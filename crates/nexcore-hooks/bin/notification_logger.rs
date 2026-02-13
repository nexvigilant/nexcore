//! Notification hook: Logs all notifications for debugging and auditing
//!
//! Creates a session log of all Claude Code notifications.
//!
//! Exit codes:
//! - 0: Always (logging is passive)

use nexcore_hooks::{HookInput, read_input};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let input: HookInput = match read_input() {
        Some(i) => i,
        None => std::process::exit(0),
    };

    let notification_type = input.notification_type.as_deref().unwrap_or("unknown");
    let message = input.message.as_deref().unwrap_or("");

    // Log to session-specific file
    log_notification(&input.session_id, notification_type, message);

    std::process::exit(0);
}

fn log_notification(session_id: &str, notification_type: &str, message: &str) {
    let log_dir = notification_log_path();

    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("Warning: Could not create log dir: {}", e);
        return;
    }

    // Use session-specific log file
    let log_file = log_dir.join(format!("{}.jsonl", &session_id[..8.min(session_id.len())]));

    let file = OpenOptions::new().create(true).append(true).open(&log_file);

    if let Ok(mut file) = file {
        let entry = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "type": notification_type,
            "message": message.chars().take(500).collect::<String>()
        });
        if let Err(e) = writeln!(file, "{}", entry) {
            eprintln!("Warning: Could not log notification: {}", e);
        }
    }
}

fn notification_log_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".claude")
        .join("logs")
        .join("notifications")
}
