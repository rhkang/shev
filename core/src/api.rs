use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Handler types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerResponse {
    pub id: String,
    pub event_type: String,
    pub shell: String,
    pub command: String,
    pub timeout: Option<u32>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateHandlerRequest {
    pub event_type: String,
    pub shell: String,
    pub command: String,
    pub timeout: Option<u32>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHandlerRequest {
    pub shell: Option<String>,
    pub command: Option<String>,
    pub timeout: Option<Option<u32>>,
    pub env: Option<HashMap<String, String>>,
}

// ============================================================================
// Timer types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerResponse {
    pub id: String,
    pub event_type: String,
    pub interval_secs: u32,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTimerRequest {
    pub event_type: String,
    pub interval_secs: u32,
    #[serde(default)]
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTimerRequest {
    pub interval_secs: Option<u32>,
    pub context: Option<String>,
}

// ============================================================================
// Schedule types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleResponse {
    pub id: String,
    pub event_type: String,
    pub scheduled_time: DateTime<Utc>,
    pub context: String,
    pub periodic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduleRequest {
    pub event_type: String,
    pub scheduled_time: DateTime<Utc>,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub periodic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScheduleRequest {
    pub scheduled_time: Option<DateTime<Utc>>,
    pub context: Option<String>,
    pub periodic: Option<bool>,
}

// ============================================================================
// Job types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponse {
    pub id: String,
    pub event: EventResponse,
    pub handler_id: String,
    pub status: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResponse {
    pub id: String,
    pub event_type: String,
    pub context: String,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Config types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub port: String,
    pub queue_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub port: Option<String>,
    pub queue_size: Option<String>,
}

// ============================================================================
// Status types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub total_jobs: usize,
    pub pending_jobs: usize,
    pub running_jobs: usize,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
}

// ============================================================================
// Event trigger types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEventRequest {
    pub event_type: String,
    #[serde(default)]
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEventResponse {
    pub triggered: bool,
    pub message: String,
}

// ============================================================================
// Reload types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReloadResponse {
    pub success: bool,
    pub handlers_loaded: usize,
    pub timers_loaded: usize,
    pub schedules_loaded: usize,
}

// ============================================================================
// Health types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WarningKind {
    MissingHandler,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    pub kind: WarningKind,
    pub event_type: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

impl Warning {
    pub fn missing_handler(event_type: &str, source: &str) -> Self {
        Self {
            kind: WarningKind::MissingHandler,
            event_type: event_type.to_string(),
            message: format!(
                "{} '{}' has no handler - events will be skipped",
                source, event_type
            ),
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub warnings: Vec<Warning>,
}
