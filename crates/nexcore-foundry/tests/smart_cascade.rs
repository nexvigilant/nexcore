//! Integration test — SMART goal cascade validation.
//!
//! Verifies the governance type system supports the full cascade pattern:
//! 1 strategic goal → 2 team goals → 6 operational goals.
//! Tests alignment checking, partial alignment detection, serialization
//! round-trips, and markdown report rendering.

use nexcore_foundry::governance::{
    AgentPerformance, CascadeValidation, DeliveryStatus, GoalLevel, GoalStatus, KpiContribution,
    SmartGoal, TeamReport, WorklogEntry,
};
use nexcore_foundry::report::render_team_report;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn strategic_goal() -> SmartGoal {
    SmartGoal {
        id: "SK-1".to_string(),
        level: GoalLevel::Strategic,
        specific: "Achieve zero critical regressions in Q1 releases".to_string(),
        measurable: "Critical regression count == 0".to_string(),
        achievable: "Historical data shows 2 per quarter; target is elimination".to_string(),
        relevant: "Directly supports patient safety and release confidence".to_string(),
        time_bound: "2026-Q1".to_string(),
        traces_to: vec![],
    }
}

fn team_goal(id: &str, traces_to: &str) -> SmartGoal {
    SmartGoal {
        id: id.to_string(),
        level: GoalLevel::Team,
        specific: format!("Deliver {id} capabilities on schedule"),
        measurable: "All station outputs validated green".to_string(),
        achievable: "Team capacity supports 3-sprint delivery".to_string(),
        relevant: format!("Traces to strategic goal {traces_to}"),
        time_bound: "Sprint 3".to_string(),
        traces_to: vec![traces_to.to_string()],
    }
}

fn operational_goal(id: &str, traces_to: &str) -> SmartGoal {
    SmartGoal {
        id: id.to_string(),
        level: GoalLevel::Operational,
        specific: format!("Complete {id} deliverable"),
        measurable: "Artifact passes validation gate".to_string(),
        achievable: "Single-agent scope".to_string(),
        relevant: format!("Required by team goal {traces_to}"),
        time_bound: "Sprint 1".to_string(),
        traces_to: vec![traces_to.to_string()],
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Full cascade: 1 strategic → 2 team → 6 operational.
/// All operational goals trace to a team goal, and both team goals trace
/// to the strategic goal. The cascade should report 100% alignment.
#[test]
fn full_cascade_is_fully_aligned() {
    let _sk1 = strategic_goal();
    let _tg1 = team_goal("TG-1", "SK-1");
    let _tg2 = team_goal("TG-2", "SK-1");
    let _op1 = operational_goal("OP-1", "TG-1");
    let _op2 = operational_goal("OP-2", "TG-1");
    let _op3 = operational_goal("OP-3", "TG-1");
    let _op4 = operational_goal("OP-4", "TG-2");
    let _op5 = operational_goal("OP-5", "TG-2");
    let _op6 = operational_goal("OP-6", "TG-2");

    let validation = CascadeValidation {
        total_operational_goals: 6,
        traced_to_team: 6,
        traced_to_strategic: 6,
        alignment_percent: 100.0,
    };

    assert!(
        validation.is_fully_aligned(),
        "full cascade should be 100% aligned"
    );
}

/// Removing one operational goal's trace should drop alignment below 100%.
#[test]
fn partial_cascade_is_not_fully_aligned() {
    let _sk1 = strategic_goal();
    let _tg1 = team_goal("TG-1", "SK-1");

    // One orphaned operational goal (no traces_to).
    let _orphan = SmartGoal {
        id: "OP-ORPHAN".to_string(),
        level: GoalLevel::Operational,
        specific: "Orphaned task".to_string(),
        measurable: "N/A".to_string(),
        achievable: "N/A".to_string(),
        relevant: "Not linked to any team goal".to_string(),
        time_bound: "Sprint 1".to_string(),
        traces_to: vec![],
    };

    let validation = CascadeValidation {
        total_operational_goals: 1,
        traced_to_team: 0,
        traced_to_strategic: 0,
        alignment_percent: 0.0,
    };

    assert!(
        !validation.is_fully_aligned(),
        "cascade with orphaned goal should not be fully aligned"
    );
}

/// Verify that `traces_to` correctly links an operational goal to its
/// team-level parent.
#[test]
fn traces_to_links_operational_to_team() {
    let tg1 = team_goal("TG-1", "SK-1");
    let op1 = operational_goal("OP-1", "TG-1");

    assert!(!op1.traces_to.is_empty());
    assert_eq!(op1.traces_to[0], "TG-1");
    assert!(!tg1.traces_to.is_empty());
    assert_eq!(tg1.traces_to[0], "SK-1");
}

/// `GoalLevel` variants distinguish strategic, team, and operational goals.
#[test]
fn goal_level_hierarchy() {
    let s = strategic_goal();
    let t = team_goal("TG-1", "SK-1");
    let o = operational_goal("OP-1", "TG-1");

    assert_eq!(s.level, GoalLevel::Strategic);
    assert_eq!(t.level, GoalLevel::Team);
    assert_eq!(o.level, GoalLevel::Operational);
}

/// `GoalStatus` tracks goal progress with target and current values.
#[test]
fn goal_status_tracks_progress() {
    let status = GoalStatus {
        goal_id: "SK-1".to_string(),
        target: "Critical regression count == 0".to_string(),
        current: "0 regressions observed".to_string(),
        met: true,
    };
    assert!(status.met);
    assert_eq!(status.goal_id, "SK-1");
}

/// `KpiContribution` round-trips through JSON.
#[test]
fn kpi_contribution_serde_roundtrip() {
    let kpi = KpiContribution {
        strategic_kpi: "regression_count".to_string(),
        measure: "critical_regression_rate".to_string(),
        value: "0.0".to_string(),
    };

    let json = serde_json::to_string(&kpi).expect("serialisation");
    let recovered: KpiContribution = serde_json::from_str(&json).expect("deserialisation");

    assert_eq!(recovered.strategic_kpi, "regression_count");
    assert_eq!(recovered.measure, "critical_regression_rate");
    assert_eq!(recovered.value, "0.0");
}

/// `TeamReport` renders to markdown with expected sections.
#[test]
fn team_report_renders_to_markdown() {
    let report = TeamReport {
        strategic_goals: vec![GoalStatus {
            goal_id: "SK-1".to_string(),
            target: "Zero regressions".to_string(),
            current: "0".to_string(),
            met: true,
        }],
        team_measures: vec![KpiContribution {
            strategic_kpi: "test_coverage".to_string(),
            measure: "line_coverage".to_string(),
            value: "95%".to_string(),
        }],
        agent_performance: vec![AgentPerformance {
            agent_name: "foundry-frame".to_string(),
            role: "Builder agent".to_string(),
            goals_met: "3/3".to_string(),
            artifacts: "2 crates".to_string(),
            time: "4h".to_string(),
        }],
        intelligence_summary: Some("All stations green.".to_string()),
        feedback_constraints: vec![],
        overall: DeliveryStatus::Delivered,
    };

    let md = render_team_report(&report);

    assert!(md.contains("# Team Report"), "should contain title");
    assert!(md.contains("Delivered"), "should contain delivery status");
    assert!(
        md.contains("test_coverage"),
        "should contain KPI name"
    );
    assert!(
        md.contains("foundry-frame"),
        "should contain agent name"
    );
    assert!(
        md.contains("All stations green."),
        "should contain intelligence summary"
    );
}

/// Delivery status labels in markdown rendering.
#[test]
fn delivery_status_variants_render() {
    for status in [
        DeliveryStatus::Delivered,
        DeliveryStatus::Blocked,
        DeliveryStatus::Iterating,
    ] {
        let report = TeamReport {
            strategic_goals: vec![],
            team_measures: vec![],
            agent_performance: vec![],
            intelligence_summary: None,
            feedback_constraints: vec![],
            overall: status,
        };
        let md = render_team_report(&report);
        assert!(!md.is_empty(), "markdown should not be empty");
    }
}

/// `WorklogEntry` records agent contributions and goals.
#[test]
fn worklog_entry_records_progress() {
    let entry = WorklogEntry {
        agent_name: "foundry-measure".to_string(),
        goals: vec![GoalStatus {
            goal_id: "OP-1".to_string(),
            target: "Coverage >= 90%".to_string(),
            current: "92%".to_string(),
            met: true,
        }],
        kpi_contributions: vec![KpiContribution {
            strategic_kpi: "code_quality".to_string(),
            measure: "coverage_percent".to_string(),
            value: "92".to_string(),
        }],
        artifacts_produced: vec!["nexcore-foundry.rlib".to_string()],
    };

    assert_eq!(entry.goals.len(), 1);
    assert!(entry.goals[0].met);
    assert_eq!(entry.kpi_contributions.len(), 1);
    assert_eq!(entry.artifacts_produced.len(), 1);
}

/// `SmartGoal` serializes and deserializes correctly.
#[test]
fn smart_goal_serde_roundtrip() {
    let goal = strategic_goal();
    let json = serde_json::to_string(&goal).expect("serialisation");
    let recovered: SmartGoal = serde_json::from_str(&json).expect("deserialisation");

    assert_eq!(recovered.id, "SK-1");
    assert_eq!(recovered.level, GoalLevel::Strategic);
    assert!(recovered.traces_to.is_empty());
}
