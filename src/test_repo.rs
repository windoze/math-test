use std::path::Path;

use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use log::debug;
use now::DateTimeNow;
use tokio_rusqlite::Connection;

use crate::question::Question;

#[derive(Clone)]
pub struct TestRepo {
    connection: Connection,
}

impl TestRepo {
    pub async fn new<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let connection = Connection::open(path).await?;
        connection
            .call(|conn| {
                conn.execute_batch(
                    "BEGIN;
                        CREATE TABLE IF NOT EXISTS questions (
                        id INTEGER PRIMARY KEY,
                        question TEXT NOT NULL,
                        expected_answer INTEGER NOT NULL,
                        answer INTEGER,
                        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                        answered_at TIMESTAMP
                    );
                    CREATE INDEX IF NOT EXISTS idx_created_at ON questions (created_at);
                    COMMIT;
                ",
                )?;
                Ok(())
            })
            .await?;

        Ok(Self { connection })
    }

    pub async fn new_question(&self) -> anyhow::Result<Question> {
        Ok(self
            .connection
            .call(|conn| {
                debug!("Finding existing unanswered question");
                let mut stmt = conn.prepare("SELECT id, question, expected_answer FROM questions WHERE answer is NULL ORDER BY RANDOM() LIMIT 1")?;
                if let Ok(question) = stmt.query_row([], |row| {
                    Ok(Question::from_question(
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        None,
                    ))
                }) {
                    debug!("Found existing unanswered question, id: {}, question: {}", question.get_id(), question.get_question());
                    return Ok(question);
                }

                debug!("Creating new question");
                let mut stmt = conn.prepare("INSERT INTO questions (question, expected_answer) VALUES (?1, ?2) RETURNING id")?;

                let question = Question::new();
                let id: i64 = stmt.query_row(
                    (question.get_question(), question.get_expected_answer()),
                    |row| row.get(0),
                )?;
                debug!("Insert new question, id: {}, question: {}", id, question.get_question());
                let mut stmt = conn.prepare("SELECT id, question, expected_answer FROM questions WHERE id = ?1")?;
                let question = stmt.query_row([id], |row| {
                    Ok(Question::from_question(
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        None,
                    ))
                })?;
                debug!("Created new question, id: {}, question: {}", question.get_id(), question.get_question());
                Ok(question)
            })
            .await?)
    }

    pub async fn answer_question(&self, id: i64, answer: i64) -> anyhow::Result<bool> {
        Ok(self
            .connection
            .call(move |conn| {
                debug!("Answering question, id: {}, answer: {}", id, answer);
                conn.execute(
                    "UPDATE questions SET answer = ?1, answered_at = CURRENT_TIMESTAMP WHERE id = ?2",
                    (answer, id),
                )?;
                let expected_answer: i64 = conn.query_row(
                    "SELECT expected_answer FROM questions WHERE id = ?1",
                    [id],
                    |row| row.get(0),
                )?;
                debug!("The answer is correct: {}", expected_answer == answer);
                Ok(expected_answer == answer)
            })
            .await?)
    }

    pub async fn get_statistics(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> anyhow::Result<(i64, i64)> {
        debug!("start: {:?}, end: {:?}", start, end);
        Ok(self
            .connection
            .call(move |conn| {
                let correct: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM questions WHERE answer is not NULL AND answer = expected_answer AND answered_at BETWEEN ?1 AND ?2",
                    [
                        start.unwrap_or_else(|| Utc::now() - chrono::Duration::days(1000)),
                        end.unwrap_or_else(Utc::now),
                    ],
                    |row| row.get(0),
                )?;
                let total: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM questions WHERE answer is not NULL AND answered_at BETWEEN ?1 AND ?2",
                    [
                        start.unwrap_or_else(|| Utc::now() - chrono::Duration::days(1000)),
                        end.unwrap_or_else(Utc::now),
                    ],
                    |row| row.get(0),
                )?;
                debug!("correct: {}, total: {}", correct, total);
                Ok((correct, total))
                    }).await?)
    }

    pub async fn mistake_collection(&self) -> anyhow::Result<Vec<(i64, String, Option<i64>)>> {
        Ok(self
            .connection
            .call(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, question, answer FROM questions WHERE answer is not null AND answer != expected_answer",
                )?;
                let mut rows = stmt.query([])?;
                let mut result = Vec::new();
                while let Some(row) = rows.next()? {
                    result.push((row.get(0)?, row.get(1)?, row.get(2)?));
                }
                Ok(result)
            })
            .await?)
    }

    pub async fn get_daily_statistics(
        &self,
        year: i32,
        month: u32,
        day: u32,
        timezone: String,
    ) -> anyhow::Result<(i64, i64)> {
        let tz: Tz = timezone.parse().unwrap();
        let date = tz
            .with_ymd_and_hms(year, month, day, 0, 0, 0)
            .single()
            .ok_or(anyhow::Error::msg("Invalid date"))?;
        let day_start = date.beginning_of_day().with_timezone(&Utc);
        debug!("day_start: {:?}", day_start);
        let day_end = date.end_of_day().with_timezone(&Utc);
        debug!("day_end: {:?}", day_end);

        self.get_statistics(Some(day_start), Some(day_end)).await
    }
}
