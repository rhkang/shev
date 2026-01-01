use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::db::{Database, Event, EventHandler, Job, JobStatus, ScheduleRecord, TimerRecord};
pub use shev_core::api::{Warning, WarningKind};

#[derive(Clone)]
pub struct JobStore {
    db: Database,
    handlers: Arc<RwLock<HashMap<String, EventHandler>>>,
    timers: Arc<RwLock<HashMap<String, TimerRecord>>>,
    schedules: Arc<RwLock<HashMap<String, ScheduleRecord>>>,
    warnings: Arc<RwLock<Vec<Warning>>>,
}

impl JobStore {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            timers: Arc::new(RwLock::new(HashMap::new())),
            schedules: Arc::new(RwLock::new(HashMap::new())),
            warnings: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_warning(&self, warning: Warning) {
        let mut warnings = self.warnings.write().await;

        if !warnings
            .iter()
            .any(|w| w.kind == warning.kind && w.event_type == warning.event_type)
        {
            warnings.push(warning);
        }
    }

    pub async fn get_warnings(&self) -> Vec<Warning> {
        let warnings = self.warnings.read().await;
        let handlers = self.handlers.read().await;

        warnings
            .iter()
            .filter(|w| match w.kind {
                WarningKind::MissingHandler => !handlers.contains_key(&w.event_type),
            })
            .cloned()
            .collect()
    }

    pub async fn has_handler(&self, event_type: &str) -> bool {
        let handlers = self.handlers.read().await;
        handlers.contains_key(event_type)
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

    pub async fn create_handler(
        &self,
        event_type: &str,
        shell: &shev_core::ShellType,
        command: &str,
        timeout: Option<u32>,
        env: &std::collections::HashMap<String, String>,
    ) -> Result<EventHandler, String> {
        let handler = self
            .db
            .insert_handler(event_type, shell, command, timeout, env)
            .await?;
        let mut handlers = self.handlers.write().await;
        handlers.insert(event_type.to_string(), handler.clone());
        Ok(handler)
    }

    pub async fn update_handler(
        &self,
        event_type: &str,
        shell: Option<&shev_core::ShellType>,
        command: Option<&str>,
        timeout: Option<Option<u32>>,
        env: Option<&std::collections::HashMap<String, String>>,
    ) -> Result<EventHandler, String> {
        let handler = self
            .db
            .update_handler(event_type, shell, command, timeout, env)
            .await?;
        let mut handlers = self.handlers.write().await;
        handlers.insert(event_type.to_string(), handler.clone());
        Ok(handler)
    }

    pub async fn delete_handler(&self, event_type: &str) -> Result<bool, String> {
        let deleted = self.db.delete_handler(event_type).await?;
        if deleted {
            let mut handlers = self.handlers.write().await;
            handlers.remove(event_type);

            // Check for orphaned timers/schedules and add warnings
            let timers = self.timers.read().await;
            if timers.contains_key(event_type) {
                drop(timers);
                self.add_warning(Warning::missing_handler(event_type, "Timer"))
                    .await;
            } else {
                drop(timers);
            }

            let schedules = self.schedules.read().await;
            if schedules.contains_key(event_type) {
                drop(schedules);
                self.add_warning(Warning::missing_handler(event_type, "Schedule"))
                    .await;
            }
        }
        Ok(deleted)
    }

    pub async fn create_timer(
        &self,
        event_type: &str,
        interval_secs: u32,
        context: &str,
    ) -> Result<TimerRecord, String> {
        self.db
            .insert_timer(event_type, interval_secs, context)
            .await
    }

    pub async fn update_timer_record(
        &self,
        event_type: &str,
        interval_secs: Option<u32>,
        context: Option<&str>,
    ) -> Result<TimerRecord, String> {
        self.db
            .update_timer(event_type, interval_secs, context)
            .await
    }

    pub async fn delete_timer(&self, event_type: &str) -> Result<bool, String> {
        let deleted = self.db.delete_timer(event_type).await?;
        if deleted {
            let mut timers = self.timers.write().await;
            timers.remove(event_type);
        }
        Ok(deleted)
    }

    pub async fn create_schedule(
        &self,
        event_type: &str,
        scheduled_time: chrono::DateTime<chrono::Utc>,
        context: &str,
        periodic: bool,
    ) -> Result<ScheduleRecord, String> {
        self.db
            .insert_schedule(event_type, scheduled_time, context, periodic)
            .await
    }

    pub async fn update_schedule_record(
        &self,
        event_type: &str,
        scheduled_time: Option<chrono::DateTime<chrono::Utc>>,
        context: Option<&str>,
        periodic: Option<bool>,
    ) -> Result<ScheduleRecord, String> {
        self.db
            .update_schedule(event_type, scheduled_time, context, periodic)
            .await
    }

    pub async fn delete_schedule(&self, event_type: &str) -> Result<bool, String> {
        let deleted = self.db.delete_schedule(event_type).await?;
        if deleted {
            let mut schedules = self.schedules.write().await;
            schedules.remove(event_type);
        }
        Ok(deleted)
    }

    pub async fn get_config(&self, key: &str) -> Option<String> {
        self.db.get_config(key).await
    }

    pub async fn set_config(&self, key: &str, value: &str) -> Result<(), String> {
        self.db.set_config(key, value).await
    }
}
