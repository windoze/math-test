use std::fmt::Display;

use rand::Rng;

#[derive(Clone)]
pub struct Question {
    question: String,
    answer: i64,
    current_input: Option<i64>,
}

impl Question {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let (question, answer) = generate_question(&mut rng);
        Self {
            question,
            answer,
            current_input: None,
        }
    }

    pub fn check_answer(&self) -> bool {
        self.current_input == Some(self.answer)
    }

    pub fn input_digit(&mut self, digit: i64) {
        self.current_input = Some(self.current_input.unwrap_or(0) * 10 + digit);
    }

    pub fn backspace(&mut self) {
        self.current_input = self.current_input.map(|x| x / 10);
        if self.current_input == Some(0) {
            self.current_input = None;
        }
    }

    pub fn can_submit(&self) -> bool {
        self.current_input.is_some()
    }

    fn get_input(&self) -> String {
        self.current_input
            .map(|x| x.to_string())
            .unwrap_or_default()
    }
}

impl Display for Question {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.question, self.get_input())
    }
}

fn generate_question<R: Rng>(rng: &mut R) -> (String, i64) {
    loop {
        let a = rng.gen_range(2..=100);
        let b = rng.gen_range(2..=100);
        let op = rng.gen_range(0..4);

        match op {
            0 => {
                // Addition
                return (format!("{} + {}", a, b), a + b);
            }
            1 => {
                // Subtraction, ensure result is non-negative
                if a >= b {
                    return (format!("{} - {}", a, b), a - b);
                }
            }
            2 => {
                // Multiplication
                let b = rng.gen_range(3..=30);
                return (format!("{} x {}", a, b), a * b);
            }
            3 => {
                // Division, ensure result is an integer
                let divisor = rng.gen_range(3..=30); // Limit divisor to ensure result is within 1-100
                let quotient = rng.gen_range(3..=30);
                let dividend = divisor * quotient;
                if dividend <= 100 {
                    return (format!("{} ÷ {}", dividend, divisor), quotient);
                }
            }
            _ => unreachable!(),
        }
    }
}


#[derive(Clone)]
pub struct Statistic {
    pub correct: i64,
    pub total: i64,
}

impl Statistic {
    pub fn new() -> Self {
        Self {
            correct: 0,
            total: 0,
        }
    }

    pub fn update(&mut self, correct: bool) {
        self.total += 1;
        if correct {
            self.correct += 1;
        }
    }

    pub fn get_accuracy(&self) -> f64 {
        100.0f64
            * if self.total == 0 {
                0.0
            } else {
                self.correct as f64 / self.total as f64
            }
    }
}