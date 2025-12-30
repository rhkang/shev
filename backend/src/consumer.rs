use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tracing::{error, info, warn};

use crate::executor::execute_command;
use crate::models::JobStatus;
use crate::queue::EventReceiver;
use crate::store::JobStore;

#[derive(Clone)]
pub struct ConsumerControl {
    running: Arc<AtomicBool>,
}

impl ConsumerControl {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Default for ConsumerControl {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn start_consumer(
    mut receiver: EventReceiver,
    store: JobStore,
    control: ConsumerControl,
) {
    info!("Event consumer started");

    while let Some(event) = receiver.recv().await {
        if !control.is_running() {
            info!("Consumer paused, skipping event: {:?}", event.id);
            continue;
        }

        info!("Processing event: {:?} (type: {})", event.id, event.event_type);

        let handler = match store.get_handler(&event.event_type).await {
            Some(h) => h,
            None => {
                warn!("No handler for event type: {}", event.event_type);
                continue;
            }
        };

        let job = store.create_job(event.clone(), handler.clone()).await;
        let job_id = job.id;

        info!("Created job: {:?}", job_id);

        if let Some(j) = store.get_job(job_id).await {
            if j.status == JobStatus::Cancelled {
                info!("Job {:?} was cancelled before execution", job_id);
                continue;
            }
        }

        store.mark_running(job_id).await;

        match execute_command(&handler, &event.context).await {
            Ok(result) => {
                if result.success {
                    info!("Job {:?} completed successfully", job_id);
                    store.mark_completed(job_id, result.stdout).await;
                } else {
                    let error_msg = if result.stderr.is_empty() {
                        format!("Exit code: {:?}", result.exit_code)
                    } else {
                        result.stderr
                    };
                    error!("Job {:?} failed: {}", job_id, error_msg);
                    store.mark_failed(job_id, error_msg).await;
                }
            }
            Err(e) => {
                error!("Job {:?} execution error: {}", job_id, e);
                store.mark_failed(job_id, e).await;
            }
        }
    }

    info!("Event consumer stopped");
}
