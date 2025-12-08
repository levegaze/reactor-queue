use actix_web::{web, App, HttpServer};
use std::sync::Arc;

use crate::state::AppState;

pub mod models;
pub mod state;
pub mod worker;
pub mod api;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 1. Setup Shared State
    let app_state = Arc::new(AppState::new());

    // 2. Spawn Worker
    let worker_state = app_state.clone();
    tokio::spawn(async move {
        worker::run_worker(worker_state).await;
    });

    // 3. Start Server
    println!("Server running at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            // เรียกใช้ Handler จาก module api
            .route("/jobs", web::post().to(api::submit_job))
            .route("/jobs/{id}", web::get().to(api::get_job))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
