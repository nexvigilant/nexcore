//! FAERS type definitions.
//!
//! Core enums used for parsing FAERS ASCII files.

use serde::{Deserialize, Serialize};

/// Drug role in adverse event report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrugRole {
    /// Primary suspect drug
    #[serde(rename = "PS")]
    PrimarySuspect,
    /// Secondary suspect drug
    #[serde(rename = "SS")]
    SecondarySuspect,
    /// Concomitant medication
    #[serde(rename = "C")]
    Concomitant,
    /// Interacting drug
    #[serde(rename = "I")]
    Interacting,
}

impl DrugRole {
    /// Parse from FAERS role code string.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_uppercase().as_str() {
            "PS" => Some(Self::PrimarySuspect),
            "SS" => Some(Self::SecondarySuspect),
            "C" => Some(Self::Concomitant),
            "I" => Some(Self::Interacting),
            _ => None,
        }
    }

    /// Check if this role indicates a suspect drug.
    #[must_use]
    pub const fn is_suspect(&self) -> bool {
        matches!(self, Self::PrimarySuspect | Self::SecondarySuspect)
    }
}

/// Outcome severity code from FAERS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutcomeCode {
    /// Death
    #[serde(rename = "DE")]
    Death,
    /// Life-threatening
    #[serde(rename = "LT")]
    LifeThreatening,
    /// Initial or prolonged hospitalization
    #[serde(rename = "HO")]
    Hospitalization,
    /// Disability or permanent damage
    #[serde(rename = "DS")]
    Disability,
    /// Congenital anomaly/birth defect
    #[serde(rename = "CA")]
    CongenitalAnomaly,
    /// Required intervention to prevent impairment
    #[serde(rename = "RI")]
    RequiredIntervention,
    /// Other serious (important medical event)
    #[serde(rename = "OT")]
    OtherSerious,
}

impl OutcomeCode {
    /// Parse from FAERS outcome code string.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_uppercase().as_str() {
            "DE" => Some(Self::Death),
            "LT" => Some(Self::LifeThreatening),
            "HO" => Some(Self::Hospitalization),
            "DS" => Some(Self::Disability),
            "CA" => Some(Self::CongenitalAnomaly),
            "RI" => Some(Self::RequiredIntervention),
            "OT" => Some(Self::OtherSerious),
            _ => None,
        }
    }

    /// Check if this outcome is considered "serious" per FDA criteria.
    #[must_use]
    pub const fn is_serious(&self) -> bool {
        // All FAERS outcome codes indicate serious outcomes
        true
    }

    /// Get severity rank (1 = most severe).
    #[must_use]
    pub const fn severity_rank(&self) -> u8 {
        match self {
            Self::Death => 1,
            Self::LifeThreatening => 2,
            Self::Hospitalization => 3,
            Self::Disability => 4,
            Self::CongenitalAnomaly => 5,
            Self::RequiredIntervention => 6,
            Self::OtherSerious => 7,
        }
    }
}

/// Age unit code from FAERS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgeUnit {
    /// Years
    #[serde(rename = "YR")]
    Year,
    /// Months
    #[serde(rename = "MON")]
    Month,
    /// Weeks
    #[serde(rename = "WK")]
    Week,
    /// Days
    #[serde(rename = "DY")]
    Day,
    /// Hours
    #[serde(rename = "HR")]
    Hour,
    /// Decades
    #[serde(rename = "DEC")]
    Decade,
}

impl AgeUnit {
    /// Parse from FAERS age code string.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_uppercase().as_str() {
            "YR" | "YEAR" | "YEARS" => Some(Self::Year),
            "MON" | "MONTH" | "MONTHS" => Some(Self::Month),
            "WK" | "WEEK" | "WEEKS" => Some(Self::Week),
            "DY" | "DAY" | "DAYS" => Some(Self::Day),
            "HR" | "HOUR" | "HOURS" => Some(Self::Hour),
            "DEC" | "DECADE" | "DECADES" => Some(Self::Decade),
            _ => None,
        }
    }

    /// Get conversion factor to years.
    #[must_use]
    pub const fn to_years_factor(&self) -> f64 {
        match self {
            Self::Year => 1.0,
            Self::Month => 1.0 / 12.0,
            Self::Week => 1.0 / 52.0,
            Self::Day => 1.0 / 365.0,
            Self::Hour => 1.0 / 8760.0,
            Self::Decade => 10.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drug_role_from_code() {
        assert_eq!(DrugRole::from_code("PS"), Some(DrugRole::PrimarySuspect));
        assert_eq!(DrugRole::from_code("ss"), Some(DrugRole::SecondarySuspect));
        assert_eq!(DrugRole::from_code("C"), Some(DrugRole::Concomitant));
        assert_eq!(DrugRole::from_code("I"), Some(DrugRole::Interacting));
        assert_eq!(DrugRole::from_code("XX"), None);
    }

    #[test]
    fn test_drug_role_is_suspect() {
        assert!(DrugRole::PrimarySuspect.is_suspect());
        assert!(DrugRole::SecondarySuspect.is_suspect());
        assert!(!DrugRole::Concomitant.is_suspect());
        assert!(!DrugRole::Interacting.is_suspect());
    }

    #[test]
    fn test_outcome_code_from_code() {
        assert_eq!(OutcomeCode::from_code("DE"), Some(OutcomeCode::Death));
        assert_eq!(
            OutcomeCode::from_code("lt"),
            Some(OutcomeCode::LifeThreatening)
        );
        assert_eq!(
            OutcomeCode::from_code("HO"),
            Some(OutcomeCode::Hospitalization)
        );
        assert_eq!(OutcomeCode::from_code("XX"), None);
    }

    #[test]
    fn test_outcome_severity_rank() {
        assert_eq!(OutcomeCode::Death.severity_rank(), 1);
        assert_eq!(OutcomeCode::OtherSerious.severity_rank(), 7);
        assert!(OutcomeCode::Death.severity_rank() < OutcomeCode::Hospitalization.severity_rank());
    }

    #[test]
    fn test_age_unit_from_code() {
        assert_eq!(AgeUnit::from_code("YR"), Some(AgeUnit::Year));
        assert_eq!(AgeUnit::from_code("mon"), Some(AgeUnit::Month));
        assert_eq!(AgeUnit::from_code("DECADE"), Some(AgeUnit::Decade));
        assert_eq!(AgeUnit::from_code("XX"), None);
    }

    #[test]
    fn test_age_unit_to_years_factor() {
        assert!((AgeUnit::Year.to_years_factor() - 1.0).abs() < f64::EPSILON);
        assert!((AgeUnit::Month.to_years_factor() * 12.0 - 1.0).abs() < 0.01);
        assert!((AgeUnit::Decade.to_years_factor() - 10.0).abs() < f64::EPSILON);
    }
}
