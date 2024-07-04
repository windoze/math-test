use std::env;

use chrono::Utc;
use chrono_tz::Tz;
use env_logger::Env;
use log::debug;
use now::DateTimeNow;
use poem::{
    endpoint::{EmbeddedFileEndpoint, EmbeddedFilesEndpoint},
    handler,
    listener::TcpListener,
    middleware::{AddData, Cors},
    post,
    web::{Data, Json},
    EndpointExt, Request, Route, Server,
};
use rust_embed::RustEmbed;

mod question;
mod test_repo;

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
pub struct Files;

#[derive(Clone)]
struct AppState {
    repo: test_repo::TestRepo,
}

#[derive(serde::Serialize)]
struct QuestionResponse {
    id: i64,
    question: String,
}

#[handler]
async fn new_question(Data(state): Data<&AppState>) -> poem::Result<Json<QuestionResponse>> {
    let question = state.repo.new_question().await.map_err(|e| {
        log::error!("Error: {:?}", e);
        anyhow::Error::msg("Failed to create new question")
    })?;
    Ok(Json(QuestionResponse {
        id: question.get_id(),
        question: question.get_question(),
    }))
}

#[derive(serde::Deserialize)]
struct SubmitAnswerRequest {
    id: i64,
    answer: i64,
}

#[derive(serde::Serialize)]
struct SubmitAnswerResponse {
    id: i64,
    correct: bool,
}

#[handler]
async fn submit_answer(
    Json(req): Json<SubmitAnswerRequest>,
    Data(state): Data<&AppState>,
) -> poem::Result<Json<SubmitAnswerResponse>> {
    let ret = state
        .repo
        .answer_question(req.id, req.answer)
        .await
        .map_err(|e| {
            log::error!("Error: {:?}", e);
            anyhow::Error::msg("Failed to answer the question")
        })?;
    Ok(Json(SubmitAnswerResponse {
        id: req.id,
        correct: ret,
    }))
}

#[derive(serde::Deserialize)]
struct GetStatisticsRequest {
    start: Option<String>,
    end: Option<String>,
}

#[derive(serde::Serialize)]
struct StatisticsResponse {
    correct: i64,
    total: i64,
}

#[handler]
async fn get_statistics(
    req: &Request,
    Data(state): Data<&AppState>,
) -> poem::Result<Json<StatisticsResponse>> {
    let GetStatisticsRequest { start, end } = req.params()?;
    let start = start
        .map(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map_err(|e| {
                    log::error!("Error: {:?}", e);
                    anyhow::Error::msg("Failed to parse start time")
                })
                .map(|dt| dt.with_timezone(&chrono::Utc))
        })
        .map_or(Ok(None), |v| v.map(Some))?;
    let end = end
        .map(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map_err(|e| {
                    log::error!("Error: {:?}", e);
                    anyhow::Error::msg("Failed to parse end time")
                })
                .map(|dt| dt.with_timezone(&chrono::Utc))
        })
        .map_or(Ok(None), |v| v.map(Some))?;
    let (correct, total) = state.repo.get_statistics(start, end).await.map_err(|e| {
        log::error!("Error: {:?}", e);
        anyhow::Error::msg("Failed to get statistics")
    })?;
    Ok(Json(StatisticsResponse { correct, total }))
}

#[handler]
async fn today_statistics(Data(state): Data<&AppState>) -> poem::Result<Json<StatisticsResponse>> {
    let tz: Tz = "Asia/Shanghai".parse().unwrap();
    let now = Utc::now().with_timezone(&tz);
    let day_start = now.beginning_of_day().with_timezone(&Utc);
    debug!("day_start: {:?}", day_start);

    let (correct, total) = state
        .repo
        .get_statistics(Some(day_start), None)
        .await
        .map_err(|e| {
            log::error!("Error: {:?}", e);
            anyhow::Error::msg("Failed to get statistics")
        })?;
    Ok(Json(StatisticsResponse { correct, total }))
}

#[handler]
async fn get_mistake_collection(
    Data(state): Data<&AppState>,
) -> poem::Result<Json<Vec<QuestionResponse>>> {
    let ret = state
        .repo
        .mistake_collection()
        .await
        .map_err(|e| {
            log::error!("Error: {:?}", e);
            anyhow::Error::msg("Failed to get mistake collection")
        })?
        .into_iter()
        .map(|(id, question, _)| QuestionResponse { id, question })
        .collect();
    Ok(Json(ret))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "questions.db".to_string());
    let state = AppState {
        repo: test_repo::TestRepo::new(&db_path).await?,
    };

    let app = Route::new()
        .at("/api/new-question", post(new_question))
        .at("/api/submit-answer", post(submit_answer))
        .at("/api/statistics", get_statistics)
        .at("/api/mistake-collection", get_mistake_collection)
        .at("/api/today", today_statistics)
        .at("/", EmbeddedFileEndpoint::<Files>::new("index.html"))
        .nest("/", EmbeddedFilesEndpoint::<Files>::new())
        .with(Cors::new().allow_methods(vec!["GET", "POST"]))
        .with(AddData::new(state));
    Server::new(TcpListener::bind("0.0.0.0:3001"))
        .run(app)
        .await?;
    Ok(())
}
