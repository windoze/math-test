use crate::test_repo::Question;

const API_BASE: &str = "http://localhost:3001/api";

#[derive(serde::Deserialize)]
struct QuestionResponse {
    id: i64,
    question: String,
}

pub async fn get_new_question() -> Result<Question, reqwest::Error> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/new_question", API_BASE))
        .send()
        .await?;
    response.json().await.map(|q: QuestionResponse| Question {
        id: q.id,
        question: q.question,
        current_input: None,
    })
}

#[derive(serde::Serialize)]
struct SubmitAnswerRequest {
    id: i64,
    answer: i64,
}

#[derive(serde::Deserialize)]
struct SubmitAnswerResponse {
    id: i64,
    correct: bool,
}

pub async fn submit_answer(id: i64, answer: i64) -> Result<bool, reqwest::Error> {
    let client = reqwest::Client::new();

    let resp: SubmitAnswerResponse = client
        .post(format!("{}/submit_answer", API_BASE))
        .json(&SubmitAnswerRequest { id, answer })
        .send()
        .await?
        .json()
        .await?;
    Ok(resp.correct)
}

#[derive(serde::Deserialize)]
struct TodayResponse {
    correct: i64,
    total: i64,
}

pub async fn today_statistics() -> Result<(i64, i64), reqwest::Error> {
    let client = reqwest::Client::new();

    let resp: TodayResponse = client
        .get(format!("{}/today", API_BASE))
        .send()
        .await?
        .json()
        .await?;
    Ok((resp.correct, resp.total))
}
