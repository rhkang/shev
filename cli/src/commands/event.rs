use clap::Subcommand;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
struct EventRequest {
    event_type: String,
    context: String,
}

#[derive(Deserialize)]
struct EventResponse {
    triggered: bool,
    message: String,
}

pub async fn execute(url: &str, action: EventAction) -> Result<(), String> {
    match action {
        EventAction::Trigger {
            event_type,
            context,
        } => {
            let client = reqwest::Client::new();
            let request = EventRequest {
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
                let body: EventResponse = resp
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
