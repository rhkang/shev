use chrono::{DateTime, Utc};
use clap::Subcommand;

use shev_core::Database;

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

pub fn execute(db_path: &str, action: ScheduleAction) -> Result<(), String> {
    let db = Database::open(db_path)?;
    db.init_schema()?;

    match action {
        ScheduleAction::Add {
            event_type,
            time,
            context,
            periodic,
        } => {
            let scheduled_time = parse_time(&time)?;
            let schedule = db.insert_schedule(&event_type, scheduled_time, &context, periodic)?;
            println!("Schedule added:");
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
        ScheduleAction::Update {
            event_type,
            time,
            context,
            periodic,
        } => {
            let scheduled_time = time.map(|t| parse_time(&t)).transpose()?;
            let schedule =
                db.update_schedule(&event_type, scheduled_time, context.as_deref(), periodic)?;
            println!("Schedule updated (new UUID generated):");
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
        ScheduleAction::Remove { event_type } => {
            if db.delete_schedule(&event_type)? {
                println!("Schedule '{}' removed", event_type);
            } else {
                println!("Schedule '{}' not found", event_type);
            }
        }
        ScheduleAction::List => {
            let schedules = db.get_all_schedules()?;
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
        }
        ScheduleAction::Show { event_type } => {
            if let Some(s) = db.get_schedule(&event_type)? {
                println!("Schedule: {}", s.event_type);
                println!("  ID: {}", s.id);
                println!("  Scheduled time: {}", s.scheduled_time);
                println!(
                    "  Periodic: {}",
                    if s.periodic {
                        "yes (daily)"
                    } else {
                        "no (one-shot)"
                    }
                );
                if !s.context.is_empty() {
                    println!("  Context: {}", s.context);
                }
            } else {
                println!("Schedule '{}' not found", event_type);
            }
        }
    }

    Ok(())
}
