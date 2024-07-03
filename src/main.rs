#![allow(non_snake_case)]

use dioxus::prelude::*;
use tracing::Level;

mod test_repo;

const _STYLE: &str = manganis::mg!(file("public/tailwind.css"));

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[derive(Clone)]
struct Question {
    question: String,
    answer: i64,
    current_input: Option<i64>,
}

impl Question {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let (question, answer) = test_repo::generate_question(&mut rng);
        Self {
            question,
            answer,
            current_input: None,
        }
    }

    fn check_answer(&self) -> bool {
        self.current_input == Some(self.answer)
    }

    fn input_digit(&mut self, digit: i64) {
        self.current_input = Some(self.current_input.unwrap_or(0) * 10 + digit);
    }

    fn backspace(&mut self) {
        self.current_input = self.current_input.map(|x| x / 10);
        if self.current_input == Some(0) {
            self.current_input = None;
        }
    }

    fn get_input(&self) -> String {
        self.current_input
            .map(|x| x.to_string())
            .unwrap_or_default()
    }

    fn can_submit(&self) -> bool {
        self.current_input.is_some()
    }
}

#[derive(Clone)]
struct Statistic {
    correct: i64,
    total: i64,
}

impl Statistic {
    fn new() -> Self {
        Self {
            correct: 0,
            total: 0,
        }
    }

    fn update(&mut self, correct: bool) {
        self.total += 1;
        if correct {
            self.correct += 1;
        }
    }

    fn get_accuracy(&self) -> f64 {
        100.0f64
            * if self.total == 0 {
                0.0
            } else {
                self.correct as f64 / self.total as f64
            }
    }
}

#[component]
fn CalcButton(digit: i64) -> Element {
    let mut input = consume_context::<Signal<Question>>();

    rsx! {
        div {
            class: "text-4xl text-center text-white bg-green-800 border-0 py-4 px-3 focus:outline-none hover:bg-green-700 rounded mt-4 md:mt-4",
            onclick: move |_| {
                input.write().input_digit(digit);
            },
            "{digit}"
        }
    }
}

#[component]
fn NumberDisplay() -> Element {
    let input = consume_context::<Signal<Question>>();

    rsx! {
        div {
            class: "font-mono font-black text-4xl bg-green-300 border-0 py-4 px-3 rounded text-base mt-10",
            "{input.read().question} = {input.read().get_input()}"
        }
    }
}

#[component]
fn Statistic() -> Element {
    let stat = consume_context::<Signal<Statistic>>();

    let accuracy = stat.read().get_accuracy();
    let color = if accuracy >= 90.0 {
        "text-green-500"
    } else if accuracy >= 70.0 {
        "text-yellow-500"
    } else {
        "text-red-500"
    };

    let accuracy = format!("{:.2}%", accuracy);

    rsx! {
        div {
            class: "mt-20 text-3xl text-center grid gap-4 grid-cols-2",
            div {
                "做对:"
            }
            div {
                "{stat.read().correct}"
            }
            div {
                "总数:"
            }
            div {
                "{stat.read().total}"
            }
            div {
                "正确率:"
            }
            div {
                class: "{color}",
                "{accuracy}"
            }
        }
    }
}

#[component]
fn Home() -> Element {
    use_context_provider(|| Signal::new(Statistic::new()));
    use_context_provider(|| Signal::new(Question::new()));

    rsx! {
        div {
            class: "w-full max-w-md m-1.5",
            NumberDisplay {}
            div {
                class: "grid gap-4 grid-cols-5 mt-10",
                CalcButton { digit: 1 }
                CalcButton { digit: 2 }
                CalcButton { digit: 3 }
                CalcButton { digit: 4 }
                CalcButton { digit: 5 }
                }
            div {
                class: "grid gap-4 grid-cols-5 mt-5",
                CalcButton { digit: 6 }
                CalcButton { digit: 7 }
                CalcButton { digit: 8 }
                CalcButton { digit: 9 }
                CalcButton { digit: 0 }
                }
            div {
                class: "grid gap-4 grid-cols-5 mt-10",
                button {
                    class: "text-3xl text-center text-white bg-red-800 border-0 py-4 px-3 focus:outline-none hover:bg-red-700 rounded text-base mt-4 md:mt-4 col-span-2",
                    onclick: |_| {
                        let mut input = consume_context::<Signal<Question>>();
                        input.write().backspace();
                    },
                    "←"
                }
                button {
                    class: "text-3xl text-center text-white bg-blue-800 border-0 py-4 px-3 focus:outline-none hover:bg-blue-700 rounded text-base mt-4 md:mt-4 col-start-4 col-span-2",
                    onclick: |_| {
                        if consume_context::<Signal<Question>>().read().can_submit() {
                            if consume_context::<Signal<Question>>().read().check_answer() {
                                let mut input = consume_context::<Signal<Question>>();
                                *input.write() = Question::new();
                                let mut stat = consume_context::<Signal<Statistic>>();
                                stat.write().update(true);
                            } else {
                                let mut input = consume_context::<Signal<Question>>();
                                *input.write() = Question::new();
                                let mut stat = consume_context::<Signal<Statistic>>();
                                stat.write().update(false);
                            }
                        }
                    },
                    "提交"
                }
            }
            Statistic {}
        }
    }
}
