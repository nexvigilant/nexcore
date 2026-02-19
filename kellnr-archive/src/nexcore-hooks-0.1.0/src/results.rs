//! Hook Result Registry for Inter-Hook Communication
//!
//! Enables hooks to share results within a single tool use. Each tool use gets
//! a unique registry file identified by tool_use_id, allowing hooks to:
//!
//! - Append their results after execution
//! - Read results from prior hooks
//! - Enable the decision_aggregator to combine all results
//!
//! Registry files are stored in `~/.cache/nexcore-hooks/{tool_use_id}.json`

use crate::protocol::{AggregatedResult, Decision, HookResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get current timestamp as f64
fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Registry of hook results for a single tool use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResultRegistry {
    /// Tool use ID this registry belongs to
    pub tool_use_id: String,
    /// All hook results in execution order
    pub results: Vec<HookResult>,
    /// When the first hook started
    pub started_at: f64,
    /// When the registry was last updated
    pub updated_at: f64,
}

impl HookResultRegistry {
    /// Create a new empty registry
    pub fn new(tool_use_id: impl Into<String>) -> Self {
        let now = now();
        Self {
            tool_use_id: tool_use_id.into(),
            results: Vec::new(),
            started_at: now,
            updated_at: now,
        }
    }

    /// Get the cache directory for hook results
    pub fn cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("nexcore-hooks")
    }

    /// Get the registry file path for a tool_use_id
    pub fn registry_path(tool_use_id: &str) -> PathBuf {
        Self::cache_dir().join(format!("results_{}.json", tool_use_id))
    }

    /// Load registry from disk for the given tool_use_id
    pub fn load(tool_use_id: &str) -> io::Result<Self> {
        let path = Self::registry_path(tool_use_id);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            serde_json::from_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        } else {
            Ok(Self::new(tool_use_id))
        }
    }

    /// Save registry to disk
    pub fn save(&mut self) -> io::Result<()> {
        self.updated_at = now();
        let path = Self::registry_path(&self.tool_use_id);

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write atomically via temp file
        let temp_path = path.with_extension("json.tmp");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&temp_path, content)?;
        fs::rename(temp_path, path)?;
        Ok(())
    }

    /// Append a hook result and save
    pub fn append(&mut self, result: HookResult) -> io::Result<()> {
        self.results.push(result);
        self.save()
    }

    /// Atomically load, append a result, and save
    pub fn append_atomic(tool_use_id: &str, result: HookResult) -> io::Result<()> {
        let mut registry = Self::load(tool_use_id)?;
        registry.append(result)
    }

    /// Get results from a specific hook
    pub fn get_by_hook(&self, hook_name: &str) -> Option<&HookResult> {
        self.results.iter().find(|r| r.hook_name == hook_name)
    }

    /// Get all results from a specific group
    pub fn get_by_group(&self, group: crate::protocol::HookGroup) -> Vec<&HookResult> {
        self.results.iter().filter(|r| r.group == group).collect()
    }

    /// Get the current aggregated decision
    pub fn current_decision(&self) -> Decision {
        self.results
            .iter()
            .fold(Decision::Allow, |acc, r| acc.combine(r.decision))
    }

    /// Get the total number of findings
    pub fn total_findings(&self) -> usize {
        self.results.iter().map(|r| r.findings.len()).sum()
    }

    /// Get total execution time
    pub fn total_duration_ms(&self) -> u64 {
        self.results.iter().map(|r| r.duration_ms).sum()
    }

    /// Aggregate all results into a final report
    pub fn aggregate(&self) -> AggregatedResult {
        AggregatedResult::from_results(&self.tool_use_id, &self.results)
    }

    /// Check if any hook has blocked
    pub fn is_blocked(&self) -> bool {
        self.results.iter().any(|r| r.decision == Decision::Block)
    }

    /// Check if any hook has warned
    pub fn has_warnings(&self) -> bool {
        self.results.iter().any(|r| r.decision == Decision::Warn)
    }

    /// Clean up old registry files (older than 1 hour)
    pub fn cleanup_old() -> io::Result<()> {
        let cache_dir = Self::cache_dir();
        if !cache_dir.exists() {
            return Ok(());
        }

        let cutoff = now() - 3600.0; // 1 hour ago

        for entry in fs::read_dir(cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = modified.elapsed() {
                            if age.as_secs_f64() > (now() - cutoff) {
                                let _ = fs::remove_file(path);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Remove the registry file for a tool_use_id
    pub fn remove(tool_use_id: &str) -> io::Result<()> {
        let path = Self::registry_path(tool_use_id);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}

/// Helper to run a hook function and record its result
pub fn with_result_tracking<F>(
    tool_use_id: &str,
    hook_name: &str,
    group: crate::protocol::HookGroup,
    f: F,
) -> io::Result<Decision>
where
    F: FnOnce() -> (Decision, Vec<crate::protocol::Finding>),
{
    use std::time::Instant;

    let start = Instant::now();
    let (decision, findings) = f();
    let duration_ms = start.elapsed().as_millis() as u64;

    let result = HookResult::new(hook_name, group)
        .with_decision(decision)
        .with_findings(findings)
        .with_duration(duration_ms);

    HookResultRegistry::append_atomic(tool_use_id, result)?;
    Ok(decision)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Finding, HookGroup, Location, Severity};
    use tempfile::tempdir;

    #[test]
    fn test_registry_new() {
        let registry = HookResultRegistry::new("test-id");
        assert_eq!(registry.tool_use_id, "test-id");
        assert!(registry.results.is_empty());
        assert!(registry.started_at > 0.0);
    }

    #[test]
    fn test_registry_append() {
        let mut registry = HookResultRegistry::new("test-id");
        let result =
            HookResult::new("test_hook", HookGroup::Security).with_decision(Decision::Warn);

        registry.results.push(result);

        assert_eq!(registry.results.len(), 1);
        assert_eq!(registry.current_decision(), Decision::Warn);
    }

    #[test]
    fn test_registry_decision_aggregation() {
        let mut registry = HookResultRegistry::new("test-id");

        registry
            .results
            .push(HookResult::new("hook1", HookGroup::Security).with_decision(Decision::Allow));
        registry
            .results
            .push(HookResult::new("hook2", HookGroup::Quality).with_decision(Decision::Warn));
        registry
            .results
            .push(HookResult::new("hook3", HookGroup::Safety).with_decision(Decision::Allow));

        assert_eq!(registry.current_decision(), Decision::Warn);
        assert!(registry.has_warnings());
        assert!(!registry.is_blocked());

        registry
            .results
            .push(HookResult::new("hook4", HookGroup::Policy).with_decision(Decision::Block));

        assert_eq!(registry.current_decision(), Decision::Block);
        assert!(registry.is_blocked());
    }

    #[test]
    fn test_registry_get_by_hook() {
        let mut registry = HookResultRegistry::new("test-id");

        registry.results.push(
            HookResult::new("secret_scanner", HookGroup::Security)
                .with_decision(Decision::Block)
                .with_findings(vec![Finding::new(
                    "API key found",
                    Severity::Critical,
                    Location::new("config.yaml"),
                )]),
        );

        let result = registry.get_by_hook("secret_scanner");
        assert!(result.is_some());
        assert_eq!(result.unwrap().decision, Decision::Block);
        assert_eq!(result.unwrap().findings.len(), 1);

        assert!(registry.get_by_hook("nonexistent").is_none());
    }

    #[test]
    fn test_registry_get_by_group() {
        let mut registry = HookResultRegistry::new("test-id");

        registry
            .results
            .push(HookResult::new("hook1", HookGroup::Security));
        registry
            .results
            .push(HookResult::new("hook2", HookGroup::Security));
        registry
            .results
            .push(HookResult::new("hook3", HookGroup::Quality));

        let security_results = registry.get_by_group(HookGroup::Security);
        assert_eq!(security_results.len(), 2);

        let quality_results = registry.get_by_group(HookGroup::Quality);
        assert_eq!(quality_results.len(), 1);

        let deps_results = registry.get_by_group(HookGroup::Dependencies);
        assert!(deps_results.is_empty());
    }

    #[test]
    fn test_registry_total_duration() {
        let mut registry = HookResultRegistry::new("test-id");

        registry
            .results
            .push(HookResult::new("hook1", HookGroup::Security).with_duration(10));
        registry
            .results
            .push(HookResult::new("hook2", HookGroup::Quality).with_duration(20));
        registry
            .results
            .push(HookResult::new("hook3", HookGroup::Safety).with_duration(15));

        assert_eq!(registry.total_duration_ms(), 45);
    }

    #[test]
    fn test_registry_total_findings() {
        let mut registry = HookResultRegistry::new("test-id");

        registry.results.push(
            HookResult::new("hook1", HookGroup::Security).with_findings(vec![
                Finding::new("f1", Severity::High, Location::new("a.rs")),
                Finding::new("f2", Severity::Low, Location::new("b.rs")),
            ]),
        );
        registry.results.push(
            HookResult::new("hook2", HookGroup::Quality).with_findings(vec![Finding::new(
                "f3",
                Severity::Medium,
                Location::new("c.rs"),
            )]),
        );

        assert_eq!(registry.total_findings(), 3);
    }

    #[test]
    fn test_registry_aggregate() {
        let mut registry = HookResultRegistry::new("test-id");

        registry.results.push(
            HookResult::new("secret_scanner", HookGroup::Security)
                .with_decision(Decision::Block)
                .with_duration(10)
                .with_findings(vec![Finding::new(
                    "Secret found",
                    Severity::Critical,
                    Location::new("config.yaml"),
                )]),
        );
        registry.results.push(
            HookResult::new("clone_detector", HookGroup::Quality)
                .with_decision(Decision::Warn)
                .with_duration(5)
                .with_findings(vec![Finding::new(
                    "Unnecessary clone",
                    Severity::Low,
                    Location::new("lib.rs"),
                )]),
        );

        let agg = registry.aggregate();

        assert_eq!(agg.decision, Decision::Block);
        assert_eq!(agg.total_findings, 2);
        assert_eq!(agg.critical_count, 1);
        assert_eq!(agg.total_duration_ms, 15);
        assert_eq!(agg.tool_use_id, "test-id");
    }
}
