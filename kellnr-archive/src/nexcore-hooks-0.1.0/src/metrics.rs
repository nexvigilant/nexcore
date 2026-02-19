//! Hook Metrics Collection and Reporting
//!
//! Tracks hook execution metrics for observability and optimization:
//! - Execution counts and durations
//! - Block/warn/allow ratios
//! - False positive tracking (user overrides)
//!
//! Metrics are persisted to `~/.cache/nexcore-hooks/metrics.json`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// Metrics for a single hook
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookMetrics {
    /// Hook name (binary name)
    pub hook_name: String,
    /// Total execution count
    pub execution_count: u64,
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    /// Number of times this hook blocked
    pub block_count: u64,
    /// Number of times this hook warned
    pub warn_count: u64,
    /// Number of times this hook allowed
    pub allow_count: u64,
    /// Number of user overrides (false positives)
    pub override_count: u64,
    /// Last execution timestamp
    pub last_execution: f64,
}

impl HookMetrics {
    /// Create new metrics for a hook
    pub fn new(hook_name: impl Into<String>) -> Self {
        Self {
            hook_name: hook_name.into(),
            ..Default::default()
        }
    }

    /// Record an execution
    pub fn record_execution(&mut self, decision: crate::protocol::Decision, duration_ms: u64) {
        self.execution_count += 1;
        self.total_duration_ms += duration_ms;
        self.last_execution = now();

        match decision {
            crate::protocol::Decision::Allow => self.allow_count += 1,
            crate::protocol::Decision::Warn => self.warn_count += 1,
            crate::protocol::Decision::Block => self.block_count += 1,
        }
    }

    /// Record a user override (false positive)
    pub fn record_override(&mut self) {
        self.override_count += 1;
    }

    /// Get average execution time in milliseconds
    pub fn avg_duration_ms(&self) -> f64 {
        if self.execution_count == 0 {
            0.0
        } else {
            self.total_duration_ms as f64 / self.execution_count as f64
        }
    }

    /// Get block rate as percentage
    pub fn block_rate(&self) -> f64 {
        if self.execution_count == 0 {
            0.0
        } else {
            (self.block_count as f64 / self.execution_count as f64) * 100.0
        }
    }

    /// Get false positive rate as percentage (overrides / blocks)
    pub fn false_positive_rate(&self) -> f64 {
        if self.block_count == 0 {
            0.0
        } else {
            (self.override_count as f64 / self.block_count as f64) * 100.0
        }
    }
}

/// Collection of metrics for all hooks
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricsRegistry {
    /// Metrics by hook name
    pub hooks: HashMap<String, HookMetrics>,
    /// When metrics were last updated
    pub last_updated: f64,
    /// When metrics collection started
    pub started_at: f64,
}

impl MetricsRegistry {
    /// Get the metrics file path
    pub fn metrics_path() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("nexcore-hooks")
            .join("metrics.json")
    }

    /// Load metrics from disk
    pub fn load() -> io::Result<Self> {
        let path = Self::metrics_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            serde_json::from_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        } else {
            Ok(Self {
                started_at: now(),
                ..Default::default()
            })
        }
    }

    /// Save metrics to disk
    pub fn save(&mut self) -> io::Result<()> {
        self.last_updated = now();
        let path = Self::metrics_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)
    }

    /// Record a hook execution
    pub fn record(
        &mut self,
        hook_name: &str,
        decision: crate::protocol::Decision,
        duration_ms: u64,
    ) {
        let metrics = self
            .hooks
            .entry(hook_name.to_string())
            .or_insert_with(|| HookMetrics::new(hook_name));
        metrics.record_execution(decision, duration_ms);
    }

    /// Record a user override
    pub fn record_override(&mut self, hook_name: &str) {
        if let Some(metrics) = self.hooks.get_mut(hook_name) {
            metrics.record_override();
        }
    }

    /// Atomically load, record, and save
    pub fn record_atomic(
        hook_name: &str,
        decision: crate::protocol::Decision,
        duration_ms: u64,
    ) -> io::Result<()> {
        let mut registry = Self::load()?;
        registry.record(hook_name, decision, duration_ms);
        registry.save()
    }

    /// Get metrics for a specific hook
    pub fn get(&self, hook_name: &str) -> Option<&HookMetrics> {
        self.hooks.get(hook_name)
    }

    /// Get all hooks sorted by execution count (descending)
    pub fn top_by_executions(&self) -> Vec<&HookMetrics> {
        let mut hooks: Vec<_> = self.hooks.values().collect();
        hooks.sort_by(|a, b| b.execution_count.cmp(&a.execution_count));
        hooks
    }

    /// Get all hooks sorted by average duration (descending)
    pub fn top_by_duration(&self) -> Vec<&HookMetrics> {
        let mut hooks: Vec<_> = self.hooks.values().collect();
        hooks.sort_by(|a, b| {
            b.avg_duration_ms()
                .partial_cmp(&a.avg_duration_ms())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hooks
    }

    /// Get hooks with high false positive rates
    pub fn high_false_positive_hooks(&self, threshold_pct: f64) -> Vec<&HookMetrics> {
        self.hooks
            .values()
            .filter(|m| m.false_positive_rate() > threshold_pct)
            .collect()
    }

    /// Get total execution count across all hooks
    pub fn total_executions(&self) -> u64 {
        self.hooks.values().map(|m| m.execution_count).sum()
    }

    /// Get total execution time across all hooks
    pub fn total_duration_ms(&self) -> u64 {
        self.hooks.values().map(|m| m.total_duration_ms).sum()
    }

    /// Generate a summary report
    pub fn generate_report(&self) -> String {
        let mut lines = Vec::new();

        lines.push("# Hook Metrics Report".to_string());
        lines.push(String::new());
        lines.push(format!("Total Executions: {}", self.total_executions()));
        lines.push(format!("Total Duration: {}ms", self.total_duration_ms()));
        lines.push(String::new());

        // Top hooks by execution
        lines.push("## Top Hooks by Execution Count".to_string());
        for (i, m) in self.top_by_executions().iter().take(10).enumerate() {
            lines.push(format!(
                "{}. {} - {} executions, {:.1}ms avg, {:.1}% block rate",
                i + 1,
                m.hook_name,
                m.execution_count,
                m.avg_duration_ms(),
                m.block_rate()
            ));
        }
        lines.push(String::new());

        // Slowest hooks
        lines.push("## Slowest Hooks".to_string());
        for (i, m) in self.top_by_duration().iter().take(5).enumerate() {
            lines.push(format!(
                "{}. {} - {:.1}ms avg ({} executions)",
                i + 1,
                m.hook_name,
                m.avg_duration_ms(),
                m.execution_count
            ));
        }
        lines.push(String::new());

        // High false positive hooks
        let high_fp = self.high_false_positive_hooks(10.0);
        if !high_fp.is_empty() {
            lines.push("## High False Positive Hooks (>10%)".to_string());
            for m in high_fp {
                lines.push(format!(
                    "- {} - {:.1}% ({} overrides / {} blocks)",
                    m.hook_name,
                    m.false_positive_rate(),
                    m.override_count,
                    m.block_count
                ));
            }
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Decision;

    #[test]
    fn test_hook_metrics_new() {
        let metrics = HookMetrics::new("test_hook");
        assert_eq!(metrics.hook_name, "test_hook");
        assert_eq!(metrics.execution_count, 0);
    }

    #[test]
    fn test_hook_metrics_record() {
        let mut metrics = HookMetrics::new("test_hook");

        metrics.record_execution(Decision::Allow, 10);
        metrics.record_execution(Decision::Warn, 20);
        metrics.record_execution(Decision::Block, 30);

        assert_eq!(metrics.execution_count, 3);
        assert_eq!(metrics.allow_count, 1);
        assert_eq!(metrics.warn_count, 1);
        assert_eq!(metrics.block_count, 1);
        assert_eq!(metrics.total_duration_ms, 60);
    }

    #[test]
    fn test_hook_metrics_avg_duration() {
        let mut metrics = HookMetrics::new("test_hook");

        metrics.record_execution(Decision::Allow, 10);
        metrics.record_execution(Decision::Allow, 20);
        metrics.record_execution(Decision::Allow, 30);

        assert!((metrics.avg_duration_ms() - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_hook_metrics_block_rate() {
        let mut metrics = HookMetrics::new("test_hook");

        metrics.record_execution(Decision::Allow, 10);
        metrics.record_execution(Decision::Allow, 10);
        metrics.record_execution(Decision::Block, 10);
        metrics.record_execution(Decision::Block, 10);

        assert!((metrics.block_rate() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_hook_metrics_false_positive_rate() {
        let mut metrics = HookMetrics::new("test_hook");

        metrics.record_execution(Decision::Block, 10);
        metrics.record_execution(Decision::Block, 10);
        metrics.record_override();

        assert!((metrics.false_positive_rate() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_registry_record() {
        let mut registry = MetricsRegistry::default();

        registry.record("hook1", Decision::Allow, 10);
        registry.record("hook1", Decision::Block, 20);
        registry.record("hook2", Decision::Warn, 15);

        assert_eq!(registry.hooks.len(), 2);
        assert_eq!(registry.get("hook1").unwrap().execution_count, 2);
        assert_eq!(registry.get("hook2").unwrap().execution_count, 1);
    }

    #[test]
    fn test_registry_totals() {
        let mut registry = MetricsRegistry::default();

        registry.record("hook1", Decision::Allow, 10);
        registry.record("hook2", Decision::Allow, 20);
        registry.record("hook3", Decision::Allow, 30);

        assert_eq!(registry.total_executions(), 3);
        assert_eq!(registry.total_duration_ms(), 60);
    }
}
