use std::path::PathBuf;

pub const DEFAULT_DB_NAME: &str = "shev.db";

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
