//! Reportability Primitives for Pharmacovigilance.
//!
//! Determines reporting requirements based on regulatory criteria:
//! - Expedited reporting (7/15 day) determination
//! - Deadline calculation from awareness date
//! - Seriousness criteria evaluation
//! - Regulatory authority-specific requirements

use serde::{Deserialize, Serialize};

use super::RegulatoryAuthority;
use crate::expectedness::Expectedness;

// =============================================================================
// Seriousness Criteria
// =============================================================================

/// ICH E2A seriousness criteria.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeriousnessCriterion {
    /// Results in death.
    Death,
    /// Is life-threatening.
    LifeThreatening,
    /// Requires or prolongs hospitalization.
    Hospitalization,
    /// Results in persistent or significant disability/incapacity.
    Disability,
    /// Is a congenital anomaly/birth defect.
    CongenitalAnomaly,
    /// Other medically important condition.
    MedicallyImportant,
}

impl SeriousnessCriterion {
    /// ICH E2A code for this criterion.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Death => "1",
            Self::LifeThreatening => "2",
            Self::Hospitalization => "3",
            Self::Disability => "4",
            Self::CongenitalAnomaly => "5",
            Self::MedicallyImportant => "6",
        }
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Death => "Results in death",
            Self::LifeThreatening => "Is life-threatening",
            Self::Hospitalization => {
                "Requires inpatient hospitalization or prolongs existing hospitalization"
            }
            Self::Disability => "Results in persistent or significant disability/incapacity",
            Self::CongenitalAnomaly => "Is a congenital anomaly/birth defect",
            Self::MedicallyImportant => "Is a medically important event or reaction",
        }
    }

    /// Urgency weight for prioritization (higher = more urgent).
    #[must_use]
    pub const fn urgency_weight(&self) -> u8 {
        match self {
            Self::Death => 10,
            Self::LifeThreatening => 9,
            Self::CongenitalAnomaly => 7,
            Self::Hospitalization => 6,
            Self::Disability => 5,
            Self::MedicallyImportant => 4,
        }
    }
}

/// Full seriousness assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriousnessAssessment {
    /// Whether the event is serious (any criterion met).
    pub is_serious: bool,
    /// Criteria that are met.
    pub criteria_met: Vec<SeriousnessCriterion>,
    /// Highest urgency weight among met criteria.
    pub max_urgency: u8,
}

/// Assess seriousness from a list of criteria.
#[must_use]
pub fn assess_seriousness(criteria: &[SeriousnessCriterion]) -> SeriousnessAssessment {
    let is_serious = !criteria.is_empty();
    let max_urgency = criteria
        .iter()
        .map(|c| c.urgency_weight())
        .max()
        .unwrap_or(0);

    SeriousnessAssessment {
        is_serious,
        criteria_met: criteria.to_vec(),
        max_urgency,
    }
}

// =============================================================================
// Expedited Reporting
// =============================================================================

/// Expedited reporting determination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpeditedReporting {
    /// Whether expedited reporting is required.
    pub required: bool,
    /// Deadline in calendar days from awareness.
    pub deadline_days: u32,
    /// Deadline category.
    pub category: ReportingCategory,
    /// Regulatory authorities requiring this report.
    pub authorities: Vec<RegulatoryAuthority>,
    /// Reason for determination.
    pub rationale: String,
}

/// Reporting category based on urgency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportingCategory {
    /// 7-day report (fatal/life-threatening unexpected).
    SevenDay,
    /// 15-day report (serious unexpected).
    FifteenDay,
    /// Periodic report (non-expedited).
    Periodic,
    /// No reporting required (non-serious, expected).
    NonReportable,
}

impl ReportingCategory {
    /// Days until deadline.
    #[must_use]
    pub const fn days(&self) -> u32 {
        match self {
            Self::SevenDay => 7,
            Self::FifteenDay => 15,
            Self::Periodic => 90,     // Quarterly
            Self::NonReportable => 0, // N/A
        }
    }
}

/// Determine expedited reporting requirements.
#[must_use]
pub fn determine_expedited(
    seriousness: &SeriousnessAssessment,
    expectedness: Expectedness,
    is_clinical_trial: bool,
) -> ExpeditedReporting {
    // Decision tree for expedited reporting
    let (required, category, rationale) = match (seriousness.is_serious, expectedness) {
        // SUSAR (Suspected Unexpected Serious Adverse Reaction)
        (true, Expectedness::Unlisted) => {
            // Check for fatal/life-threatening
            let is_fatal_lt = seriousness.criteria_met.iter().any(|c| {
                matches!(
                    c,
                    SeriousnessCriterion::Death | SeriousnessCriterion::LifeThreatening
                )
            });

            if is_fatal_lt {
                (
                    true,
                    ReportingCategory::SevenDay,
                    "Fatal/life-threatening SUSAR".to_string(),
                )
            } else {
                (
                    true,
                    ReportingCategory::FifteenDay,
                    "Serious unexpected ADR".to_string(),
                )
            }
        }

        // Serious but expected - different rules for trials vs marketed
        (true, Expectedness::Listed) => {
            if is_clinical_trial {
                // Clinical trials: still expedited if serious
                (
                    true,
                    ReportingCategory::FifteenDay,
                    "Serious ADR in clinical trial".to_string(),
                )
            } else {
                // Marketed: periodic reporting
                (
                    false,
                    ReportingCategory::Periodic,
                    "Serious expected - periodic report".to_string(),
                )
            }
        }

        // Serious but unknown expectedness - treat as unexpected (precautionary)
        (true, Expectedness::Unknown) => (
            true,
            ReportingCategory::FifteenDay,
            "Serious ADR - expectedness unknown, treat as unexpected".to_string(),
        ),

        // Non-serious
        (false, _) => (
            false,
            ReportingCategory::NonReportable,
            "Non-serious event".to_string(),
        ),
    };

    // Determine applicable authorities
    let authorities = if required {
        vec![
            RegulatoryAuthority::FdaCfr21,
            RegulatoryAuthority::EmaGvpIx,
            RegulatoryAuthority::IchE2b,
        ]
    } else {
        vec![]
    };

    ExpeditedReporting {
        required,
        deadline_days: category.days(),
        category,
        authorities,
        rationale,
    }
}

// =============================================================================
// Deadline Calculator
// =============================================================================

/// Calculated deadline with business day awareness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingDeadline {
    /// Awareness date (YYYYMMDD).
    pub awareness_date: String,
    /// Deadline date (YYYYMMDD).
    pub deadline_date: String,
    /// Calendar days until deadline.
    pub calendar_days: u32,
    /// Business days until deadline (approximate).
    pub business_days: u32,
    /// Whether deadline has passed.
    pub is_overdue: bool,
}

/// Calculate reporting deadline from awareness date.
///
/// # Arguments
///
/// * `awareness_date` - Date company became aware (YYYYMMDD).
/// * `deadline_days` - Number of calendar days allowed.
/// * `current_date` - Today's date (YYYYMMDD).
///
/// # Returns
///
/// `ReportingDeadline` with calculated dates and status.
#[must_use]
pub fn calculate_deadline(
    awareness_date: &str,
    deadline_days: u32,
    current_date: &str,
) -> Option<ReportingDeadline> {
    let awareness = parse_date(awareness_date)?;
    let current = parse_date(current_date)?;

    let deadline = awareness + deadline_days as i64;
    let days_remaining = deadline - current;

    // Approximate business days (5/7 of calendar days)
    let business_days = if days_remaining > 0 {
        ((days_remaining as f64) * 5.0 / 7.0).ceil() as u32
    } else {
        0
    };

    Some(ReportingDeadline {
        awareness_date: awareness_date.to_string(),
        deadline_date: format_date(deadline),
        calendar_days: days_remaining.max(0) as u32,
        business_days,
        is_overdue: days_remaining < 0,
    })
}

/// Check if a report is within compliance window.
#[must_use]
pub fn is_within_deadline(awareness_date: &str, submission_date: &str, deadline_days: u32) -> bool {
    let awareness = match parse_date(awareness_date) {
        Some(d) => d,
        None => return false,
    };
    let submission = match parse_date(submission_date) {
        Some(d) => d,
        None => return false,
    };

    (submission - awareness) <= deadline_days as i64
}

// =============================================================================
// Clock Start Determination
// =============================================================================

/// What starts the regulatory clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClockStartEvent {
    /// Date information received by company.
    DateReceived,
    /// Date company became aware of reportability.
    DateOfAwareness,
    /// Date of event occurrence.
    DateOfEvent,
}

/// Determine clock start date based on authority.
#[must_use]
pub fn clock_start_rule(authority: RegulatoryAuthority) -> ClockStartEvent {
    match authority {
        RegulatoryAuthority::FdaCfr21 => ClockStartEvent::DateReceived,
        RegulatoryAuthority::EmaGvpIx => ClockStartEvent::DateOfAwareness,
        RegulatoryAuthority::WhoUmc => ClockStartEvent::DateOfAwareness,
        RegulatoryAuthority::IchE2b => ClockStartEvent::DateOfAwareness,
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Parse YYYYMMDD date to days since epoch (simplified).
fn parse_date(date: &str) -> Option<i64> {
    if date.len() < 8 {
        return None;
    }

    let year: i64 = date[0..4].parse().ok()?;
    let month: i64 = date[4..6].parse().ok()?;
    let day: i64 = date[6..8].parse().ok()?;

    Some(year * 365 + month * 30 + day)
}

/// Format days since epoch back to YYYYMMDD (simplified).
fn format_date(days: i64) -> String {
    // Reverse of simplified parse (not calendar-accurate, but consistent)
    let year = days / 365;
    let remaining = days % 365;
    let month = remaining / 30;
    let day = remaining % 30;

    format!("{year:04}{month:02}{day:02}")
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seriousness_criteria_codes() {
        assert_eq!(SeriousnessCriterion::Death.code(), "1");
        assert_eq!(SeriousnessCriterion::MedicallyImportant.code(), "6");
    }

    #[test]
    fn test_seriousness_assessment() {
        let criteria = vec![
            SeriousnessCriterion::Death,
            SeriousnessCriterion::Hospitalization,
        ];
        let assessment = assess_seriousness(&criteria);

        assert!(assessment.is_serious);
        assert_eq!(assessment.criteria_met.len(), 2);
        assert_eq!(assessment.max_urgency, 10); // Death = 10
    }

    #[test]
    fn test_seriousness_non_serious() {
        let assessment = assess_seriousness(&[]);
        assert!(!assessment.is_serious);
        assert_eq!(assessment.max_urgency, 0);
    }

    #[test]
    fn test_expedited_susar() {
        let seriousness = assess_seriousness(&[SeriousnessCriterion::Death]);
        let expedited = determine_expedited(&seriousness, Expectedness::Unlisted, false);

        assert!(expedited.required);
        assert_eq!(expedited.category, ReportingCategory::SevenDay);
        assert_eq!(expedited.deadline_days, 7);
    }

    #[test]
    fn test_expedited_serious_unexpected() {
        let seriousness = assess_seriousness(&[SeriousnessCriterion::Hospitalization]);
        let expedited = determine_expedited(&seriousness, Expectedness::Unlisted, false);

        assert!(expedited.required);
        assert_eq!(expedited.category, ReportingCategory::FifteenDay);
        assert_eq!(expedited.deadline_days, 15);
    }

    #[test]
    fn test_expedited_serious_expected_marketed() {
        let seriousness = assess_seriousness(&[SeriousnessCriterion::Hospitalization]);
        let expedited = determine_expedited(&seriousness, Expectedness::Listed, false);

        assert!(!expedited.required);
        assert_eq!(expedited.category, ReportingCategory::Periodic);
    }

    #[test]
    fn test_expedited_serious_expected_clinical_trial() {
        let seriousness = assess_seriousness(&[SeriousnessCriterion::Hospitalization]);
        let expedited = determine_expedited(&seriousness, Expectedness::Listed, true);

        // Clinical trials still require expedited for serious events
        assert!(expedited.required);
        assert_eq!(expedited.category, ReportingCategory::FifteenDay);
    }

    #[test]
    fn test_expedited_non_serious() {
        let seriousness = assess_seriousness(&[]);
        let expedited = determine_expedited(&seriousness, Expectedness::Unlisted, false);

        assert!(!expedited.required);
        assert_eq!(expedited.category, ReportingCategory::NonReportable);
    }

    #[test]
    fn test_deadline_calculation() {
        let deadline = calculate_deadline("20240101", 15, "20240110");

        assert!(deadline.is_some());
        let deadline = deadline.unwrap();
        assert!(!deadline.is_overdue);
        assert!(deadline.calendar_days > 0);
    }

    #[test]
    fn test_deadline_overdue() {
        let deadline = calculate_deadline("20240101", 7, "20240115");

        assert!(deadline.is_some());
        let deadline = deadline.unwrap();
        assert!(deadline.is_overdue);
    }

    #[test]
    fn test_within_deadline() {
        assert!(is_within_deadline("20240101", "20240110", 15));
        assert!(!is_within_deadline("20240101", "20240120", 15));
    }

    #[test]
    fn test_clock_start_rules() {
        assert_eq!(
            clock_start_rule(RegulatoryAuthority::FdaCfr21),
            ClockStartEvent::DateReceived
        );
        assert_eq!(
            clock_start_rule(RegulatoryAuthority::EmaGvpIx),
            ClockStartEvent::DateOfAwareness
        );
    }

    #[test]
    fn test_reporting_category_days() {
        assert_eq!(ReportingCategory::SevenDay.days(), 7);
        assert_eq!(ReportingCategory::FifteenDay.days(), 15);
        assert_eq!(ReportingCategory::Periodic.days(), 90);
    }
}
