use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content, ErrorCode};
use serde_json::Value;
use std::path::PathBuf;

/// Brain directory root.
pub fn brain_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".into());
    PathBuf::from(&home).join(".claude").join("brain")
}

/// Brain database path.
pub fn brain_db_path() -> PathBuf {
    brain_dir().join("brain.db")
}

/// Telemetry directory.
pub fn telemetry_dir() -> PathBuf {
    brain_dir().join("telemetry")
}

/// Implicit knowledge directory.
pub fn implicit_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".into());
    PathBuf::from(&home).join(".claude").join("implicit")
}

/// Hormones state file.
pub fn hormones_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".into());
    PathBuf::from(&home)
        .join(".claude")
        .join("hormones")
        .join("state.json")
}

/// Skills directory.
pub fn skills_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".into());
    PathBuf::from(&home).join(".claude").join("skills")
}

/// Settings.json path.
pub fn settings_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".into());
    PathBuf::from(&home).join(".claude").join("settings.json")
}

/// ~/.claude.json path (MCP server config).
pub fn claude_json_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".into());
    PathBuf::from(&home).join(".claude.json")
}

/// MEMORY.md path.
pub fn memory_md_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".into());
    PathBuf::from(&home)
        .join(".claude")
        .join("projects")
        .join("-home-matthew")
        .join("memory")
        .join("MEMORY.md")
}

/// Open brain.db read-only.
pub fn open_brain_db() -> Result<rusqlite::Connection, McpError> {
    let path = brain_db_path();
    rusqlite::Connection::open_with_flags(
        &path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| McpError::new(ErrorCode(500), format!("brain.db open failed: {e}"), None))
}

/// Open brain.db read-write.
pub fn open_brain_db_rw() -> Result<rusqlite::Connection, McpError> {
    let path = brain_db_path();
    rusqlite::Connection::open(&path)
        .map_err(|e| McpError::new(ErrorCode(500), format!("brain.db open failed: {e}"), None))
}

/// Read a JSON file, return parsed Value.
pub fn read_json_file(path: &std::path::Path) -> Result<Value, McpError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        McpError::new(
            ErrorCode(500),
            format!("read {}: {e}", path.display()),
            None,
        )
    })?;
    serde_json::from_str(&content).map_err(|e| {
        McpError::new(
            ErrorCode(500),
            format!("parse {}: {e}", path.display()),
            None,
        )
    })
}

/// Read JSONL file, return Vec of Value.
pub fn read_jsonl_file(path: &std::path::Path) -> Vec<Value> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return vec![];
    };
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

/// Count lines in a file.
pub fn count_lines(path: &std::path::Path) -> usize {
    std::fs::read_to_string(path)
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).count())
        .unwrap_or(0)
}

/// Count rows in a brain.db table.
pub fn db_count(conn: &rusqlite::Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap_or(0)
}

/// Shorthand for text result.
pub fn text_result(s: &str) -> CallToolResult {
    CallToolResult::success(vec![Content::text(s)])
}

/// Shorthand for JSON result.
pub fn json_result(v: &Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(v.to_string())])
}

/// Shorthand for error.
pub fn mcp_err(msg: &str) -> McpError {
    McpError::new(ErrorCode(500), msg.to_string(), None)
}

/// File age in hours.
pub fn file_age_hours(path: &std::path::Path) -> Option<f64> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    let elapsed = std::time::SystemTime::now().duration_since(modified).ok()?;
    Some(elapsed.as_secs_f64() / 3600.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brain_dir_ends_with_brain() {
        let p = brain_dir();
        assert!(p.ends_with(".claude/brain"), "got: {}", p.display());
    }

    #[test]
    fn brain_db_path_ends_with_db() {
        let p = brain_db_path();
        assert!(p.ends_with("brain.db"), "got: {}", p.display());
    }

    #[test]
    fn implicit_dir_ends_with_implicit() {
        let p = implicit_dir();
        assert!(p.ends_with(".claude/implicit"), "got: {}", p.display());
    }

    #[test]
    fn skills_dir_ends_with_skills() {
        let p = skills_dir();
        assert!(p.ends_with(".claude/skills"), "got: {}", p.display());
    }

    #[test]
    fn settings_path_ends_with_json() {
        let p = settings_path();
        assert!(p.ends_with("settings.json"), "got: {}", p.display());
    }

    #[test]
    fn text_result_contains_text() {
        let r = text_result("hello");
        assert!(!r.content.is_empty());
    }

    #[test]
    fn json_result_contains_json() {
        let v = serde_json::json!({"key": "val"});
        let r = json_result(&v);
        assert!(!r.content.is_empty());
    }

    #[test]
    fn mcp_err_has_message() {
        let e = mcp_err("test error");
        assert_eq!(e.message, "test error");
    }

    #[test]
    fn count_lines_on_missing_file() {
        let p = std::path::Path::new("/tmp/nonexistent-skills-mcp-test-file");
        assert_eq!(count_lines(p), 0);
    }

    #[test]
    fn read_jsonl_on_missing_file() {
        let p = std::path::Path::new("/tmp/nonexistent-skills-mcp-test-file");
        assert!(read_jsonl_file(p).is_empty());
    }

    #[test]
    fn file_age_on_missing_returns_none() {
        let p = std::path::Path::new("/tmp/nonexistent-skills-mcp-test-file");
        assert!(file_age_hours(p).is_none());
    }
}
