use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum JobStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Job {
    pub id: i64,
    pub name: String,
    pub status: JobStatus,
    pub retry_count: i32,
    pub max_retries: i32,
    pub created_at: i64,           // Unix timestamp
    pub started_at: Option<i64>,   // Unix timestamp
    pub completed_at: Option<i64>, // Unix timestamp
    pub failed_reason: Option<String>,
}

#[derive(Deserialize)]
pub struct JobRequest {
    pub name: String,
}
