//! # CEP Feedback System
//!
//! Feedback signals and aggregation for the IMPROVE → SEE loop.

use super::stages::StageId;
use serde::{Deserialize, Serialize};

/// Priority level for feedback signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FeedbackPriority {
    /// Low priority - nice to address.
    Low = 1,
    /// Medium priority - should address.
    Medium = 2,
    /// High priority - must address.
    High = 3,
    /// Critical - blocks further progress.
    Critical = 4,
}

impl Default for FeedbackPriority {
    fn default() -> Self {
        Self::Medium
    }
}

/// Source of a feedback signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackSource {
    /// Automated validation check.
    Validation {
        /// Which metric triggered feedback.
        metric: String,
        /// Actual value.
        actual: f64,
        /// Required threshold.
        threshold: f64,
    },
    /// Stage execution issue.
    StageExecution {
        /// Which stage.
        stage: StageId,
        /// What went wrong.
        issue: String,
    },
    /// Translation gap.
    TranslationGap {
        /// Unmapped concept.
        concept: String,
        /// Suggested action.
        suggestion: Option<String>,
    },
    /// User/expert feedback.
    Expert {
        /// Who provided feedback.
        source: String,
        /// Category.
        category: String,
    },
    /// System observation.
    System {
        /// Component.
        component: String,
    },
}

/// A feedback signal from any CEP stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSignal {
    /// Unique identifier.
    pub id: String,
    /// Which stage generated this feedback.
    pub source_stage: StageId,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Priority level.
    pub priority: FeedbackPriority,
    /// Feedback source details.
    pub source: FeedbackSource,
    /// Human-readable message.
    pub message: String,
    /// Suggested improvement.
    #[serde(default)]
    pub suggestion: Option<String>,
    /// Metadata.
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

impl FeedbackSignal {
    /// Creates a new feedback signal.
    #[must_use]
    pub fn new(
        source_stage: StageId,
        priority: FeedbackPriority,
        source: FeedbackSource,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: nexcore_id::NexId::v4().to_string(),
            source_stage,
            timestamp: chrono::Utc::now(),
            priority,
            source,
            message: message.into(),
            suggestion: None,
            metadata: None,
        }
    }

    /// Creates a validation feedback signal.
    #[must_use]
    pub fn validation(
        stage: StageId,
        metric: impl Into<String>,
        actual: f64,
        threshold: f64,
    ) -> Self {
        let metric_name = metric.into();
        let message = format!(
            "{} below threshold: {:.2} < {:.2}",
            metric_name, actual, threshold
        );
        let priority = if actual < threshold * 0.8 {
            FeedbackPriority::Critical
        } else if actual < threshold * 0.9 {
            FeedbackPriority::High
        } else {
            FeedbackPriority::Medium
        };

        Self::new(
            stage,
            priority,
            FeedbackSource::Validation {
                metric: metric_name,
                actual,
                threshold,
            },
            message,
        )
    }

    /// Creates a stage execution feedback signal.
    #[must_use]
    pub fn stage_issue(stage: StageId, issue: impl Into<String>) -> Self {
        let issue_str = issue.into();
        Self::new(
            stage,
            FeedbackPriority::High,
            FeedbackSource::StageExecution {
                stage,
                issue: issue_str.clone(),
            },
            issue_str,
        )
    }

    /// Creates a translation gap feedback signal.
    #[must_use]
    pub fn translation_gap(stage: StageId, concept: impl Into<String>) -> Self {
        let concept_name = concept.into();
        Self::new(
            stage,
            FeedbackPriority::Medium,
            FeedbackSource::TranslationGap {
                concept: concept_name.clone(),
                suggestion: None,
            },
            format!("No mapping found for concept: {}", concept_name),
        )
    }

    /// Adds a suggestion.
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Aggregator for feedback signals.
#[derive(Debug, Clone, Default)]
pub struct FeedbackAggregator {
    /// All collected signals.
    signals: Vec<FeedbackSignal>,
}

impl FeedbackAggregator {
    /// Creates a new aggregator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a signal.
    pub fn add(&mut self, signal: FeedbackSignal) {
        self.signals.push(signal);
    }

    /// Returns all signals.
    #[must_use]
    pub fn signals(&self) -> &[FeedbackSignal] {
        &self.signals
    }

    /// Returns signals filtered by stage.
    pub fn by_stage(&self, stage: StageId) -> impl Iterator<Item = &FeedbackSignal> {
        self.signals.iter().filter(move |s| s.source_stage == stage)
    }

    /// Returns signals filtered by priority.
    pub fn by_priority(
        &self,
        min_priority: FeedbackPriority,
    ) -> impl Iterator<Item = &FeedbackSignal> {
        self.signals
            .iter()
            .filter(move |s| s.priority >= min_priority)
    }

    /// Returns critical signals.
    pub fn critical(&self) -> impl Iterator<Item = &FeedbackSignal> {
        self.by_priority(FeedbackPriority::Critical)
    }

    /// Returns signals sorted by priority (highest first).
    #[must_use]
    pub fn sorted_by_priority(&self) -> Vec<&FeedbackSignal> {
        let mut sorted: Vec<_> = self.signals.iter().collect();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
        sorted
    }

    /// Counts signals by stage.
    #[must_use]
    pub fn count_by_stage(&self) -> std::collections::HashMap<StageId, usize> {
        let mut counts = std::collections::HashMap::new();
        for signal in &self.signals {
            *counts.entry(signal.source_stage).or_default() += 1;
        }
        counts
    }

    /// Returns the stage with most feedback signals.
    #[must_use]
    pub fn hottest_stage(&self) -> Option<(StageId, usize)> {
        self.count_by_stage()
            .into_iter()
            .max_by_key(|(_, count)| *count)
    }

    /// Clears all signals.
    pub fn clear(&mut self) {
        self.signals.clear();
    }

    /// Returns total signal count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.signals.len()
    }

    /// Checks if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_signal_creation() {
        let signal = FeedbackSignal::validation(StageId::Validate, "coverage", 0.85, 0.95);
        assert_eq!(signal.source_stage, StageId::Validate);
        assert_eq!(signal.priority, FeedbackPriority::High); // 0.85 < 0.95 * 0.9
    }

    #[test]
    fn test_aggregator() {
        let mut agg = FeedbackAggregator::new();
        agg.add(FeedbackSignal::validation(
            StageId::Validate,
            "coverage",
            0.85,
            0.95,
        ));
        agg.add(FeedbackSignal::stage_issue(
            StageId::Decompose,
            "Failed to parse",
        ));
        agg.add(FeedbackSignal::translation_gap(
            StageId::Translate,
            "harm_type",
        ));

        assert_eq!(agg.len(), 3);
        assert_eq!(agg.by_stage(StageId::Validate).count(), 1);
    }

    #[test]
    fn test_priority_sorting() {
        let mut agg = FeedbackAggregator::new();
        agg.add(FeedbackSignal::new(
            StageId::See,
            FeedbackPriority::Low,
            FeedbackSource::System {
                component: "test".into(),
            },
            "Low priority",
        ));
        agg.add(FeedbackSignal::new(
            StageId::See,
            FeedbackPriority::Critical,
            FeedbackSource::System {
                component: "test".into(),
            },
            "Critical",
        ));

        let sorted = agg.sorted_by_priority();
        assert_eq!(sorted[0].priority, FeedbackPriority::Critical);
    }
}
