mod commands;

use clap::{Parser, Subcommand};

use commands::{config, event, handler, job, schedule, timer};

const DEFAULT_URL: &str = "http://127.0.0.1:3000";

fn get_default_url() -> String {
    std::env::var("SHEV_URL").unwrap_or_else(|_| DEFAULT_URL.to_string())
}

#[derive(Parser)]
#[command(name = "shev")]
#[command(about = "Shell Event System CLI", long_about = None)]
struct Cli {
    /// Server URL (default: SHEV_URL env or http://127.0.0.1:3000)
    #[arg(long, short)]
    url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage event handlers
    Handler {
        #[command(subcommand)]
        action: handler::HandlerAction,
    },
    /// Manage timers
    Timer {
        #[command(subcommand)]
        action: timer::TimerAction,
    },
    /// Manage scheduled events
    Schedule {
        #[command(subcommand)]
        action: schedule::ScheduleAction,
    },
    /// Query jobs
    Job {
        #[command(subcommand)]
        action: job::JobAction,
    },
    /// Trigger events
    Event {
        #[command(subcommand)]
        action: event::EventAction,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: config::ConfigAction,
    },
    /// Reload handlers/timers/schedules in running server
    Reload,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let url = cli.url.unwrap_or_else(get_default_url);

    let result = match cli.command {
        Commands::Handler { action } => handler::execute(&url, action).await,
        Commands::Timer { action } => timer::execute(&url, action).await,
        Commands::Schedule { action } => schedule::execute(&url, action).await,
        Commands::Job { action } => job::execute(&url, action).await,
        Commands::Event { action } => event::execute(&url, action).await,
        Commands::Config { action } => config::execute(&url, action).await,
        Commands::Reload => reload(&url).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn reload(url: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/reload", url))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    if resp.status().is_success() {
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        println!("Reload successful:");
        println!("  Handlers loaded: {}", body["handlers_loaded"]);
        println!("  Timers loaded: {}", body["timers_loaded"]);
        println!("  Schedules loaded: {}", body["schedules_loaded"]);
        Ok(())
    } else {
        Err(format!("Server returned error: {}", resp.status()))
    }
}
