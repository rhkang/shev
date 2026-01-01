use tracing::{error, info, warn};

use crate::db::JobStatus;
use crate::executor::execute_command;
use crate::queue::EventReceiver;
use crate::store::JobStore;

pub async fn start_consumer(mut receiver: EventReceiver, store: JobStore) {
    info!("Event consumer started");

    while let Some(event) = receiver.recv().await {
        info!(
            "Processing event: {:?} (type: {})",
            event.id, event.event_type
        );

        let handler = match store.get_handler(&event.event_type).await {
            Some(h) => h,
            None => {
                warn!("No handler for event type: {}", event.event_type);
                continue;
            }
        };

        let job = store.create_job(event.clone(), &handler).await;
        let job_id = job.id;

        info!("Created job: {:?} (handler: {:?})", job_id, handler.id);

        let store = store.clone();
        tokio::spawn(async move {
            if let Some(j) = store.get_job(job_id).await {
                if j.status == JobStatus::Cancelled {
                    info!("Job {:?} was cancelled before execution", job_id);
                    return;
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
                        error!("Job {:?} failed", job_id);
                        store.mark_failed(job_id, error_msg).await;
                    }
                }
                Err(e) => {
                    error!("Job {:?} execution error: {}", job_id, e);
                    store.mark_failed(job_id, e).await;
                }
            }
        });
    }

    info!("Event consumer stopped");
}
