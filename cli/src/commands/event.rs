use clap::Subcommand;
use shev_core::api::{TriggerEventRequest, TriggerEventResponse};

#[derive(Subcommand)]
pub enum EventAction {
    /// Trigger an event
    Trigger {
        /// Event type name
        event_type: String,
        /// Context to pass to handler
        #[arg(long, short, default_value = "")]
        context: String,
    },
}

pub async fn execute(url: &str, action: EventAction) -> Result<(), String> {
    match action {
        EventAction::Trigger {
            event_type,
            context,
        } => {
            let client = reqwest::Client::new();
            let request = TriggerEventRequest {
                event_type: event_type.clone(),
                context,
            };

            let resp = client
                .post(format!("{}/events", url))
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let body: TriggerEventResponse = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

                if body.triggered {
                    println!("Event '{}' triggered successfully", event_type);
                    println!("  {}", body.message);
                } else {
                    println!("Event '{}' was not triggered", event_type);
                    println!("  {}", body.message);
                }
            } else {
                return Err(format!("Server returned error: {}", resp.status()));
            }
        }
    }

    Ok(())
}
