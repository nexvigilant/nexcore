//! Stop hook that blocks session completion until a unique artifact file is created.
//!
//! Checks for artifacts created during the session in:
//! - ~/.claude/brain/sessions/{session_id}/
//! - Requires at least one .md file with session-unique content
//!
//! Exit codes:
//! - 0: Artifact exists, allow stop
//! - 2: No artifact found, block stop

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

fn main() {
    let result = run();
    std::process::exit(result);
}

fn run() -> i32 {
    // Get session start time marker
    let marker_path = dirs::home_dir()
        .map(|h| h.join(".claude/session_start_marker"))
        .unwrap_or_default();

    let session_start = fs::metadata(&marker_path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    // Check brain sessions directory for artifacts created this session
    let brain_sessions = dirs::home_dir()
        .map(|h| h.join(".claude/brain/sessions"))
        .unwrap_or_default();

    if !brain_sessions.exists() {
        print_block("No brain sessions directory found");
        return 2;
    }

    // Find artifacts created after session start
    let artifacts = find_session_artifacts(&brain_sessions, session_start);

    if artifacts.is_empty() {
        // Also check for contribution marker (capability improvement counts)
        let contribution_marker = dirs::home_dir()
            .map(|h| h.join(".claude/.contribution_marker"))
            .unwrap_or_default();

        if contribution_marker.exists() {
            if let Ok(modified) = fs::metadata(&contribution_marker).and_then(|m| m.modified()) {
                if modified > session_start {
                    // Contribution was made, allow stop
                    return 0;
                }
            }
        }

        print_block("No artifact created this session. Create a unique file before stopping.");
        return 2;
    }

    // Print success with artifact list
    eprintln!("✅ Session artifacts found:");
    for artifact in &artifacts {
        eprintln!("   📄 {}", artifact.display());
    }

    0
}

fn find_session_artifacts(brain_dir: &PathBuf, after: SystemTime) -> Vec<PathBuf> {
    let mut artifacts = Vec::new();

    if let Ok(sessions) = fs::read_dir(brain_dir) {
        for session in sessions.flatten() {
            if let Ok(files) = fs::read_dir(session.path()) {
                for file in files.flatten() {
                    let path = file.path();

                    // Only check .md files
                    if path.extension().map_or(false, |e| e == "md") {
                        if let Ok(meta) = fs::metadata(&path) {
                            if let Ok(modified) = meta.modified() {
                                if modified > after {
                                    artifacts.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Also check scratchpad for artifacts
    let scratchpad = dirs::home_dir()
        .map(|h| h.join(".claude/scratchpad"))
        .unwrap_or_default();

    if let Ok(files) = fs::read_dir(&scratchpad) {
        for file in files.flatten() {
            let path = file.path();
            if path.extension().map_or(false, |e| e == "md") {
                if let Ok(meta) = fs::metadata(&path) {
                    if let Ok(modified) = meta.modified() {
                        if modified > after {
                            artifacts.push(path);
                        }
                    }
                }
            }
        }
    }

    artifacts
}

fn print_block(message: &str) {
    let output = serde_json::json!({
        "decision": "block",
        "reason": message
    });
    println!("{}", output);
    eprintln!("🛑 {}", message);
}
