use std::collections::HashMap;

use clap::Subcommand;

use shev_core::{Database, ShellType};

#[derive(Subcommand)]
pub enum HandlerAction {
    /// Add a new handler
    Add {
        /// Event type name
        event_type: String,
        /// Shell to use (pwsh, bash, sh)
        #[arg(long, short)]
        shell: String,
        /// Command to execute
        #[arg(long, short)]
        command: String,
        /// Timeout in seconds
        #[arg(long, short)]
        timeout: Option<u64>,
    },
    /// Update an existing handler (generates new UUID)
    Update {
        /// Event type name
        event_type: String,
        /// Shell to use (pwsh, bash, sh)
        #[arg(long, short)]
        shell: Option<String>,
        /// Command to execute
        #[arg(long, short)]
        command: Option<String>,
        /// Timeout in seconds
        #[arg(long, short)]
        timeout: Option<u64>,
        /// Set environment variable (can be used multiple times): KEY=VALUE
        #[arg(long, short)]
        env: Option<Vec<String>>,
        /// Clear all environment variables
        #[arg(long)]
        clear_env: bool,
    },
    /// Remove a handler
    Remove {
        /// Event type name
        event_type: String,
    },
    /// List all handlers
    List,
    /// Show details of a handler
    Show {
        /// Event type name
        event_type: String,
    },
}

pub fn execute(db_path: &str, action: HandlerAction) -> Result<(), String> {
    let db = Database::open(db_path)?;
    db.init_schema()?;

    match action {
        HandlerAction::Add {
            event_type,
            shell,
            command,
            timeout,
        } => {
            let shell = parse_shell(&shell)?;
            let env: HashMap<String, String> = HashMap::new();
            let handler = db.insert_handler(&event_type, &shell, &command, timeout, &env)?;
            println!("Handler added:");
            println!("  ID: {}", handler.id);
            println!("  Event type: {}", handler.event_type);
            println!("  Shell: {}", handler.shell.as_str());
            println!("  Command: {}", handler.command);
            if let Some(t) = handler.timeout {
                println!("  Timeout: {}s", t);
            }
        }
        HandlerAction::Update {
            event_type,
            shell,
            command,
            timeout,
            env,
            clear_env,
        } => {
            let shell = shell.map(|s| parse_shell(&s)).transpose()?;

            let env_map = if clear_env {
                Some(HashMap::new())
            } else if let Some(env_vars) = env {
                let existing = db
                    .get_handler(&event_type)?
                    .map(|h| h.env)
                    .unwrap_or_default();
                let mut new_env = existing;
                for var in env_vars {
                    if let Some((key, value)) = var.split_once('=') {
                        new_env.insert(key.to_string(), value.to_string());
                    } else {
                        return Err(format!("Invalid env format '{}', use KEY=VALUE", var));
                    }
                }
                Some(new_env)
            } else {
                None
            };

            let handler = db.update_handler(
                &event_type,
                shell.as_ref(),
                command.as_deref(),
                timeout.map(Some),
                env_map.as_ref(),
            )?;
            println!("Handler updated (new UUID generated):");
            println!("  ID: {}", handler.id);
            println!("  Event type: {}", handler.event_type);
            println!("  Shell: {}", handler.shell.as_str());
            println!("  Command: {}", handler.command);
            if let Some(t) = handler.timeout {
                println!("  Timeout: {}s", t);
            }
            if !handler.env.is_empty() {
                println!("  Environment:");
                for (k, v) in &handler.env {
                    println!("    {}={}", k, v);
                }
            }
        }
        HandlerAction::Remove { event_type } => {
            if db.delete_handler(&event_type)? {
                println!("Handler '{}' removed", event_type);
            } else {
                println!("Handler '{}' not found", event_type);
            }
        }
        HandlerAction::List => {
            let handlers = db.get_all_handlers()?;
            if handlers.is_empty() {
                println!("No handlers configured");
            } else {
                println!(
                    "{:<20} {:<8} {:<10} {}",
                    "EVENT_TYPE", "SHELL", "TIMEOUT", "ID"
                );
                println!("{}", "-".repeat(70));
                for h in handlers {
                    let timeout = h
                        .timeout
                        .map(|t| format!("{}s", t))
                        .unwrap_or_else(|| "-".to_string());
                    println!(
                        "{:<20} {:<8} {:<10} {}",
                        h.event_type,
                        h.shell.as_str(),
                        timeout,
                        h.id
                    );
                }
            }
        }
        HandlerAction::Show { event_type } => {
            if let Some(h) = db.get_handler(&event_type)? {
                println!("Handler: {}", h.event_type);
                println!("  ID: {}", h.id);
                println!("  Shell: {}", h.shell.as_str());
                println!("  Command: {}", h.command);
                if let Some(t) = h.timeout {
                    println!("  Timeout: {}s", t);
                }
                if !h.env.is_empty() {
                    println!("  Environment:");
                    for (k, v) in &h.env {
                        println!("    {}={}", k, v);
                    }
                }
            } else {
                println!("Handler '{}' not found", event_type);
            }
        }
    }

    Ok(())
}

fn parse_shell(shell: &str) -> Result<ShellType, String> {
    ShellType::from_str(shell)
        .ok_or_else(|| format!("Invalid shell '{}'. Use: pwsh, bash, or sh", shell))
}
