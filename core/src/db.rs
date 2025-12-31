use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::models::{Event, EventHandler, Job, JobStatus, ShellType};

pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS handlers (
    id TEXT PRIMARY KEY,
    event_type TEXT UNIQUE NOT NULL,
    shell TEXT NOT NULL,
    command TEXT NOT NULL,
    timeout INTEGER,
    env TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS timers (
    id TEXT PRIMARY KEY,
    event_type TEXT UNIQUE NOT NULL,
    context TEXT DEFAULT '',
    interval_secs INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    event_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_context TEXT,
    event_timestamp TEXT NOT NULL,
    handler_id TEXT NOT NULL,
    status TEXT NOT NULL,
    output TEXT,
    error TEXT,
    started_at TEXT,
    finished_at TEXT
);

CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO config (key, value) VALUES ('port', '3000');
INSERT OR IGNORE INTO config (key, value) VALUES ('queue_size', '100');
"#;

#[derive(Debug, Clone)]
pub struct TimerRecord {
    pub id: Uuid,
    pub event_type: String,
    pub context: String,
    pub interval_secs: u64,
}

impl TimerRecord {
    pub fn new(event_type: String, context: String, interval_secs: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            context,
            interval_secs,
        }
    }
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("Failed to open database: {}", e))?;
        Ok(Self { conn })
    }

    pub fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(SCHEMA)
            .map_err(|e| format!("Failed to init schema: {}", e))?;
        Ok(())
    }

    // Config operations
    pub fn get_config(&self, key: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT value FROM config WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .ok()
    }

    pub fn set_config(&self, key: &str, value: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .map_err(|e| format!("Failed to set config: {}", e))?;
        Ok(())
    }

    pub fn get_port(&self) -> u16 {
        self.get_config("port")
            .and_then(|v| v.parse().ok())
            .unwrap_or(3000)
    }

    pub fn get_queue_size(&self) -> usize {
        self.get_config("queue_size")
            .and_then(|v| v.parse().ok())
            .unwrap_or(100)
    }

    // Handler operations
    pub fn insert_handler(
        &self,
        event_type: &str,
        shell: &ShellType,
        command: &str,
        timeout: Option<u64>,
        env: &HashMap<String, String>,
    ) -> Result<EventHandler, String> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        let env_json = serde_json::to_string(env).unwrap_or_default();

        self.conn
            .execute(
                r#"INSERT INTO handlers (id, event_type, shell, command, timeout, env, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
                params![
                    id.to_string(),
                    event_type,
                    shell.as_str(),
                    command,
                    timeout,
                    env_json,
                    now,
                    now
                ],
            )
            .map_err(|e| format!("Failed to insert handler: {}", e))?;

        Ok(EventHandler {
            id,
            event_type: event_type.to_string(),
            shell: shell.clone(),
            command: command.to_string(),
            timeout,
            env: env.clone(),
        })
    }

    pub fn update_handler(
        &self,
        event_type: &str,
        shell: Option<&ShellType>,
        command: Option<&str>,
        timeout: Option<Option<u64>>,
    ) -> Result<EventHandler, String> {
        let existing = self
            .get_handler(event_type)?
            .ok_or_else(|| format!("Handler '{}' not found", event_type))?;

        let new_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        let new_shell = shell.unwrap_or(&existing.shell);
        let new_command = command.unwrap_or(&existing.command);
        let new_timeout = timeout.unwrap_or(existing.timeout);

        self.conn
            .execute(
                r#"UPDATE handlers SET id = ?1, shell = ?2, command = ?3, timeout = ?4, updated_at = ?5
               WHERE event_type = ?6"#,
                params![
                    new_id.to_string(),
                    new_shell.as_str(),
                    new_command,
                    new_timeout,
                    now,
                    event_type
                ],
            )
            .map_err(|e| format!("Failed to update handler: {}", e))?;

        Ok(EventHandler {
            id: new_id,
            event_type: event_type.to_string(),
            shell: new_shell.clone(),
            command: new_command.to_string(),
            timeout: new_timeout,
            env: existing.env,
        })
    }

    pub fn delete_handler(&self, event_type: &str) -> Result<bool, String> {
        let rows = self
            .conn
            .execute(
                "DELETE FROM handlers WHERE event_type = ?1",
                params![event_type],
            )
            .map_err(|e| format!("Failed to delete handler: {}", e))?;
        Ok(rows > 0)
    }

    pub fn get_handler(&self, event_type: &str) -> Result<Option<EventHandler>, String> {
        self.conn
            .query_row(
                "SELECT id, event_type, shell, command, timeout, env FROM handlers WHERE event_type = ?1",
                params![event_type],
                |row| Self::row_to_handler(row),
            )
            .optional()
            .map_err(|e| format!("Failed to get handler: {}", e))
    }

    /// Get the current handler UUID for an event type (for checking if a job's handler is still current)
    pub fn get_handler_id(&self, event_type: &str) -> Result<Option<Uuid>, String> {
        self.conn
            .query_row(
                "SELECT id FROM handlers WHERE event_type = ?1",
                params![event_type],
                |row| {
                    let id: String = row.get(0)?;
                    Ok(Uuid::parse_str(&id).ok())
                },
            )
            .optional()
            .map_err(|e| format!("Failed to get handler id: {}", e))
            .map(|opt| opt.flatten())
    }

    pub fn get_all_handlers(&self) -> Result<Vec<EventHandler>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, event_type, shell, command, timeout, env FROM handlers ORDER BY event_type")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let iter = stmt
            .query_map([], |row| Self::row_to_handler(row))
            .map_err(|e| format!("Failed to query handlers: {}", e))?;

        Ok(iter.filter_map(|r| r.ok()).collect())
    }

    fn row_to_handler(row: &rusqlite::Row) -> rusqlite::Result<EventHandler> {
        let id: String = row.get(0)?;
        let event_type: String = row.get(1)?;
        let shell_str: String = row.get(2)?;
        let command: String = row.get(3)?;
        let timeout: Option<u64> = row.get(4)?;
        let env_json: String = row.get(5)?;

        let shell = ShellType::from_str(&shell_str).unwrap_or(ShellType::Sh);
        let env: HashMap<String, String> = serde_json::from_str(&env_json).unwrap_or_default();

        Ok(EventHandler {
            id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4()),
            event_type,
            shell,
            command,
            timeout,
            env,
        })
    }

    // Timer operations
    pub fn insert_timer(
        &self,
        event_type: &str,
        interval_secs: u64,
        context: &str,
    ) -> Result<TimerRecord, String> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();

        self.conn
            .execute(
                r#"INSERT INTO timers (id, event_type, context, interval_secs, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
                params![id.to_string(), event_type, context, interval_secs, now, now],
            )
            .map_err(|e| format!("Failed to insert timer: {}", e))?;

        Ok(TimerRecord {
            id,
            event_type: event_type.to_string(),
            context: context.to_string(),
            interval_secs,
        })
    }

    pub fn update_timer(
        &self,
        event_type: &str,
        interval_secs: Option<u64>,
        context: Option<&str>,
    ) -> Result<TimerRecord, String> {
        let existing = self
            .get_timer(event_type)?
            .ok_or_else(|| format!("Timer '{}' not found", event_type))?;

        let new_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        let new_interval = interval_secs.unwrap_or(existing.interval_secs);
        let new_context = context.unwrap_or(&existing.context);

        self.conn
            .execute(
                r#"UPDATE timers SET id = ?1, context = ?2, interval_secs = ?3, updated_at = ?4
               WHERE event_type = ?5"#,
                params![
                    new_id.to_string(),
                    new_context,
                    new_interval,
                    now,
                    event_type
                ],
            )
            .map_err(|e| format!("Failed to update timer: {}", e))?;

        Ok(TimerRecord {
            id: new_id,
            event_type: event_type.to_string(),
            context: new_context.to_string(),
            interval_secs: new_interval,
        })
    }

    pub fn delete_timer(&self, event_type: &str) -> Result<bool, String> {
        let rows = self
            .conn
            .execute(
                "DELETE FROM timers WHERE event_type = ?1",
                params![event_type],
            )
            .map_err(|e| format!("Failed to delete timer: {}", e))?;
        Ok(rows > 0)
    }

    /// Get the current timer UUID for an event type (for checking if a timer is still current)
    pub fn get_timer_id(&self, event_type: &str) -> Result<Option<Uuid>, String> {
        self.conn
            .query_row(
                "SELECT id FROM timers WHERE event_type = ?1",
                params![event_type],
                |row| {
                    let id: String = row.get(0)?;
                    Ok(Uuid::parse_str(&id).ok())
                },
            )
            .optional()
            .map_err(|e| format!("Failed to get timer id: {}", e))
            .map(|opt| opt.flatten())
    }

    pub fn get_timer(&self, event_type: &str) -> Result<Option<TimerRecord>, String> {
        self.conn
            .query_row(
                "SELECT id, event_type, context, interval_secs FROM timers WHERE event_type = ?1",
                params![event_type],
                |row| {
                    let id: String = row.get(0)?;
                    let event_type: String = row.get(1)?;
                    let context: String = row.get(2)?;
                    let interval_secs: u64 = row.get(3)?;

                    Ok(TimerRecord {
                        id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4()),
                        event_type,
                        context,
                        interval_secs,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to get timer: {}", e))
    }

    pub fn get_all_timers(&self) -> Result<Vec<TimerRecord>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, event_type, context, interval_secs FROM timers ORDER BY event_type",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let iter = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let event_type: String = row.get(1)?;
                let context: String = row.get(2)?;
                let interval_secs: u64 = row.get(3)?;

                Ok(TimerRecord {
                    id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4()),
                    event_type,
                    context,
                    interval_secs,
                })
            })
            .map_err(|e| format!("Failed to query timers: {}", e))?;

        Ok(iter.filter_map(|r| r.ok()).collect())
    }

    // Job operations
    pub fn insert_job(&self, job: &Job) -> Result<(), String> {
        self.conn
            .execute(
                r#"INSERT INTO jobs (id, event_id, event_type, event_context, event_timestamp, handler_id, status, output, error, started_at, finished_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
                params![
                    job.id.to_string(),
                    job.event.id.to_string(),
                    job.event.event_type,
                    job.event.context,
                    job.event.timestamp.to_rfc3339(),
                    job.handler_id.to_string(),
                    job.status.as_str(),
                    job.output,
                    job.error,
                    job.started_at.map(|t| t.to_rfc3339()),
                    job.finished_at.map(|t| t.to_rfc3339())
                ],
            )
            .map_err(|e| format!("Failed to insert job: {}", e))?;
        Ok(())
    }

    pub fn update_job(&self, job: &Job) -> Result<(), String> {
        self.conn
            .execute(
                r#"UPDATE jobs SET status = ?1, output = ?2, error = ?3, started_at = ?4, finished_at = ?5
               WHERE id = ?6"#,
                params![
                    job.status.as_str(),
                    job.output,
                    job.error,
                    job.started_at.map(|t| t.to_rfc3339()),
                    job.finished_at.map(|t| t.to_rfc3339()),
                    job.id.to_string()
                ],
            )
            .map_err(|e| format!("Failed to update job: {}", e))?;
        Ok(())
    }

    pub fn get_job(&self, job_id: Uuid) -> Result<Option<Job>, String> {
        self.conn
            .query_row(
                r#"SELECT id, event_id, event_type, event_context, event_timestamp, handler_id, status, output, error, started_at, finished_at
               FROM jobs WHERE id = ?1"#,
                params![job_id.to_string()],
                |row| Self::row_to_job(row),
            )
            .optional()
            .map_err(|e| format!("Failed to get job: {}", e))
    }

    pub fn get_all_jobs(
        &self,
        status: Option<&JobStatus>,
        limit: usize,
    ) -> Result<Vec<Job>, String> {
        let query = match status {
            Some(s) => format!(
                "SELECT id, event_id, event_type, event_context, event_timestamp, handler_id, status, output, error, started_at, finished_at
                 FROM jobs WHERE status = '{}' ORDER BY event_timestamp DESC LIMIT {}",
                s.as_str(), limit
            ),
            None => format!(
                "SELECT id, event_id, event_type, event_context, event_timestamp, handler_id, status, output, error, started_at, finished_at
                 FROM jobs ORDER BY event_timestamp DESC LIMIT {}",
                limit
            ),
        };

        let mut stmt = self
            .conn
            .prepare(&query)
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let iter = stmt
            .query_map([], |row| Self::row_to_job(row))
            .map_err(|e| format!("Failed to query jobs: {}", e))?;

        Ok(iter.filter_map(|r| r.ok()).collect())
    }

    pub fn has_active_job(&self, event_type: &str) -> bool {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM jobs WHERE event_type = ?1 AND (status = 'pending' OR status = 'running')",
                params![event_type],
                |row| {
                    let count: i64 = row.get(0)?;
                    Ok(count > 0)
                },
            )
            .unwrap_or(false)
    }

    /// Mark all pending/running jobs as cancelled (used on startup to clean up stale jobs from unexpected shutdown)
    pub fn cancel_stale_jobs(&self) -> Result<usize, String> {
        let now = Utc::now().to_rfc3339();
        let rows = self
            .conn
            .execute(
                "UPDATE jobs SET status = 'cancelled', error = 'Backend restarted', finished_at = ?1 WHERE status = 'pending' OR status = 'running'",
                params![now],
            )
            .map_err(|e| format!("Failed to cancel stale jobs: {}", e))?;
        Ok(rows)
    }

    fn row_to_job(row: &rusqlite::Row) -> rusqlite::Result<Job> {
        let id: String = row.get(0)?;
        let event_id: String = row.get(1)?;
        let event_type: String = row.get(2)?;
        let event_context: String = row.get(3)?;
        let event_timestamp: String = row.get(4)?;
        let handler_id: String = row.get(5)?;
        let status_str: String = row.get(6)?;
        let output: Option<String> = row.get(7)?;
        let error: Option<String> = row.get(8)?;
        let started_at: Option<String> = row.get(9)?;
        let finished_at: Option<String> = row.get(10)?;

        let status = JobStatus::from_str(&status_str).unwrap_or(JobStatus::Cancelled);

        Ok(Job {
            id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4()),
            event: Event {
                id: Uuid::parse_str(&event_id).unwrap_or_else(|_| Uuid::new_v4()),
                event_type,
                context: event_context,
                timestamp: DateTime::parse_from_rfc3339(&event_timestamp)
                    .map(|t| t.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            },
            handler_id: Uuid::parse_str(&handler_id).unwrap_or_else(|_| Uuid::new_v4()),
            status,
            output,
            error,
            started_at: started_at.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|t| t.with_timezone(&Utc))
                    .ok()
            }),
            finished_at: finished_at.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|t| t.with_timezone(&Utc))
                    .ok()
            }),
        })
    }
}

trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
