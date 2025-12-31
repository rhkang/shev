use std::path::Path;
use std::sync::Arc;

use tokio::sync::Mutex;
use uuid::Uuid;

pub use shev_core::{Database as SyncDatabase, ScheduleRecord, TimerRecord};
pub use shev_core::{Event, EventHandler, Job, JobStatus};

/// Async wrapper around the sync shev_core::Database
pub struct Database {
    inner: Arc<Mutex<SyncDatabase>>,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let db = SyncDatabase::open(path)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(db)),
        })
    }

    pub async fn init_schema(&self) -> Result<(), String> {
        let db = self.inner.lock().await;
        db.init_schema()
    }

    pub async fn get_port(&self) -> u16 {
        let db = self.inner.lock().await;
        db.get_port()
    }

    pub async fn get_queue_size(&self) -> usize {
        let db = self.inner.lock().await;
        db.get_queue_size()
    }

    pub async fn get_all_handlers(&self) -> Vec<EventHandler> {
        let db = self.inner.lock().await;
        db.get_all_handlers().unwrap_or_default()
    }

    pub async fn get_all_timers(&self) -> Vec<TimerRecord> {
        let db = self.inner.lock().await;
        db.get_all_timers().unwrap_or_default()
    }

    pub async fn insert_job(&self, job: &Job) -> Result<(), String> {
        let db = self.inner.lock().await;
        db.insert_job(job)
    }

    pub async fn update_job(&self, job: &Job) -> Result<(), String> {
        let db = self.inner.lock().await;
        db.update_job(job)
    }

    pub async fn get_job(&self, job_id: Uuid) -> Option<Job> {
        let db = self.inner.lock().await;
        db.get_job(job_id).ok().flatten()
    }

    pub async fn get_all_jobs(&self) -> Vec<Job> {
        let db = self.inner.lock().await;
        db.get_all_jobs(None, 1000).unwrap_or_default()
    }

    pub async fn get_jobs_by_status(&self, status: JobStatus) -> Vec<Job> {
        let db = self.inner.lock().await;
        db.get_all_jobs(Some(&status), 1000).unwrap_or_default()
    }

    pub async fn has_active_job(&self, event_type: &str) -> bool {
        let db = self.inner.lock().await;
        db.has_active_job(event_type)
    }

    pub async fn get_timer_id(&self, event_type: &str) -> Option<Uuid> {
        let db = self.inner.lock().await;
        db.get_timer_id(event_type).ok().flatten()
    }

    pub async fn cancel_stale_jobs(&self) -> usize {
        let db = self.inner.lock().await;
        db.cancel_stale_jobs().unwrap_or(0)
    }

    pub async fn get_all_schedules(&self) -> Vec<ScheduleRecord> {
        let db = self.inner.lock().await;
        db.get_all_schedules().unwrap_or_default()
    }

    pub async fn get_schedule_id(&self, event_type: &str) -> Option<Uuid> {
        let db = self.inner.lock().await;
        db.get_schedule_id(event_type).ok().flatten()
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
