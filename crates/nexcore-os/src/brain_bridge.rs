// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Brain bridge — persists OS state to the NexCore Brain artifact system.
//!
//! ## Primitive Grounding
//!
//! - π Persistence: Snapshots survive across emulator restarts
//! - ∃ Existence: Sessions and artifacts are created with unique IDs
//! - ς State: OS state serialized and versioned
//! - ∝ Irreversibility: Resolved artifacts are immutable snapshots

use nexcore_brain::{Artifact, ArtifactType, BrainSession};

/// Bridge between NexCore OS and the Brain working memory system.
///
/// Creates a brain session at boot and saves OS state snapshots
/// as versioned artifacts.
pub struct BrainBridge {
    /// Active brain session.
    session: BrainSession,
}

impl BrainBridge {
    /// Create a new bridge with a fresh brain session.
    pub fn new() -> Result<Self, String> {
        let session = BrainSession::create_with_options(
            Some("nexcore-os".to_string()),
            None,
            Some("NexCore OS emulator session".to_string()),
        )
        .map_err(|e| format!("Brain session creation failed: {e}"))?;

        Ok(Self { session })
    }

    /// Load the most recent brain session (for resuming).
    pub fn load_latest() -> Result<Self, String> {
        let session = BrainSession::load_latest().map_err(|e| format!("Brain load failed: {e}"))?;
        Ok(Self { session })
    }

    /// Save an OS state snapshot as a brain artifact.
    pub fn save_snapshot(&self, content: &str) -> Result<(), String> {
        let artifact = Artifact::new("os-snapshot.md", ArtifactType::Custom, content.to_string());
        self.session
            .save_artifact(&artifact)
            .map_err(|e| format!("Artifact save failed: {e}"))
    }

    /// Resolve the current snapshot (create an immutable version).
    pub fn resolve_snapshot(&self) -> Result<u32, String> {
        self.session
            .resolve_artifact("os-snapshot.md")
            .map_err(|e| format!("Artifact resolve failed: {e}"))
    }

    /// Load the latest snapshot content.
    pub fn load_snapshot(&self) -> Result<String, String> {
        let artifact = self
            .session
            .get_artifact("os-snapshot.md", None)
            .map_err(|e| format!("Artifact load failed: {e}"))?;
        Ok(artifact.content)
    }

    /// Load a specific resolved version.
    pub fn load_version(&self, version: u32) -> Result<String, String> {
        let artifact = self
            .session
            .get_artifact("os-snapshot.md", Some(version))
            .map_err(|e| format!("Artifact load failed: {e}"))?;
        Ok(artifact.content)
    }

    /// List all resolved version numbers.
    pub fn list_versions(&self) -> Result<Vec<u32>, String> {
        self.session
            .list_versions("os-snapshot.md")
            .map_err(|e| format!("Version list failed: {e}"))
    }

    /// Save an arbitrary named artifact.
    pub fn save_artifact(&self, name: &str, content: &str) -> Result<(), String> {
        let artifact = Artifact::new(name, ArtifactType::Custom, content.to_string());
        self.session
            .save_artifact(&artifact)
            .map_err(|e| format!("Artifact save failed: {e}"))
    }

    /// Get the session ID.
    pub fn session_id(&self) -> &str {
        &self.session.id
    }

    /// Format brain status for display.
    pub fn format_status(&self) -> String {
        use std::fmt::Write;

        let mut out = String::new();
        out.push_str("Brain Working Memory\n");
        out.push_str("────────────────────\n");
        let _ = writeln!(out, "Session:    {}", self.session.id);

        match self.session.list_artifacts() {
            Ok(artifacts) => {
                let count: usize = artifacts.len();
                let _ = writeln!(out, "Artifacts:  {count}");
                for name in &artifacts {
                    let versions = self
                        .session
                        .list_versions(name)
                        .map(|v: Vec<u32>| v.len())
                        .unwrap_or(0);
                    let _ = writeln!(out, "  {name} ({versions} versions)");
                }
            }
            Err(_) => {
                out.push_str("Artifacts:  (error reading)\n");
            }
        }

        out
    }
}
