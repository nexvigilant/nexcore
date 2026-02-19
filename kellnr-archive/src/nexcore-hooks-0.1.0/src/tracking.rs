//! Unified Tracking Registry for Claude Code Artifacts
//!
//! Provides sequential tracking IDs (00001, 00002, ...) for all artifacts
//! in ~/.claude/ to enable easy recognition of stale files and session continuity.
//!
//! Tracked artifact types:
//! - Handoffs (H): Session handoff documents
//! - Plans (P): Implementation plans
//! - Audits (A): Verification and audit logs
//! - Tasks (T): Task state snapshots
//! - Sessions (S): Session state files

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Artifact types that can be tracked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtifactType {
    Handoff,
    Plan,
    Audit,
    Task,
    Session,
    Requirement,
    Assumption,
    Other,
}

impl ArtifactType {
    /// Short prefix for display
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Handoff => "H",
            Self::Plan => "P",
            Self::Audit => "A",
            Self::Task => "T",
            Self::Session => "S",
            Self::Requirement => "R",
            Self::Assumption => "X",
            Self::Other => "O",
        }
    }

    /// Directory name for this artifact type
    pub fn directory(&self) -> &'static str {
        match self {
            Self::Handoff => "handoffs",
            Self::Plan => "plans",
            Self::Audit => "verified",
            Self::Task => "tasks",
            Self::Session => "sessions",
            Self::Requirement => "requirements",
            Self::Assumption => "assumptions",
            Self::Other => "other",
        }
    }
}

/// A tracked artifact entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedArtifact {
    pub id: u32,
    pub artifact_type: ArtifactType,
    pub path: String,
    pub created_at: f64,
    pub updated_at: f64,
    pub session_id: Option<String>,
    pub description: Option<String>,
}

/// The tracking registry - maintains all artifact IDs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrackingRegistry {
    /// Next available tracking ID (global across all types)
    pub next_id: u32,
    /// All tracked artifacts by ID
    pub artifacts: Vec<TrackedArtifact>,
    /// Current session's handoff ID (for incremental updates)
    pub current_handoff_id: Option<u32>,
    /// Current session ID
    pub current_session_id: Option<String>,
    /// Registry last updated timestamp
    pub last_updated: f64,
}

impl TrackingRegistry {
    /// Registry file path
    pub fn registry_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("tracking_registry.json")
    }

    /// Load registry from disk (with file locking for safety)
    pub fn load() -> Self {
        let path = Self::registry_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(registry) = serde_json::from_str(&content) {
                    return registry;
                }
            }
        }
        // Initialize with ID starting at 1
        Self {
            next_id: 1,
            ..Default::default()
        }
    }

    /// Save registry to disk
    pub fn save(&mut self) -> std::io::Result<()> {
        self.last_updated = crate::state::now();
        let path = Self::registry_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write atomically via temp file
        let temp_path = path.with_extension("json.tmp");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&temp_path, &content)?;
        fs::rename(temp_path, path)?;
        Ok(())
    }

    /// Allocate and return the next tracking ID
    pub fn allocate_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Format a tracking ID as 5-digit string
    pub fn format_id(id: u32) -> String {
        format!("{:05}", id)
    }

    /// Register a new artifact and return its tracking ID
    pub fn register_artifact(
        &mut self,
        artifact_type: ArtifactType,
        path: &str,
        session_id: Option<&str>,
        description: Option<&str>,
    ) -> u32 {
        let id = self.allocate_id();
        let now = crate::state::now();

        let artifact = TrackedArtifact {
            id,
            artifact_type,
            path: path.to_string(),
            created_at: now,
            updated_at: now,
            session_id: session_id.map(String::from),
            description: description.map(String::from),
        };

        self.artifacts.push(artifact);
        id
    }

    /// Update an existing artifact's timestamp
    pub fn update_artifact(&mut self, id: u32) {
        if let Some(artifact) = self.artifacts.iter_mut().find(|a| a.id == id) {
            artifact.updated_at = crate::state::now();
        }
    }

    /// Get artifact by ID
    pub fn get_artifact(&self, id: u32) -> Option<&TrackedArtifact> {
        self.artifacts.iter().find(|a| a.id == id)
    }

    /// Get artifact by path
    pub fn get_artifact_by_path(&self, path: &str) -> Option<&TrackedArtifact> {
        self.artifacts.iter().find(|a| a.path == path)
    }

    /// Get or create current session's handoff ID
    pub fn get_or_create_handoff_id(&mut self, session_id: &str) -> u32 {
        // Check if we already have a handoff for this session
        if let Some(id) = self.current_handoff_id {
            if self.current_session_id.as_deref() == Some(session_id) {
                return id;
            }
        }

        // Create new handoff
        let handoff_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("handoffs");

        let id = self.allocate_id();
        let path = handoff_dir
            .join(format!("{}.md", Self::format_id(id)))
            .to_string_lossy()
            .to_string();

        let artifact = TrackedArtifact {
            id,
            artifact_type: ArtifactType::Handoff,
            path,
            created_at: crate::state::now(),
            updated_at: crate::state::now(),
            session_id: Some(session_id.to_string()),
            description: Some("Session handoff document".to_string()),
        };

        self.artifacts.push(artifact);
        self.current_handoff_id = Some(id);
        self.current_session_id = Some(session_id.to_string());

        id
    }

    /// Generate a tracked filename for a new artifact
    pub fn generate_tracked_filename(
        &mut self,
        _artifact_type: ArtifactType,
        description: &str,
        extension: &str,
    ) -> (u32, String) {
        let id = self.allocate_id();
        let id_str = Self::format_id(id);

        // Sanitize description for filename
        let safe_desc: String = description
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_string();

        let truncated = if safe_desc.len() > 50 {
            safe_desc[..50].to_string()
        } else {
            safe_desc
        };

        let filename = format!("{}_{}.{}", id_str, truncated, extension);
        (id, filename)
    }

    /// List artifacts by type
    pub fn list_by_type(&self, artifact_type: ArtifactType) -> Vec<&TrackedArtifact> {
        self.artifacts
            .iter()
            .filter(|a| a.artifact_type == artifact_type)
            .collect()
    }

    /// Get recent artifacts (last 24 hours)
    pub fn recent_artifacts(&self, hours: f64) -> Vec<&TrackedArtifact> {
        let cutoff = crate::state::now() - (hours * 3600.0);
        self.artifacts
            .iter()
            .filter(|a| a.updated_at >= cutoff)
            .collect()
    }

    /// Clean up old artifacts (remove entries for non-existent files)
    pub fn cleanup(&mut self) {
        self.artifacts.retain(|a| {
            let path = PathBuf::from(&a.path);
            path.exists()
        });
    }
}

/// Helper to atomically load, modify, and save the registry
pub fn with_registry<F, T>(f: F) -> std::io::Result<T>
where
    F: FnOnce(&mut TrackingRegistry) -> T,
{
    let mut registry = TrackingRegistry::load();
    let result = f(&mut registry);
    registry.save()?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_id() {
        assert_eq!(TrackingRegistry::format_id(1), "00001");
        assert_eq!(TrackingRegistry::format_id(42), "00042");
        assert_eq!(TrackingRegistry::format_id(12345), "12345");
        assert_eq!(TrackingRegistry::format_id(99999), "99999");
    }

    #[test]
    fn test_allocate_id() {
        let mut registry = TrackingRegistry::default();
        registry.next_id = 1;

        assert_eq!(registry.allocate_id(), 1);
        assert_eq!(registry.allocate_id(), 2);
        assert_eq!(registry.allocate_id(), 3);
        assert_eq!(registry.next_id, 4);
    }

    #[test]
    fn test_generate_tracked_filename() {
        let mut registry = TrackingRegistry::default();
        registry.next_id = 42;

        let (id, filename) =
            registry.generate_tracked_filename(ArtifactType::Plan, "My Cool Plan!", "md");

        assert_eq!(id, 42);
        // "My Cool Plan!" -> "my-cool-plan" (spaces/'!' become '-', then trimmed)
        assert_eq!(filename, "00042_my-cool-plan.md");
    }
}
