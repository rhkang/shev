use clap::Subcommand;
use shev_core::api::{CreateTimerRequest, TimerResponse, UpdateTimerRequest};

#[derive(Subcommand)]
pub enum TimerAction {
    /// Add a new timer
    Add {
        /// Event type name
        event_type: String,
        /// Interval in seconds
        #[arg(long, short)]
        interval: u32,
        /// Context to pass to handler
        #[arg(long, short, default_value = "")]
        context: String,
    },
    /// Update an existing timer (generates new UUID)
    Update {
        /// Event type name
        event_type: String,
        /// Interval in seconds
        #[arg(long, short)]
        interval: Option<u32>,
        /// Context to pass to handler
        #[arg(long, short)]
        context: Option<String>,
    },
    /// Remove a timer
    Remove {
        /// Event type name
        event_type: String,
    },
    /// List all timers
    List,
    /// Show details of a timer
    Show {
        /// Event type name
        event_type: String,
    },
}

fn print_timer(timer: &TimerResponse) {
    println!("  ID: {}", timer.id);
    println!("  Event type: {}", timer.event_type);
    println!("  Interval: {}s", timer.interval_secs);
    if !timer.context.is_empty() {
        println!("  Context: {}", timer.context);
    }
}

pub async fn execute(url: &str, action: TimerAction) -> Result<(), String> {
    let client = reqwest::Client::new();

    match action {
        TimerAction::Add {
            event_type,
            interval,
            context,
        } => {
            let request = CreateTimerRequest {
                event_type: event_type.clone(),
                interval_secs: interval,
                context,
            };

            let resp = client
                .post(format!("{}/timers", url))
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let timer: TimerResponse = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;
                println!("Timer added:");
                print_timer(&timer);
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
        TimerAction::Update {
            event_type,
            interval,
            context,
        } => {
            let request = UpdateTimerRequest {
                interval_secs: interval,
                context,
            };

            let resp = client
                .put(format!("{}/timers/{}", url, event_type))
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let timer: TimerResponse = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;
                println!("Timer updated (new UUID generated):");
                print_timer(&timer);
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
        TimerAction::Remove { event_type } => {
            let resp = client
                .delete(format!("{}/timers/{}", url, event_type))
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                println!("Timer '{}' removed", event_type);
            } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
                println!("Timer '{}' not found", event_type);
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
        TimerAction::List => {
            let resp = client
                .get(format!("{}/timers", url))
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let timers: Vec<TimerResponse> = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

                if timers.is_empty() {
                    println!("No timers configured");
                } else {
                    println!(
                        "{:<20} {:<12} {:<20} {}",
                        "EVENT_TYPE", "INTERVAL", "CONTEXT", "ID"
                    );
                    println!("{}", "-".repeat(80));
                    for t in timers {
                        let context = if t.context.is_empty() {
                            "-".to_string()
                        } else if t.context.len() > 18 {
                            format!("{}...", &t.context[..15])
                        } else {
                            t.context.clone()
                        };
                        println!(
                            "{:<20} {:<12} {:<20} {}",
                            t.event_type,
                            format!("{}s", t.interval_secs),
                            context,
                            t.id
                        );
                    }
                }
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
        TimerAction::Show { event_type } => {
            let resp = client
                .get(format!("{}/timers/{}", url, event_type))
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let timer: TimerResponse = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;
                println!("Timer: {}", timer.event_type);
                print_timer(&timer);
            } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
                println!("Timer '{}' not found", event_type);
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
    }

    Ok(())
}
