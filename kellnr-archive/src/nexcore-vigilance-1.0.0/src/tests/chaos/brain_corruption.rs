//! # Brain Corruption Tests
//!
//! CTVP Phase 1: Test recovery from corrupt artifact files.
//!
//! ## Test Scenarios
//!
//! 1. Corrupted index.json - Verify recovery rebuilds from session directories
//! 2. Partial writes - Artifact exists but metadata missing
//! 3. Invalid JSON in artifacts - Verify graceful handling
//! 4. Missing session directories - Verify proper error propagation

use std::fs;
use std::path::PathBuf;

use super::{ChaosTestResult, FaultInjector};
use tempfile::TempDir;

/// Simulated brain state for testing
pub struct SimulatedBrain {
    pub root: TempDir,
    pub brain_dir: PathBuf,
    pub sessions_dir: PathBuf,
    pub index_path: PathBuf,
}

impl SimulatedBrain {
    /// Create a new simulated brain with proper directory structure
    pub fn new() -> std::io::Result<Self> {
        let root = TempDir::new()?;
        let brain_dir = root.path().join(".claude/brain");
        let sessions_dir = brain_dir.join("sessions");
        let index_path = brain_dir.join("index.json");

        fs::create_dir_all(&sessions_dir)?;
        fs::create_dir_all(brain_dir.join("annotations"))?;
        fs::write(&index_path, "[]")?;

        Ok(Self {
            root,
            brain_dir,
            sessions_dir,
            index_path,
        })
    }

    /// Create a valid session with artifact
    pub fn create_session(
        &self,
        session_id: &str,
        artifact_name: &str,
        content: &str,
    ) -> std::io::Result<PathBuf> {
        let session_dir = self.sessions_dir.join(session_id);
        fs::create_dir_all(&session_dir)?;

        let artifact_path = session_dir.join(artifact_name);
        fs::write(&artifact_path, content)?;

        // Create metadata
        let metadata_path = session_dir.join(format!("{artifact_name}.metadata.json"));
        fs::write(
            &metadata_path,
            r#"{"type":"Task","summary":"Test artifact"}"#,
        )?;

        Ok(session_dir)
    }

    /// Corrupt the index file
    pub fn corrupt_index(&self) -> std::io::Result<()> {
        fs::write(&self.index_path, "{ invalid json }")
    }

    /// Write truncated (partial) index
    pub fn truncate_index(&self) -> std::io::Result<()> {
        fs::write(&self.index_path, r#"[{"id":"abc"#) // Truncated JSON
    }

    /// Remove index file
    pub fn remove_index(&self) -> std::io::Result<()> {
        if self.index_path.exists() {
            fs::remove_file(&self.index_path)?;
        }
        Ok(())
    }

    /// Create artifact without metadata (partial write)
    pub fn create_partial_artifact(
        &self,
        session_id: &str,
        artifact_name: &str,
    ) -> std::io::Result<PathBuf> {
        let session_dir = self.sessions_dir.join(session_id);
        fs::create_dir_all(&session_dir)?;

        let artifact_path = session_dir.join(artifact_name);
        fs::write(&artifact_path, "# Task\nContent here")?;
        // Intentionally skip metadata creation

        Ok(artifact_path)
    }
}

/// Attempt to read and parse an index file, returning structured result
fn try_read_index(index_path: &PathBuf) -> Result<Vec<serde_json::Value>, String> {
    let content = fs::read_to_string(index_path).map_err(|e| format!("IO error: {e}"))?;

    serde_json::from_str(&content).map_err(|e| format!("Parse error: {e}"))
}

/// Attempt recovery by scanning session directories
fn attempt_index_recovery(sessions_dir: &PathBuf, index_path: &PathBuf) -> Result<usize, String> {
    if !sessions_dir.exists() {
        return Err("Sessions directory does not exist".to_string());
    }

    let mut recovered = Vec::new();

    let entries =
        fs::read_dir(sessions_dir).map_err(|e| format!("Cannot read sessions directory: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Only recover valid UUID-like directory names
                if name.len() >= 8 && name.chars().all(|c| c.is_alphanumeric() || c == '-') {
                    recovered.push(serde_json::json!({
                        "id": name,
                        "created_at": chrono::Utc::now().to_rfc3339(),
                        "project": null,
                        "description": "Recovered session"
                    }));
                }
            }
        }
    }

    let index_json = serde_json::to_string_pretty(&recovered)
        .map_err(|e| format!("Serialization error: {e}"))?;

    fs::write(index_path, index_json).map_err(|e| format!("Write error: {e}"))?;

    Ok(recovered.len())
}

/// Detect partial writes in a session directory
fn detect_partial_writes(session_dir: &PathBuf) -> Vec<String> {
    let mut partial = Vec::new();

    if let Ok(entries) = fs::read_dir(session_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Skip metadata files, resolved files, and hidden files
                if name.ends_with(".metadata.json")
                    || name.contains(".resolved")
                    || name.starts_with('.')
                {
                    continue;
                }

                // Check if metadata exists
                let metadata_path = session_dir.join(format!("{name}.metadata.json"));
                if !metadata_path.exists() {
                    partial.push(name.to_string());
                }
            }
        }
    }

    partial
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ========== Basic Corruption Tests ==========

    #[test]
    fn test_corrupt_index_recovery() {
        let brain = SimulatedBrain::new().expect("Failed to create simulated brain");

        // Create a valid session
        let session_id = "12345678-1234-1234-1234-123456789abc";
        brain
            .create_session(session_id, "task.md", "# Task\nContent")
            .expect("Failed to create session");

        // Corrupt the index
        brain.corrupt_index().expect("Failed to corrupt index");

        // Verify index is corrupted
        let read_result = try_read_index(&brain.index_path);
        assert!(read_result.is_err(), "Corrupted index should fail to parse");

        // Attempt recovery
        let recovery_result = attempt_index_recovery(&brain.sessions_dir, &brain.index_path);
        assert!(recovery_result.is_ok(), "Recovery should succeed");
        assert_eq!(
            recovery_result.expect("Already checked"),
            1,
            "Should recover 1 session"
        );

        // Verify index is now valid
        let read_result = try_read_index(&brain.index_path);
        assert!(read_result.is_ok(), "Recovered index should be valid");
    }

    #[test]
    fn test_truncated_index_recovery() {
        let brain = SimulatedBrain::new().expect("Failed to create simulated brain");

        // Create sessions
        brain
            .create_session("session-001", "task.md", "# Task 1")
            .expect("create");
        brain
            .create_session("session-002", "plan.md", "# Plan 2")
            .expect("create");

        // Truncate index (simulates crash during write)
        brain.truncate_index().expect("Failed to truncate index");

        // Verify index is invalid
        let read_result = try_read_index(&brain.index_path);
        assert!(read_result.is_err(), "Truncated index should fail to parse");

        // Recovery
        let recovery_result = attempt_index_recovery(&brain.sessions_dir, &brain.index_path);
        assert!(recovery_result.is_ok());
        assert_eq!(recovery_result.expect("Already checked"), 2);
    }

    #[test]
    fn test_missing_index_recovery() {
        let brain = SimulatedBrain::new().expect("Failed to create simulated brain");

        brain
            .create_session("test-session-1", "task.md", "content")
            .expect("create");

        // Remove index entirely
        brain.remove_index().expect("Failed to remove index");
        assert!(!brain.index_path.exists());

        // Recovery should still work
        let recovery_result = attempt_index_recovery(&brain.sessions_dir, &brain.index_path);
        assert!(recovery_result.is_ok());

        // Index should be recreated
        assert!(brain.index_path.exists());
    }

    // ========== Partial Write Tests ==========

    #[test]
    fn test_detect_partial_writes() {
        let brain = SimulatedBrain::new().expect("Failed to create simulated brain");

        // Create a session with partial artifact (no metadata)
        let session_dir = brain
            .create_partial_artifact("session-partial", "orphan.md")
            .expect("Failed to create partial artifact")
            .parent()
            .expect("Should have parent")
            .to_path_buf();

        let partial = detect_partial_writes(&session_dir);
        assert_eq!(partial.len(), 1);
        assert_eq!(partial[0], "orphan.md");
    }

    #[test]
    fn test_no_panic_on_corrupt_index() {
        let result = std::panic::catch_unwind(|| {
            let brain = SimulatedBrain::new().expect("create");
            brain.corrupt_index().expect("corrupt");

            // This should NOT panic even with corrupted data
            let _ = try_read_index(&brain.index_path);
        });

        assert!(result.is_ok(), "Should not panic on corrupt index");
    }

    #[test]
    fn test_recovery_with_empty_sessions() {
        let brain = SimulatedBrain::new().expect("Failed to create simulated brain");

        brain.corrupt_index().expect("corrupt");

        // Recovery with no sessions should succeed but recover nothing
        let recovery_result = attempt_index_recovery(&brain.sessions_dir, &brain.index_path);
        assert!(recovery_result.is_ok());
        assert_eq!(recovery_result.expect("Already checked"), 0);
    }

    // ========== Graceful Degradation Tests ==========

    #[test]
    fn test_graceful_degradation_unreadable_sessions() {
        let brain = SimulatedBrain::new().expect("Failed to create simulated brain");

        // Remove sessions directory entirely
        fs::remove_dir_all(&brain.sessions_dir).expect("remove");

        // Recovery should fail gracefully with error, not panic
        let recovery_result = attempt_index_recovery(&brain.sessions_dir, &brain.index_path);
        assert!(recovery_result.is_err());
        assert!(
            recovery_result
                .err()
                .expect("Already checked")
                .contains("does not exist")
        );
    }

    // ========== Fault Injector Integration ==========

    #[test]
    fn test_fault_injector_corruption_scenario() {
        let injector = FaultInjector::new("index corruption");
        let mut result = ChaosTestResult::new("brain_corruption_recovery");

        let brain = SimulatedBrain::new().expect("create brain");
        brain
            .create_session("test-session", "task.md", "# Test")
            .expect("create session");

        // Phase 1: Normal operation
        let read_result = try_read_index(&brain.index_path);
        assert!(read_result.is_ok());
        result.add_degradation("Normal read succeeded");

        // Phase 2: Inject fault
        injector.inject();
        brain.corrupt_index().expect("corrupt");

        // Phase 3: Verify graceful degradation
        let read_result = try_read_index(&brain.index_path);
        assert!(read_result.is_err());
        result.add_propagated_error(read_result.err().expect("Already checked"));

        // Phase 4: Recovery
        let recovery = attempt_index_recovery(&brain.sessions_dir, &brain.index_path);
        assert!(recovery.is_ok());
        result.add_recovery(format!(
            "Recovered {} sessions",
            recovery.expect("Already checked")
        ));

        // Phase 5: Clear fault and verify normal operation
        injector.clear();
        let read_result = try_read_index(&brain.index_path);
        assert!(read_result.is_ok());
        result.add_recovery("Normal operation restored");

        assert!(result.passed);
    }

    // ========== Property-Based Tests ==========

    proptest! {
        #[test]
        fn prop_no_panic_on_arbitrary_index_content(content in ".*") {
            let temp = TempDir::new().unwrap();
            let index_path = temp.path().join("index.json");
            fs::write(&index_path, &content).unwrap();

            // Should never panic regardless of content
            let _ = try_read_index(&index_path.to_path_buf());
        }

        #[test]
        fn prop_recovery_idempotent(session_count in 0usize..10) {
            let brain = SimulatedBrain::new().unwrap();

            // Create sessions
            for i in 0..session_count {
                let id = format!("{:08x}-0000-0000-0000-000000000000", i);
                let _ = brain.create_session(&id, "task.md", "# Task");
            }

            // Corrupt and recover
            brain.corrupt_index().unwrap();
            let result1 = attempt_index_recovery(&brain.sessions_dir, &brain.index_path).unwrap();

            // Recover again (should be idempotent)
            brain.corrupt_index().unwrap();
            let result2 = attempt_index_recovery(&brain.sessions_dir, &brain.index_path).unwrap();

            prop_assert_eq!(result1, result2, "Recovery should be idempotent");
        }
    }
}
