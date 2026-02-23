//! Markdown report renderers for The Foundry architecture.
//!
//! Converts typed artifacts from the analyst and governance modules into
//! human-readable markdown suitable for terminal display, GitHub comments,
//! or Brain artifact storage.
//!
//! # Example
//!
//! ```
//! use nexcore_foundry::analyst::{IntelligenceReport, RiskLevel};
//! use nexcore_foundry::report::render_intelligence_report;
//!
//! let report = IntelligenceReport {
//!     findings: vec!["High complexity in 3 modules".to_string()],
//!     recommendations: vec!["Decompose large functions".to_string()],
//!     risk_level: RiskLevel::Moderate,
//!     confidence: 0.85,
//! };
//!
//! let md = render_intelligence_report(&report);
//! assert!(md.contains("## Findings"));
//! assert!(md.contains("Moderate"));
//! ```

use crate::analyst::{IntelligenceReport, RiskLevel};
use crate::governance::{DeliveryStatus, TeamReport};

// ---------------------------------------------------------------------------
// Intelligence report renderer
// ---------------------------------------------------------------------------

/// Renders an [`IntelligenceReport`] as a markdown document.
///
/// The output includes sections for risk level, confidence, findings, and
/// recommendations.
///
/// # Example
///
/// ```
/// use nexcore_foundry::analyst::{IntelligenceReport, RiskLevel};
/// use nexcore_foundry::report::render_intelligence_report;
///
/// let report = IntelligenceReport {
///     findings: vec!["Elevated cyclomatic complexity".to_string()],
///     recommendations: vec!["Extract helper functions".to_string()],
///     risk_level: RiskLevel::Low,
///     confidence: 0.92,
/// };
///
/// let md = render_intelligence_report(&report);
/// assert!(md.contains("# Intelligence Report"));
/// assert!(md.contains("Low"));
/// ```
#[must_use]
pub fn render_intelligence_report(report: &IntelligenceReport) -> String {
    let mut out = String::with_capacity(512);

    out.push_str("# Intelligence Report\n\n");

    out.push_str("| Field | Value |\n|-------|-------|\n");
    out.push_str(&format!(
        "| Risk Level | {} |\n",
        risk_level_label(&report.risk_level)
    ));
    out.push_str(&format!(
        "| Confidence | {:.0}% |\n\n",
        report.confidence * 100.0
    ));

    if !report.findings.is_empty() {
        out.push_str("## Findings\n\n");
        for f in &report.findings {
            out.push_str(&format!("- {f}\n"));
        }
        out.push('\n');
    }

    if !report.recommendations.is_empty() {
        out.push_str("## Recommendations\n\n");
        for (i, r) in report.recommendations.iter().enumerate() {
            out.push_str(&format!("{}. {r}\n", i + 1));
        }
        out.push('\n');
    }

    out
}

/// Returns a human-readable label for a [`RiskLevel`].
#[must_use]
pub fn risk_level_label(level: &RiskLevel) -> &'static str {
    match level {
        RiskLevel::Low => "Low",
        RiskLevel::Moderate => "Moderate",
        RiskLevel::High => "High",
        RiskLevel::Critical => "Critical",
    }
}

// ---------------------------------------------------------------------------
// Team report renderer
// ---------------------------------------------------------------------------

/// Renders a [`TeamReport`] as a markdown document.
///
/// The output includes sections for delivery status, strategic goals, team
/// KPI contributions, per-agent performance, and feedback constraints.
///
/// # Example
///
/// ```
/// use nexcore_foundry::governance::{
///     AgentPerformance, DeliveryStatus, GoalStatus, KpiContribution, TeamReport,
/// };
/// use nexcore_foundry::report::render_team_report;
///
/// let report = TeamReport {
///     strategic_goals: vec![GoalStatus {
///         goal_id: "S1".to_string(),
///         target: "P95 < 4h".to_string(),
///         current: "3.1h".to_string(),
///         met: true,
///     }],
///     team_measures: vec![],
///     agent_performance: vec![AgentPerformance {
///         agent_name: "rust-eng".to_string(),
///         role: "Backend".to_string(),
///         goals_met: "3/3".to_string(),
///         artifacts: "2 crates".to_string(),
///         time: "4h".to_string(),
///     }],
///     intelligence_summary: Some("All green.".to_string()),
///     feedback_constraints: vec![],
///     overall: DeliveryStatus::Delivered,
/// };
///
/// let md = render_team_report(&report);
/// assert!(md.contains("# Team Report"));
/// assert!(md.contains("Delivered"));
/// ```
#[must_use]
pub fn render_team_report(report: &TeamReport) -> String {
    let mut out = String::with_capacity(1024);

    out.push_str("# Team Report\n\n");
    out.push_str(&format!(
        "**Status:** {}\n\n",
        delivery_status_label(&report.overall)
    ));

    // Strategic goals table
    if !report.strategic_goals.is_empty() {
        out.push_str("## Strategic Goals\n\n");
        out.push_str("| Goal | Target | Current | Met |\n");
        out.push_str("|------|--------|---------|-----|\n");
        for g in &report.strategic_goals {
            let check = if g.met { "yes" } else { "no" };
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                g.goal_id, g.target, g.current, check
            ));
        }
        out.push('\n');
    }

    // Team KPI measures
    if !report.team_measures.is_empty() {
        out.push_str("## Team KPI Contributions\n\n");
        out.push_str("| KPI | Measure | Value |\n");
        out.push_str("|-----|---------|-------|\n");
        for m in &report.team_measures {
            out.push_str(&format!(
                "| {} | {} | {} |\n",
                m.strategic_kpi, m.measure, m.value
            ));
        }
        out.push('\n');
    }

    // Agent performance table
    if !report.agent_performance.is_empty() {
        out.push_str("## Agent Performance\n\n");
        out.push_str("| Agent | Role | Goals Met | Artifacts | Time |\n");
        out.push_str("|-------|------|-----------|-----------|------|\n");
        for a in &report.agent_performance {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                a.agent_name, a.role, a.goals_met, a.artifacts, a.time
            ));
        }
        out.push('\n');
    }

    // Intelligence summary
    if let Some(summary) = &report.intelligence_summary {
        out.push_str("## Intelligence Summary\n\n");
        out.push_str(summary);
        out.push_str("\n\n");
    }

    // Feedback constraints
    if !report.feedback_constraints.is_empty() {
        out.push_str("## Feedback Constraints\n\n");
        for c in &report.feedback_constraints {
            out.push_str(&format!("- {c}\n"));
        }
        out.push('\n');
    }

    out
}

/// Returns a human-readable label for a [`DeliveryStatus`].
#[must_use]
pub fn delivery_status_label(status: &DeliveryStatus) -> &'static str {
    match status {
        DeliveryStatus::Delivered => "Delivered",
        DeliveryStatus::Blocked => "Blocked",
        DeliveryStatus::Iterating => "Iterating",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyst::{IntelligenceReport, RiskLevel};
    use crate::governance::{
        AgentPerformance, DeliveryStatus, GoalStatus, KpiContribution, TeamReport,
    };

    #[test]
    fn intelligence_report_markdown_contains_sections() {
        let report = IntelligenceReport {
            findings: vec![
                "Module X exceeds complexity threshold".to_string(),
                "Test coverage regressed by 3%".to_string(),
            ],
            recommendations: vec![
                "Decompose process_pipeline into smaller functions".to_string(),
                "Add integration tests for the payment module".to_string(),
            ],
            risk_level: RiskLevel::High,
            confidence: 0.88,
        };

        let md = render_intelligence_report(&report);

        assert!(md.contains("# Intelligence Report"));
        assert!(md.contains("High"));
        assert!(md.contains("88%"));
        assert!(md.contains("## Findings"));
        assert!(md.contains("Module X exceeds complexity threshold"));
        assert!(md.contains("## Recommendations"));
        assert!(md.contains("1. Decompose"));
        assert!(md.contains("2. Add integration"));
    }

    #[test]
    fn intelligence_report_empty_sections_omitted() {
        let report = IntelligenceReport {
            findings: vec![],
            recommendations: vec![],
            risk_level: RiskLevel::Low,
            confidence: 0.99,
        };

        let md = render_intelligence_report(&report);

        assert!(md.contains("Low"));
        assert!(!md.contains("## Findings"));
        assert!(!md.contains("## Recommendations"));
    }

    #[test]
    fn team_report_markdown_contains_sections() {
        let report = TeamReport {
            strategic_goals: vec![GoalStatus {
                goal_id: "S1".to_string(),
                target: "P95 < 4h".to_string(),
                current: "3.1h".to_string(),
                met: true,
            }],
            team_measures: vec![KpiContribution {
                strategic_kpi: "Latency".to_string(),
                measure: "p95_hours".to_string(),
                value: "3.1".to_string(),
            }],
            agent_performance: vec![AgentPerformance {
                agent_name: "rust-eng".to_string(),
                role: "Backend".to_string(),
                goals_met: "4/4".to_string(),
                artifacts: "2 crates".to_string(),
                time: "6h".to_string(),
            }],
            intelligence_summary: Some("All systems nominal.".to_string()),
            feedback_constraints: vec!["Maintain zero-alloc API".to_string()],
            overall: DeliveryStatus::Delivered,
        };

        let md = render_team_report(&report);

        assert!(md.contains("# Team Report"));
        assert!(md.contains("Delivered"));
        assert!(md.contains("## Strategic Goals"));
        assert!(md.contains("S1"));
        assert!(md.contains("## Team KPI Contributions"));
        assert!(md.contains("Latency"));
        assert!(md.contains("## Agent Performance"));
        assert!(md.contains("rust-eng"));
        assert!(md.contains("## Intelligence Summary"));
        assert!(md.contains("All systems nominal."));
        assert!(md.contains("## Feedback Constraints"));
        assert!(md.contains("Maintain zero-alloc API"));
    }

    #[test]
    fn team_report_blocked_status() {
        let report = TeamReport {
            strategic_goals: vec![],
            team_measures: vec![],
            agent_performance: vec![],
            intelligence_summary: None,
            feedback_constraints: vec![],
            overall: DeliveryStatus::Blocked,
        };

        let md = render_team_report(&report);
        assert!(md.contains("Blocked"));
    }

    #[test]
    fn risk_level_labels_correct() {
        assert_eq!(risk_level_label(&RiskLevel::Low), "Low");
        assert_eq!(risk_level_label(&RiskLevel::Moderate), "Moderate");
        assert_eq!(risk_level_label(&RiskLevel::High), "High");
        assert_eq!(risk_level_label(&RiskLevel::Critical), "Critical");
    }

    #[test]
    fn delivery_status_labels_correct() {
        assert_eq!(
            delivery_status_label(&DeliveryStatus::Delivered),
            "Delivered"
        );
        assert_eq!(delivery_status_label(&DeliveryStatus::Blocked), "Blocked");
        assert_eq!(
            delivery_status_label(&DeliveryStatus::Iterating),
            "Iterating"
        );
    }
}
