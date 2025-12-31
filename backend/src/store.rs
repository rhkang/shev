use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::db::{Database, Event, EventHandler, Job, JobStatus, ScheduleRecord, TimerRecord};

#[derive(Clone)]
pub struct JobStore {
    db: Database,
    handlers: Arc<RwLock<HashMap<String, EventHandler>>>,
    timers: Arc<RwLock<HashMap<String, TimerRecord>>>,
    schedules: Arc<RwLock<HashMap<String, ScheduleRecord>>>,
}

impl JobStore {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            timers: Arc::new(RwLock::new(HashMap::new())),
            schedules: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn load_handlers(&self) {
        let db_handlers = self.db.get_all_handlers().await;
        let mut handlers = self.handlers.write().await;

        handlers.clear();
        for handler in db_handlers {
            handlers.insert(handler.event_type.clone(), handler);
        }
    }

    pub async fn load_timers(&self) -> Vec<TimerRecord> {
        let db_timers = self.db.get_all_timers().await;
        let mut timers = self.timers.write().await;

        timers.clear();
        for timer in &db_timers {
            timers.insert(timer.event_type.clone(), timer.clone());
        }

        db_timers
    }

    pub async fn get_handler(&self, event_type: &str) -> Option<EventHandler> {
        let handlers = self.handlers.read().await;
        handlers.get(event_type).cloned()
    }

    pub async fn get_handlers(&self) -> Vec<EventHandler> {
        let handlers = self.handlers.read().await;
        handlers.values().cloned().collect()
    }

    pub async fn register_timer(&self, timer: TimerRecord) {
        let mut timers = self.timers.write().await;
        timers.insert(timer.event_type.clone(), timer);
    }

    pub async fn get_timer(&self, event_type: &str) -> Option<TimerRecord> {
        let timers = self.timers.read().await;
        timers.get(event_type).cloned()
    }

    pub async fn get_timers(&self) -> Vec<TimerRecord> {
        let timers = self.timers.read().await;
        timers.values().cloned().collect()
    }

    pub async fn create_job(&self, event: Event, handler: &EventHandler) -> Job {
        let job = Job::new(event, handler.id);
        let _ = self.db.insert_job(&job).await;
        job
    }

    pub async fn mark_running(&self, job_id: Uuid) {
        if let Some(mut job) = self.db.get_job(job_id).await {
            job.status = JobStatus::Running;
            job.started_at = Some(Utc::now());
            let _ = self.db.update_job(&job).await;
        }
    }

    pub async fn mark_completed(&self, job_id: Uuid, output: String) {
        if let Some(mut job) = self.db.get_job(job_id).await {
            job.status = JobStatus::Completed;
            job.output = Some(output);
            job.finished_at = Some(Utc::now());
            let _ = self.db.update_job(&job).await;
        }
    }

    pub async fn mark_failed(&self, job_id: Uuid, error: String) {
        if let Some(mut job) = self.db.get_job(job_id).await {
            job.status = JobStatus::Failed;
            job.error = Some(error);
            job.finished_at = Some(Utc::now());
            let _ = self.db.update_job(&job).await;
        }
    }

    pub async fn cancel_job(&self, job_id: Uuid) -> bool {
        if let Some(mut job) = self.db.get_job(job_id).await {
            if job.status == JobStatus::Pending || job.status == JobStatus::Running {
                job.status = JobStatus::Cancelled;
                job.finished_at = Some(Utc::now());
                let _ = self.db.update_job(&job).await;
                return true;
            }
        }
        false
    }

    pub async fn get_job(&self, job_id: Uuid) -> Option<Job> {
        self.db.get_job(job_id).await
    }

    pub async fn get_all_jobs(&self) -> Vec<Job> {
        self.db.get_all_jobs().await
    }

    pub async fn get_jobs_by_status(&self, status: JobStatus) -> Vec<Job> {
        self.db.get_jobs_by_status(status).await
    }

    pub async fn get_completed_jobs(&self) -> Vec<Job> {
        self.get_jobs_by_status(JobStatus::Completed).await
    }

    pub async fn has_active_job(&self, event_type: &str) -> bool {
        self.db.has_active_job(event_type).await
    }

    pub async fn get_timer_id(&self, event_type: &str) -> Option<Uuid> {
        self.db.get_timer_id(event_type).await
    }

    pub async fn load_schedules(&self) -> Vec<ScheduleRecord> {
        let db_schedules = self.db.get_all_schedules().await;
        let mut schedules = self.schedules.write().await;

        schedules.clear();
        for schedule in &db_schedules {
            schedules.insert(schedule.event_type.clone(), schedule.clone());
        }

        db_schedules
    }

    pub async fn register_schedule(&self, schedule: ScheduleRecord) {
        let mut schedules = self.schedules.write().await;
        schedules.insert(schedule.event_type.clone(), schedule);
    }

    pub async fn get_schedule(&self, event_type: &str) -> Option<ScheduleRecord> {
        let schedules = self.schedules.read().await;
        schedules.get(event_type).cloned()
    }

    pub async fn get_schedules(&self) -> Vec<ScheduleRecord> {
        let schedules = self.schedules.read().await;
        schedules.values().cloned().collect()
    }

    pub async fn get_schedule_id(&self, event_type: &str) -> Option<Uuid> {
        self.db.get_schedule_id(event_type).await
    }
}
