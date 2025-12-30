use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    Pwsh,
    Bash,
    Sh,
}

impl ShellType {
    pub fn command_args<'a>(&self, command: &'a str) -> (&'static str, Vec<&'a str>) {
        match self {
            ShellType::Pwsh => ("pwsh", vec!["-Command", command]),
            ShellType::Bash => ("bash", vec!["-c", command]),
            ShellType::Sh => ("sh", vec!["-c", command]),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub context: String,
    pub timestamp: DateTime<Utc>,
}

impl Event {
    pub fn new(event_type: String, context: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            context,
            timestamp: Utc::now(),
        }
    }
}

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHandler {
    pub event_type: String,
    pub shell: ShellType,
    #[serde(skip_serializing)]
    pub command: String,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default, skip_serializing)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub event: Event,
    pub handler: EventHandler,
    pub status: JobStatus,
    pub output: Option<String>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

impl Job {
    pub fn new(event: Event, handler: EventHandler) -> Self {
        Self {
            id: Uuid::new_v4(),
            event,
            handler,
            status: JobStatus::Pending,
            output: None,
            error: None,
            started_at: None,
            finished_at: None,
        }
    }
}
