use std::{env, path::PathBuf, time::Duration};

use chrono::Utc;
use chrono_tz::Tz;
use clap::{command, Parser};
use env_logger::Env;
use log::{debug, info};
use now::DateTimeNow;
use poem::{
    endpoint::{EmbeddedFileEndpoint, EmbeddedFilesEndpoint},
    handler,
    listener::{Listener, RustlsCertificate, RustlsConfig, TcpListener},
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
    time_zone: String,
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
    debug!(
        "id: {}, question: {}",
        question.get_id(),
        question.get_question()
    );
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
    debug!("id: {}, answer: {}", req.id, req.answer);
    let ret = state
        .repo
        .answer_question(req.id, req.answer)
        .await
        .map_err(|e| {
            log::error!("Error: {:?}", e);
            anyhow::Error::msg("Failed to answer the question")
        })?;
    debug!("correct: {}", ret);
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
    debug!("start: {:?}, end: {:?}", start, end);
    let (correct, total) = state.repo.get_statistics(start, end).await.map_err(|e| {
        log::error!("Error: {:?}", e);
        anyhow::Error::msg("Failed to get statistics")
    })?;
    debug!("correct: {}, total: {}", correct, total);
    Ok(Json(StatisticsResponse { correct, total }))
}

#[handler]
async fn today_statistics(Data(state): Data<&AppState>) -> poem::Result<Json<StatisticsResponse>> {
    let tz: Tz = state.time_zone.parse().unwrap();
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
    debug!("correct: {}, total: {}", correct, total);
    Ok(Json(StatisticsResponse { correct, total }))
}

#[handler]
async fn get_mistake_collection(
    Data(state): Data<&AppState>,
) -> poem::Result<Json<Vec<QuestionResponse>>> {
    let ret: Vec<QuestionResponse> = state
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
    debug!("mistake_collection contains {} items", ret.len());
    Ok(Json(ret))
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Listen address
    #[arg(short, long, default_value = "localhost:3001")]
    listen: String,

    /// Default time zone, used to calculate today's statistics
    #[arg(short, long, default_value = "Asia/Shanghai")]
    timezone: String,

    /// Database path, default to "questions.db" under the current directory
    #[arg(short, long)]
    database: Option<PathBuf>,

    /// Enable TLS
    #[arg(short, long, default_value = "false")]
    tls: bool,

    /// Path to the certificate file
    #[arg(long)]
    cert: Option<PathBuf>,

    /// Path to the private key file
    #[arg(long)]
    key: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    if args.tls && (args.cert.is_none() || args.key.is_none()) {
        log::error!("Certificate and private key are required for TLS");
        return Ok(());
    }

    info!("Initializing the database");
    let db_path = args.database.clone().unwrap_or_else(|| {
        env::var("DB_PATH")
            .unwrap_or_else(|_| "questions.db".to_string())
            .into()
    });
    let state = AppState {
        time_zone: args.timezone.clone(),
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

    if args.tls {
        info!("Starting server at https://{}", &args.listen);
        let listener = TcpListener::bind(args.listen.clone()).rustls(async_stream::stream! {
            loop {
                if let Ok(tls_config) = load_tls_config(&args) {
                    yield tls_config;
                }
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        });
        Server::new(listener)
            .run_with_graceful_shutdown(
                app,
                async move {
                    let _ = tokio::signal::ctrl_c().await;
                },
                Some(Duration::from_secs(5)),
            )
            .await
    } else {
        info!("Starting server at http://{}", &args.listen); // DevSkim: ignore DS137138
        Server::new(TcpListener::bind(args.listen.clone()))
            .run_with_graceful_shutdown(
                app,
                async move {
                    let _ = tokio::signal::ctrl_c().await;
                },
                Some(Duration::from_secs(5)),
            )
            .await
    }?;
    info!("Server stopped");
    Ok(())
}

fn load_tls_config(args: &Args) -> Result<RustlsConfig, std::io::Error> {
    Ok(RustlsConfig::new().fallback(
        RustlsCertificate::new()
            .cert(std::fs::read(args.cert.to_owned().unwrap())?)
            .key(std::fs::read(args.key.to_owned().unwrap())?),
    ))
}
