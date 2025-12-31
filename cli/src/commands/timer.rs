use clap::Subcommand;

use shev_core::Database;

#[derive(Subcommand)]
pub enum TimerAction {
    /// Add a new timer
    Add {
        /// Event type name
        event_type: String,
        /// Interval in seconds
        #[arg(long, short)]
        interval: u64,
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
        interval: Option<u64>,
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

pub fn execute(db_path: &str, action: TimerAction) -> Result<(), String> {
    let db = Database::open(db_path)?;
    db.init_schema()?;

    match action {
        TimerAction::Add {
            event_type,
            interval,
            context,
        } => {
            let timer = db.insert_timer(&event_type, interval, &context)?;
            println!("Timer added:");
            println!("  ID: {}", timer.id);
            println!("  Event type: {}", timer.event_type);
            println!("  Interval: {}s", timer.interval_secs);
            if !timer.context.is_empty() {
                println!("  Context: {}", timer.context);
            }
        }
        TimerAction::Update {
            event_type,
            interval,
            context,
        } => {
            let timer = db.update_timer(&event_type, interval, context.as_deref())?;
            println!("Timer updated (new UUID generated):");
            println!("  ID: {}", timer.id);
            println!("  Event type: {}", timer.event_type);
            println!("  Interval: {}s", timer.interval_secs);
            if !timer.context.is_empty() {
                println!("  Context: {}", timer.context);
            }
        }
        TimerAction::Remove { event_type } => {
            if db.delete_timer(&event_type)? {
                println!("Timer '{}' removed", event_type);
            } else {
                println!("Timer '{}' not found", event_type);
            }
        }
        TimerAction::List => {
            let timers = db.get_all_timers()?;
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
        }
        TimerAction::Show { event_type } => {
            if let Some(t) = db.get_timer(&event_type)? {
                println!("Timer: {}", t.event_type);
                println!("  ID: {}", t.id);
                println!("  Interval: {}s", t.interval_secs);
                if !t.context.is_empty() {
                    println!("  Context: {}", t.context);
                }
            } else {
                println!("Timer '{}' not found", event_type);
            }
        }
    }

    Ok(())
}
