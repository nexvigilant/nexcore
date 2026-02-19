//! # Periodic Safety Reporting (PSUR/PBRER)
//!
//! Models periodic safety update reports per ICH E2C(R2).
//!
//! ## Report Types
//!
//! | Report | Authority | Regulation |
//! |--------|-----------|------------|
//! | PSUR | EMA | ICH E2C(R2) |
//! | PBRER | ICH harmonized | ICH E2C(R2) |
//! | PADER | FDA | 21 CFR 314.80 |
//! | DSUR | ICH (clinical trials) | ICH E2F |
//!
//! ## E2C(R2) PBRER Sections
//!
//! 1. Introduction
//! 2. Worldwide marketing authorization status
//! 3. Actions taken for safety reasons
//! 4. Changes to reference safety information
//! 5. Estimated exposure
//! 6. Data in summary tabulations
//! 7. Summaries of significant findings
//! 8. Signal evaluation
//! 9. Evaluation of risks and new information
//! 10. Benefit evaluation
//! 11. Integrated benefit-risk analysis
//! 12. Conclusions and actions
//!
//! ## T1 Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|-------------|--------|
//! | Reporting period | Sequence | σ |
//! | Data aggregation | Quantity | N |
//! | Benefit-risk eval | Comparison | κ |
//! | Report state | State | ς |

use serde::{Deserialize, Serialize};

use super::icsr::CaseId;

// ═══════════════════════════════════════════════════════════════════════════
// PERIODIC REPORT
// ═══════════════════════════════════════════════════════════════════════════

/// A periodic safety report (PSUR/PBRER/PADER/DSUR).
///
/// # Tier: T3 (Domain-Specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodicReport {
    /// Report identifier
    pub id: String,
    /// Drug or product name
    pub drug_name: String,
    /// Report type
    pub report_type: PeriodicReportType,
    /// International Birth Date (IBD) — first marketing authorization date
    pub international_birth_date: String,
    /// Data Lock Point — end of reporting period
    pub data_lock_point: String,
    /// Reporting period start
    pub period_start: String,
    /// Reporting period end (= DLP)
    pub period_end: String,
    /// Report lifecycle state
    pub state: ReportState,
    /// Report sections (E2C(R2) structure)
    pub sections: ReportSections,
}

/// Periodic report types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeriodicReportType {
    /// Periodic Safety Update Report (EMA term)
    Psur,
    /// Periodic Benefit-Risk Evaluation Report (ICH harmonized)
    Pbrer,
    /// Periodic Adverse Drug Experience Report (FDA)
    Pader,
    /// Development Safety Update Report (clinical trials)
    Dsur,
}

/// Report lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportState {
    /// Data collection in progress
    DataCollection,
    /// Drafting in progress
    Drafting,
    /// Under medical review
    MedicalReview,
    /// Under quality review
    QualityReview,
    /// Submitted to authority
    Submitted,
    /// Accepted by authority
    Accepted,
    /// Authority requests supplemental data
    QueryReceived,
}

// ═══════════════════════════════════════════════════════════════════════════
// REPORT SECTIONS (E2C(R2))
// ═══════════════════════════════════════════════════════════════════════════

/// E2C(R2) PBRER sections.
///
/// # Tier: T2-C (composed from T1: Sequence + Quantity + Comparison)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportSections {
    /// Section 5: Patient exposure estimates
    pub exposure: ExposureData,
    /// Section 6: Summary tabulations
    pub tabulations: SummaryTabulations,
    /// Section 8: Signal evaluation
    pub signal_evaluation: SignalEvaluation,
    /// Section 11: Integrated benefit-risk analysis
    pub benefit_risk: BenefitRiskSummary,
    /// Section 12: Conclusions and actions
    pub conclusions: Vec<Conclusion>,
}

/// Patient exposure estimates (Section 5).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExposureData {
    /// Cumulative patient-years since IBD
    pub cumulative_patient_years: f64,
    /// Patient-years during this reporting period
    pub interval_patient_years: f64,
    /// Estimated number of patients exposed (cumulative)
    pub cumulative_patients: u64,
    /// Estimated number of patients exposed (interval)
    pub interval_patients: u64,
    /// Data sources for exposure estimates
    pub sources: Vec<String>,
}

/// Summary tabulations of ICSRs (Section 6).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SummaryTabulations {
    /// Total ICSRs in interval
    pub interval_case_count: u32,
    /// Total ICSRs cumulative
    pub cumulative_case_count: u32,
    /// Serious cases in interval
    pub interval_serious: u32,
    /// Fatal cases in interval
    pub interval_fatal: u32,
    /// Case IDs referenced in this report
    pub referenced_cases: Vec<CaseId>,
}

/// Signal evaluation summary (Section 8).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignalEvaluation {
    /// New signals detected during interval
    pub new_signals: Vec<SignalSummary>,
    /// Ongoing signals under evaluation
    pub ongoing_signals: Vec<SignalSummary>,
    /// Signals closed during interval
    pub closed_signals: Vec<SignalSummary>,
}

/// Summary of a single signal for periodic reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalSummary {
    /// Drug-event pair
    pub drug_event: String,
    /// Signal detection method used
    pub method: String,
    /// Current signal status
    pub status: SignalStatus,
    /// Evaluation conclusion
    pub evaluation: Option<String>,
}

/// Signal lifecycle status within periodic reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalStatus {
    /// Newly detected, under evaluation
    New,
    /// Under ongoing evaluation
    Ongoing,
    /// Confirmed — action taken
    Confirmed,
    /// Refuted — false positive
    Refuted,
    /// Closed — no further action
    Closed,
}

/// Benefit-risk analysis summary (Section 11).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BenefitRiskSummary {
    /// Overall benefit-risk assessment
    pub assessment: BenefitRiskAssessment,
    /// Whether the B/R profile has changed since last report
    pub profile_changed: bool,
    /// Key new benefits identified
    pub new_benefits: Vec<String>,
    /// Key new risks identified
    pub new_risks: Vec<String>,
}

/// Overall benefit-risk assessment category.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenefitRiskAssessment {
    /// Benefits clearly outweigh risks
    Favorable,
    /// Benefits outweigh risks with additional monitoring
    #[default]
    FavorableWithMonitoring,
    /// Benefit-risk balance is uncertain
    Uncertain,
    /// Risks may outweigh benefits
    Unfavorable,
}

/// Report conclusion and recommended action (Section 12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conclusion {
    /// Finding description
    pub finding: String,
    /// Recommended action
    pub action: ConclusionAction,
}

/// Actions from periodic report conclusions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConclusionAction {
    /// No change to current management
    NoChange,
    /// Update product labeling
    LabelUpdate { section: String },
    /// Update the Risk Management Plan/REMS
    UpdateRiskManagement,
    /// Submit urgent safety restriction
    UrgentSafetyRestriction,
    /// Initiate clinical study for further evaluation
    InitiateStudy,
    /// Submit supplemental data to authority
    SupplementalData,
}

impl PeriodicReport {
    /// Create a new periodic report in DataCollection state.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        drug_name: impl Into<String>,
        report_type: PeriodicReportType,
        period_start: impl Into<String>,
        period_end: impl Into<String>,
    ) -> Self {
        let period_end_str = period_end.into();
        Self {
            id: id.into(),
            drug_name: drug_name.into(),
            report_type,
            international_birth_date: String::new(),
            data_lock_point: period_end_str.clone(),
            period_start: period_start.into(),
            period_end: period_end_str,
            state: ReportState::DataCollection,
            sections: ReportSections::default(),
        }
    }

    /// Reporting rate: interval cases per 1000 patient-years.
    #[must_use]
    pub fn reporting_rate_per_1k_py(&self) -> f64 {
        let py = self.sections.exposure.interval_patient_years;
        if py <= 0.0 {
            return 0.0;
        }
        f64::from(self.sections.tabulations.interval_case_count) / py * 1000.0
    }

    /// Serious case proportion in interval.
    #[must_use]
    pub fn serious_proportion(&self) -> f64 {
        let total = self.sections.tabulations.interval_case_count;
        if total == 0 {
            return 0.0;
        }
        f64::from(self.sections.tabulations.interval_serious) / f64::from(total)
    }

    /// Total new + ongoing signals.
    #[must_use]
    pub fn active_signal_count(&self) -> usize {
        self.sections.signal_evaluation.new_signals.len()
            + self.sections.signal_evaluation.ongoing_signals.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pbrer() -> PeriodicReport {
        let mut report = PeriodicReport::new(
            "PBRER-2024-H2",
            "Rivaroxaban",
            PeriodicReportType::Pbrer,
            "2024-07-01",
            "2024-12-31",
        );
        report.international_birth_date = "2008-09-30".into();

        // Section 5: Exposure
        report.sections.exposure = ExposureData {
            cumulative_patient_years: 250_000.0,
            interval_patient_years: 25_000.0,
            cumulative_patients: 500_000,
            interval_patients: 50_000,
            sources: vec!["IMS Health".into(), "Registry data".into()],
        };

        // Section 6: Tabulations
        report.sections.tabulations = SummaryTabulations {
            interval_case_count: 150,
            cumulative_case_count: 3200,
            interval_serious: 45,
            interval_fatal: 5,
            referenced_cases: vec![CaseId::new("US-FDA-2024-001")],
        };

        // Section 8: Signal evaluation
        report.sections.signal_evaluation = SignalEvaluation {
            new_signals: vec![SignalSummary {
                drug_event: "Rivaroxaban-Liver injury".into(),
                method: "EBGM".into(),
                status: SignalStatus::New,
                evaluation: None,
            }],
            ongoing_signals: vec![],
            closed_signals: vec![SignalSummary {
                drug_event: "Rivaroxaban-GI bleeding".into(),
                method: "PRR".into(),
                status: SignalStatus::Confirmed,
                evaluation: Some("Label updated 2024-Q1".into()),
            }],
        };

        // Section 11: B/R
        report.sections.benefit_risk = BenefitRiskSummary {
            assessment: BenefitRiskAssessment::FavorableWithMonitoring,
            profile_changed: false,
            new_benefits: vec![],
            new_risks: vec!["Potential liver injury signal".into()],
        };

        // Section 12: Conclusions
        report.sections.conclusions = vec![Conclusion {
            finding: "New liver injury signal requires further evaluation".into(),
            action: ConclusionAction::InitiateStudy,
        }];

        report
    }

    #[test]
    fn reporting_rate_calculation() {
        let report = sample_pbrer();
        let rate = report.reporting_rate_per_1k_py();
        // 150 / 25000 * 1000 = 6.0
        assert!((rate - 6.0).abs() < 0.01);
    }

    #[test]
    fn serious_proportion_calculation() {
        let report = sample_pbrer();
        let prop = report.serious_proportion();
        // 45 / 150 = 0.3
        assert!((prop - 0.3).abs() < 0.01);
    }

    #[test]
    fn active_signal_count() {
        let report = sample_pbrer();
        assert_eq!(report.active_signal_count(), 1); // 1 new + 0 ongoing
    }

    #[test]
    fn zero_exposure_handling() {
        let report = PeriodicReport::new(
            "PBRER-EMPTY",
            "DrugX",
            PeriodicReportType::Pbrer,
            "2024-01-01",
            "2024-06-30",
        );
        assert_eq!(report.reporting_rate_per_1k_py(), 0.0);
        assert_eq!(report.serious_proportion(), 0.0);
    }

    #[test]
    fn benefit_risk_default() {
        let br = BenefitRiskAssessment::default();
        assert_eq!(br, BenefitRiskAssessment::FavorableWithMonitoring);
    }

    #[test]
    fn signal_status_variants() {
        assert_ne!(SignalStatus::New, SignalStatus::Confirmed);
        assert_ne!(SignalStatus::Refuted, SignalStatus::Closed);
    }

    #[test]
    fn conclusion_action_equality() {
        assert_eq!(ConclusionAction::NoChange, ConclusionAction::NoChange);
        assert_ne!(
            ConclusionAction::NoChange,
            ConclusionAction::UrgentSafetyRestriction
        );
    }

    #[test]
    fn dsur_creation() {
        let dsur = PeriodicReport::new(
            "DSUR-2024-001",
            "Experimental-123",
            PeriodicReportType::Dsur,
            "2024-01-01",
            "2024-12-31",
        );
        assert_eq!(dsur.report_type, PeriodicReportType::Dsur);
        assert_eq!(dsur.state, ReportState::DataCollection);
    }
}
