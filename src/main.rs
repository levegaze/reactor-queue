use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::broadcast;
use tokio::task::JoinSet;
use tokio::time::timeout;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

pub mod models;
pub mod state;
pub mod worker;
pub mod api;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Load .env file
    dotenvy::dotenv().ok();

    // 1. Setup Database Connection Pool
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    let db_pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("Connected to database");

    // 2. Setup shutdown channel
    let (shutdown_tx, _) = broadcast::channel(1);

    let app_state = Arc::new(AppState {
        db_pool,
        shutdown_tx: shutdown_tx.clone(),
    });

    // 3. Spawn Worker Pool with JoinSet
    const NUM_WORKERS: usize = 4;
    let mut workers = JoinSet::new();

    for worker_id in 0..NUM_WORKERS {
        let worker_state = app_state.clone();
        workers.spawn(async move {
            worker::run_worker(worker_id, worker_state).await;
        });
    }

    // 4. Build Router
    let app = Router::new()
        .route("/jobs", post(api::submit_job))
        .route("/jobs/{id}", get(api::get_job))
        .with_state(app_state.clone())
        .layer(TraceLayer::new_for_http());

    // 5. Start Server
    let addr = "127.0.0.1:8080";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Server running at http://{}", addr);
    println!("Worker pool started with {} workers", NUM_WORKERS);

    let server_task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // 6. Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("\nReceived shutdown signal (Ctrl+C)...");
        }
    }

    // 7. Graceful shutdown
    println!("Notifying workers to shutdown...");
    let _ = shutdown_tx.send(());

    println!("Waiting for workers to finish (timeout: 30s)...");
    match timeout(Duration::from_secs(30), async {
        while let Some(result) = workers.join_next().await {
            if let Err(e) = result {
                eprintln!("Worker error during shutdown: {}", e);
            }
        }
    }).await {
        Ok(_) => println!("All workers shutdown successfully"),
        Err(_) => println!("Timeout reached, forcing shutdown"),
    }

    // Stop the server by dropping the listener (axum will gracefully shutdown)
    let _ = server_task.await;
    println!("Shutdown complete");

    Ok(())
}
