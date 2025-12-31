use clap::Subcommand;

use shev_core::Database;

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        /// Configuration key (port, queue_size)
        key: String,
        /// Configuration value
        value: String,
    },
}

pub fn execute(db_path: &str, action: ConfigAction) -> Result<(), String> {
    let db = Database::open(db_path)?;
    db.init_schema()?;

    match action {
        ConfigAction::Show => {
            let port = db.get_config("port").unwrap_or_else(|| "3000".to_string());
            let queue_size = db
                .get_config("queue_size")
                .unwrap_or_else(|| "100".to_string());

            println!("Configuration:");
            println!("  port: {}", port);
            println!("  queue_size: {}", queue_size);
            println!();
            println!("Database: {}", db_path);
        }
        ConfigAction::Set { key, value } => {
            match key.as_str() {
                "port" => {
                    let port: u16 = value
                        .parse()
                        .map_err(|_| format!("Invalid port: {}", value))?;
                    if port == 0 {
                        return Err("Port cannot be 0".to_string());
                    }
                    db.set_config("port", &value)?;
                    println!("Set port = {}", port);
                }
                "queue_size" => {
                    let size: usize = value
                        .parse()
                        .map_err(|_| format!("Invalid queue_size: {}", value))?;
                    if size == 0 {
                        return Err("Queue size cannot be 0".to_string());
                    }
                    db.set_config("queue_size", &value)?;
                    println!("Set queue_size = {}", size);
                }
                _ => {
                    return Err(format!(
                        "Unknown config key '{}'. Valid keys: port, queue_size",
                        key
                    ));
                }
            }
            println!();
            println!("Note: Restart the server for changes to take effect");
        }
    }

    Ok(())
}
