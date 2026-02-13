//! Feedback loop for metrics collection with disk persistence.
//!
//! Persists execution metrics to disk for learning across sessions:
//! - `chain_metrics.jsonl` - Append-only log of all executions
//! - `skill_stats.json` - Aggregated skill success rates and chain rankings

use super::models::{AndonSignal, Chain, ChainMetrics, ExecutionResult, ExecutionStatus};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Cached statistics persisted to disk.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CachedStats {
    skill_success_rates: HashMap<String, f64>,
    chain_rankings: HashMap<String, f64>,
    #[serde(default)]
    updated_at: Option<String>,
}

/// Feedback loop for learning from executions.
pub struct FeedbackLoop {
    metrics_dir: PathBuf,
    metrics_file: PathBuf,
    stats_file: PathBuf,
    /// Skill success rates (exponential moving average)
    pub skill_success_rates: HashMap<String, f64>,
    /// Chain rankings
    pub chain_rankings: HashMap<String, f64>,
}

impl FeedbackLoop {
    /// Create a new feedback loop with disk persistence.
    #[must_use]
    pub fn new(metrics_dir: &Path) -> Self {
        std::fs::create_dir_all(metrics_dir).unwrap_or_default();

        let metrics_file = metrics_dir.join("chain_metrics.jsonl");
        let stats_file = metrics_dir.join("skill_stats.json");

        let mut feedback = Self {
            metrics_dir: metrics_dir.to_path_buf(),
            metrics_file,
            stats_file,
            skill_success_rates: HashMap::new(),
            chain_rankings: HashMap::new(),
        };

        // Load cached stats from disk
        feedback.load_cached_stats();

        feedback
    }

    /// Load cached statistics from disk.
    fn load_cached_stats(&mut self) {
        if !self.stats_file.exists() {
            return;
        }

        let file = match File::open(&self.stats_file) {
            Ok(f) => f,
            Err(_) => return,
        };

        let reader = BufReader::new(file);
        if let Ok(stats) = serde_json::from_reader::<_, CachedStats>(reader) {
            self.skill_success_rates = stats.skill_success_rates;
            self.chain_rankings = stats.chain_rankings;
        }
    }

    /// Save cached statistics to disk.
    fn save_cached_stats(&self) {
        let stats = CachedStats {
            skill_success_rates: self.skill_success_rates.clone(),
            chain_rankings: self.chain_rankings.clone(),
            updated_at: Some(Utc::now().to_rfc3339()),
        };

        let file = match File::create(&self.stats_file) {
            Ok(f) => f,
            Err(_) => return,
        };

        let _ = serde_json::to_writer_pretty(file, &stats);
    }

    /// Append a metrics record to the JSONL file.
    fn append_metrics(&self, metrics: &ChainMetrics) {
        let file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.metrics_file)
        {
            Ok(f) => f,
            Err(_) => return,
        };

        let mut writer = std::io::BufWriter::new(file);
        if let Ok(json) = serde_json::to_string(metrics) {
            let _ = writeln!(writer, "{json}");
        }
    }

    /// Record execution results for learning.
    pub fn record(&mut self, chain: &Chain, result: &ExecutionResult) {
        let chain_expr = self.to_expression(chain);

        // Create metrics record
        let metrics = ChainMetrics {
            chain_expression: chain_expr.clone(),
            domain: chain
                .analysis
                .as_ref()
                .map(|a| a.domain.clone())
                .unwrap_or_else(|| "general".to_string()),
            success: result.status == ExecutionStatus::Completed,
            duration_seconds: result.total_duration_seconds,
            skills_used: chain.nodes.iter().map(|n| n.skill_name.clone()).collect(),
            skill_success_rates: result
                .skill_results
                .iter()
                .map(|r| {
                    (
                        r.skill_name.clone(),
                        if r.signal == AndonSignal::Green {
                            1.0
                        } else {
                            0.0
                        },
                    )
                })
                .collect(),
        };

        // Append to JSONL file
        self.append_metrics(&metrics);

        // Update skill stats with exponential moving average
        for res in &result.skill_results {
            let success = res.signal == AndonSignal::Green;
            let current = self
                .skill_success_rates
                .entry(res.skill_name.clone())
                .or_insert(0.5);
            *current = 0.8 * (*current) + 0.2 * (if success { 1.0 } else { 0.0 });
        }

        // Update chain ranking
        let success = result.status == ExecutionStatus::Completed;
        let current = self.chain_rankings.entry(chain_expr).or_insert(0.5);
        *current = 0.8 * (*current) + 0.2 * (if success { 1.0 } else { 0.0 });

        // Persist to disk
        self.save_cached_stats();
    }

    /// Get metrics directory path.
    #[must_use]
    pub fn metrics_dir(&self) -> &Path {
        &self.metrics_dir
    }

    /// Get historical success rate for a skill.
    #[must_use]
    pub fn get_skill_success_rate(&self, skill_name: &str) -> f64 {
        self.skill_success_rates
            .get(skill_name)
            .copied()
            .unwrap_or(0.5)
    }

    /// Get historical ranking for a chain expression.
    #[must_use]
    pub fn get_chain_ranking(&self, chain_expr: &str) -> f64 {
        self.chain_rankings.get(chain_expr).copied().unwrap_or(0.5)
    }

    /// Suggest improvements based on historical data.
    #[must_use]
    pub fn suggest_improvements(&self, chain: &Chain) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        let common_pairs = self.get_common_pairs();

        for node in &chain.nodes {
            let rate = self.get_skill_success_rate(&node.skill_name);

            // Warn about low success rate skills
            if rate < 0.6 {
                suggestions.push(Suggestion {
                    kind: SuggestionKind::Warn,
                    skill: node.skill_name.clone(),
                    message: format!("{} has {:.0}% success rate", node.skill_name, rate * 100.0),
                    recommendation: "Consider alternative skill or additional validation"
                        .to_string(),
                });
            }

            // Suggest missing common pairings
            if let Some(pairs) = common_pairs.get(node.skill_name.as_str()) {
                for pair in pairs {
                    if !chain.nodes.iter().any(|n| n.skill_name == *pair) {
                        suggestions.push(Suggestion {
                            kind: SuggestionKind::Add,
                            skill: (*pair).to_string(),
                            message: format!("{pair} commonly follows {}", node.skill_name),
                            recommendation: format!(
                                "Consider adding {pair} after {}",
                                node.skill_name
                            ),
                        });
                    }
                }
            }
        }

        suggestions
    }

    /// Get commonly paired skills.
    fn get_common_pairs(&self) -> HashMap<&'static str, Vec<&'static str>> {
        let mut pairs = HashMap::new();
        pairs.insert("algorithm", vec!["proceed-lite", "proceed"]);
        pairs.insert("explore", vec!["bluf-report", "algorithm"]);
        pairs.insert("proceed-lite", vec!["commit"]);
        pairs.insert("proceed", vec!["commit"]);
        pairs.insert("lint", vec!["typecheck", "test"]);
        pairs.insert("typecheck", vec!["lint", "test"]);
        pairs
    }

    /// Get top N chains by ranking.
    #[must_use]
    pub fn get_top_chains(&self, n: usize) -> Vec<(&str, f64)> {
        let mut sorted: Vec<_> = self.chain_rankings.iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted
            .into_iter()
            .take(n)
            .map(|(k, v)| (k.as_str(), *v))
            .collect()
    }

    /// Get top N skills by success rate.
    #[must_use]
    pub fn get_skill_leaderboard(&self, n: usize) -> Vec<(&str, f64)> {
        let mut sorted: Vec<_> = self.skill_success_rates.iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted
            .into_iter()
            .take(n)
            .map(|(k, v)| (k.as_str(), *v))
            .collect()
    }

    /// Get summary statistics.
    #[must_use]
    pub fn get_stats_summary(&self) -> StatsSummary {
        let avg_skill_success = if self.skill_success_rates.is_empty() {
            0.5
        } else {
            self.skill_success_rates.values().sum::<f64>() / self.skill_success_rates.len() as f64
        };

        StatsSummary {
            total_skills_tracked: self.skill_success_rates.len(),
            total_chains_tracked: self.chain_rankings.len(),
            avg_skill_success_rate: avg_skill_success,
            top_chains: self
                .get_top_chains(3)
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            top_skills: self
                .get_skill_leaderboard(5)
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        }
    }

    /// Read metrics history from JSONL file.
    #[must_use]
    pub fn read_metrics_history(&self, limit: usize) -> Vec<ChainMetrics> {
        if !self.metrics_file.exists() {
            return Vec::new();
        }

        let file = match File::open(&self.metrics_file) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let reader = BufReader::new(file);
        let mut metrics: Vec<ChainMetrics> = reader
            .lines()
            .map_while(Result::ok)
            .filter_map(|line| serde_json::from_str(&line).ok())
            .collect();

        // Return most recent entries
        if metrics.len() > limit {
            metrics.drain(0..metrics.len() - limit);
        }

        metrics
    }

    fn to_expression(&self, chain: &Chain) -> String {
        chain
            .nodes
            .iter()
            .map(|n| n.skill_name.clone())
            .collect::<Vec<_>>()
            .join(" -> ")
    }
}

/// A suggestion for improving a chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub kind: SuggestionKind,
    pub skill: String,
    pub message: String,
    pub recommendation: String,
}

/// The type of suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SuggestionKind {
    Warn,
    Add,
}

/// Summary statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsSummary {
    pub total_skills_tracked: usize,
    pub total_chains_tracked: usize,
    pub avg_skill_success_rate: f64,
    pub top_chains: Vec<(String, f64)>,
    pub top_skills: Vec<(String, f64)>,
}
