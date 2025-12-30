use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::models::{EventHandler, ShellType};
use crate::producer::TimerProducerConfig;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_queue_size")]
    pub queue_size: usize,
    #[serde(default)]
    pub handlers: Vec<HandlerConfig>,
    #[serde(default)]
    pub timers: Vec<TimerConfig>,
}

fn default_port() -> u16 {
    3000
}

fn default_queue_size() -> usize {
    100
}

#[derive(Debug, Deserialize)]
pub struct HandlerConfig {
    pub event_type: String,
    pub shell: ShellType,
    pub command: String,
    pub timeout: Option<u64>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl From<HandlerConfig> for EventHandler {
    fn from(config: HandlerConfig) -> Self {
        EventHandler {
            event_type: config.event_type,
            shell: config.shell,
            command: config.command,
            timeout: config.timeout,
            env: config.env,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TimerConfig {
    pub event_type: String,
    #[serde(default)]
    pub context: String,
    pub interval_secs: u64,
}

impl From<TimerConfig> for TimerProducerConfig {
    fn from(config: TimerConfig) -> Self {
        TimerProducerConfig {
            event_type: config.event_type,
            context: config.context,
            interval_secs: config.interval_secs,
        }
    }
}

impl Config {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config file: {}", e))
    }

    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        match Self::load_from_file(path) {
            Ok(config) => config,
            Err(_) => Self::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: default_port(),
            queue_size: default_queue_size(),
            handlers: Vec::new(),
            timers: Vec::new(),
        }
    }
}
