// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # SMART Goals for Value Intelligence
//!
//! Structured goal tracking for the value intelligence system using
//! the SMART framework (Specific, Measurable, Achievable, Relevant, Time-bound).
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │              IntelligenceGoal (T2-C)              │
//! ├────────────┬──────────┬─────────┬───────────────┤
//! │  GoalMetric │ Progress │ Status  │  Priority     │
//! │  (what)     │ (where)  │ (state) │  (P0-P5)      │
//! ├────────────┴──────────┴─────────┴───────────────┤
//! │  GoalPortfolio — collection of active goals      │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ## Primitive Grounding
//!
//! | Symbol | Primitive | Role |
//! |:------:|:----------|:-----|
//! | **π** | Persistence | Goal state persistence (dominant) |
//! | **N** | Quantity | Measurable target values |
//! | **∂** | Boundary | Target thresholds |
//! | **ς** | State | Goal status lifecycle |
//! | **σ** | Sequence | Time-bound deadlines |
//! | **κ** | Comparison | Progress vs target |
//!
//! ## Default Goal Set
//!
//! | # | Metric | Target | Deadline | Priority |
//! |---|--------|--------|----------|----------|
//! | G1 | DetectionCoverage | 5/5 signal types | 7 days | P1 |
//! | G2 | ConvergenceThreshold | ≥ 0.7 | 14 days | P1 |
//! | G3 | TimeToIntelligence | ≤ 3600s (1h) | 30 days | P4 |
//! | G4 | SourceDiversityMin | ≥ 3 sources | 14 days | P2 |
//! | G5 | ConfidenceFloor | ≥ 0.7 | 7 days | P1 |
//! | G6 | SignalFreshness | ≤ 1800s (30m) | 30 days | P3 |
//! | G7 | ActionableRate | ≥ 0.3 (30%) | 30 days | P4 |
//! | G8 | FalsePositiveRate | ≤ 0.1 (10%) | 60 days | P0 |

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::intelligence::MonitoringDashboard;

// ============================================================================
// Goal Metric — What is measured
// ============================================================================

/// Measurable metric for intelligence system goals.
///
/// Each metric maps to a specific field in [`MonitoringDashboard`]
/// and has a direction (higher-is-better or lower-is-better).
///
/// ## Tier: T2-P (N dominant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalMetric {
    /// Number of active signal types (out of 5). Target: maximize.
    DetectionCoverage,
    /// Cross-signal convergence score [0, 1]. Target: maximize.
    ConvergenceThreshold,
    /// Seconds from first signal to actionable state. Target: minimize.
    TimeToIntelligence,
    /// Number of unique sources. Target: maximize.
    SourceDiversityMin,
    /// Synthesized confidence floor [0, 1]. Target: maximize.
    ConfidenceFloor,
    /// Seconds since last signal. Target: minimize.
    SignalFreshness,
    /// Ratio of signals reaching actionable state [0, 1]. Target: maximize.
    ActionableRate,
    /// Historical false positive rate [0, 1]. Target: minimize.
    FalsePositiveRate,
}

impl GoalMetric {
    /// Whether higher values are better for this metric.
    pub fn higher_is_better(&self) -> bool {
        matches!(
            self,
            Self::DetectionCoverage
                | Self::ConvergenceThreshold
                | Self::SourceDiversityMin
                | Self::ConfidenceFloor
                | Self::ActionableRate
        )
    }

    /// Extract current value from monitoring dashboard.
    pub fn extract_from(&self, dashboard: &MonitoringDashboard) -> f64 {
        match self {
            Self::DetectionCoverage => dashboard.active_signal_types as f64,
            Self::ConvergenceThreshold => dashboard.convergence_score,
            Self::TimeToIntelligence => dashboard.time_to_current_state,
            Self::SourceDiversityMin => dashboard.source_diversity_score,
            Self::ConfidenceFloor => dashboard.confidence_lower_bound,
            Self::SignalFreshness => dashboard.seconds_since_last_signal,
            Self::ActionableRate => dashboard.actionable_ratio,
            Self::FalsePositiveRate => dashboard.false_positive_rate,
        }
    }

    /// Unit label for display.
    pub fn unit(&self) -> &'static str {
        match self {
            Self::DetectionCoverage => "types",
            Self::ConvergenceThreshold => "score",
            Self::TimeToIntelligence => "seconds",
            Self::SourceDiversityMin => "score",
            Self::ConfidenceFloor => "score",
            Self::SignalFreshness => "seconds",
            Self::ActionableRate => "ratio",
            Self::FalsePositiveRate => "ratio",
        }
    }

    /// NexVigilant P0-P5 priority alignment.
    pub fn default_priority(&self) -> Priority {
        match self {
            Self::FalsePositiveRate => Priority::P0, // Patient Safety
            Self::DetectionCoverage => Priority::P1, // Signal Integrity
            Self::ConvergenceThreshold => Priority::P1,
            Self::ConfidenceFloor => Priority::P1,
            Self::SourceDiversityMin => Priority::P2, // Data Quality
            Self::SignalFreshness => Priority::P3,    // Operational
            Self::TimeToIntelligence => Priority::P4, // Efficiency
            Self::ActionableRate => Priority::P4,
        }
    }

    /// Human-readable description of what SMART-Specific means for this metric.
    pub fn specific_description(&self) -> &'static str {
        match self {
            Self::DetectionCoverage => {
                "Activate all 5 signal detectors (Sentiment, Trend, Engagement, Virality, Controversy)"
            }
            Self::ConvergenceThreshold => {
                "Achieve cross-signal convergence score >= target across active detectors"
            }
            Self::TimeToIntelligence => {
                "Reduce elapsed time from first signal detection to Actionable state"
            }
            Self::SourceDiversityMin => {
                "Maintain signals from >= 3 independent sources with HHI-based diversity >= target"
            }
            Self::ConfidenceFloor => {
                "Ensure synthesized confidence lower bound exceeds minimum threshold"
            }
            Self::SignalFreshness => "Keep time since last signal below staleness threshold",
            Self::ActionableRate => {
                "Increase proportion of intelligence assessments reaching Actionable state"
            }
            Self::FalsePositiveRate => {
                "Reduce false positive signals to below acceptable threshold via validation"
            }
        }
    }
}

impl std::fmt::Display for GoalMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DetectionCoverage => write!(f, "Detection Coverage"),
            Self::ConvergenceThreshold => write!(f, "Convergence Threshold"),
            Self::TimeToIntelligence => write!(f, "Time to Intelligence"),
            Self::SourceDiversityMin => write!(f, "Source Diversity Minimum"),
            Self::ConfidenceFloor => write!(f, "Confidence Floor"),
            Self::SignalFreshness => write!(f, "Signal Freshness"),
            Self::ActionableRate => write!(f, "Actionable Rate"),
            Self::FalsePositiveRate => write!(f, "False Positive Rate"),
        }
    }
}

// ============================================================================
// Priority — P0-P5 alignment
// ============================================================================

/// NexVigilant priority levels for goal alignment.
///
/// ## Tier: T2-P (κ dominant — comparison/ordering)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Priority {
    /// P0: Patient Safety — supreme directive.
    P0,
    /// P1: Signal Integrity — no signal lost or downgraded.
    P1,
    /// P2: Data Quality.
    P2,
    /// P3: Operational Efficiency.
    P3,
    /// P4: Cost Optimization.
    P4,
    /// P5: Enhancement.
    P5,
}

impl Priority {
    /// Numeric weight for prioritization (higher = more important).
    pub fn weight(&self) -> f64 {
        match self {
            Self::P0 => 1.0,
            Self::P1 => 0.85,
            Self::P2 => 0.70,
            Self::P3 => 0.55,
            Self::P4 => 0.40,
            Self::P5 => 0.25,
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::P0 => write!(f, "P0-Safety"),
            Self::P1 => write!(f, "P1-Signal"),
            Self::P2 => write!(f, "P2-Quality"),
            Self::P3 => write!(f, "P3-Operations"),
            Self::P4 => write!(f, "P4-Efficiency"),
            Self::P5 => write!(f, "P5-Enhancement"),
        }
    }
}

// ============================================================================
// Goal Status — ς (State)
// ============================================================================

/// Goal lifecycle status.
///
/// ```text
/// NotStarted ──→ InProgress ──→ OnTrack ──→ Achieved
///                    │              │
///                    └──→ AtRisk ──→ Failed
/// ```
///
/// ## Tier: T2-P (ς dominant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalStatus {
    /// Goal defined but work not started.
    NotStarted,
    /// Work in progress, insufficient data to assess trajectory.
    InProgress,
    /// Current trajectory predicts goal achievement by deadline.
    OnTrack,
    /// Current trajectory predicts goal will NOT be met.
    AtRisk,
    /// Goal achieved — target met before or at deadline.
    Achieved,
    /// Goal failed — deadline passed without achievement.
    Failed,
}

impl GoalStatus {
    /// Whether the goal is still active (not terminal).
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::NotStarted | Self::InProgress | Self::OnTrack | Self::AtRisk
        )
    }

    /// Whether this is a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Achieved | Self::Failed)
    }
}

impl std::fmt::Display for GoalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotStarted => write!(f, "NOT_STARTED"),
            Self::InProgress => write!(f, "IN_PROGRESS"),
            Self::OnTrack => write!(f, "ON_TRACK"),
            Self::AtRisk => write!(f, "AT_RISK"),
            Self::Achieved => write!(f, "ACHIEVED"),
            Self::Failed => write!(f, "FAILED"),
        }
    }
}

// ============================================================================
// Goal Progress — N + κ (measured comparison)
// ============================================================================

/// Tracks measurable progress toward a goal target.
///
/// ## Tier: T2-C (N + κ + σ)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GoalProgress {
    /// Current measured value.
    pub current: f64,
    /// Target value.
    pub target: f64,
    /// Starting value (baseline when goal was created).
    pub baseline: f64,
    /// Whether higher is better for this metric.
    pub higher_is_better: bool,
}

impl GoalProgress {
    /// Create new progress tracker.
    pub fn new(current: f64, target: f64, baseline: f64, higher_is_better: bool) -> Self {
        Self {
            current,
            target,
            baseline,
            higher_is_better,
        }
    }

    /// Completion percentage [0.0, 1.0].
    ///
    /// 0.0 = at baseline, 1.0 = target achieved.
    /// Can exceed 1.0 if target is surpassed.
    pub fn completion(&self) -> f64 {
        let total_distance = if self.higher_is_better {
            self.target - self.baseline
        } else {
            self.baseline - self.target
        };

        if total_distance.abs() < f64::EPSILON {
            return if self.is_met() { 1.0 } else { 0.0 };
        }

        let current_distance = if self.higher_is_better {
            self.current - self.baseline
        } else {
            self.baseline - self.current
        };

        (current_distance / total_distance).clamp(0.0, 2.0)
    }

    /// Whether the target is met.
    pub fn is_met(&self) -> bool {
        if self.higher_is_better {
            self.current >= self.target
        } else {
            self.current <= self.target
        }
    }

    /// Gap remaining to target (always positive when not met, negative when exceeded).
    pub fn gap(&self) -> f64 {
        if self.higher_is_better {
            self.target - self.current
        } else {
            self.current - self.target
        }
    }

    /// Improvement from baseline (always positive if moving toward target).
    pub fn improvement(&self) -> f64 {
        if self.higher_is_better {
            self.current - self.baseline
        } else {
            self.baseline - self.current
        }
    }
}

impl std::fmt::Display for GoalProgress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pct = self.completion() * 100.0;
        let status = if self.is_met() { "MET" } else { "OPEN" };
        write!(
            f,
            "{:.1} / {:.1} ({:.0}% {}) [gap: {:.2}]",
            self.current,
            self.target,
            pct,
            status,
            self.gap()
        )
    }
}

// ============================================================================
// Intelligence Goal — SMART composite
// ============================================================================

/// A SMART goal for the value intelligence system.
///
/// SMART decomposition:
/// - **S**pecific: `metric` + `metric.specific_description()`
/// - **M**easurable: `progress.current` vs `progress.target`
/// - **A**chievable: `achievability_check()` validates target is realistic
/// - **R**elevant: `priority` aligns to P0-P5 hierarchy
/// - **T**ime-bound: `deadline` with elapsed/remaining tracking
///
/// ## Tier: T2-C (π + N + ∂ + ς + σ + κ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceGoal {
    /// Unique goal identifier (e.g., "G1", "G2").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// What metric this goal measures.
    pub metric: GoalMetric,
    /// Priority alignment.
    pub priority: Priority,
    /// Measurable progress.
    pub progress: GoalProgress,
    /// Current status.
    pub status: GoalStatus,
    /// When goal was created.
    pub created_at: DateTime<Utc>,
    /// When goal must be achieved.
    pub deadline: DateTime<Utc>,
    /// When goal was achieved (if applicable).
    pub achieved_at: Option<DateTime<Utc>>,
    /// Number of times progress has been updated.
    pub check_count: u32,
}

impl IntelligenceGoal {
    /// Create a new SMART goal.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        metric: GoalMetric,
        target: f64,
        baseline: f64,
        deadline_days: i64,
    ) -> Self {
        let priority = metric.default_priority();
        let higher_is_better = metric.higher_is_better();
        let progress = GoalProgress::new(baseline, target, baseline, higher_is_better);
        let now = Utc::now();

        Self {
            id: id.into(),
            name: name.into(),
            metric,
            priority,
            progress,
            status: GoalStatus::NotStarted,
            created_at: now,
            deadline: now + Duration::days(deadline_days),
            achieved_at: None,
            check_count: 0,
        }
    }

    /// Update progress from a monitoring dashboard snapshot.
    pub fn update_from_dashboard(&mut self, dashboard: &MonitoringDashboard) {
        let new_value = self.metric.extract_from(dashboard);
        self.progress.current = new_value;
        self.check_count += 1;

        // Update status
        if self.progress.is_met() {
            self.status = GoalStatus::Achieved;
            if self.achieved_at.is_none() {
                self.achieved_at = Some(Utc::now());
            }
        } else if Utc::now() > self.deadline {
            self.status = GoalStatus::Failed;
        } else {
            // Assess trajectory
            let elapsed_ratio = self.elapsed_ratio();
            let completion = self.progress.completion();

            if self.check_count < 3 {
                self.status = GoalStatus::InProgress;
            } else if completion >= elapsed_ratio * 0.8 {
                self.status = GoalStatus::OnTrack;
            } else {
                self.status = GoalStatus::AtRisk;
            }
        }
    }

    /// Elapsed ratio [0.0, 1.0] of time consumed.
    pub fn elapsed_ratio(&self) -> f64 {
        let total = (self.deadline - self.created_at).num_seconds() as f64;
        if total <= 0.0 {
            return 1.0;
        }
        let elapsed = (Utc::now() - self.created_at).num_seconds() as f64;
        (elapsed / total).clamp(0.0, 1.0)
    }

    /// Remaining time in seconds.
    pub fn remaining_seconds(&self) -> f64 {
        let remaining = self.deadline - Utc::now();
        remaining.num_seconds().max(0) as f64
    }

    /// SMART achievability check.
    ///
    /// Returns `true` if the target is within realistic bounds for this metric.
    pub fn achievability_check(&self) -> bool {
        match self.metric {
            GoalMetric::DetectionCoverage => self.progress.target <= 5.0,
            GoalMetric::ConvergenceThreshold => self.progress.target <= 1.0,
            GoalMetric::TimeToIntelligence => self.progress.target >= 60.0, // At least 1 minute
            GoalMetric::SourceDiversityMin => self.progress.target <= 1.0,
            GoalMetric::ConfidenceFloor => self.progress.target <= 1.0,
            GoalMetric::SignalFreshness => self.progress.target >= 10.0, // At least 10 seconds
            GoalMetric::ActionableRate => self.progress.target <= 1.0,
            GoalMetric::FalsePositiveRate => self.progress.target >= 0.0,
        }
    }

    /// SMART relevance score — how well this goal aligns with current priority.
    pub fn relevance_score(&self) -> f64 {
        self.priority.weight()
    }

    /// Full SMART assessment as structured report.
    pub fn smart_assessment(&self) -> SmartAssessment {
        SmartAssessment {
            specific: self.metric.specific_description().to_string(),
            measurable: format!(
                "{:.2} {} (current) vs {:.2} {} (target)",
                self.progress.current,
                self.metric.unit(),
                self.progress.target,
                self.metric.unit()
            ),
            achievable: self.achievability_check(),
            relevant: format!("{} (weight={:.2})", self.priority, self.priority.weight()),
            time_bound: format!(
                "{:.0}s remaining ({:.0}% elapsed)",
                self.remaining_seconds(),
                self.elapsed_ratio() * 100.0
            ),
            completion_pct: self.progress.completion() * 100.0,
            status: self.status,
        }
    }
}

impl std::fmt::Display for IntelligenceGoal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} | {} | {} | {} | {:.0}% complete",
            self.id,
            self.name,
            self.metric,
            self.priority,
            self.status,
            self.progress.completion() * 100.0
        )
    }
}

// ============================================================================
// SMART Assessment — Structured report
// ============================================================================

/// Structured SMART assessment for a goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartAssessment {
    /// S: What specifically will be accomplished?
    pub specific: String,
    /// M: How will progress be measured?
    pub measurable: String,
    /// A: Is the target realistic?
    pub achievable: bool,
    /// R: How does this align with organizational priorities?
    pub relevant: String,
    /// T: What is the deadline and time remaining?
    pub time_bound: String,
    /// Current completion percentage.
    pub completion_pct: f64,
    /// Current goal status.
    pub status: GoalStatus,
}

impl std::fmt::Display for SmartAssessment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SMART Assessment ({}):\n  S: {}\n  M: {}\n  A: {}\n  R: {}\n  T: {}\n  Progress: {:.0}%",
            self.status,
            self.specific,
            self.measurable,
            if self.achievable { "YES" } else { "NO" },
            self.relevant,
            self.time_bound,
            self.completion_pct
        )
    }
}

// ============================================================================
// Goal Portfolio — Collection of active goals
// ============================================================================

/// Portfolio of intelligence goals with aggregate health assessment.
///
/// ## Tier: T2-C (Σ + π)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalPortfolio {
    /// All goals in the portfolio.
    pub goals: Vec<IntelligenceGoal>,
    /// When the portfolio was last assessed.
    pub last_assessed: DateTime<Utc>,
}

impl GoalPortfolio {
    /// Create the default goal set for a new intelligence system.
    pub fn default_goals() -> Self {
        let goals = vec![
            IntelligenceGoal::new(
                "G1",
                "Full Detection Coverage",
                GoalMetric::DetectionCoverage,
                5.0, // All 5 signal types
                0.0, // Starting from zero
                7,   // 7 days
            ),
            IntelligenceGoal::new(
                "G2",
                "Strong Signal Convergence",
                GoalMetric::ConvergenceThreshold,
                0.7, // 70% convergence
                0.0,
                14,
            ),
            IntelligenceGoal::new(
                "G3",
                "Rapid Intelligence Synthesis",
                GoalMetric::TimeToIntelligence,
                3600.0,  // 1 hour
                86400.0, // Starting at 24h
                30,
            ),
            IntelligenceGoal::new(
                "G4",
                "Multi-Source Diversity",
                GoalMetric::SourceDiversityMin,
                0.67, // HHI diversity ≥ 0.67 (3+ equal sources)
                0.0,
                14,
            ),
            IntelligenceGoal::new(
                "G5",
                "Reliable Confidence Floor",
                GoalMetric::ConfidenceFloor,
                0.7, // 70% confidence lower bound
                0.0,
                7,
            ),
            IntelligenceGoal::new(
                "G6",
                "Fresh Signal Pipeline",
                GoalMetric::SignalFreshness,
                1800.0, // 30 minutes max staleness
                7200.0, // Starting at 2h
                30,
            ),
            IntelligenceGoal::new(
                "G7",
                "Actionable Intelligence Rate",
                GoalMetric::ActionableRate,
                0.3, // 30% actionable
                0.0,
                30,
            ),
            IntelligenceGoal::new(
                "G8",
                "False Positive Containment",
                GoalMetric::FalsePositiveRate,
                0.1, // ≤ 10% FPR
                0.5, // Assuming 50% initially
                60,
            ),
        ];

        Self {
            goals,
            last_assessed: Utc::now(),
        }
    }

    /// Update all goals from a monitoring dashboard.
    pub fn update_all(&mut self, dashboard: &MonitoringDashboard) {
        for goal in &mut self.goals {
            if goal.status.is_active() {
                goal.update_from_dashboard(dashboard);
            }
        }
        self.last_assessed = Utc::now();
    }

    /// Number of achieved goals.
    pub fn achieved_count(&self) -> usize {
        self.goals
            .iter()
            .filter(|g| g.status == GoalStatus::Achieved)
            .count()
    }

    /// Number of at-risk goals.
    pub fn at_risk_count(&self) -> usize {
        self.goals
            .iter()
            .filter(|g| g.status == GoalStatus::AtRisk)
            .count()
    }

    /// Number of failed goals.
    pub fn failed_count(&self) -> usize {
        self.goals
            .iter()
            .filter(|g| g.status == GoalStatus::Failed)
            .count()
    }

    /// Overall portfolio completion [0.0, 1.0].
    pub fn overall_completion(&self) -> f64 {
        if self.goals.is_empty() {
            return 0.0;
        }

        // Priority-weighted completion
        let total_weight: f64 = self.goals.iter().map(|g| g.priority.weight()).sum();
        let weighted_completion: f64 = self
            .goals
            .iter()
            .map(|g| g.progress.completion().min(1.0) * g.priority.weight())
            .sum();

        if total_weight > 0.0 {
            weighted_completion / total_weight
        } else {
            0.0
        }
    }

    /// Portfolio health classification.
    pub fn health(&self) -> &'static str {
        let completion = self.overall_completion();
        let at_risk = self.at_risk_count();
        let failed = self.failed_count();

        if failed > 0 {
            "critical"
        } else if at_risk > 2 {
            "degraded"
        } else if completion >= 0.8 {
            "excellent"
        } else if completion >= 0.5 {
            "good"
        } else {
            "developing"
        }
    }

    /// Get highest-priority unmet goal.
    pub fn highest_priority_gap(&self) -> Option<&IntelligenceGoal> {
        self.goals
            .iter()
            .filter(|g| g.status.is_active() && !g.progress.is_met())
            .min_by_key(|g| g.priority)
    }

    /// Get goals sorted by priority then completion.
    pub fn prioritized(&self) -> Vec<&IntelligenceGoal> {
        let mut sorted: Vec<_> = self.goals.iter().collect();
        sorted.sort_by(|a, b| {
            a.priority.cmp(&b.priority).then_with(|| {
                a.progress
                    .completion()
                    .partial_cmp(&b.progress.completion())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });
        sorted
    }

    /// Generate portfolio summary.
    pub fn summary(&self) -> PortfolioSummary {
        PortfolioSummary {
            total_goals: self.goals.len(),
            achieved: self.achieved_count(),
            on_track: self
                .goals
                .iter()
                .filter(|g| g.status == GoalStatus::OnTrack)
                .count(),
            in_progress: self
                .goals
                .iter()
                .filter(|g| g.status == GoalStatus::InProgress)
                .count(),
            at_risk: self.at_risk_count(),
            not_started: self
                .goals
                .iter()
                .filter(|g| g.status == GoalStatus::NotStarted)
                .count(),
            failed: self.failed_count(),
            overall_completion: self.overall_completion(),
            health: self.health().to_string(),
            highest_priority_gap: self
                .highest_priority_gap()
                .map(|g| format!("{}: {}", g.id, g.name)),
        }
    }
}

impl std::fmt::Display for GoalPortfolio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Goal Portfolio [{}] — {:.0}% complete",
            self.health().to_uppercase(),
            self.overall_completion() * 100.0
        )?;
        writeln!(
            f,
            "  Achieved: {} | On Track: {} | At Risk: {} | Failed: {}",
            self.achieved_count(),
            self.goals
                .iter()
                .filter(|g| g.status == GoalStatus::OnTrack)
                .count(),
            self.at_risk_count(),
            self.failed_count()
        )?;
        for goal in &self.goals {
            writeln!(f, "  {}", goal)?;
        }
        Ok(())
    }
}

/// Aggregate portfolio summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSummary {
    /// Total number of goals.
    pub total_goals: usize,
    /// Goals achieved.
    pub achieved: usize,
    /// Goals on track.
    pub on_track: usize,
    /// Goals in progress.
    pub in_progress: usize,
    /// Goals at risk.
    pub at_risk: usize,
    /// Goals not yet started.
    pub not_started: usize,
    /// Goals failed.
    pub failed: usize,
    /// Priority-weighted completion [0.0, 1.0].
    pub overall_completion: f64,
    /// Portfolio health classification.
    pub health: String,
    /// Highest priority unmet goal.
    pub highest_priority_gap: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence::{IntelligenceState, ResponseAction};

    // ── GoalMetric ──

    #[test]
    fn metric_direction() {
        assert!(GoalMetric::DetectionCoverage.higher_is_better());
        assert!(GoalMetric::ConvergenceThreshold.higher_is_better());
        assert!(!GoalMetric::TimeToIntelligence.higher_is_better());
        assert!(!GoalMetric::FalsePositiveRate.higher_is_better());
    }

    #[test]
    fn metric_priority_alignment() {
        assert_eq!(
            GoalMetric::FalsePositiveRate.default_priority(),
            Priority::P0
        );
        assert_eq!(
            GoalMetric::DetectionCoverage.default_priority(),
            Priority::P1
        );
        assert_eq!(
            GoalMetric::SourceDiversityMin.default_priority(),
            Priority::P2
        );
        assert_eq!(GoalMetric::SignalFreshness.default_priority(), Priority::P3);
        assert_eq!(
            GoalMetric::TimeToIntelligence.default_priority(),
            Priority::P4
        );
    }

    #[test]
    fn metric_extracts_from_dashboard() {
        let dashboard = MonitoringDashboard {
            active_signal_count: 5,
            active_signal_types: 4,
            convergence_score: 0.75,
            source_diversity_score: 0.68,
            temporal_alignment_score: 0.82,
            confidence_score: 0.78,
            confidence_lower_bound: 0.65,
            state: IntelligenceState::Confirmed,
            recommended_action: ResponseAction::Investigate,
            conviction: 0.72,
            seconds_since_last_signal: 300.0,
            decay_rate: 0.01,
            false_positive_rate: 0.08,
            actionable_ratio: 0.35,
            time_to_current_state: 1800.0,
            snapshot_at: Utc::now(),
        };

        assert!(
            (GoalMetric::DetectionCoverage.extract_from(&dashboard) - 4.0).abs() < f64::EPSILON
        );
        assert!(
            (GoalMetric::ConvergenceThreshold.extract_from(&dashboard) - 0.75).abs() < f64::EPSILON
        );
        assert!(
            (GoalMetric::FalsePositiveRate.extract_from(&dashboard) - 0.08).abs() < f64::EPSILON
        );
    }

    // ── GoalProgress ──

    #[test]
    fn progress_higher_is_better() {
        let p = GoalProgress::new(0.5, 1.0, 0.0, true);
        assert!((p.completion() - 0.5).abs() < f64::EPSILON);
        assert!(!p.is_met());
        assert!((p.gap() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn progress_lower_is_better() {
        let p = GoalProgress::new(0.3, 0.1, 0.5, false);
        assert!((p.completion() - 0.5).abs() < f64::EPSILON); // 50% of the way from 0.5 to 0.1
        assert!(!p.is_met());
    }

    #[test]
    fn progress_met() {
        let p = GoalProgress::new(1.0, 0.8, 0.0, true);
        assert!(p.is_met());
        assert!(p.completion() >= 1.0);
    }

    #[test]
    fn progress_lower_met() {
        let p = GoalProgress::new(0.05, 0.1, 0.5, false);
        assert!(p.is_met());
    }

    // ── IntelligenceGoal ──

    #[test]
    fn goal_smart_components() {
        let goal = IntelligenceGoal::new(
            "G1",
            "Full Detection Coverage",
            GoalMetric::DetectionCoverage,
            5.0,
            0.0,
            7,
        );

        assert_eq!(goal.status, GoalStatus::NotStarted);
        assert!(goal.achievability_check()); // 5 types is achievable
        assert!(goal.relevance_score() > 0.0);
        assert_eq!(goal.priority, Priority::P1);

        let assessment = goal.smart_assessment();
        assert!(!assessment.specific.is_empty());
        assert!(!assessment.measurable.is_empty());
        assert!(assessment.achievable);
        assert!(!assessment.relevant.is_empty());
        assert!(!assessment.time_bound.is_empty());
    }

    #[test]
    fn goal_unrealistic_target() {
        let goal = IntelligenceGoal::new(
            "BAD",
            "Impossible Coverage",
            GoalMetric::DetectionCoverage,
            10.0, // Only 5 signal types exist
            0.0,
            7,
        );
        assert!(!goal.achievability_check());
    }

    #[test]
    fn goal_update_from_dashboard() {
        let mut goal = IntelligenceGoal::new(
            "G5",
            "Confidence Floor",
            GoalMetric::ConfidenceFloor,
            0.7,
            0.0,
            7,
        );

        let dashboard = MonitoringDashboard {
            active_signal_count: 4,
            active_signal_types: 3,
            convergence_score: 0.8,
            source_diversity_score: 0.7,
            temporal_alignment_score: 0.9,
            confidence_score: 0.85,
            confidence_lower_bound: 0.72,
            state: IntelligenceState::Actionable,
            recommended_action: ResponseAction::Act,
            conviction: 0.88,
            seconds_since_last_signal: 60.0,
            decay_rate: 0.005,
            false_positive_rate: 0.05,
            actionable_ratio: 0.4,
            time_to_current_state: 900.0,
            snapshot_at: Utc::now(),
        };

        goal.update_from_dashboard(&dashboard);
        assert!(goal.progress.is_met()); // 0.72 >= 0.7
        assert_eq!(goal.status, GoalStatus::Achieved);
        assert!(goal.achieved_at.is_some());
    }

    // ── GoalPortfolio ──

    #[test]
    fn default_portfolio_has_8_goals() {
        let portfolio = GoalPortfolio::default_goals();
        assert_eq!(portfolio.goals.len(), 8);

        // Verify each goal ID
        let ids: Vec<_> = portfolio.goals.iter().map(|g| g.id.as_str()).collect();
        assert_eq!(ids, vec!["G1", "G2", "G3", "G4", "G5", "G6", "G7", "G8"]);
    }

    #[test]
    fn portfolio_initial_health() {
        let portfolio = GoalPortfolio::default_goals();
        assert_eq!(portfolio.achieved_count(), 0);
        assert_eq!(portfolio.failed_count(), 0);
        // All at baseline = 0% completion
        assert!(portfolio.overall_completion() < f64::EPSILON);
        assert_eq!(portfolio.health(), "developing");
    }

    #[test]
    fn portfolio_prioritization() {
        let portfolio = GoalPortfolio::default_goals();
        let sorted = portfolio.prioritized();

        // G8 (FPR, P0) should come first
        assert_eq!(sorted[0].id, "G8");
    }

    #[test]
    fn portfolio_highest_priority_gap() {
        let portfolio = GoalPortfolio::default_goals();
        let gap = portfolio.highest_priority_gap();
        assert!(gap.is_some());
        // Should be G8 (P0 — FalsePositiveRate)
        let g = gap.unwrap_or_else(|| panic!("expected Some"));
        assert_eq!(g.id, "G8");
    }

    #[test]
    fn portfolio_summary() {
        let portfolio = GoalPortfolio::default_goals();
        let summary = portfolio.summary();
        assert_eq!(summary.total_goals, 8);
        assert_eq!(summary.not_started, 8);
        assert_eq!(summary.achieved, 0);
        assert!(summary.highest_priority_gap.is_some());
    }

    // ── Priority ──

    #[test]
    fn priority_ordering() {
        assert!(Priority::P0 < Priority::P1);
        assert!(Priority::P1 < Priority::P2);
        assert!(Priority::P4 < Priority::P5);
    }

    #[test]
    fn priority_weights_descending() {
        assert!(Priority::P0.weight() > Priority::P1.weight());
        assert!(Priority::P1.weight() > Priority::P2.weight());
        assert!(Priority::P4.weight() > Priority::P5.weight());
    }

    // ── Display traits ──

    #[test]
    fn display_formats_exist() {
        let goal = IntelligenceGoal::new(
            "G1",
            "Test Goal",
            GoalMetric::DetectionCoverage,
            5.0,
            0.0,
            7,
        );
        let s = format!("{}", goal);
        assert!(s.contains("G1"));
        assert!(s.contains("Test Goal"));

        let assessment = goal.smart_assessment();
        let s = format!("{}", assessment);
        assert!(s.contains("SMART Assessment"));

        let portfolio = GoalPortfolio::default_goals();
        let s = format!("{}", portfolio);
        assert!(s.contains("Goal Portfolio"));
    }
}
