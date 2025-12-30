use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::{Event, EventHandler, Job, JobStatus};

#[derive(Debug, Clone)]
pub struct JobStore {
    jobs: Arc<RwLock<HashMap<Uuid, Job>>>,
    handlers: Arc<RwLock<HashMap<String, EventHandler>>>,
}

impl JobStore {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_handler(&self, handler: EventHandler) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(handler.event_type.clone(), handler);
    }

    pub async fn get_handler(&self, event_type: &str) -> Option<EventHandler> {
        let handlers = self.handlers.read().await;
        handlers.get(event_type).cloned()
    }

    pub async fn get_handlers(&self) -> Vec<EventHandler> {
        let handlers = self.handlers.read().await;
        handlers.values().cloned().collect()
    }

    pub async fn create_job(&self, event: Event, handler: EventHandler) -> Job {
        let job = Job::new(event, handler);
        let mut jobs = self.jobs.write().await;
        jobs.insert(job.id, job.clone());
        job
    }

    pub async fn mark_running(&self, job_id: Uuid) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            job.status = JobStatus::Running;
            job.started_at = Some(Utc::now());
        }
    }

    pub async fn mark_completed(&self, job_id: Uuid, output: String) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            job.status = JobStatus::Completed;
            job.output = Some(output);
            job.finished_at = Some(Utc::now());
        }
    }

    pub async fn mark_failed(&self, job_id: Uuid, error: String) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            job.status = JobStatus::Failed;
            job.error = Some(error);
            job.finished_at = Some(Utc::now());
        }
    }

    pub async fn cancel_job(&self, job_id: Uuid) -> bool {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            if job.status == JobStatus::Pending || job.status == JobStatus::Running {
                job.status = JobStatus::Cancelled;
                job.finished_at = Some(Utc::now());
                return true;
            }
        }
        false
    }

    pub async fn get_job(&self, job_id: Uuid) -> Option<Job> {
        let jobs = self.jobs.read().await;
        jobs.get(&job_id).cloned()
    }

    pub async fn get_all_jobs(&self) -> Vec<Job> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }

    pub async fn get_jobs_by_status(&self, status: JobStatus) -> Vec<Job> {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|j| j.status == status)
            .cloned()
            .collect()
    }

    pub async fn get_completed_jobs(&self) -> Vec<Job> {
        self.get_jobs_by_status(JobStatus::Completed).await
    }

    pub async fn has_active_job(&self, event_type: &str) -> bool {
        let jobs = self.jobs.read().await;
        jobs.values().any(|j| {
            j.event.event_type == event_type
                && (j.status == JobStatus::Pending || j.status == JobStatus::Running)
        })
    }
}

impl Default for JobStore {
    fn default() -> Self {
        Self::new()
    }
}
