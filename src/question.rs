use std::fmt::Display;

use log::trace;
use rand::Rng;

#[derive(Clone)]
pub struct Question {
    id: i64,
    question: String,
    expected_answer: i64,
    user_answer: Option<i64>,
}

impl Question {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let (question, answer) = generate_question(&mut rng);
        trace!("Generated question: {} = {}", question, answer);
        Self {
            id: 0,
            question,
            expected_answer: answer,
            user_answer: None,
        }
    }

    pub fn from_question(
        id: i64,
        question: String,
        expected_answer: i64,
        user_answer: Option<i64>,
    ) -> Self {
        Self {
            id,
            question,
            expected_answer,
            user_answer,
        }
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn get_question(&self) -> String {
        self.question.clone()
    }

    pub fn get_expected_answer(&self) -> i64 {
        self.expected_answer
    }

    pub fn get_answer(&self) -> Option<i64> {
        self.user_answer
    }

    fn get_input(&self) -> String {
        self.user_answer.map(|x| x.to_string()).unwrap_or_default()
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
