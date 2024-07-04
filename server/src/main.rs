use std::env;

use poem::{
    endpoint::{EmbeddedFileEndpoint, EmbeddedFilesEndpoint},
    handler,
    listener::TcpListener,
    middleware::AddData,
    post,
    web::{Data, Json},
    EndpointExt, Request, Route, Server,
};
use rust_embed::RustEmbed;

mod question;
mod test_repo;

#[derive(RustEmbed)]
#[folder = "../ui/dist"]
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
    let (id, question) = state.repo.new_question().await.map_err(|e| {
        log::error!("Error: {:?}", e);
        anyhow::Error::msg("Failed to create new question")
    })?;
    Ok(Json(QuestionResponse {
        id,
        question: question.get_question(),
    }))
}

#[derive(serde::Deserialize)]
struct SubmitAnswerRequest {
    id: i64,
    answer: i64,
}

#[handler]
async fn submit_answer(
    Json(req): Json<SubmitAnswerRequest>,
    Data(state): Data<&AppState>,
) -> poem::Result<Json<bool>> {
    let ret = state
        .repo
        .answer_question(req.id, req.answer)
        .await
        .map_err(|e| {
            log::error!("Error: {:?}", e);
            anyhow::Error::msg("Failed to answer the question")
        })?;
    Ok(Json(ret))
}

#[derive(serde::Deserialize)]
struct GetStatisticsRequest {
    start: Option<String>,
    end: Option<String>,
}

#[handler]
async fn get_statistics(
    req: &Request,
    Data(state): Data<&AppState>,
) -> poem::Result<Json<(i64, i64)>> {
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
    Ok(Json((correct, total)))
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
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=warn");
    }

    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "questions.db".to_string());
    let state = AppState {
        repo: test_repo::TestRepo::new(&db_path).await?,
    };

    let app = Route::new()
        .at("/api/new-question", post(new_question))
        .at("/api/submit-answer", post(submit_answer))
        .at("/api/statistics", get_statistics)
        .at("/api/mistake-collection", get_mistake_collection)
        .at("/", EmbeddedFileEndpoint::<Files>::new("index.html"))
        .nest("/", EmbeddedFilesEndpoint::<Files>::new())
        .with(AddData::new(state));
    Server::new(TcpListener::bind("0.0.0.0:3001"))
        .run(app)
        .await?;
    Ok(())
}
