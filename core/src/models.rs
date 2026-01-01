use std::collections::HashMap;

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

    pub fn as_str(&self) -> &'static str {
        match self {
            ShellType::Pwsh => "pwsh",
            ShellType::Bash => "bash",
            ShellType::Sh => "sh",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pwsh" | "powershell" => Some(ShellType::Pwsh),
            "bash" => Some(ShellType::Bash),
            "sh" => Some(ShellType::Sh),
            _ => None,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHandler {
    pub id: Uuid,
    pub event_type: String,
    pub shell: ShellType,
    #[serde(skip_serializing)]
    pub command: String,
    #[serde(default)]
    pub timeout: Option<u32>,
    #[serde(default, skip_serializing)]
    pub env: HashMap<String, String>,
}

impl EventHandler {
    pub fn new(
        event_type: String,
        shell: ShellType,
        command: String,
        timeout: Option<u32>,
        env: HashMap<String, String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            shell,
            command,
            timeout,
            env,
        }
    }
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

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pending" => Some(JobStatus::Pending),
            "running" => Some(JobStatus::Running),
            "completed" => Some(JobStatus::Completed),
            "failed" => Some(JobStatus::Failed),
            "cancelled" => Some(JobStatus::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub event: Event,
    pub handler_id: Uuid,
    pub status: JobStatus,
    pub output: Option<String>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

impl Job {
    pub fn new(event: Event, handler_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            event,
            handler_id,
            status: JobStatus::Pending,
            output: None,
            error: None,
            started_at: None,
            finished_at: None,
        }
    }
}
