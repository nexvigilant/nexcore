//! SOC 2 control evidence collection and compliance scoring.
//!
//! Implements the Trust Services Criteria (TSC) framework for SOC 2 Type II:
//! - **Security** (CC): Common criteria — logical/physical access, system operations
//! - **Availability** (A): System uptime and disaster recovery
//! - **Processing Integrity** (PI): Complete, valid, accurate processing
//! - **Confidentiality** (C): Protection of confidential information
//! - **Privacy** (P): Personal information collection, use, retention, disposal
//!
//! Each control tracks its compliance status and evidence type.
//! Scorecards aggregate control status per category with weighted
//! overall scoring for audit readiness assessment.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// SOC 2 Categories
// ============================================================================

/// The five Trust Services Categories defined by AICPA for SOC 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Soc2Category {
    /// Common Criteria (CC) — mandatory for all SOC 2 reports.
    /// Covers logical access, system operations, change management, risk mitigation.
    Security,
    /// System uptime commitments, failover, disaster recovery, BCP.
    Availability,
    /// Data processing completeness, accuracy, timeliness, authorization.
    ProcessingIntegrity,
    /// Protection of information designated as confidential.
    Confidentiality,
    /// Personal information lifecycle: collection, use, retention, disposal.
    Privacy,
}

impl Soc2Category {
    /// Weight of this category in the overall compliance score.
    ///
    /// Weights reflect the relative importance in a pharma SaaS context:
    /// - Security (30%): Foundation of all trust, mandatory for SOC 2
    /// - Confidentiality (25%): Pharma IP and trade secrets
    /// - Privacy (20%): GDPR/CCPA personal data obligations
    /// - Availability (15%): Platform uptime commitments
    /// - Processing Integrity (10%): Data accuracy (lower weight because
    ///   pharma compounds are validated upstream)
    #[must_use]
    pub fn weight(&self) -> f64 {
        match self {
            Self::Security => 0.30,
            Self::Confidentiality => 0.25,
            Self::Privacy => 0.20,
            Self::Availability => 0.15,
            Self::ProcessingIntegrity => 0.10,
        }
    }
}

// ============================================================================
// Evidence Types
// ============================================================================

/// How evidence is collected for a SOC 2 control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    /// Evidence collected automatically (e.g., system logs, config checks).
    Automated,
    /// Evidence collected through manual review (e.g., policy documents).
    Manual,
    /// Combination of automated and manual evidence collection.
    Hybrid,
}

// ============================================================================
// Control Status
// ============================================================================

/// Current compliance status of a SOC 2 control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlStatus {
    /// Control has not been assessed yet.
    NotStarted,
    /// Assessment is in progress (evidence being gathered).
    InProgress,
    /// Control is compliant — evidence demonstrates effectiveness.
    Compliant,
    /// Control is non-compliant — remediation required.
    NonCompliant,
    /// Control does not apply to this system/scope.
    NotApplicable,
}

// ============================================================================
// SOC 2 Control
// ============================================================================

/// A single SOC 2 control with its assessment status and evidence metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soc2Control {
    /// Control identifier (e.g., "CC6.1", "A1.2", "P1.1").
    pub control_id: String,
    /// Trust Services Category this control belongs to.
    pub category: Soc2Category,
    /// Human-readable description of what this control requires.
    pub description: String,
    /// How evidence is collected for this control.
    pub evidence_type: EvidenceType,
    /// Current compliance status.
    pub status: ControlStatus,
    /// When this control was last assessed (None if never).
    pub last_assessed_at: Option<DateTime<Utc>>,
    /// When the next assessment is due (None if not scheduled).
    pub next_assessment_due: Option<DateTime<Utc>>,
}

// ============================================================================
// Compliance Scorecard
// ============================================================================

/// Aggregated compliance metrics for a single SOC 2 category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScorecard {
    /// The category being scored.
    pub category: Soc2Category,
    /// Total number of applicable controls in this category.
    pub controls_total: u32,
    /// Number of controls with Compliant status.
    pub controls_compliant: u32,
    /// Number of controls with NonCompliant status.
    pub controls_non_compliant: u32,
    /// Number of controls with InProgress status.
    pub controls_in_progress: u32,
    /// Compliance percentage (compliant / total * 100). 0.0 if no controls.
    pub compliance_percentage: f64,
}

// ============================================================================
// Scoring Functions
// ============================================================================

/// Calculate a compliance scorecard for a specific SOC 2 category.
///
/// Counts controls by status and computes the compliance percentage.
/// Controls with `NotApplicable` status are excluded from the total.
/// Controls with `NotStarted` status count toward the total but not
/// toward compliance.
#[must_use]
pub fn calculate_scorecard(
    controls: &[Soc2Control],
    category: Soc2Category,
) -> ComplianceScorecard {
    let category_controls: Vec<&Soc2Control> = controls
        .iter()
        .filter(|c| c.category == category && c.status != ControlStatus::NotApplicable)
        .collect();

    let controls_total = category_controls.len() as u32;
    let controls_compliant = category_controls
        .iter()
        .filter(|c| c.status == ControlStatus::Compliant)
        .count() as u32;
    let controls_non_compliant = category_controls
        .iter()
        .filter(|c| c.status == ControlStatus::NonCompliant)
        .count() as u32;
    let controls_in_progress = category_controls
        .iter()
        .filter(|c| c.status == ControlStatus::InProgress)
        .count() as u32;

    let compliance_percentage = if controls_total == 0 {
        0.0
    } else {
        (f64::from(controls_compliant) / f64::from(controls_total)) * 100.0
    };

    ComplianceScorecard {
        category,
        controls_total,
        controls_compliant,
        controls_non_compliant,
        controls_in_progress,
        compliance_percentage,
    }
}

/// Calculate the overall weighted compliance score across all categories.
///
/// Category weights (pharma SaaS context):
/// - Security: 30%
/// - Confidentiality: 25%
/// - Privacy: 20%
/// - Availability: 15%
/// - Processing Integrity: 10%
///
/// Returns a weighted average percentage (0.0 to 100.0).
/// If no scorecards are provided, returns 0.0.
#[must_use]
pub fn overall_compliance_score(scorecards: &[ComplianceScorecard]) -> f64 {
    if scorecards.is_empty() {
        return 0.0;
    }

    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;

    for card in scorecards {
        let weight = card.category.weight();
        weighted_sum += card.compliance_percentage * weight;
        total_weight += weight;
    }

    if total_weight == 0.0 {
        return 0.0;
    }

    weighted_sum / total_weight
}

/// Determine whether the system is ready for a SOC 2 audit.
///
/// The threshold is 95% overall compliance — below this, the auditor
/// is likely to issue exceptions that would result in a qualified opinion.
#[must_use]
pub fn is_audit_ready(score: f64) -> bool {
    score >= 95.0
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_control(
        control_id: &str,
        category: Soc2Category,
        status: ControlStatus,
    ) -> Soc2Control {
        Soc2Control {
            control_id: control_id.to_string(),
            category,
            description: format!("Test control {control_id}"),
            evidence_type: EvidenceType::Automated,
            status,
            last_assessed_at: Some(Utc::now()),
            next_assessment_due: None,
        }
    }

    #[test]
    fn category_weights_sum_to_one() {
        let categories = [
            Soc2Category::Security,
            Soc2Category::Availability,
            Soc2Category::ProcessingIntegrity,
            Soc2Category::Confidentiality,
            Soc2Category::Privacy,
        ];

        let total: f64 = categories.iter().map(|c| c.weight()).sum();
        assert!((total - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn scorecard_all_compliant() {
        let controls = vec![
            make_control("CC1.1", Soc2Category::Security, ControlStatus::Compliant),
            make_control("CC1.2", Soc2Category::Security, ControlStatus::Compliant),
            make_control("CC1.3", Soc2Category::Security, ControlStatus::Compliant),
        ];

        let scorecard = calculate_scorecard(&controls, Soc2Category::Security);
        assert_eq!(scorecard.controls_total, 3);
        assert_eq!(scorecard.controls_compliant, 3);
        assert_eq!(scorecard.controls_non_compliant, 0);
        assert_eq!(scorecard.controls_in_progress, 0);
        assert!((scorecard.compliance_percentage - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn scorecard_mixed_statuses() {
        let controls = vec![
            make_control("CC2.1", Soc2Category::Security, ControlStatus::Compliant),
            make_control("CC2.2", Soc2Category::Security, ControlStatus::NonCompliant),
            make_control("CC2.3", Soc2Category::Security, ControlStatus::InProgress),
            make_control("CC2.4", Soc2Category::Security, ControlStatus::NotStarted),
        ];

        let scorecard = calculate_scorecard(&controls, Soc2Category::Security);
        assert_eq!(scorecard.controls_total, 4);
        assert_eq!(scorecard.controls_compliant, 1);
        assert_eq!(scorecard.controls_non_compliant, 1);
        assert_eq!(scorecard.controls_in_progress, 1);
        assert!((scorecard.compliance_percentage - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn scorecard_excludes_not_applicable() {
        let controls = vec![
            make_control("A1.1", Soc2Category::Availability, ControlStatus::Compliant),
            make_control("A1.2", Soc2Category::Availability, ControlStatus::Compliant),
            make_control(
                "A1.3",
                Soc2Category::Availability,
                ControlStatus::NotApplicable,
            ),
        ];

        let scorecard = calculate_scorecard(&controls, Soc2Category::Availability);
        // NotApplicable is excluded from total
        assert_eq!(scorecard.controls_total, 2);
        assert_eq!(scorecard.controls_compliant, 2);
        assert!((scorecard.compliance_percentage - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn scorecard_filters_by_category() {
        let controls = vec![
            make_control("CC3.1", Soc2Category::Security, ControlStatus::Compliant),
            make_control(
                "A2.1",
                Soc2Category::Availability,
                ControlStatus::NonCompliant,
            ),
            make_control("CC3.2", Soc2Category::Security, ControlStatus::Compliant),
        ];

        let security_card = calculate_scorecard(&controls, Soc2Category::Security);
        assert_eq!(security_card.controls_total, 2);
        assert_eq!(security_card.controls_compliant, 2);

        let avail_card = calculate_scorecard(&controls, Soc2Category::Availability);
        assert_eq!(avail_card.controls_total, 1);
        assert_eq!(avail_card.controls_non_compliant, 1);
    }

    #[test]
    fn scorecard_empty_category_is_zero() {
        let controls: Vec<Soc2Control> = vec![];
        let scorecard = calculate_scorecard(&controls, Soc2Category::Privacy);
        assert_eq!(scorecard.controls_total, 0);
        assert!((scorecard.compliance_percentage - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn overall_score_all_categories_at_100() {
        let scorecards = vec![
            ComplianceScorecard {
                category: Soc2Category::Security,
                controls_total: 10,
                controls_compliant: 10,
                controls_non_compliant: 0,
                controls_in_progress: 0,
                compliance_percentage: 100.0,
            },
            ComplianceScorecard {
                category: Soc2Category::Availability,
                controls_total: 5,
                controls_compliant: 5,
                controls_non_compliant: 0,
                controls_in_progress: 0,
                compliance_percentage: 100.0,
            },
            ComplianceScorecard {
                category: Soc2Category::ProcessingIntegrity,
                controls_total: 3,
                controls_compliant: 3,
                controls_non_compliant: 0,
                controls_in_progress: 0,
                compliance_percentage: 100.0,
            },
            ComplianceScorecard {
                category: Soc2Category::Confidentiality,
                controls_total: 8,
                controls_compliant: 8,
                controls_non_compliant: 0,
                controls_in_progress: 0,
                compliance_percentage: 100.0,
            },
            ComplianceScorecard {
                category: Soc2Category::Privacy,
                controls_total: 6,
                controls_compliant: 6,
                controls_non_compliant: 0,
                controls_in_progress: 0,
                compliance_percentage: 100.0,
            },
        ];

        let score = overall_compliance_score(&scorecards);
        assert!((score - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn overall_score_weighted_correctly() {
        // Security at 100%, everything else at 0%
        // Expected: 100.0 * 0.30 / 1.0 = 30.0 if only Security provided
        // But with all categories: 100 * 0.30 + 0 * 0.25 + 0 * 0.20 + 0 * 0.15 + 0 * 0.10 = 30.0
        let scorecards = vec![
            ComplianceScorecard {
                category: Soc2Category::Security,
                controls_total: 10,
                controls_compliant: 10,
                controls_non_compliant: 0,
                controls_in_progress: 0,
                compliance_percentage: 100.0,
            },
            ComplianceScorecard {
                category: Soc2Category::Availability,
                controls_total: 5,
                controls_compliant: 0,
                controls_non_compliant: 5,
                controls_in_progress: 0,
                compliance_percentage: 0.0,
            },
            ComplianceScorecard {
                category: Soc2Category::ProcessingIntegrity,
                controls_total: 3,
                controls_compliant: 0,
                controls_non_compliant: 3,
                controls_in_progress: 0,
                compliance_percentage: 0.0,
            },
            ComplianceScorecard {
                category: Soc2Category::Confidentiality,
                controls_total: 8,
                controls_compliant: 0,
                controls_non_compliant: 8,
                controls_in_progress: 0,
                compliance_percentage: 0.0,
            },
            ComplianceScorecard {
                category: Soc2Category::Privacy,
                controls_total: 6,
                controls_compliant: 0,
                controls_non_compliant: 6,
                controls_in_progress: 0,
                compliance_percentage: 0.0,
            },
        ];

        let score = overall_compliance_score(&scorecards);
        // 100 * 0.30 / (0.30 + 0.25 + 0.20 + 0.15 + 0.10) = 30.0 / 1.0 = 30.0
        assert!((score - 30.0).abs() < 0.01);
    }

    #[test]
    fn overall_score_partial_categories() {
        // Only Security (weight 0.30) and Privacy (weight 0.20) provided
        let scorecards = vec![
            ComplianceScorecard {
                category: Soc2Category::Security,
                controls_total: 10,
                controls_compliant: 10,
                controls_non_compliant: 0,
                controls_in_progress: 0,
                compliance_percentage: 100.0,
            },
            ComplianceScorecard {
                category: Soc2Category::Privacy,
                controls_total: 6,
                controls_compliant: 3,
                controls_non_compliant: 3,
                controls_in_progress: 0,
                compliance_percentage: 50.0,
            },
        ];

        let score = overall_compliance_score(&scorecards);
        // (100 * 0.30 + 50 * 0.20) / (0.30 + 0.20) = (30 + 10) / 0.50 = 80.0
        assert!((score - 80.0).abs() < 0.01);
    }

    #[test]
    fn overall_score_empty_is_zero() {
        let score = overall_compliance_score(&[]);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn audit_ready_at_95_percent() {
        assert!(is_audit_ready(95.0));
        assert!(is_audit_ready(100.0));
        assert!(is_audit_ready(99.5));
    }

    #[test]
    fn not_audit_ready_below_95() {
        assert!(!is_audit_ready(94.99));
        assert!(!is_audit_ready(90.0));
        assert!(!is_audit_ready(0.0));
    }

    #[test]
    fn soc2_control_serialization_roundtrip() {
        let control = make_control("CC6.1", Soc2Category::Security, ControlStatus::Compliant);
        let json = serde_json::to_string(&control).unwrap();
        let deserialized: Soc2Control = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.control_id, "CC6.1");
        assert_eq!(deserialized.category, Soc2Category::Security);
        assert_eq!(deserialized.status, ControlStatus::Compliant);
    }

    #[test]
    fn compliance_scorecard_serialization_roundtrip() {
        let scorecard = ComplianceScorecard {
            category: Soc2Category::Confidentiality,
            controls_total: 8,
            controls_compliant: 6,
            controls_non_compliant: 1,
            controls_in_progress: 1,
            compliance_percentage: 75.0,
        };

        let json = serde_json::to_string(&scorecard).unwrap();
        let deserialized: ComplianceScorecard = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.category, Soc2Category::Confidentiality);
        assert_eq!(deserialized.controls_total, 8);
        assert!((deserialized.compliance_percentage - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn security_has_highest_weight() {
        assert!(Soc2Category::Security.weight() > Soc2Category::Confidentiality.weight());
        assert!(Soc2Category::Confidentiality.weight() > Soc2Category::Privacy.weight());
        assert!(Soc2Category::Privacy.weight() > Soc2Category::Availability.weight());
        assert!(Soc2Category::Availability.weight() > Soc2Category::ProcessingIntegrity.weight());
    }
}
