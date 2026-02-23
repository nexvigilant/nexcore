//! Snapshot (brain artifact) parser.
//!
//! Parses brain artifacts from ~/.gemini/antigravity/brain/{uuid}/
//! including *.md files, *.metadata.json files, and *.resolved.N versions.

use nexcore_fs::dirs;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Result, TelemetryError};
use crate::types::{Snapshot, SnapshotMetadata, SnapshotType};

/// A discovered brain session directory.
#[derive(Debug, Clone)]
pub struct BrainSession {
    /// Session UUID
    pub id: String,
    /// Path to the session directory
    pub path: PathBuf,
}

/// Get the brain directory path.
fn brain_home() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or(TelemetryError::HomeNotFound)?;
    let brain_dir = home.join(".gemini").join("antigravity").join("brain");

    if !brain_dir.exists() {
        return Err(TelemetryError::BrainNotFound);
    }

    Ok(brain_dir)
}

/// Discover all brain session directories.
///
/// Scans ~/.gemini/antigravity/brain/ for UUID directories.
///
/// # Errors
///
/// Returns `TelemetryError::BrainNotFound` if the brain directory doesn't exist.
/// Returns `TelemetryError::Io` for filesystem errors.
pub fn discover_brain_sessions() -> Result<Vec<BrainSession>> {
    let brain_dir = brain_home()?;
    let mut sessions = Vec::new();

    let entries = fs::read_dir(&brain_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let id = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Basic UUID validation (should be 36 chars with hyphens)
        if id.len() == 36 && id.chars().filter(|c| *c == '-').count() == 4 {
            sessions.push(BrainSession { id, path });
        }
    }

    // Sort by ID for consistent ordering
    sessions.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(sessions)
}

/// Parse all snapshots for a brain session.
///
/// # Arguments
///
/// * `session_id` - The UUID of the brain session
///
/// # Errors
///
/// Returns `TelemetryError::SnapshotNotFound` if the session directory doesn't exist.
/// Returns `TelemetryError::Io` for filesystem errors.
pub fn parse_snapshots(session_id: &str) -> Result<Vec<Snapshot>> {
    let brain_dir = brain_home()?;
    let session_dir = brain_dir.join(session_id);

    if !session_dir.exists() {
        return Err(TelemetryError::SnapshotNotFound { path: session_dir });
    }

    parse_snapshots_from_dir(&session_dir, session_id)
}

/// Parse snapshots from a specific directory path.
///
/// # Arguments
///
/// * `session_dir` - Path to the session directory
/// * `session_id` - The session ID to associate with snapshots
///
/// # Errors
///
/// Returns `TelemetryError::Io` for filesystem errors.
pub fn parse_snapshots_from_dir(session_dir: &Path, session_id: &str) -> Result<Vec<Snapshot>> {
    let mut snapshots = Vec::new();
    let mut processed_bases: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Single directory read: classify all files in one pass
    let mut md_files: Vec<PathBuf> = Vec::new();
    let mut resolved_map: std::collections::HashMap<String, Vec<u32>> =
        std::collections::HashMap::new();

    for entry in fs::read_dir(session_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let filename = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Collect resolved versions: "foo.md.resolved.3" → resolved_map["foo.md"] = [3]
        if let Some((base, version)) = parse_resolved_filename(&filename) {
            resolved_map.entry(base).or_default().push(version);
        } else if filename.ends_with(".md")
            && !filename.contains(".metadata")
            && !filename.contains(".resolved")
        {
            md_files.push(path);
        }
    }

    // Sort each version list
    for versions in resolved_map.values_mut() {
        versions.sort_unstable();
    }

    // Process each base markdown file
    for md_path in md_files {
        let filename = match md_path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        if processed_bases.contains(&filename) {
            continue;
        }
        processed_bases.insert(filename.clone());

        let versions = resolved_map.remove(&filename).unwrap_or_default();
        let snapshot =
            parse_single_snapshot_with_versions(session_dir, session_id, &filename, versions)?;
        snapshots.push(snapshot);
    }

    // Sort by name for consistent ordering
    snapshots.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(snapshots)
}

/// Extract (base_name, version) from "foo.md.resolved.3" → ("foo.md", 3)
fn parse_resolved_filename(filename: &str) -> Option<(String, u32)> {
    let idx = filename.find(".resolved.")?;
    let base = &filename[..idx];
    // Only match base files ending with .md
    if !base.ends_with(".md") {
        return None;
    }
    let suffix = &filename[idx + ".resolved.".len()..];
    let version = suffix.parse::<u32>().ok()?;
    Some((base.to_string(), version))
}

/// Parse a single snapshot with its metadata and versions.
#[allow(dead_code)]
fn parse_single_snapshot(
    session_dir: &Path,
    session_id: &str,
    base_name: &str,
) -> Result<Snapshot> {
    let versions = find_resolved_versions(session_dir, base_name)?;
    parse_single_snapshot_with_versions(session_dir, session_id, base_name, versions)
}

/// Parse a single snapshot with pre-computed resolved versions (avoids repeated dir reads).
fn parse_single_snapshot_with_versions(
    session_dir: &Path,
    session_id: &str,
    base_name: &str,
    versions: Vec<u32>,
) -> Result<Snapshot> {
    let base_path = session_dir.join(base_name);
    let metadata_path = session_dir.join(format!("{base_name}.metadata.json"));

    // Read content
    let content = if base_path.exists() {
        fs::read_to_string(&base_path)?
    } else {
        String::new()
    };

    // Read metadata if available
    let metadata = if metadata_path.exists() {
        let meta_content = fs::read_to_string(&metadata_path)?;
        let meta: SnapshotMetadata = serde_json::from_str(&meta_content)?;
        Some(meta)
    } else {
        None
    };

    // Determine snapshot type from metadata or filename
    let snapshot_type = metadata
        .as_ref()
        .map(|m| SnapshotType::from_str(&m.artifact_type))
        .unwrap_or_else(|| infer_snapshot_type(base_name));

    Ok(Snapshot {
        session_id: session_id.to_string(),
        name: base_name.to_string(),
        snapshot_type,
        content,
        metadata,
        versions,
        path: base_path,
    })
}

/// Find all resolved version numbers for a base file.
#[allow(dead_code)]
fn find_resolved_versions(session_dir: &Path, base_name: &str) -> Result<Vec<u32>> {
    let mut versions = Vec::new();
    let prefix = format!("{base_name}.resolved.");

    let entries = fs::read_dir(session_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let filename = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Match pattern: base_name.resolved.N
        if let Some(suffix) = filename.strip_prefix(&prefix) {
            if let Ok(version) = suffix.parse::<u32>() {
                versions.push(version);
            }
        }
    }

    versions.sort();
    Ok(versions)
}

/// Infer snapshot type from filename.
///
/// Checks for more specific patterns first (e.g., "implementation" before "plan")
/// to handle compound names like "implementation_plan.md".
fn infer_snapshot_type(filename: &str) -> SnapshotType {
    let lower = filename.to_lowercase();

    // Check more specific patterns first
    if lower.contains("implementation") {
        SnapshotType::Implementation
    } else if lower.contains("walkthrough") {
        SnapshotType::Walkthrough
    } else if lower.contains("task") {
        SnapshotType::Task
    } else if lower.contains("plan") {
        SnapshotType::Plan
    } else {
        SnapshotType::Unknown
    }
}

/// Get the content of a specific resolved version.
///
/// # Arguments
///
/// * `session_id` - The brain session UUID
/// * `base_name` - The base artifact name (e.g., "task.md")
/// * `version` - The version number
///
/// # Errors
///
/// Returns `TelemetryError::SnapshotNotFound` if the version doesn't exist.
pub fn get_resolved_version(session_id: &str, base_name: &str, version: u32) -> Result<String> {
    let brain_dir = brain_home()?;
    let version_path = brain_dir
        .join(session_id)
        .join(format!("{base_name}.resolved.{version}"));

    if !version_path.exists() {
        return Err(TelemetryError::SnapshotNotFound { path: version_path });
    }

    let content = fs::read_to_string(&version_path)?;
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_snapshot(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        fs::write(&path, content).expect("Failed to write snapshot");
    }

    fn create_test_metadata(dir: &Path, name: &str) {
        let metadata = r#"{
            "artifactType": "ARTIFACT_TYPE_TASK",
            "summary": "Test summary",
            "updatedAt": "2026-02-02T07:04:37.230274091Z"
        }"#;

        let path = dir.join(format!("{name}.metadata.json"));
        fs::write(&path, metadata).expect("Failed to write metadata");
    }

    #[test]
    fn test_parse_single_snapshot() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let session_id = "test-session";

        create_test_snapshot(temp_dir.path(), "task.md", "# Test Task");
        create_test_metadata(temp_dir.path(), "task.md");
        create_test_snapshot(temp_dir.path(), "task.md.resolved.0", "# Test Task v0");
        create_test_snapshot(temp_dir.path(), "task.md.resolved.1", "# Test Task v1");

        let snapshot =
            parse_single_snapshot(temp_dir.path(), session_id, "task.md").expect("Failed to parse");

        assert_eq!(snapshot.session_id, session_id);
        assert_eq!(snapshot.name, "task.md");
        assert_eq!(snapshot.snapshot_type, SnapshotType::Task);
        assert_eq!(snapshot.content, "# Test Task");
        assert!(snapshot.metadata.is_some());
        assert_eq!(snapshot.versions, vec![0, 1]);
    }

    #[test]
    fn test_parse_snapshots_from_dir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let session_id = "test-session";

        create_test_snapshot(temp_dir.path(), "task.md", "# Task");
        create_test_snapshot(temp_dir.path(), "plan.md", "# Plan");

        let snapshots =
            parse_snapshots_from_dir(temp_dir.path(), session_id).expect("Failed to parse");

        assert_eq!(snapshots.len(), 2);
        assert!(snapshots.iter().any(|s| s.name == "task.md"));
        assert!(snapshots.iter().any(|s| s.name == "plan.md"));
    }

    #[test]
    fn test_infer_snapshot_type() {
        assert_eq!(infer_snapshot_type("task.md"), SnapshotType::Task);
        assert_eq!(
            infer_snapshot_type("implementation_plan.md"),
            SnapshotType::Implementation
        );
        assert_eq!(
            infer_snapshot_type("walkthrough.md"),
            SnapshotType::Walkthrough
        );
        assert_eq!(infer_snapshot_type("random.md"), SnapshotType::Unknown);
    }

    #[test]
    fn test_find_resolved_versions() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        create_test_snapshot(temp_dir.path(), "task.md", "content");
        create_test_snapshot(temp_dir.path(), "task.md.resolved.0", "v0");
        create_test_snapshot(temp_dir.path(), "task.md.resolved.1", "v1");
        create_test_snapshot(temp_dir.path(), "task.md.resolved.5", "v5");

        let versions =
            find_resolved_versions(temp_dir.path(), "task.md").expect("Failed to find versions");

        assert_eq!(versions, vec![0, 1, 5]);
    }
}
