use crate::models::Job;
use crate::state::AppState;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

fn get_current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub async fn run_worker(worker_id: usize, data: Arc<AppState>) {
    println!("[Worker {}] Started and waiting for jobs...", worker_id);

    let mut shutdown_rx = data.shutdown_tx.subscribe();

    loop {
        // Check for shutdown signal
        if shutdown_rx.try_recv().is_ok() {
            println!("[Worker {}] Received shutdown signal, finishing current job...", worker_id);
            break;
        }
        // Dequeue job from database with SKIP LOCKED (atomic operation)
        let job_result = sqlx::query_as!(
            Job,
            r#"
            SELECT id, name, status as "status: crate::models::JobStatus", retry_count, max_retries, created_at, started_at, completed_at, failed_reason
            FROM jobs
            WHERE status = 'Queued'
            ORDER BY created_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
            "#
        )
        .fetch_optional(&data.db_pool)
        .await;

        match job_result {
            Ok(Some(job)) => {
                println!("[Worker {}] Processing job #{}", worker_id, job.id);

                // Mark as Processing and set started_at timestamp
                let now = get_current_timestamp();
                sqlx::query!(
                    "UPDATE jobs SET status = 'Processing', started_at = $1 WHERE id = $2",
                    now,
                    job.id
                )
                .execute(&data.db_pool)
                .await
                .ok();

                // Simulate work
                sleep(Duration::from_secs(5)).await;

                // Simulate 30% failure rate
                let failed = rand::random::<f64>() < 0.3;

                if failed {
                    let new_retry_count = job.retry_count + 1;

                    println!("[Worker {}] Job #{} failed! (retry {}/{})", worker_id, job.id, job.retry_count, job.max_retries);

                    // Retry logic
                    if new_retry_count <= job.max_retries {
                        sqlx::query!(
                            "UPDATE jobs SET status = 'Queued', retry_count = $1, started_at = NULL, failed_reason = NULL WHERE id = $2",
                            new_retry_count,
                            job.id
                        )
                        .execute(&data.db_pool)
                        .await
                        .ok();

                        println!("[Worker {}] Job #{} re-queued for retry {}/{}", worker_id, job.id, new_retry_count, job.max_retries);
                    } else {
                        sqlx::query!(
                            "UPDATE jobs SET status = 'Failed', failed_reason = $1 WHERE id = $2",
                            "Max retries exceeded",
                            job.id
                        )
                        .execute(&data.db_pool)
                        .await
                        .ok();

                        println!("[Worker {}] Job #{} permanently failed after {} retries", worker_id, job.id, new_retry_count);
                    }
                } else {
                    // Job succeeded
                    let completed_at = get_current_timestamp();
                    sqlx::query!(
                        "UPDATE jobs SET status = 'Completed', completed_at = $1 WHERE id = $2",
                        completed_at,
                        job.id
                    )
                    .execute(&data.db_pool)
                    .await
                    .ok();

                    println!("[Worker {}] Job #{} completed successfully!", worker_id, job.id);
                }
            }
            Ok(None) => {
                // No jobs available, sleep briefly
                sleep(Duration::from_millis(500)).await;
            }
            Err(e) => {
                eprintln!("[Worker {}] Database error: {}", worker_id, e);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    println!("[Worker {}] Shutdown complete", worker_id);
}
