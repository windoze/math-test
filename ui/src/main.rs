#![allow(non_snake_case)]

use dioxus::prelude::*;
use test_repo::{Question, Statistic};
use tracing::Level;

mod api;
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

#[component]
fn NumberDisplay() -> Element {
    let input = consume_context::<Signal<Question>>();

    rsx! {
        div {
            class: "font-mono font-black text-4xl bg-green-300 border-0 py-4 px-3 rounded text-base mt-10",
            "{input.read()}"
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
    let question = use_resource(|| Question::new());
    let question: Question = match *question.read_unchecked() {
        Some(Ok(question)) => question,
        Some(Err(err)) => {
            // if there was an error, render the error
            return rsx! {"An error occurred while fetching stories {err}"};
        }
        None => {
            // if the future is not resolved yet, render a loading message
            return rsx! {"Loading items"};
        }
    };
    use_context_provider(|| Signal::new(Statistic::new()));
    use_context_provider(|| Signal::new(question));

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
