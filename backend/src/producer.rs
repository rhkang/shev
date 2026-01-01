use std::time::Duration;

use chrono::Utc;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::db::{Event, ScheduleRecord, TimerRecord};
use crate::queue::EventSender;
use crate::store::JobStore;

#[derive(Clone)]
pub struct TimerManager {
    store: JobStore,
}

impl TimerManager {
    pub fn new(store: JobStore) -> Self {
        Self { store }
    }

    pub async fn register_timer(&self, config: TimerRecord, sender: EventSender) {
        let event_type = config.event_type.clone();
        let timer_id = config.id;

        {
            let existing_timer = self.store.get_timer(&event_type).await;
            if let Some(existing) = existing_timer {
                if existing.id == timer_id {
                    info!(
                        "Timer '{}' (id: {}) already running, skipping",
                        event_type, timer_id
                    );
                    return;
                }

                info!(
                    "Timer '{}' updated (old: {}, new: {}), old will stop on next cycle",
                    event_type, existing.id, timer_id
                );
            }
        }

        info!("Starting timer '{}' (id: {})", event_type, timer_id);

        self.store.register_timer(config.clone()).await;

        let store = self.store.clone();
        tokio::spawn(async move {
            run_timer(config, sender, store).await;
        });
    }
}

#[derive(Clone)]
pub struct ScheduleManager {
    store: JobStore,
}

impl ScheduleManager {
    pub fn new(store: JobStore) -> Self {
        Self { store }
    }

    pub async fn register_schedule(&self, config: ScheduleRecord, sender: EventSender) {
        let event_type = config.event_type.clone();
        let schedule_id = config.id;

        {
            let existing_schedule = self.store.get_schedule(&event_type).await;
            if let Some(existing) = existing_schedule {
                if existing.id == schedule_id {
                    info!(
                        "Schedule '{}' (id: {}) already running, skipping",
                        event_type, schedule_id
                    );
                    return;
                }

                info!(
                    "Schedule '{}' updated (old: {}, new: {}), old will stop on next cycle",
                    event_type, existing.id, schedule_id
                );
            }
        }

        info!("Starting schedule '{}' (id: {})", event_type, schedule_id);

        self.store.register_schedule(config.clone()).await;

        let store = self.store.clone();
        tokio::spawn(async move {
            run_schedule(config, sender, store).await;
        });
    }
}

async fn run_timer(config: TimerRecord, sender: EventSender, store: JobStore) {
    let timer_id = config.id;
    info!(
        "Timer started for '{}' (id: {}) with interval {}s",
        config.event_type, timer_id, config.interval_secs
    );

    loop {
        sleep(Duration::from_secs(config.interval_secs.into())).await;

        let current_id = store.get_timer_id(&config.event_type).await;
        if current_id != Some(timer_id) {
            info!(
                "Timer '{}' (id: {}) is outdated or removed, stopping",
                config.event_type, timer_id
            );
            break;
        }

        if !store.has_handler(&config.event_type).await {
            warn!(
                "Timer '{}': No handler found, skipping event",
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
    }
}

async fn run_schedule(config: ScheduleRecord, sender: EventSender, store: JobStore) {
    use chrono::Duration as ChronoDuration;

    let schedule_id = config.id;
    let mode = if config.periodic {
        "periodic"
    } else {
        "one-shot"
    };
    info!(
        "Schedule started for '{}' (id: {}) at {} ({})",
        config.event_type, schedule_id, config.scheduled_time, mode
    );

    let mut next_time = config.scheduled_time;

    loop {
        let now = Utc::now();

        if config.periodic {
            while next_time <= now {
                next_time = next_time + ChronoDuration::days(1);
            }
        }

        let wait_duration = if next_time > now {
            let duration = next_time - now;
            Duration::from_secs(duration.num_seconds().max(0) as u64)
        } else {
            Duration::from_secs(0)
        };

        if wait_duration.as_secs() > 0 {
            info!(
                "Schedule '{}' waiting {}s until {}",
                config.event_type,
                wait_duration.as_secs(),
                next_time
            );
            sleep(wait_duration).await;
        }

        let current_id = store.get_schedule_id(&config.event_type).await;
        if current_id != Some(schedule_id) {
            info!(
                "Schedule '{}' (id: {}) is outdated or removed, stopping",
                config.event_type, schedule_id
            );
            break;
        }

        if !store.has_handler(&config.event_type).await {
            warn!(
                "Schedule '{}': No handler found, skipping event",
                config.event_type
            );
            if config.periodic {
                next_time = next_time + ChronoDuration::days(1);
                continue;
            } else {
                break;
            }
        }

        let event = Event::new(config.event_type.clone(), config.context.clone());
        info!("Schedule producing event: {:?}", event.id);

        if sender.send(event).await.is_err() {
            warn!("Schedule channel closed for '{}'", config.event_type);
            break;
        }

        if config.periodic {
            info!(
                "Schedule '{}' fired, next run at {}",
                config.event_type,
                next_time + ChronoDuration::days(1)
            );
            next_time = next_time + ChronoDuration::days(1);
        } else {
            info!(
                "Schedule '{}' fired (one-shot), stopping",
                config.event_type
            );
            break;
        }
    }
}
