//! Lightweight decision insert binary for the decision-journal hook.
//!
//! Usage: `brain-db-insert <timestamp> <session_id> <tool> <classification> <target>`
//!
//! Designed to be called from bash hooks with minimal overhead.
//! Opens the database, inserts one row, and exits.

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 6 {
        eprintln!(
            "Usage: {} <timestamp> <session_id> <tool> <classification> <target>",
            args.first()
                .map(|s| s.as_str())
                .unwrap_or("brain-db-insert")
        );
        std::process::exit(1);
    }

    let timestamp = &args[1];
    let session_id = &args[2];
    let tool = &args[3];
    let classification = &args[4];
    let target = &args[5];

    let risk_level = match classification.as_str() {
        "dependency" | "infrastructure" | "ci" => "MEDIUM",
        "hook" | "configuration" | "mcp" => "MEDIUM",
        _ => "LOW",
    };

    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let db_path = PathBuf::from(home).join(".claude/brain/brain.db");

    let pool = match nexcore_db::pool::DbPool::open(&db_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("DB open failed: {e}");
            std::process::exit(1);
        }
    };

    let ts = nexcore_chrono::DateTime::parse_from_rfc3339(timestamp)
        .unwrap_or_else(|_| nexcore_chrono::DateTime::now());

    let result = pool.with_conn(|conn| {
        nexcore_db::decisions::insert(
            conn,
            &nexcore_db::decisions::DecisionRow {
                id: None,
                timestamp: ts,
                session_id: session_id.to_string(),
                tool: tool.to_string(),
                action: classification.to_string(),
                target: target.to_string(),
                risk_level: risk_level.to_string(),
                reversible: true,
            },
        )
    });

    if let Err(e) = result {
        eprintln!("Insert failed: {e}");
        std::process::exit(1);
    }
}
