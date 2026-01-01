use chrono::{DateTime, Utc};
use clap::Subcommand;
use serde::{Deserialize, Serialize};

#[derive(Subcommand)]
pub enum ScheduleAction {
    /// Add a new scheduled event
    Add {
        /// Event type name
        event_type: String,
        /// Scheduled time in RFC3339/ISO8601 format (e.g., 2025-01-15T14:30:00Z)
        #[arg(long, short)]
        time: String,
        /// Context to pass to handler
        #[arg(long, short, default_value = "")]
        context: String,
        /// Run periodically (daily at the same time)
        #[arg(long, short)]
        periodic: bool,
    },
    /// Update an existing schedule (generates new UUID)
    Update {
        /// Event type name
        event_type: String,
        /// Scheduled time in RFC3339/ISO8601 format
        #[arg(long, short)]
        time: Option<String>,
        /// Context to pass to handler
        #[arg(long, short)]
        context: Option<String>,
        /// Run periodically (daily at the same time)
        #[arg(long, short)]
        periodic: Option<bool>,
    },
    /// Remove a schedule
    Remove {
        /// Event type name
        event_type: String,
    },
    /// List all schedules
    List,
    /// Show details of a schedule
    Show {
        /// Event type name
        event_type: String,
    },
}

fn parse_time(time_str: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(time_str)
        .map(|t| t.with_timezone(&Utc))
        .map_err(|e| {
            format!(
                "Invalid time format '{}': {}. Use RFC3339 format like 2025-01-15T14:30:00Z",
                time_str, e
            )
        })
}

#[derive(Serialize)]
struct CreateScheduleRequest {
    event_type: String,
    scheduled_time: DateTime<Utc>,
    context: String,
    periodic: bool,
}

#[derive(Serialize)]
struct UpdateScheduleRequest {
    scheduled_time: Option<DateTime<Utc>>,
    context: Option<String>,
    periodic: Option<bool>,
}

#[derive(Deserialize)]
struct ScheduleResponse {
    id: String,
    event_type: String,
    scheduled_time: DateTime<Utc>,
    context: String,
    periodic: bool,
}

fn print_schedule(schedule: &ScheduleResponse) {
    println!("  ID: {}", schedule.id);
    println!("  Event type: {}", schedule.event_type);
    println!("  Scheduled time: {}", schedule.scheduled_time);
    println!(
        "  Periodic: {}",
        if schedule.periodic {
            "yes (daily)"
        } else {
            "no (one-shot)"
        }
    );
    if !schedule.context.is_empty() {
        println!("  Context: {}", schedule.context);
    }
}

pub async fn execute(url: &str, action: ScheduleAction) -> Result<(), String> {
    let client = reqwest::Client::new();

    match action {
        ScheduleAction::Add {
            event_type,
            time,
            context,
            periodic,
        } => {
            let scheduled_time = parse_time(&time)?;
            let request = CreateScheduleRequest {
                event_type: event_type.clone(),
                scheduled_time,
                context,
                periodic,
            };

            let resp = client
                .post(format!("{}/schedules", url))
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let schedule: ScheduleResponse = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;
                println!("Schedule added:");
                print_schedule(&schedule);
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
        ScheduleAction::Update {
            event_type,
            time,
            context,
            periodic,
        } => {
            let scheduled_time = time.map(|t| parse_time(&t)).transpose()?;
            let request = UpdateScheduleRequest {
                scheduled_time,
                context,
                periodic,
            };

            let resp = client
                .put(format!("{}/schedules/{}", url, event_type))
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let schedule: ScheduleResponse = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;
                println!("Schedule updated (new UUID generated):");
                print_schedule(&schedule);
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
        ScheduleAction::Remove { event_type } => {
            let resp = client
                .delete(format!("{}/schedules/{}", url, event_type))
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                println!("Schedule '{}' removed", event_type);
            } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
                println!("Schedule '{}' not found", event_type);
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
        ScheduleAction::List => {
            let resp = client
                .get(format!("{}/schedules", url))
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let schedules: Vec<ScheduleResponse> = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

                if schedules.is_empty() {
                    println!("No schedules configured");
                } else {
                    println!(
                        "{:<20} {:<26} {:<10} {:<15} {}",
                        "EVENT_TYPE", "SCHEDULED_TIME", "PERIODIC", "CONTEXT", "ID"
                    );
                    println!("{}", "-".repeat(110));
                    for s in schedules {
                        let context = if s.context.is_empty() {
                            "-".to_string()
                        } else if s.context.len() > 13 {
                            format!("{}...", &s.context[..10])
                        } else {
                            s.context.clone()
                        };
                        let periodic = if s.periodic { "daily" } else { "one-shot" };
                        println!(
                            "{:<20} {:<26} {:<10} {:<15} {}",
                            s.event_type,
                            s.scheduled_time.format("%Y-%m-%dT%H:%M:%SZ"),
                            periodic,
                            context,
                            s.id
                        );
                    }
                }
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
        ScheduleAction::Show { event_type } => {
            let resp = client
                .get(format!("{}/schedules/{}", url, event_type))
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let schedule: ScheduleResponse = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;
                println!("Schedule: {}", schedule.event_type);
                print_schedule(&schedule);
            } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
                println!("Schedule '{}' not found", event_type);
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
    }

    Ok(())
}
