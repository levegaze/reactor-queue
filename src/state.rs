use std::collections::{HashMap, VecDeque};
use std::sync::{atomic::AtomicU64, Mutex};
use crate::models::Job;

pub struct AppState {
    pub jobs: Mutex<HashMap<u64, Job>>,
    pub queue: Mutex<VecDeque<u64>>,
    pub job_counter: AtomicU64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            jobs: Mutex::new(HashMap::new()),
            queue: Mutex::new(VecDeque::new()),
            job_counter: AtomicU64::new(1),
        }
    }
}
