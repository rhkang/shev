use std::collections::HashMap;

use clap::Subcommand;
use shev_core::api::{CreateHandlerRequest, HandlerResponse, UpdateHandlerRequest};

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
        timeout: Option<u32>,
        /// Set environment variable (can be used multiple times): KEY=VALUE
        #[arg(long, short)]
        env: Option<Vec<String>>,
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
        timeout: Option<u32>,
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

fn parse_env_vars(env: Option<Vec<String>>) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    if let Some(env_vars) = env {
        for var in env_vars {
            if let Some((key, value)) = var.split_once('=') {
                map.insert(key.to_string(), value.to_string());
            } else {
                return Err(format!("Invalid env format '{}', use KEY=VALUE", var));
            }
        }
    }
    Ok(map)
}

fn print_handler(handler: &HandlerResponse) {
    println!("  ID: {}", handler.id);
    println!("  Event type: {}", handler.event_type);
    println!("  Shell: {}", handler.shell);
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

pub async fn execute(url: &str, action: HandlerAction) -> Result<(), String> {
    let client = reqwest::Client::new();

    match action {
        HandlerAction::Add {
            event_type,
            shell,
            command,
            timeout,
            env,
        } => {
            let env_map = parse_env_vars(env)?;
            let request = CreateHandlerRequest {
                event_type,
                shell,
                command,
                timeout,
                env: env_map,
            };

            let resp = client
                .post(format!("{}/handlers", url))
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let handler: HandlerResponse = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;
                println!("Handler added:");
                print_handler(&handler);
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
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
            let env_map = if clear_env {
                Some(HashMap::new())
            } else if env.is_some() {
                // Fetch existing handler to merge env vars
                let existing = client
                    .get(format!("{}/handlers", url))
                    .send()
                    .await
                    .map_err(|e| format!("Failed to connect to server: {}", e))?;

                if existing.status().is_success() {
                    let handlers: Vec<HandlerResponse> = existing
                        .json()
                        .await
                        .map_err(|e| format!("Failed to parse response: {}", e))?;

                    let current_env = handlers
                        .iter()
                        .find(|h| h.event_type == event_type)
                        .map(|h| h.env.clone())
                        .unwrap_or_default();

                    let mut new_env = current_env;
                    for (k, v) in parse_env_vars(env)? {
                        new_env.insert(k, v);
                    }
                    Some(new_env)
                } else {
                    Some(parse_env_vars(env)?)
                }
            } else {
                None
            };

            let request = UpdateHandlerRequest {
                shell,
                command,
                timeout: timeout.map(Some),
                env: env_map,
            };

            let resp = client
                .put(format!("{}/handlers/{}", url, event_type))
                .json(&request)
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let handler: HandlerResponse = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;
                println!("Handler updated (new UUID generated):");
                print_handler(&handler);
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
        HandlerAction::Remove { event_type } => {
            let resp = client
                .delete(format!("{}/handlers/{}", url, event_type))
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                println!("Handler '{}' removed", event_type);
            } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
                println!("Handler '{}' not found", event_type);
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
        HandlerAction::List => {
            let resp = client
                .get(format!("{}/handlers", url))
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let handlers: Vec<HandlerResponse> = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

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
                            h.event_type, h.shell, timeout, h.id
                        );
                    }
                }
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
        HandlerAction::Show { event_type } => {
            let resp = client
                .get(format!("{}/handlers", url))
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let handlers: Vec<HandlerResponse> = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

                if let Some(h) = handlers.iter().find(|h| h.event_type == event_type) {
                    println!("Handler: {}", h.event_type);
                    print_handler(h);
                } else {
                    println!("Handler '{}' not found", event_type);
                }
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
    }

    Ok(())
}
