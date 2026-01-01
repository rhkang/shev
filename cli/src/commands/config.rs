use clap::Subcommand;
use shev_core::api::{ConfigResponse, UpdateConfigRequest};

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

pub async fn execute(url: &str, action: ConfigAction) -> Result<(), String> {
    let client = reqwest::Client::new();

    match action {
        ConfigAction::Show => {
            let resp = client
                .get(format!("{}/config", url))
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let config: ConfigResponse = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

                println!("Configuration:");
                println!("  port: {}", config.port);
                println!("  queue_size: {}", config.queue_size);
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
        ConfigAction::Set { key, value } => {
            let request = match key.as_str() {
                "port" => {
                    let _: u16 = value
                        .parse()
                        .map_err(|_| format!("Invalid port: {}", value))?;
                    UpdateConfigRequest {
                        port: Some(value.clone()),
                        queue_size: None,
                    }
                }
                "queue_size" => {
                    let _: usize = value
                        .parse()
                        .map_err(|_| format!("Invalid queue_size: {}", value))?;
                    UpdateConfigRequest {
                        port: None,
                        queue_size: Some(value.clone()),
                    }
                }
                _ => {
                    return Err(format!(
                        "Unknown config key '{}'. Valid keys: port, queue_size",
                        key
                    ));
                }
            };

            let resp = client
                .put(format!("{}/config", url))
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                println!("Set {} = {}", key, value);
                println!();
                println!("Note: Restart the server for changes to take effect");
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
    }

    Ok(())
}
