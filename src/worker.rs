use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::models::JobStatus;
use crate::state::AppState;

pub async fn run_worker(data: Arc<AppState>) {
    println!("Worker started and waiting for jobs...");

    loop {
        
        let next_job_id = {
            let mut queue = data.queue.lock().unwrap();
            queue.pop_front()
        };

        if let Some(id) = next_job_id {
            println!(" Processing job #{}", id);

            {
                let mut jobs = data.jobs.lock().unwrap();
                if let Some(job) = jobs.get_mut(&id) {
                    job.status = JobStatus::Processing;
                }
            }

            sleep(Duration::from_secs(5)).await;

            {
                let mut jobs = data.jobs.lock().unwrap();
                if let Some(job) = jobs.get_mut(&id) {
                    job.status = JobStatus::Completed;
                    println!("Job #{} finished!", id);
                }
            }
        } else {
            sleep(Duration::from_millis(500)).await;
        }
    }
}
