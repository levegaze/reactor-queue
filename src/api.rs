use actix_web::{web, HttpResponse, Responder};
use std::sync::{atomic::Ordering, Arc};

use crate::models::{Job, JobRequest, JobStatus};
use crate::state::AppState;

// POST /jobs
pub async fn submit_job(
    data: web::Data<Arc<AppState>>, 
    req: web::Json<JobRequest>
) -> impl Responder {
    let id = data.job_counter.fetch_add(1, Ordering::SeqCst);

    let new_job = Job {
        id,
        name: req.name.clone(),
        status: JobStatus::Queued,
    };

    {
        let mut jobs = data.jobs.lock().unwrap();
        let mut queue = data.queue.lock().unwrap();
        jobs.insert(id, new_job.clone());
        queue.push_back(id);
    }

    println!("Received Job #{} ({})", id, req.name);
    HttpResponse::Ok().json(new_job)
}

// GET /jobs/{id}
pub async fn get_job(
    data: web::Data<Arc<AppState>>, 
    path: web::Path<u64>
) -> impl Responder {
    let id = path.into_inner();
    let jobs = data.jobs.lock().unwrap();

    match jobs.get(&id) {
        Some(job) => HttpResponse::Ok().json(job),
        None => HttpResponse::NotFound().body("Job not found"),
    }
}