use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;
use tokio::time::timeout;

use crate::models::EventHandler;

#[derive(Debug)]
pub struct ExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

pub async fn execute_command(
    handler: &EventHandler,
    event_context: &str,
) -> Result<ExecutionResult, String> {
    let (shell_cmd, args) = handler.shell.command_args(&handler.command);

    let mut cmd = Command::new(shell_cmd);
    cmd.args(&args)
        .env("EVENT_CONTEXT", event_context)
        .envs(&handler.env)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let future = async {
        let child = cmd.spawn().map_err(|e| format!("Failed to spawn process: {}", e))?;
        child
            .wait_with_output()
            .await
            .map_err(|e| format!("Failed to wait for process: {}", e))
    };

    let output = if let Some(timeout_secs) = handler.timeout {
        match timeout(Duration::from_secs(timeout_secs), future).await {
            Ok(result) => result?,
            Err(_) => {
                return Err(format!(
                    "Command timed out after {} seconds",
                    timeout_secs
                ))
            }
        }
    } else {
        future.await?
    };

    Ok(ExecutionResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code(),
    })
}
