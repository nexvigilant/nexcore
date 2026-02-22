// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Task classifier — maps task specifications to categories.
//!
//! ## Primitive Grounding: mapping(mu) + comparison(kappa)
//!
//! Classifies by comparing (kappa) task keywords against known
//! category signal words, then mapping (mu) to the strongest match.
//!
//! This is a rules engine, not an ML model. Latency is O(keywords).

use crate::types::TaskCategory;

/// Keyword-based task classifier.
///
/// Uses weighted keyword matching to assign a [`TaskCategory`].
/// Designed to be fast and deterministic — no model calls.
#[derive(Debug, Clone)]
pub struct TaskClassifier {
    /// Keyword → (category, weight) mappings
    rules: Vec<ClassifierRule>,
}

#[derive(Debug, Clone)]
struct ClassifierRule {
    keywords: Vec<&'static str>,
    category: TaskCategory,
    weight: f32,
}

impl Default for TaskClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskClassifier {
    /// Create classifier with default keyword rules.
    ///
    /// Rules derived from Phase 0 empirical analysis of 151K tool calls.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: vec![
                // Explore signals
                ClassifierRule {
                    keywords: vec![
                        "read", "search", "find", "look", "explore", "understand",
                        "analyze", "investigate", "scan", "check", "examine", "review",
                        "what is", "how does", "where is", "list",
                    ],
                    category: TaskCategory::Explore,
                    weight: 1.0,
                },
                // Mutate signals
                ClassifierRule {
                    keywords: vec![
                        "write", "create", "build", "implement", "add", "fix", "edit",
                        "modify", "update", "change", "refactor", "rename", "delete",
                        "remove", "replace", "generate", "code",
                    ],
                    category: TaskCategory::Mutate,
                    weight: 1.0,
                },
                // Orchestrate signals
                ClassifierRule {
                    keywords: vec![
                        "team", "parallel", "coordinate", "spawn", "delegate",
                        "orchestrate", "pipeline", "workflow", "batch", "multi",
                    ],
                    category: TaskCategory::Orchestrate,
                    weight: 1.2,
                },
                // Compute signals
                ClassifierRule {
                    keywords: vec![
                        "calculate", "compute", "signal", "detect", "prr", "ror",
                        "faers", "disproportionality", "statistics", "measure",
                        "score", "evaluate", "benchmark",
                    ],
                    category: TaskCategory::Compute,
                    weight: 1.1,
                },
                // Verify signals
                ClassifierRule {
                    keywords: vec![
                        "test", "verify", "validate", "assert", "check", "lint",
                        "clippy", "build", "compile", "ci", "gate",
                    ],
                    category: TaskCategory::Verify,
                    weight: 1.0,
                },
                // Browse signals
                ClassifierRule {
                    keywords: vec![
                        "browser", "chrome", "navigate", "click", "screenshot",
                        "page", "tab", "website", "url", "dom",
                    ],
                    category: TaskCategory::Browse,
                    weight: 1.3,
                },
            ],
        }
    }

    /// Classify a task specification into a category.
    ///
    /// Returns `TaskCategory::Mixed` if no single category dominates
    /// (two or more categories each have >30% of total score).
    #[must_use]
    pub fn classify(&self, task_spec: &str) -> TaskCategory {
        if task_spec.trim().is_empty() {
            return TaskCategory::Mixed;
        }

        let lower = task_spec.to_lowercase();
        let mut scores: Vec<(TaskCategory, f32)> = Vec::new();

        for rule in &self.rules {
            let mut score = 0.0f32;
            for kw in &rule.keywords {
                if lower.contains(kw) {
                    score += rule.weight;
                }
            }
            if score > 0.0 {
                scores.push((rule.category, score));
            }
        }

        if scores.is_empty() {
            return TaskCategory::Mixed;
        }

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let total: f32 = scores.iter().map(|(_, s)| s).sum();

        // Check for mixed: if top two both >30% of total
        if scores.len() >= 2 {
            let top_pct = scores[0].1 / total;
            let second_pct = scores[1].1 / total;
            if top_pct < 0.5 && second_pct > 0.3 {
                return TaskCategory::Mixed;
            }
        }

        scores[0].0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explore_classification() {
        let c = TaskClassifier::new();
        assert_eq!(c.classify("read the file and understand the architecture"), TaskCategory::Explore);
    }

    #[test]
    fn test_mutate_classification() {
        let c = TaskClassifier::new();
        assert_eq!(c.classify("implement a new feature and add tests"), TaskCategory::Mutate);
    }

    #[test]
    fn test_compute_classification() {
        let c = TaskClassifier::new();
        assert_eq!(c.classify("calculate PRR and ROR for the drug signal"), TaskCategory::Compute);
    }

    #[test]
    fn test_orchestrate_classification() {
        let c = TaskClassifier::new();
        assert_eq!(c.classify("spawn a team to coordinate parallel batch fixes"), TaskCategory::Orchestrate);
    }

    #[test]
    fn test_verify_classification() {
        let c = TaskClassifier::new();
        assert_eq!(c.classify("run tests and validate the build passes lint"), TaskCategory::Verify);
    }

    #[test]
    fn test_browse_classification() {
        let c = TaskClassifier::new();
        assert_eq!(c.classify("navigate to the website and take a screenshot"), TaskCategory::Browse);
    }

    #[test]
    fn test_empty_is_mixed() {
        let c = TaskClassifier::new();
        assert_eq!(c.classify(""), TaskCategory::Mixed);
    }

    #[test]
    fn test_mixed_classification() {
        let c = TaskClassifier::new();
        // Three-way split: explore(1.0) + mutate(1.0) + compute(1.1)
        // No single category exceeds 50%, second exceeds 30% → Mixed
        assert_eq!(
            c.classify("search then change and calculate"),
            TaskCategory::Mixed
        );
    }
}
