use std::path::Path;

use chrono::{DateTime, Utc};
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
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS questions (
                    id INTEGER PRIMARY KEY,
                    question TEXT NOT NULL,
                    expected_answer INTEGER NOT NULL,
                    answer INTEGER,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    answered_at TIMESTAMP
                )",
                    [],
                )?;
                Ok(())
            })
            .await?;

        Ok(Self { connection })
    }

    pub async fn new_question(&self) -> anyhow::Result<(i64, Question)> {
        Ok(self
            .connection
            .call(|conn| {
                let mut stmt = conn.prepare(
                "INSERT INTO questions (question, expected_answer) VALUES (?1, ?2) RETURNING id",
            )?;

                let question = Question::new();
                let id: i64 = stmt.query_row(
                    (question.get_question(), question.get_expected_answer()),
                    |row| row.get(0),
                )?;
                Ok((id, question))
            })
            .await?)
    }

    pub async fn answer_question(&self, id: i64, answer: i64) -> anyhow::Result<bool> {
        Ok(self
            .connection
            .call(move |conn| {
                conn.execute(
                    "UPDATE questions SET answer = ?1, answered_at = CURRENT_TIMESTAMP WHERE id = ?2",
                    (answer, id),
                )?;
                let expected_answer: i64 = conn.query_row(
                    "SELECT expected_answer FROM questions WHERE id = ?1",
                    [id],
                    |row| row.get(0),
                )?;
                Ok(expected_answer == answer)
            })
            .await?)
    }

    pub async fn get_statistics(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> anyhow::Result<(i64, i64)> {
        Ok(self
            .connection
            .call(move |conn| {
                let correct: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM questions WHERE answer = expected_answer AND answered_at BETWEEN ?1 AND ?2",
                    [
                        start.unwrap_or_else(|| Utc::now() - chrono::Duration::days(1000)),
                        end.unwrap_or_else(Utc::now),
                    ],
                    |row| row.get(0),
                )?;
                let total: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM questions WHERE answered_at BETWEEN ?1 AND ?2",
                    [
                        start.unwrap_or_else(|| Utc::now() - chrono::Duration::days(1000)),
                        end.unwrap_or_else(Utc::now),
                    ],
                    |row| row.get(0),
                )?;
                Ok((correct, total))
                    }).await?)
    }

    pub async fn mistake_collection(&self) -> anyhow::Result<Vec<(i64, String, Option<i64>)>> {
        Ok(self
            .connection
            .call(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, question, answer FROM questions WHERE answer != expected_answer",
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
}
