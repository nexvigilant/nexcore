//! Source session parser.
//!
//! Parses JSON session files from external telemetry sources.
//! Source files are located at ~/.gemini/tmp/{project_hash}/chats/session-*.json

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Result, TelemetryError};
use crate::types::{ProjectHash, Source};

/// Discovered source with metadata.
#[derive(Debug, Clone)]
pub struct DiscoveredSource {
    /// Project hash (directory name)
    pub project_hash: ProjectHash,
    /// Path to the session file
    pub path: PathBuf,
    /// Session filename
    pub filename: String,
}

/// Get the telemetry home directory.
fn telemetry_home() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or(TelemetryError::HomeNotFound)?;
    let telemetry_dir = home.join(".gemini").join("tmp");

    if !telemetry_dir.exists() {
        return Err(TelemetryError::HomeNotFound);
    }

    Ok(telemetry_dir)
}

/// Discover all source sessions across all projects.
///
/// Scans ~/.gemini/tmp/{hash}/chats/ for session-*.json files.
///
/// # Errors
///
/// Returns `TelemetryError::HomeNotFound` if the telemetry directory doesn't exist.
/// Returns `TelemetryError::Io` for filesystem errors.
pub fn discover_sources() -> Result<Vec<DiscoveredSource>> {
    let telemetry_dir = telemetry_home()?;
    let mut sources = Vec::new();

    // Read project hash directories
    let entries = fs::read_dir(&telemetry_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Skip non-directories and special directories
        if !path.is_dir() {
            continue;
        }

        let hash = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Skip 'bin' directory
        if hash == "bin" {
            continue;
        }

        // Look for and process chats directory
        let chats_dir = path.join("chats");
        if chats_dir.exists() && chats_dir.is_dir() {
            scan_chat_directory(&chats_dir, &hash, &mut sources)?;
        }
    }

    // Sort by path for consistent ordering
    sources.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(sources)
}

/// Helper to scan a single project's chat directory for session files.
fn scan_chat_directory(
    chats_dir: &Path,
    hash: &str,
    sources: &mut Vec<DiscoveredSource>,
) -> Result<()> {
    for chat_entry in fs::read_dir(chats_dir)? {
        let chat_entry = chat_entry?;
        let chat_path = chat_entry.path();

        if !chat_path.is_file() {
            continue;
        }

        let filename = match chat_path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Only include session-*.json files
        if filename.starts_with("session-") && filename.ends_with(".json") {
            sources.push(DiscoveredSource {
                project_hash: ProjectHash(hash.to_string()),
                path: chat_path,
                filename,
            });
        }
    }
    Ok(())
}

/// Discover sources for a specific project hash.
///
/// # Errors
///
/// Returns `TelemetryError::ProjectNotFound` if the project directory doesn't exist.
pub fn discover_sources_for_project(project_hash: &str) -> Result<Vec<DiscoveredSource>> {
    let telemetry_dir = telemetry_home()?;
    let project_dir = telemetry_dir.join(project_hash);

    if !project_dir.exists() {
        return Err(TelemetryError::ProjectNotFound {
            hash: project_hash.to_string(),
        });
    }

    let chats_dir = project_dir.join("chats");
    if !chats_dir.exists() || !chats_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut sources = Vec::new();
    let entries = fs::read_dir(&chats_dir)?;

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

        if filename.starts_with("session-") && filename.ends_with(".json") {
            sources.push(DiscoveredSource {
                project_hash: ProjectHash(project_hash.to_string()),
                path,
                filename,
            });
        }
    }

    sources.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(sources)
}

/// Parse a source session file.
///
/// # Arguments
///
/// * `path` - Path to the session JSON file
///
/// # Errors
///
/// Returns `TelemetryError::SourceNotFound` if the file doesn't exist.
/// Returns `TelemetryError::Json` for JSON parsing errors.
/// Returns `TelemetryError::Io` for read errors.
pub fn parse_source(path: &Path) -> Result<Source> {
    if !path.exists() {
        return Err(TelemetryError::SourceNotFound {
            path: path.to_path_buf(),
        });
    }

    let content = fs::read_to_string(path)?;
    let source: Source = serde_json::from_str(&content)?;

    Ok(source)
}

/// Parse all sources for a project.
///
/// # Errors
///
/// Returns errors for any source that fails to parse.
pub fn parse_all_sources(project_hash: &str) -> Result<Vec<Source>> {
    let discovered = discover_sources_for_project(project_hash)?;
    let mut sources = Vec::with_capacity(discovered.len());

    for discovered_source in discovered {
        let source = parse_source(&discovered_source.path)?;
        sources.push(source);
    }

    Ok(sources)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_session(dir: &Path, filename: &str) -> PathBuf {
        let session = r#"{
            "sessionId": "test-session-id",
            "projectHash": "testhash",
            "startTime": "2026-01-29T05:26:25.195Z",
            "lastUpdated": "2026-01-29T05:26:56.228Z",
            "messages": []
        }"#;

        let path = dir.join(filename);
        fs::write(&path, session).expect("Failed to write test session");
        path
    }

    #[test]
    fn test_parse_source_valid() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let session_path = create_test_session(temp_dir.path(), "session-test.json");

        let source = parse_source(&session_path).expect("Failed to parse source");

        assert_eq!(source.id, "test-session-id");
        assert_eq!(source.project_hash, "testhash");
        assert!(source.messages.is_empty());
    }

    #[test]
    fn test_parse_source_not_found() {
        let result = parse_source(Path::new("/nonexistent/path.json"));

        assert!(matches!(result, Err(TelemetryError::SourceNotFound { .. })));
    }

    #[test]
    fn test_parse_source_invalid_json() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().join("invalid.json");
        fs::write(&path, "not valid json").expect("Failed to write");

        let result = parse_source(&path);

        assert!(matches!(result, Err(TelemetryError::Json(_))));
    }
}
