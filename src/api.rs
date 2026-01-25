use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;

use crate::models::{Job, JobRequest};
use crate::state::AppState;

// POST /jobs
pub async fn submit_job(
    State(data): State<Arc<AppState>>,
    Json(req): Json<JobRequest>,
) -> impl IntoResponse {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Insert job into database
    let result = sqlx::query_as::<_, Job>(
        r#"
        INSERT INTO jobs (name, status, retry_count, max_retries, created_at, started_at, completed_at, failed_reason)
        VALUES ($1, 'Queued', 0, 3, $2, NULL, NULL, NULL)
        RETURNING id, name, status, retry_count, max_retries, created_at, started_at, completed_at, failed_reason
        "#
    )
    .bind(&req.name)
    .bind(now)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(job) => {
            println!("Received Job #{} ({})", job.id, job.name);
            (StatusCode::OK, Json(job)).into_response()
        }
        Err(e) => {
            eprintln!("Failed to create job: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create job").into_response()
        }
    }
}

// GET /jobs/{id}
pub async fn get_job(
    State(data): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Job>(
        "SELECT id, name, status, retry_count, max_retries, created_at, started_at, completed_at, failed_reason FROM jobs WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&data.db_pool)
    .await;

    match result {
        Ok(Some(job)) => (StatusCode::OK, Json(job)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Job not found").into_response(),
        Err(e) => {
            eprintln!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}
