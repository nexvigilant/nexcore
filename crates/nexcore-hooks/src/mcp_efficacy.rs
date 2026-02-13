//! MCP Tool Efficacy Tracking (CTVP Phase 2)
//!
//! Tracks the effectiveness of MCP tool suggestions by measuring when suggestions
//! are made versus when MCP tools are actually used.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

/// Event: MCP tool suggestion was made
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSuggestionEvent {
    /// Unix timestamp
    pub timestamp: f64,
    /// Session that received the suggestion
    pub session_id: String,
    /// Tools that were suggested
    pub suggested_tools: Vec<String>,
    /// Keywords that triggered the suggestion
    pub matched_keywords: Vec<String>,
    /// Category of the suggestion
    pub category: String,
}

/// Event: MCP tool was actually used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpUsageEvent {
    /// Unix timestamp
    pub timestamp: f64,
    /// Session that used the tool
    pub session_id: String,
    /// Tool that was used
    pub tool_name: String,
    /// Whether this followed a suggestion
    pub followed_suggestion: bool,
}

/// Efficacy metrics for a specific tool
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolMetrics {
    /// Number of times this tool was suggested
    pub suggestion_count: u32,
    /// Number of times this tool was used
    pub usage_count: u32,
    /// Number of times usage followed a suggestion
    pub adopted_count: u32,
}

impl ToolMetrics {
    /// Returns the adoption rate: adopted / suggested
    ///
    /// # Returns
    /// Adoption rate as f64 between 0.0 and 1.0
    pub fn adoption_rate(&self) -> f64 {
        if self.suggestion_count == 0 {
            0.0
        } else {
            self.adopted_count as f64 / self.suggestion_count as f64
        }
    }
}

/// Overall efficacy metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EfficacyMetrics {
    /// Total sessions where suggestions were made
    pub sessions_with_suggestions: u32,
    /// Sessions where MCP tools were used after suggestion
    pub sessions_with_followup_usage: u32,
    /// Total suggestions made
    pub total_suggestions: u32,
    /// Total MCP tool usages
    pub total_usages: u32,
    /// Per-tool metrics
    pub tool_metrics: HashMap<String, ToolMetrics>,
    /// Time period start
    pub period_start: f64,
    /// Time period end
    pub period_end: f64,
}

impl EfficacyMetrics {
    /// Returns the Capability Achievement Rate (CAR)
    ///
    /// # Returns
    /// CAR as f64 between 0.0 and 1.0
    pub fn capability_achievement_rate(&self) -> f64 {
        if self.sessions_with_suggestions == 0 {
            0.0
        } else {
            self.sessions_with_followup_usage as f64 / self.sessions_with_suggestions as f64
        }
    }

    /// Returns the overall adoption rate across all tools
    ///
    /// # Returns
    /// Adoption rate as f64 between 0.0 and 1.0
    pub fn overall_adoption_rate(&self) -> f64 {
        if self.total_suggestions == 0 {
            0.0
        } else {
            self.tool_metrics
                .values()
                .map(|m| m.adopted_count)
                .sum::<u32>() as f64
                / self.total_suggestions as f64
        }
    }
}

/// The MCP Efficacy Registry - persists suggestion and usage events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpEfficacyRegistry {
    /// All suggestion events
    pub suggestions: Vec<McpSuggestionEvent>,
    /// All usage events
    pub usages: Vec<McpUsageEvent>,
    /// Last updated timestamp
    pub last_updated: f64,
}

impl McpEfficacyRegistry {
    /// Returns the registry file path
    ///
    /// # Returns
    /// PathBuf to ~/.claude/mcp_efficacy.json
    pub fn registry_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("mcp_efficacy.json")
    }

    /// Loads registry from disk
    ///
    /// # Returns
    /// Loaded registry or default if not found
    pub fn load() -> Self {
        let path = Self::registry_path();
        if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Saves registry to disk atomically
    ///
    /// # Errors
    /// Returns error if write fails
    pub fn save(&mut self) -> Result<(), std::io::Error> {
        self.last_updated = crate::state::now();
        let path = Self::registry_path();
        if let Some(p) = path.parent() {
            fs::create_dir_all(p)?;
        }
        let tmp = path.with_extension("json.tmp");
        fs::write(
            &tmp,
            serde_json::to_string_pretty(self).map_err(|e| std::io::Error::other(e.to_string()))?,
        )?;
        fs::rename(tmp, path)
    }

    /// Records a suggestion event
    ///
    /// # Arguments
    /// * `session_id` - Session identifier
    /// * `tools` - Tools that were suggested
    /// * `keywords` - Keywords that triggered the suggestion
    /// * `category` - Category of the suggestion
    pub fn record_suggestion(
        &mut self,
        session_id: &str,
        tools: Vec<String>,
        keywords: Vec<String>,
        category: &str,
    ) {
        self.suggestions.push(McpSuggestionEvent {
            timestamp: crate::state::now(),
            session_id: session_id.to_string(),
            suggested_tools: tools,
            matched_keywords: keywords,
            category: category.to_string(),
        });
    }

    /// Records an MCP tool usage event
    ///
    /// # Arguments
    /// * `session_id` - Session identifier
    /// * `tool_name` - Name of the tool used
    pub fn record_usage(&mut self, session_id: &str, tool_name: &str) {
        let index: HashMap<&str, HashSet<&str>> = self
            .suggestions
            .iter()
            .map(|s| {
                (
                    s.session_id.as_str(),
                    s.suggested_tools.iter().map(String::as_str).collect(),
                )
            })
            .fold(
                HashMap::new(),
                |mut acc, (sid, tools): (&str, HashSet<&str>)| {
                    acc.entry(sid).or_default().extend(tools);
                    acc
                },
            );
        let followed = index
            .get(session_id)
            .is_some_and(|tools| tools.contains(tool_name));
        self.usages.push(McpUsageEvent {
            timestamp: crate::state::now(),
            session_id: session_id.to_string(),
            tool_name: tool_name.to_string(),
            followed_suggestion: followed,
        });
    }

    /// Generates an efficacy report
    ///
    /// # Arguments
    /// * `hours` - Optional time window in hours (None = all time)
    ///
    /// # Returns
    /// Formatted report string
    pub fn report(&self, hours: Option<f64>) -> String {
        let m = compute_metrics_impl(&self.suggestions, &self.usages, hours);
        let period = hours.map_or_else(|| "All time".to_string(), |h| format!("Last {h} hours"));
        format!(
            "\n📊 MCP EFFICACY ({period})\n  CAR: {:.1}%\n  Sessions: {} suggested, {} followed\n  Status: {}\n",
            m.capability_achievement_rate() * 100.0,
            m.sessions_with_suggestions,
            m.sessions_with_followup_usage,
            if m.sessions_with_suggestions < 10 {
                "⚠️ Need more data"
            } else if m.capability_achievement_rate() >= 0.8 {
                "✅ Phase 2 validated"
            } else {
                "❌ Below threshold"
            }
        )
    }

    /// Removes events older than specified days
    ///
    /// # Arguments
    /// * `days` - Number of days to retain
    pub fn cleanup(&mut self, days: u32) {
        let cutoff = crate::state::now() - (days as f64 * 86400.0);
        self.suggestions.retain(|s| s.timestamp >= cutoff);
        self.usages.retain(|u| u.timestamp >= cutoff);
    }
}

/// Computes efficacy metrics from filtered data
fn compute_metrics_impl(
    suggestions: &[McpSuggestionEvent],
    usages: &[McpUsageEvent],
    hours: Option<f64>,
) -> EfficacyMetrics {
    let now = crate::state::now();
    let cutoff = hours.map_or(0.0, |h| now - h * 3600.0);

    let filtered_suggestions: Vec<_> = suggestions
        .iter()
        .filter(|s| s.timestamp >= cutoff)
        .collect();
    let sessions_sug: HashSet<_> = filtered_suggestions.iter().map(|s| &s.session_id).collect();
    let tool_counts: HashMap<String, u32> = filtered_suggestions
        .iter()
        .flat_map(|s| s.suggested_tools.iter().cloned())
        .fold(HashMap::new(), |mut acc, tool| {
            *acc.entry(tool).or_insert(0) += 1;
            acc
        });

    let filtered_usages: Vec<_> = usages.iter().filter(|u| u.timestamp >= cutoff).collect();
    let sessions_follow: HashSet<_> = filtered_usages
        .iter()
        .filter(|u| u.followed_suggestion)
        .map(|u| &u.session_id)
        .collect();

    let mut tool_metrics: HashMap<String, ToolMetrics> = HashMap::new();
    tool_counts.into_iter().for_each(|(tool, count)| {
        tool_metrics.entry(tool).or_default().suggestion_count = count;
    });
    filtered_usages.iter().for_each(|u| {
        let m = tool_metrics.entry(u.tool_name.clone()).or_default();
        m.usage_count += 1;
        if u.followed_suggestion {
            m.adopted_count += 1;
        }
    });

    EfficacyMetrics {
        sessions_with_suggestions: sessions_sug.len() as u32,
        sessions_with_followup_usage: sessions_follow.len() as u32,
        total_suggestions: filtered_suggestions.len() as u32,
        total_usages: filtered_usages.len() as u32,
        tool_metrics,
        period_start: cutoff,
        period_end: now,
    }
}

/// Helper for atomic registry operations
///
/// # Arguments
/// * `f` - Function to apply to the registry
///
/// # Errors
/// Returns error if save fails
pub fn with_efficacy_registry<F, T>(f: F) -> Result<T, std::io::Error>
where
    F: FnOnce(&mut McpEfficacyRegistry) -> T,
{
    let mut r = McpEfficacyRegistry::load();
    let result = f(&mut r);
    r.save()?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_car_with_followup() {
        let mut r = McpEfficacyRegistry::default();
        r.record_suggestion("s1", vec!["t1".into()], vec![], "C");
        r.record_usage("s1", "t1");
        r.record_suggestion("s2", vec!["t2".into()], vec![], "C");
        let m = compute_metrics_impl(&r.suggestions, &r.usages, None);
        assert_eq!(m.sessions_with_suggestions, 2);
        assert_eq!(m.sessions_with_followup_usage, 1);
        assert!((m.capability_achievement_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_car_perfect_adoption() {
        let mut r = McpEfficacyRegistry::default();
        r.record_suggestion("s1", vec!["t1".into()], vec![], "C");
        r.record_usage("s1", "t1");
        r.record_suggestion("s2", vec!["t2".into()], vec![], "C");
        r.record_usage("s2", "t2");
        let m = compute_metrics_impl(&r.suggestions, &r.usages, None);
        assert_eq!(m.capability_achievement_rate(), 1.0);
    }

    #[test]
    fn test_car_zero_adoption() {
        let mut r = McpEfficacyRegistry::default();
        r.record_suggestion("s1", vec!["t1".into()], vec![], "C");
        r.record_suggestion("s2", vec!["t2".into()], vec![], "C");
        r.record_usage("s3", "t3");
        let m = compute_metrics_impl(&r.suggestions, &r.usages, None);
        assert_eq!(m.capability_achievement_rate(), 0.0);
    }

    #[test]
    fn test_empty_registry() {
        let r = McpEfficacyRegistry::default();
        let m = compute_metrics_impl(&r.suggestions, &r.usages, None);
        assert_eq!(m.capability_achievement_rate(), 0.0);
        assert_eq!(m.overall_adoption_rate(), 0.0);
    }

    #[test]
    fn test_tool_metrics() {
        let mut r = McpEfficacyRegistry::default();
        r.record_suggestion("s1", vec!["t1".into()], vec![], "C");
        r.record_suggestion("s2", vec!["t1".into()], vec![], "C");
        r.record_suggestion("s3", vec!["t1".into()], vec![], "C");
        r.record_usage("s1", "t1");
        r.record_usage("s2", "t1");
        let m = compute_metrics_impl(&r.suggestions, &r.usages, None);
        // Verify via Option::map_or to avoid panic paths
        let suggestion_count = m.tool_metrics.get("t1").map_or(0, |tm| tm.suggestion_count);
        let adopted_count = m.tool_metrics.get("t1").map_or(0, |tm| tm.adopted_count);
        assert_eq!(suggestion_count, 3);
        assert_eq!(adopted_count, 2);
    }

    #[test]
    fn test_report_format() {
        let mut r = McpEfficacyRegistry::default();
        r.record_suggestion("s1", vec!["t1".into()], vec![], "C");
        r.record_usage("s1", "t1");
        let report = r.report(None);
        assert!(report.contains("MCP EFFICACY"));
        assert!(report.contains("CAR:"));
    }

    #[test]
    fn test_ctvp_threshold_check() {
        let mut r = McpEfficacyRegistry::default();
        for i in 0..12 {
            let sid = format!("s{i}");
            r.record_suggestion(&sid, vec!["t1".into()], vec![], "C");
            if i < 10 {
                r.record_usage(&sid, "t1");
            }
        }
        let report = r.report(None);
        assert!(report.contains("Phase 2 validated"));
    }
}
