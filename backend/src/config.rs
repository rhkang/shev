use std::net::IpAddr;
use std::path::PathBuf;

use clap::Parser;

pub const DEFAULT_DB_NAME: &str = "shev.db";

#[derive(Parser)]
#[command(name = "shev-backend", about = "Shell Event System backend server")]
pub struct Args {
    /// Listen on all interfaces (0.0.0.0) instead of localhost only
    #[arg(short, long)]
    pub listen: bool,

    /// Allowed IP addresses (can be specified multiple times). If not set, all IPs are allowed when --listen is used.
    #[arg(long = "allow")]
    pub allowed_ips: Vec<IpAddr>,
}

pub fn get_db_path() -> String {
    if let Ok(path) = std::env::var("SHEV_DB") {
        return path;
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let db_path: PathBuf = exe_dir.join(DEFAULT_DB_NAME);
            return db_path.to_string_lossy().to_string();
        }
    }

    DEFAULT_DB_NAME.to_string()
}
