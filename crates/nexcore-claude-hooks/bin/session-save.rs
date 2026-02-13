//! Session Save - SessionEnd Hook
//!
//! Saves session state to the brain system when session ends.
//! Resolves open artifacts to create immutable snapshots.
//!
//! Hook Protocol:
//! - Input: JSON on stdin with session_id, cwd, etc.
//! - Output: Empty JSON `{}` on stdout
//! - Exit: 0 = pass (always, never blocks session end)

use nexcore_brain::BrainSession;
use nexcore_hook_lib::cytokine::emit_hook_completed;
use nexcore_hook_lib::pass;

const HOOK_NAME: &str = "session-save";
use serde::Deserialize;
use std::fmt;
use std::io::{self, Read};

/// SessionEnd input structure from Claude Code
/// Tier: T2-C (cross-domain composite)
/// Grounds to: T1(String) via Option.
#[derive(Debug, Deserialize)]
struct SessionEndInput {
    #[serde(default)]
    #[allow(dead_code)]
    session_id: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    cwd: Option<String>,
}

fn main() {
    // Read stdin (SessionEnd event data)
    let mut buffer = String::new();
    if io::stdin().read_to_string(&mut buffer).is_err() {
        pass();
    }

    // Parse input (gracefully handle empty/malformed)
    let _input: SessionEndInput = serde_json::from_str(&buffer).unwrap_or(SessionEndInput {
        session_id: None,
        cwd: None,
    });

    // Try to save session state
    if let Err(e) = save_session_state() {
        // Log error but never block session end
        eprintln!("[session-save] Warning: {e}");
    }

    pass();
}

/// Error type for session save operations.
///
/// # Tier: T3 (Domain-Specific Error)
/// Grounds to: T1(String) via Display.
#[derive(Debug)]
enum SaveError {
    BrainAccess(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BrainAccess(msg) => write!(f, "brain access: {msg}"),
        }
    }
}

/// Save current session state to brain system
fn save_session_state() -> Result<(), SaveError> {
    // Get the latest brain session
    let session = BrainSession::load_latest().map_err(|e| SaveError::BrainAccess(e.to_string()))?;

    // Track which artifacts were resolved
    let mut resolved = Vec::new();

    // Try to resolve common artifacts (create immutable snapshots)
    for artifact_name in &["task.md", "plan.md", "notes.md", "decisions.md"] {
        // Only resolve if artifact exists
        if session.get_artifact(artifact_name, None).is_ok() {
            match session.resolve_artifact(artifact_name) {
                Ok(version) => {
                    resolved.push(format!("{artifact_name} -> v{version}"));
                }
                Err(_err) => {
                    // Artifact might not have changed - that's fine
                }
            }
        }
    }

    // Emit cytokine signal (TGF-beta = regulation, session saved)
    let artifact_count = resolved.len();
    emit_hook_completed(HOOK_NAME, 0, &format!("saved_{artifact_count}_artifacts"));

    // Report what was saved
    if !resolved.is_empty() {
        eprintln!();
        eprintln!("\u{1F4BE} SESSION STATE SAVED");
        for item in &resolved {
            eprintln!("  - {item}");
        }
        eprintln!();
    }

    Ok(())
}
