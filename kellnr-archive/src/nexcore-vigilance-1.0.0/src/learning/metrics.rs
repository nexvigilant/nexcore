//! Performance metrics and A/B testing evaluation.

use chrono::{DateTime, Utc};
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};

/// Performance metrics for a single task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub task_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub completion_time_ms: u64,
    pub success: bool,
    pub skills_used: Vec<String>,
    pub errors_encountered: u32,
    pub interventions_received: u32,
}

/// Key Performance Indicators for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KPIs {
    /// Used skills / Available skills
    pub skill_adoption_rate: f64,
    /// Average task completion time
    pub avg_completion_time_ms: f64,
    /// Success rate on repeated tasks
    pub knowledge_retention_score: f64,
    /// Time saved compared to baseline
    pub efficiency_gain: f64,
}

/// A/B test variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Variant {
    /// No teaching ecosystem
    Control,
    /// With teaching ecosystem
    Treatment,
}

/// A/B test session data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestSession {
    pub session_id: NexId,
    pub variant: Variant,
    pub user_id: String,
    pub metrics: Vec<PerformanceMetrics>,
}

/// A/B test results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestResult {
    pub control_kpis: KPIs,
    pub treatment_kpis: KPIs,
    /// Percentage improvement
    pub lift: f64,
    pub statistical_significance: bool,
}

/// Evaluation engine for A/B testing.
pub struct EvaluationEngine {
    sessions: Vec<ABTestSession>,
}

impl Default for EvaluationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EvaluationEngine {
    /// Create a new evaluation engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }

    /// Record a test session.
    pub fn record_session(&mut self, session: ABTestSession) {
        self.sessions.push(session);
    }

    /// Get all recorded sessions.
    #[must_use]
    pub fn sessions(&self) -> &[ABTestSession] {
        &self.sessions
    }

    /// Calculate A/B test results.
    #[must_use]
    pub fn calculate_results(&self) -> ABTestResult {
        let control_metrics: Vec<&PerformanceMetrics> = self
            .sessions
            .iter()
            .filter(|s| s.variant == Variant::Control)
            .flat_map(|s| &s.metrics)
            .collect();

        let treatment_metrics: Vec<&PerformanceMetrics> = self
            .sessions
            .iter()
            .filter(|s| s.variant == Variant::Treatment)
            .flat_map(|s| &s.metrics)
            .collect();

        let control_kpis = Self::compute_kpis(&control_metrics);
        let treatment_kpis = Self::compute_kpis(&treatment_metrics);

        let lift = if control_kpis.avg_completion_time_ms > 0.0 {
            (control_kpis.avg_completion_time_ms - treatment_kpis.avg_completion_time_ms)
                / control_kpis.avg_completion_time_ms
        } else {
            0.0
        };

        ABTestResult {
            control_kpis,
            treatment_kpis,
            lift,
            statistical_significance: lift > 0.05,
        }
    }

    fn compute_kpis(metrics: &[&PerformanceMetrics]) -> KPIs {
        if metrics.is_empty() {
            return KPIs::default();
        }

        let total_time: u64 = metrics.iter().map(|m| m.completion_time_ms).sum();
        let avg_time = total_time as f64 / metrics.len() as f64;

        let success_count = metrics.iter().filter(|m| m.success).count();
        let success_rate = success_count as f64 / metrics.len() as f64;

        KPIs {
            avg_completion_time_ms: avg_time,
            knowledge_retention_score: success_rate,
            ..KPIs::default()
        }
    }
}
