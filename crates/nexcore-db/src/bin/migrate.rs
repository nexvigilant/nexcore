//! One-shot migration binary: imports JSON Brain data into SQLite.
//!
//! Usage: `cargo run -p nexcore-db --bin brain-migrate`
//!
//! Reads from:
//!   - `~/.claude/brain/` (sessions, artifacts)
//!   - `~/.claude/implicit/` (preferences, patterns, corrections, beliefs, trust)
//!   - `~/.claude/code_tracker/` (tracked files)
//!
//! Writes to:
//!   - `~/.claude/brain/brain.db` (SQLite)

use std::path::PathBuf;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let home = PathBuf::from(home);

    let claude_dir = home.join(".claude");
    let brain_dir = claude_dir.join("brain");
    let implicit_dir = claude_dir.join("implicit");
    let tracker_dir = claude_dir.join("code_tracker");
    let db_path = brain_dir.join("brain.db");

    eprintln!("=== Brain JSON -> SQLite Migration (V1 + V2) ===");
    eprintln!("Claude dir:   {}", claude_dir.display());
    eprintln!("Database:     {}", db_path.display());
    eprintln!();

    // Open (or create) the database — schema auto-migrates
    let pool = match nexcore_db::pool::DbPool::open(&db_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ERROR: Failed to open database: {e}");
            std::process::exit(1);
        }
    };

    // Run V1 migration (sessions, artifacts, implicit, tracker)
    let v1_result = pool
        .with_conn(|conn| nexcore_db::migrate::run(conn, &brain_dir, &implicit_dir, &tracker_dir));

    match v1_result {
        Ok(stats) => {
            eprintln!("V1 Migration (Brain data):");
            eprintln!("  Sessions:     {}", stats.sessions);
            eprintln!("  Artifacts:    {}", stats.artifacts);
            eprintln!("  Versions:     {}", stats.versions);
            eprintln!("  Tracked:      {}", stats.tracked_files);
            eprintln!("  Preferences:  {}", stats.preferences);
            eprintln!("  Patterns:     {}", stats.patterns);
            eprintln!("  Corrections:  {}", stats.corrections);
            eprintln!("  Beliefs:      {}", stats.beliefs);
            eprintln!("  Trust:        {}", stats.trust_accumulators);
            eprintln!("  Implications: {}", stats.implications);
            print_errors(&stats.errors);
        }
        Err(e) => {
            eprintln!("WARNING: V1 migration had errors: {e}");
        }
    }

    eprintln!();

    // Run V2 migration (dotfile telemetry + knowledge)
    let v2_result = pool.with_conn(|conn| nexcore_db::migrate::run_v2(conn, &claude_dir));

    match v2_result {
        Ok(stats) => {
            eprintln!("V2 Migration (Dotfile data):");
            eprintln!("  Decisions:    {}", stats.decisions);
            eprintln!("  Tool usage:   {}", stats.tool_usage);
            eprintln!("  Efficiency:   {}", stats.token_efficiency);
            eprintln!("  Tasks:        {}", stats.tasks);
            eprintln!("  Handoffs:     {}", stats.handoffs);
            eprintln!("  Antibodies:   {}", stats.antibodies);
            print_errors(&stats.errors);
        }
        Err(e) => {
            eprintln!("ERROR: V2 migration failed: {e}");
            std::process::exit(1);
        }
    }

    eprintln!();
    eprintln!("Database size: {}", file_size_display(&db_path));
}

fn print_errors(errors: &[String]) {
    if errors.is_empty() {
        eprintln!("  Errors: 0");
    } else {
        eprintln!("  {} non-fatal errors:", errors.len());
        for (i, err) in errors.iter().enumerate() {
            if i < 20 {
                eprintln!("    - {err}");
            }
        }
        if errors.len() > 20 {
            eprintln!("    ... and {} more", errors.len() - 20);
        }
    }
}

fn file_size_display(path: &std::path::Path) -> String {
    match std::fs::metadata(path) {
        Ok(meta) => {
            let bytes = meta.len();
            if bytes < 1024 {
                format!("{bytes} B")
            } else if bytes < 1024 * 1024 {
                format!("{:.1} KB", bytes as f64 / 1024.0)
            } else {
                format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
            }
        }
        Err(_) => "unknown".into(),
    }
}
