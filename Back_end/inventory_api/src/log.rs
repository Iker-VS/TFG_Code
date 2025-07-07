use chrono::Local;
use once_cell::sync::Lazy;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

static LOG_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub fn write_log(message: &str) -> std::io::Result<()> {
    let now = Local::now();
    let today = now.format("%Y-%m-%d").to_string();
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("logs");
    let log_file = base_dir.join(format!("{}.log", today));
    if !base_dir.exists() {
        create_dir_all(&base_dir)?;
    }
    let timestamp = now.format("[%Y-%m-%d %H:%M:%S]");
    // Solo la apertura y escritura del archivo se protege con el mutex
    let _lock = LOG_MUTEX.lock().unwrap();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)?;
    writeln!(file, "{} {}", timestamp, message)?;
    Ok(())
}
