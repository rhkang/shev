use clap::Subcommand;
use uuid::Uuid;

use shev_core::{Database, JobStatus};

#[derive(Subcommand)]
pub enum JobAction {
    /// List jobs
    List {
        /// Filter by status (pending, running, completed, failed, cancelled)
        #[arg(long, short)]
        status: Option<String>,
        /// Maximum number of jobs to show
        #[arg(long, short, default_value = "50")]
        limit: usize,
    },
    /// Show details of a job
    Show {
        /// Job ID
        job_id: String,
    },
}

pub fn execute(db_path: &str, action: JobAction) -> Result<(), String> {
    let db = Database::open(db_path)?;
    db.init_schema()?;

    match action {
        JobAction::List { status, limit } => {
            let status = status.map(|s| parse_status(&s)).transpose()?;
            let jobs = db.get_all_jobs(status.as_ref(), limit)?;

            if jobs.is_empty() {
                println!("No jobs found");
            } else {
                println!(
                    "{:<36} {:<15} {:<12} {}",
                    "JOB_ID", "EVENT_TYPE", "STATUS", "TIMESTAMP"
                );
                println!("{}", "-".repeat(90));
                for j in jobs {
                    let timestamp = j.event.timestamp.format("%Y-%m-%d %H:%M").to_string();
                    println!(
                        "{:<36} {:<15} {:<12} {}",
                        j.id,
                        truncate(&j.event.event_type, 15),
                        j.status.as_str(),
                        timestamp
                    );
                }
            }
        }
        JobAction::Show { job_id } => {
            let uuid =
                Uuid::parse_str(&job_id).map_err(|_| format!("Invalid job ID: {}", job_id))?;

            if let Some(j) = db.get_job(uuid)? {
                // Check if handler is still current
                let current_handler_id = db.get_handler_id(&j.event.event_type)?;
                let handler_current = current_handler_id == Some(j.handler_id);

                println!("Job: {}", j.id);
                println!("  Status: {}", j.status.as_str());
                println!("  Event type: {}", j.event.event_type);
                println!("  Event ID: {}", j.event.id);
                println!(
                    "  Handler ID: {}{}",
                    j.handler_id,
                    if handler_current { "" } else { " (outdated)" }
                );
                println!("  Timestamp: {}", j.event.timestamp.to_rfc3339());
                if !j.event.context.is_empty() {
                    println!("  Context: {}", j.event.context);
                }
                if let Some(ref started) = j.started_at {
                    println!("  Started: {}", started.to_rfc3339());
                }
                if let Some(ref finished) = j.finished_at {
                    println!("  Finished: {}", finished.to_rfc3339());
                }
                if let Some(ref output) = j.output {
                    println!("  Output:");
                    for line in output.lines().take(20) {
                        println!("    {}", line);
                    }
                    if output.lines().count() > 20 {
                        println!("    ... (truncated)");
                    }
                }
                if let Some(ref error) = j.error {
                    println!("  Error:");
                    for line in error.lines().take(10) {
                        println!("    {}", line);
                    }
                }
            } else {
                println!("Job '{}' not found", job_id);
            }
        }
    }

    Ok(())
}

fn parse_status(status: &str) -> Result<JobStatus, String> {
    JobStatus::from_str(status).ok_or_else(|| {
        format!(
            "Invalid status '{}'. Use: pending, running, completed, failed, or cancelled",
            status
        )
    })
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}
