//! SMART governance types for The Foundry architecture.
//!
//! Provides a three-level (Strategic → Team → Operational) goal cascade,
//! progress tracking, KPI contribution recording, cascade-alignment
//! validation, per-agent worklog entries, and consolidated team reports.
//!
//! # Goal cascade
//!
//! Every [`SmartGoal`] carries a `traces_to` field that links it upward
//! through the hierarchy.  Strategic goals leave that vector empty.  Team
//! and Operational goals each name the IDs of the parent goals they
//! contribute to.  [`CascadeValidation`] computes what fraction of
//! Operational goals can be traced all the way to the Strategic level,
//! surfacing misalignment before delivery.
//!
//! # Example
//!
//! ```
//! use nexcore_foundry::governance::{
//!     GoalLevel, SmartGoal, GoalStatus, CascadeValidation, DeliveryStatus, TeamReport,
//! };
//!
//! let strategic = SmartGoal {
//!     id: "S1".to_string(),
//!     level: GoalLevel::Strategic,
//!     specific: "Reduce adverse-event review latency".to_string(),
//!     measurable: "P95 review time < 4 h".to_string(),
//!     achievable: "Within current team capacity".to_string(),
//!     relevant: "Core SLA commitment".to_string(),
//!     time_bound: "Q3 2026".to_string(),
//!     traces_to: vec![],
//! };
//!
//! let operational = SmartGoal {
//!     id: "O1".to_string(),
//!     level: GoalLevel::Operational,
//!     specific: "Cache FAERS query results".to_string(),
//!     measurable: "Cache hit-rate >= 80 %".to_string(),
//!     achievable: "Redis already provisioned".to_string(),
//!     relevant: "Directly reduces query latency".to_string(),
//!     time_bound: "Sprint 12".to_string(),
//!     traces_to: vec!["S1".to_string()],
//! };
//!
//! assert!(operational.traces_to.contains(&strategic.id));
//!
//! let validation = CascadeValidation {
//!     total_operational_goals: 1,
//!     traced_to_team: 1,
//!     traced_to_strategic: 1,
//!     alignment_percent: 100.0,
//! };
//! assert!(validation.is_fully_aligned());
//! ```

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Goal hierarchy
// ---------------------------------------------------------------------------

/// Organisational level at which a [`SmartGoal`] lives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalLevel {
    /// Top-level, multi-quarter objectives owned by leadership.
    Strategic,
    /// Mid-level objectives owned by a cross-functional team.
    Team,
    /// Sprint- or task-level objectives owned by individual agents.
    Operational,
}

/// A SMART goal expressed at any level of the organisational hierarchy.
///
/// The `traces_to` field links this goal upward: an [`GoalLevel::Operational`]
/// goal names the IDs of the [`GoalLevel::Team`] (or [`GoalLevel::Strategic`])
/// goals it contributes to.  [`GoalLevel::Strategic`] goals leave this empty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartGoal {
    /// Unique identifier for this goal (e.g. `"S1"`, `"T2"`, `"O7"`).
    pub id: String,
    /// Level within the cascade hierarchy.
    pub level: GoalLevel,
    /// **S** — What exactly will be accomplished?
    pub specific: String,
    /// **M** — How will progress and completion be measured?
    pub measurable: String,
    /// **A** — Why is this attainable given current resources?
    pub achievable: String,
    /// **R** — Why does this matter to the parent goal / strategy?
    pub relevant: String,
    /// **T** — By when must this be complete?
    pub time_bound: String,
    /// IDs of parent goals this goal contributes to.
    ///
    /// Empty for [`GoalLevel::Strategic`] goals.
    pub traces_to: Vec<String>,
}

// ---------------------------------------------------------------------------
// Progress tracking
// ---------------------------------------------------------------------------

/// Snapshot of a single goal's progress at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalStatus {
    /// ID of the [`SmartGoal`] being tracked.
    pub goal_id: String,
    /// The target value or condition (mirrors `measurable`).
    pub target: String,
    /// The current measured value or state.
    pub current: String,
    /// Whether the goal has been fully met.
    pub met: bool,
}

// ---------------------------------------------------------------------------
// KPI contribution
// ---------------------------------------------------------------------------

/// Records how an agent's work moves a strategic KPI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiContribution {
    /// Name of the strategic KPI being influenced.
    pub strategic_kpi: String,
    /// The specific metric or measure being reported.
    pub measure: String,
    /// Observed value for `measure` during this reporting period.
    pub value: String,
}

// ---------------------------------------------------------------------------
// Cascade validation
// ---------------------------------------------------------------------------

/// Validates that the Operational → Team → Strategic goal cascade is complete.
///
/// Produced by comparing the full set of Operational goals against those that
/// have been traced up through Team goals to a Strategic goal.
///
/// # Example
///
/// ```
/// use nexcore_foundry::governance::CascadeValidation;
///
/// let v = CascadeValidation {
///     total_operational_goals: 8,
///     traced_to_team: 8,
///     traced_to_strategic: 8,
///     alignment_percent: 100.0,
/// };
/// assert!(v.is_fully_aligned());
///
/// let partial = CascadeValidation {
///     total_operational_goals: 8,
///     traced_to_team: 6,
///     traced_to_strategic: 5,
///     alignment_percent: 62.5,
/// };
/// assert!(!partial.is_fully_aligned());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeValidation {
    /// Total number of Operational goals in the plan.
    pub total_operational_goals: u32,
    /// Number of Operational goals that trace to at least one Team goal.
    pub traced_to_team: u32,
    /// Number of Operational goals that trace (directly or via Team) to a
    /// Strategic goal.
    pub traced_to_strategic: u32,
    /// `traced_to_strategic / total_operational_goals * 100.0`.
    pub alignment_percent: f64,
}

impl CascadeValidation {
    /// Returns `true` when every Operational goal traces to the Strategic
    /// level, i.e. `alignment_percent >= 100.0`.
    #[must_use]
    pub fn is_fully_aligned(&self) -> bool {
        self.alignment_percent >= 100.0
    }
}

// ---------------------------------------------------------------------------
// Worklog
// ---------------------------------------------------------------------------

/// Per-agent progress report submitted at the end of a work cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorklogEntry {
    /// Name of the reporting agent.
    pub agent_name: String,
    /// Status snapshots for each goal the agent owns.
    pub goals: Vec<GoalStatus>,
    /// KPI contributions made during this cycle.
    pub kpi_contributions: Vec<KpiContribution>,
    /// Names or paths of artifacts produced (e.g. compiled binaries, reports).
    pub artifacts_produced: Vec<String>,
}

// ---------------------------------------------------------------------------
// Team report
// ---------------------------------------------------------------------------

/// Summary of a single agent's performance, suitable for inclusion in a
/// consolidated [`TeamReport`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformance {
    /// Name of the agent.
    pub agent_name: String,
    /// Role or function of the agent in the team.
    pub role: String,
    /// Human-readable goals-met summary (e.g. `"3 / 4"`).
    pub goals_met: String,
    /// Human-readable artifacts summary (e.g. `"2 binaries, 1 report"`).
    pub artifacts: String,
    /// Wall-clock or sprint time consumed (e.g. `"4 h 12 m"`).
    pub time: String,
}

/// High-level delivery state for a [`TeamReport`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    /// All acceptance criteria have been met and deliverables are ready.
    Delivered,
    /// Progress is stopped by an unresolved dependency or blocker.
    Blocked,
    /// In active refinement; further cycles are expected.
    Iterating,
}

/// Orchestrator's consolidated end-of-cycle report for the whole team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamReport {
    /// Status of every Strategic goal tracked during this cycle.
    pub strategic_goals: Vec<GoalStatus>,
    /// Team-level KPI contributions aggregated across all agents.
    pub team_measures: Vec<KpiContribution>,
    /// Per-agent performance summaries.
    pub agent_performance: Vec<AgentPerformance>,
    /// Optional high-level intelligence or retrospective commentary.
    pub intelligence_summary: Option<String>,
    /// Feedback constraints or guard-rails to carry forward.
    pub feedback_constraints: Vec<String>,
    /// Overall delivery state for this cycle.
    pub overall: DeliveryStatus,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Strategic goals have an empty `traces_to`; Operational goals list the
    /// IDs of the Strategic (or Team) goals they contribute to.
    #[test]
    fn smart_goal_cascade_traces() {
        let strategic = SmartGoal {
            id: "S1".to_string(),
            level: GoalLevel::Strategic,
            specific: "Improve signal detection recall".to_string(),
            measurable: "Recall >= 0.95 on held-out set".to_string(),
            achievable: "Model retraining pipeline in place".to_string(),
            relevant: "Core product SLA".to_string(),
            time_bound: "Q2 2026".to_string(),
            traces_to: vec![],
        };

        let operational = SmartGoal {
            id: "O1".to_string(),
            level: GoalLevel::Operational,
            specific: "Add synonym expansion to query parser".to_string(),
            measurable: "Coverage of synonym variants >= 90 %".to_string(),
            achievable: "RxNorm API already integrated".to_string(),
            relevant: "Directly improves recall for drug name variants".to_string(),
            time_bound: "Sprint 8".to_string(),
            traces_to: vec!["S1".to_string()],
        };

        assert!(strategic.traces_to.is_empty());
        assert!(operational.traces_to.contains(&strategic.id));
    }

    /// A [`CascadeValidation`] with 12/12 traced and 100 % alignment reports
    /// full alignment.
    #[test]
    fn cascade_validation_checks_completeness() {
        let validation = CascadeValidation {
            total_operational_goals: 12,
            traced_to_team: 12,
            traced_to_strategic: 12,
            alignment_percent: 100.0,
        };

        assert!(validation.is_fully_aligned());
    }

    /// When fewer Operational goals are traced than exist, alignment is below
    /// 100 % and [`CascadeValidation::is_fully_aligned`] returns `false`.
    #[test]
    fn cascade_validation_incomplete() {
        let validation = CascadeValidation {
            total_operational_goals: 12,
            traced_to_team: 10,
            traced_to_strategic: 10,
            alignment_percent: (10.0_f64 / 12.0_f64) * 100.0,
        };

        assert!(!validation.is_fully_aligned());
    }

    /// A [`WorklogEntry`] correctly records a met goal and a KPI contribution.
    #[test]
    fn worklog_entry_tracks_kpi() {
        let entry = WorklogEntry {
            agent_name: "rust-engineer".to_string(),
            goals: vec![GoalStatus {
                goal_id: "O1".to_string(),
                target: "Cache hit-rate >= 80 %".to_string(),
                current: "84 %".to_string(),
                met: true,
            }],
            kpi_contributions: vec![KpiContribution {
                strategic_kpi: "P95 review latency".to_string(),
                measure: "cache_hit_rate".to_string(),
                value: "0.84".to_string(),
            }],
            artifacts_produced: vec!["nexcore-foundry.rlib".to_string()],
        };

        assert_eq!(entry.goals.len(), 1);
        assert!(entry.goals[0].met);
        assert_eq!(entry.kpi_contributions.len(), 1);
        assert_eq!(
            entry.kpi_contributions[0].strategic_kpi,
            "P95 review latency"
        );
    }

    /// A [`TeamReport`] carries the [`DeliveryStatus`] set by the
    /// orchestrator, and pattern matching on it is exhaustive.
    #[test]
    fn team_report_overall_status() {
        let report = TeamReport {
            strategic_goals: vec![GoalStatus {
                goal_id: "S1".to_string(),
                target: "P95 < 4 h".to_string(),
                current: "3.2 h".to_string(),
                met: true,
            }],
            team_measures: vec![KpiContribution {
                strategic_kpi: "P95 review latency".to_string(),
                measure: "p95_hours".to_string(),
                value: "3.2".to_string(),
            }],
            agent_performance: vec![AgentPerformance {
                agent_name: "rust-engineer".to_string(),
                role: "Backend implementation".to_string(),
                goals_met: "4 / 4".to_string(),
                artifacts: "2 crates".to_string(),
                time: "6 h".to_string(),
            }],
            intelligence_summary: Some("All objectives met; no blockers.".to_string()),
            feedback_constraints: vec!["Maintain zero-allocation public API".to_string()],
            overall: DeliveryStatus::Delivered,
        };

        match report.overall {
            DeliveryStatus::Delivered => {}
            DeliveryStatus::Blocked => panic!("expected Delivered, got Blocked"),
            DeliveryStatus::Iterating => panic!("expected Delivered, got Iterating"),
        }
    }
}
