use chrono::{DateTime, Utc};
use clap::Subcommand;
use serde::Deserialize;

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
        /// Maximum output lines to show (0 for no limit)
        #[arg(long, short = 'n', default_value = "20")]
        max_lines: usize,
    },
    /// Cancel a pending or running job
    Cancel {
        /// Job ID
        job_id: String,
    },
}

#[derive(Deserialize)]
struct Event {
    id: String,
    event_type: String,
    context: String,
    timestamp: DateTime<Utc>,
}

#[derive(Deserialize)]
struct JobResponse {
    id: String,
    event: Event,
    handler_id: String,
    status: String,
    output: Option<String>,
    error: Option<String>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}

fn print_lines(text: &str, max_lines: usize) {
    let lines: Vec<&str> = text.lines().collect();
    let total = lines.len();

    if max_lines == 0 {
        for line in lines {
            println!("    {}", line);
        }
    } else {
        for line in lines.iter().take(max_lines) {
            println!("    {}", line);
        }
        if total > max_lines {
            println!("    ... ({} more lines)", total - max_lines);
        }
    }
}

pub async fn execute(url: &str, action: JobAction) -> Result<(), String> {
    let client = reqwest::Client::new();

    match action {
        JobAction::List { status, limit } => {
            let mut request_url = format!("{}/jobs", url);
            if let Some(ref s) = status {
                request_url = format!("{}?status={}", request_url, s);
            }

            let resp = client
                .get(&request_url)
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let jobs: Vec<JobResponse> = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

                if jobs.is_empty() {
                    println!("No jobs found");
                } else {
                    println!(
                        "{:<36} {:<15} {:<12} {}",
                        "JOB_ID", "EVENT_TYPE", "STATUS", "TIMESTAMP"
                    );
                    println!("{}", "-".repeat(90));
                    for j in jobs.iter().take(limit) {
                        let timestamp = j.event.timestamp.format("%Y-%m-%d %H:%M").to_string();
                        println!(
                            "{:<36} {:<15} {:<12} {}",
                            j.id,
                            truncate(&j.event.event_type, 15),
                            j.status,
                            timestamp
                        );
                    }
                }
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
        JobAction::Show { job_id, max_lines } => {
            let resp = client
                .get(format!("{}/jobs/{}", url, job_id))
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                let j: JobResponse = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

                println!("Job: {}", j.id);
                println!("  Status: {}", j.status);
                println!("  Event type: {}", j.event.event_type);
                println!("  Event ID: {}", j.event.id);
                println!("  Handler ID: {}", j.handler_id);
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
                    print_lines(output, max_lines);
                }
                if let Some(ref error) = j.error {
                    println!("  Error:");
                    print_lines(error, max_lines);
                }
            } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
                println!("Job '{}' not found", job_id);
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
        JobAction::Cancel { job_id } => {
            let resp = client
                .post(format!("{}/jobs/{}/cancel", url, job_id))
                .send()
                .await
                .map_err(|e| format!("Failed to connect to server: {}", e))?;

            if resp.status().is_success() {
                println!("Job '{}' cancelled", job_id);
            } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
                println!("Job '{}' not found", job_id);
            } else if resp.status() == reqwest::StatusCode::BAD_REQUEST {
                println!(
                    "Job '{}' not cancellable (only pending/running jobs can be cancelled)",
                    job_id
                );
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Server returned error {}: {}", status, body));
            }
        }
    }

    Ok(())
}
