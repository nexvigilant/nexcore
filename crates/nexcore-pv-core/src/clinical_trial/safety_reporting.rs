//! FDA Safety Reporting (December 2025 Final Guidance)
//!
//! Based on:
//! - "Sponsor Responsibilities: Safety Reporting Requirements" (Dec 2025)
//! - "Investigator Responsibilities: Safety Reporting" (Dec 2025)
//!
//! Key requirement: Investigators must report SAEs "immediately" (≤1 calendar day)

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Serious Adverse Event (SAE) criteria per 21 CFR 312.32
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeriousnessCriterion {
    Death,
    LifeThreatening,
    Hospitalization,
    Disability,
    CongenitalAnomaly,
    RequiresIntervention,
    MedicallyImportant,
}

impl SeriousnessCriterion {
    #[must_use]
    pub const fn priority(&self) -> u8 {
        match self {
            Self::Death => 1,
            Self::LifeThreatening => 2,
            Self::CongenitalAnomaly => 3,
            Self::Hospitalization => 4,
            Self::Disability => 5,
            Self::RequiresIntervention => 6,
            Self::MedicallyImportant => 7,
        }
    }

    #[must_use]
    pub const fn is_fatal_or_life_threatening(&self) -> bool {
        matches!(self, Self::Death | Self::LifeThreatening)
    }
}

impl fmt::Display for SeriousnessCriterion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Death => "Death",
            Self::LifeThreatening => "Life-threatening",
            Self::Hospitalization => "Hospitalization",
            Self::Disability => "Disability/Incapacity",
            Self::CongenitalAnomaly => "Congenital anomaly",
            Self::RequiresIntervention => "Requires intervention",
            Self::MedicallyImportant => "Medically important",
        };
        write!(f, "{s}")
    }
}

/// Causality/relatedness assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Relatedness {
    Definite,
    Probable,
    Possible,
    Unlikely,
    Unrelated,
    NotAssessable,
}

impl Relatedness {
    #[must_use]
    pub const fn is_related(&self) -> bool {
        matches!(self, Self::Definite | Self::Probable | Self::Possible)
    }
}

/// Event outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventOutcome {
    Recovered,
    RecoveredWithSequelae,
    Recovering,
    NotRecovered,
    Fatal,
    Unknown,
}

/// Serious Adverse Event (T3 domain type)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriousAdverseEvent {
    pub id: String,
    pub event_term: String,
    pub seriousness_criteria: Vec<SeriousnessCriterion>,
    pub onset_date: DateTime<Utc>,
    pub awareness_date: DateTime<Utc>,
    pub expected: bool,
    pub relatedness: Relatedness,
    pub outcome: EventOutcome,
    pub subject_id: String,
}

impl SeriousAdverseEvent {
    #[must_use]
    pub fn is_susar(&self) -> bool {
        !self.expected && self.relatedness.is_related()
    }

    #[must_use]
    pub fn has_fatal_or_life_threatening(&self) -> bool {
        self.seriousness_criteria
            .iter()
            .any(SeriousnessCriterion::is_fatal_or_life_threatening)
    }

    #[must_use]
    pub fn requires_expedited_reporting(&self) -> bool {
        self.is_susar() || self.has_fatal_or_life_threatening()
    }

    #[must_use]
    pub fn highest_priority_criterion(&self) -> Option<&SeriousnessCriterion> {
        self.seriousness_criteria
            .iter()
            .min_by_key(|c| c.priority())
    }
}

/// Safety reporting timeline (FDA Dec 2025)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyReportingTimeline {
    pub event: SeriousAdverseEvent,
    pub awareness_time: DateTime<Utc>,
    pub reported_to_sponsor: Option<DateTime<Utc>>,
    pub reported_to_fda: Option<DateTime<Utc>>,
}

impl SafetyReportingTimeline {
    #[must_use]
    pub fn new(event: SeriousAdverseEvent) -> Self {
        let awareness_time = event.awareness_date;
        Self {
            event,
            awareness_time,
            reported_to_sponsor: None,
            reported_to_fda: None,
        }
    }

    #[must_use]
    pub fn investigator_deadline(&self) -> DateTime<Utc> {
        self.awareness_time + Duration::days(1)
    }

    #[must_use]
    pub fn sponsor_expedited_deadline(&self) -> DateTime<Utc> {
        self.awareness_time + Duration::days(7)
    }

    #[must_use]
    pub fn sponsor_standard_deadline(&self) -> DateTime<Utc> {
        self.awareness_time + Duration::days(15)
    }

    #[must_use]
    pub fn applicable_fda_deadline(&self) -> DateTime<Utc> {
        if self.event.has_fatal_or_life_threatening() {
            self.sponsor_expedited_deadline()
        } else {
            self.sponsor_standard_deadline()
        }
    }

    #[must_use]
    pub fn investigator_timely(&self) -> Option<bool> {
        self.reported_to_sponsor
            .map(|t| t <= self.investigator_deadline())
    }

    #[must_use]
    pub fn sponsor_timely(&self) -> Option<bool> {
        self.reported_to_fda
            .map(|t| t <= self.applicable_fda_deadline())
    }

    #[must_use]
    pub fn compliance_status(&self) -> ComplianceStatus {
        match (
            self.reported_to_sponsor.is_some(),
            self.reported_to_fda.is_some(),
        ) {
            (false, _) => ComplianceStatus::PendingInvestigator,
            (true, false) => ComplianceStatus::PendingSponsor,
            (true, true) => self.evaluate_timeliness(),
        }
    }

    fn evaluate_timeliness(&self) -> ComplianceStatus {
        let inv_ok = self.investigator_timely().unwrap_or(false);
        let sponsor_ok = self.sponsor_timely().unwrap_or(false);
        if inv_ok && sponsor_ok {
            ComplianceStatus::Compliant
        } else {
            ComplianceStatus::Late
        }
    }
}

/// Compliance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    PendingInvestigator,
    PendingSponsor,
    Compliant,
    Late,
}

/// Aggregate safety report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyReport {
    pub report_type: SafetyReportType,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub subjects_exposed: usize,
    pub total_saes: usize,
    pub susars: usize,
    pub fatal_saes: usize,
    pub drug_related_saes: usize,
}

impl SafetyReport {
    #[must_use]
    pub fn sae_rate(&self) -> f64 {
        rate_per_hundred(self.total_saes, self.subjects_exposed)
    }

    #[must_use]
    pub fn susar_rate(&self) -> f64 {
        rate_per_hundred(self.susars, self.subjects_exposed)
    }
}

fn rate_per_hundred(events: usize, population: usize) -> f64 {
    if population == 0 {
        0.0
    } else {
        (events as f64 / population as f64) * 100.0
    }
}

/// Safety report types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyReportType {
    IndSafetyReport,
    IndAnnualReport,
    Dsur,
    Pader,
    Psur,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_susar_detection() {
        let sae = SeriousAdverseEvent {
            id: "SAE-001".into(),
            event_term: "Hepatotoxicity".into(),
            seriousness_criteria: vec![SeriousnessCriterion::Hospitalization],
            onset_date: Utc::now(),
            awareness_date: Utc::now(),
            expected: false,
            relatedness: Relatedness::Probable,
            outcome: EventOutcome::Recovering,
            subject_id: "SUBJ-001".into(),
        };
        assert!(sae.is_susar());
        assert!(sae.requires_expedited_reporting());
    }

    #[test]
    fn test_reporting_timeline() {
        let sae = SeriousAdverseEvent {
            id: "SAE-002".into(),
            event_term: "Death".into(),
            seriousness_criteria: vec![SeriousnessCriterion::Death],
            onset_date: Utc::now(),
            awareness_date: Utc::now(),
            expected: false,
            relatedness: Relatedness::Possible,
            outcome: EventOutcome::Fatal,
            subject_id: "SUBJ-002".into(),
        };
        let timeline = SafetyReportingTimeline::new(sae);
        let deadline = timeline.applicable_fda_deadline();
        assert!(deadline <= timeline.awareness_time + Duration::days(7));
    }
}
