use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, RwLock};
use tokio::time::sleep;
use tracing::{info, warn};

use crate::models::Event;
use crate::queue::EventSender;
use crate::store::JobStore;

#[derive(Debug, Clone)]
pub struct TimerProducerConfig {
    pub event_type: String,
    pub context: String,
    pub interval_secs: u64,
}

#[derive(Debug)]
struct TimerState {
    trigger: Arc<Notify>,
}

#[derive(Clone)]
pub struct TimerManager {
    timers: Arc<RwLock<HashMap<String, Arc<TimerState>>>>,
    sender: EventSender,
    store: JobStore,
}

impl TimerManager {
    pub fn new(sender: EventSender, store: JobStore) -> Self {
        Self {
            timers: Arc::new(RwLock::new(HashMap::new())),
            sender,
            store,
        }
    }

    pub async fn register_timer(&self, config: TimerProducerConfig) {
        let event_type = config.event_type.clone();
        let trigger = Arc::new(Notify::new());

        let state = Arc::new(TimerState {
            trigger: trigger.clone(),
        });

        {
            let mut timers = self.timers.write().await;
            timers.insert(event_type.clone(), state);
        }

        let sender = self.sender.clone();
        let store = self.store.clone();
        tokio::spawn(async move {
            run_timer(config, sender, store, trigger).await;
        });
    }

    pub async fn trigger(&self, event_type: &str) -> bool {
        if self.store.has_active_job(event_type).await {
            info!(
                "Manual trigger ignored for '{}': job already active",
                event_type
            );
            return false;
        }

        let timers = self.timers.read().await;
        if let Some(state) = timers.get(event_type) {
            state.trigger.notify_one();
            info!("Manual trigger accepted for '{}'", event_type);
            true
        } else {
            false
        }
    }

    pub async fn has_timer(&self, event_type: &str) -> bool {
        let timers = self.timers.read().await;
        timers.contains_key(event_type)
    }
}

async fn run_timer(
    config: TimerProducerConfig,
    sender: EventSender,
    store: JobStore,
    trigger: Arc<Notify>,
) {
    info!(
        "Timer started for '{}' with interval {}s",
        config.event_type, config.interval_secs
    );

    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(config.interval_secs)) => {
                info!("Timer interval elapsed for '{}'", config.event_type);
            }
            _ = trigger.notified() => {
                info!("Timer manually triggered for '{}'", config.event_type);
            }
        }

        if store.has_active_job(&config.event_type).await {
            info!(
                "Skipping timer event for '{}': job already active",
                config.event_type
            );
            continue;
        }

        let event = Event::new(config.event_type.clone(), config.context.clone());
        info!("Timer producing event: {:?}", event.id);

        if sender.send(event).await.is_err() {
            warn!("Timer channel closed for '{}'", config.event_type);
            break;
        }

        loop {
            sleep(Duration::from_millis(100)).await;
            if !store.has_active_job(&config.event_type).await {
                break;
            }
        }

        info!("Job completed, timer resuming for '{}'", config.event_type);
    }
}

#[derive(Debug, Deserialize)]
pub struct EventRequest {
    pub event_type: String,
    #[serde(default)]
    pub context: String,
}

#[derive(Debug, Serialize)]
pub struct EventResponse {
    #[serde(flatten)]
    pub event: Option<Event>,
    pub triggered: bool,
    pub message: String,
}

#[derive(Clone)]
pub struct HttpProducerState {
    pub sender: EventSender,
    pub timer_manager: TimerManager,
}

async fn handle_event(
    State(state): State<HttpProducerState>,
    Json(request): Json<EventRequest>,
) -> Result<Json<EventResponse>, StatusCode> {
    if state.timer_manager.has_timer(&request.event_type).await {
        let triggered = state.timer_manager.trigger(&request.event_type).await;
        if triggered {
            Ok(Json(EventResponse {
                event: None,
                triggered: true,
                message: format!("Timer triggered for '{}', timer reset", request.event_type),
            }))
        } else {
            Ok(Json(EventResponse {
                event: None,
                triggered: false,
                message: format!(
                    "Event '{}' already has active job, request ignored",
                    request.event_type
                ),
            }))
        }
    } else {
        let event = Event::new(request.event_type, request.context);
        info!("HTTP producing event: {:?}", event.id);

        state
            .sender
            .send(event.clone())
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(EventResponse {
            event: Some(event),
            triggered: true,
            message: "Event queued".to_string(),
        }))
    }
}

pub fn create_http_producer_router(sender: EventSender, timer_manager: TimerManager) -> Router {
    let state = HttpProducerState {
        sender,
        timer_manager,
    };

    Router::new()
        .route("/events", post(handle_event))
        .with_state(state)
}
