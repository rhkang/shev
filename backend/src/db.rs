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

    pub async fn insert_handler(
        &self,
        event_type: &str,
        shell: &shev_core::ShellType,
        command: &str,
        timeout: Option<u64>,
        env: &std::collections::HashMap<String, String>,
    ) -> Result<EventHandler, String> {
        let db = self.inner.lock().await;
        db.insert_handler(event_type, shell, command, timeout, env)
    }

    pub async fn update_handler(
        &self,
        event_type: &str,
        shell: Option<&shev_core::ShellType>,
        command: Option<&str>,
        timeout: Option<Option<u64>>,
        env: Option<&std::collections::HashMap<String, String>>,
    ) -> Result<EventHandler, String> {
        let db = self.inner.lock().await;
        db.update_handler(event_type, shell, command, timeout, env)
    }

    pub async fn delete_handler(&self, event_type: &str) -> Result<bool, String> {
        let db = self.inner.lock().await;
        db.delete_handler(event_type)
    }

    pub async fn insert_timer(
        &self,
        event_type: &str,
        interval_secs: u64,
        context: &str,
    ) -> Result<TimerRecord, String> {
        let db = self.inner.lock().await;
        db.insert_timer(event_type, interval_secs, context)
    }

    pub async fn update_timer(
        &self,
        event_type: &str,
        interval_secs: Option<u64>,
        context: Option<&str>,
    ) -> Result<TimerRecord, String> {
        let db = self.inner.lock().await;
        db.update_timer(event_type, interval_secs, context)
    }

    pub async fn delete_timer(&self, event_type: &str) -> Result<bool, String> {
        let db = self.inner.lock().await;
        db.delete_timer(event_type)
    }

    pub async fn insert_schedule(
        &self,
        event_type: &str,
        scheduled_time: chrono::DateTime<chrono::Utc>,
        context: &str,
        periodic: bool,
    ) -> Result<ScheduleRecord, String> {
        let db = self.inner.lock().await;
        db.insert_schedule(event_type, scheduled_time, context, periodic)
    }

    pub async fn update_schedule(
        &self,
        event_type: &str,
        scheduled_time: Option<chrono::DateTime<chrono::Utc>>,
        context: Option<&str>,
        periodic: Option<bool>,
    ) -> Result<ScheduleRecord, String> {
        let db = self.inner.lock().await;
        db.update_schedule(event_type, scheduled_time, context, periodic)
    }

    pub async fn delete_schedule(&self, event_type: &str) -> Result<bool, String> {
        let db = self.inner.lock().await;
        db.delete_schedule(event_type)
    }

    pub async fn get_config(&self, key: &str) -> Option<String> {
        let db = self.inner.lock().await;
        db.get_config(key)
    }

    pub async fn set_config(&self, key: &str, value: &str) -> Result<(), String> {
        let db = self.inner.lock().await;
        db.set_config(key, value)
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
