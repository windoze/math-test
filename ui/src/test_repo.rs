use std::fmt::Display;

use crate::api::{get_new_question, submit_answer};

#[derive(Clone)]
pub struct Question {
    pub id: i64,
    pub question: String,
    pub current_input: Option<i64>,
}

impl Question {
    pub async fn new() -> anyhow::Result<Self> {
        let question = get_new_question().await?;
        Ok(question)
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

    // Submit the current input and get a new question
    pub async fn submit(self) -> anyhow::Result<Self> {
        submit_answer(self.id, self.current_input.unwrap_or_default()).await?;
        Self::new().await
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

    pub fn get_accuracy(&self) -> f64 {
        100.0f64
            * if self.total == 0 {
                0.0
            } else {
                self.correct as f64 / self.total as f64
            }
    }
}
