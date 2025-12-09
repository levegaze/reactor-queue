use actix_web::{web, HttpResponse, Responder};
use std::sync::Arc;

use crate::models::{Job, JobRequest};
use crate::state::AppState;

// POST /jobs
pub async fn submit_job(
    data: web::Data<Arc<AppState>>,
    req: web::Json<JobRequest>
) -> impl Responder {
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
            HttpResponse::Ok().json(job)
        }
        Err(e) => {
            eprintln!("Failed to create job: {}", e);
            HttpResponse::InternalServerError().body("Failed to create job")
        }
    }
}

// GET /jobs/{id}
pub async fn get_job(
    data: web::Data<Arc<AppState>>,
    path: web::Path<i64>
) -> impl Responder {
    let id = path.into_inner();

    let result = sqlx::query_as::<_, Job>(
        "SELECT id, name, status, retry_count, max_retries, created_at, started_at, completed_at, failed_reason FROM jobs WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&data.db_pool)
    .await;

    match result {
        Ok(Some(job)) => HttpResponse::Ok().json(job),
        Ok(None) => HttpResponse::NotFound().body("Job not found"),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}