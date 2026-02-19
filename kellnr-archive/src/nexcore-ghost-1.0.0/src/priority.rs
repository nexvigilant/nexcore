//! # Data Privacy Priority
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | κ Comparison | Priority ordering within Guardian hierarchy |
//!
//! ## Tier: T2-P (κ Comparison)
//!
//! Privacy sits at P2b in the Guardian priority hierarchy:
//! P0 Patient Safety > P1 Signal Integrity > P2 Regulatory > P2b Privacy > P3 Data Quality

use serde::{Deserialize, Serialize};
use std::fmt;

/// Data privacy priority level within the Guardian hierarchy.
///
/// P2b: yields to P0 (patient safety), P1 (signal integrity),
/// and P2 (regulatory compliance). Outranks P3 (data quality),
/// P4 (operational efficiency), and P5 (cost optimization).
///
/// Rationale: ICH E2B may require identifiable data for regulatory
/// submissions. Patient safety always trumps privacy. But privacy
/// outranks convenience and cost.
///
/// ## Tier: T2-P (κ Comparison)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DataPrivacyPriority {
    /// P0: Patient safety always wins.
    PatientSafety = 0,
    /// P1: Signal integrity preserved.
    SignalIntegrity = 1,
    /// P2: Regulatory compliance (ICH E2B submissions).
    RegulatoryCompliance = 2,
    /// P2b: Data privacy enforcement.
    DataPrivacy = 3,
    /// P3: Data quality.
    DataQuality = 4,
    /// P4: Operational efficiency.
    OperationalEfficiency = 5,
    /// P5: Cost optimization.
    CostOptimization = 6,
}

impl DataPrivacyPriority {
    /// Whether this priority outranks another (lower ordinal = higher priority).
    #[must_use]
    pub const fn outranks(&self, other: &Self) -> bool {
        (*self as u8) < (*other as u8)
    }

    /// Whether data privacy outranks this level.
    #[must_use]
    pub const fn privacy_outranks(&self) -> bool {
        Self::DataPrivacy.outranks(self)
    }

    /// Human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::PatientSafety => "P0: Patient Safety",
            Self::SignalIntegrity => "P1: Signal Integrity",
            Self::RegulatoryCompliance => "P2: Regulatory Compliance",
            Self::DataPrivacy => "P2b: Data Privacy",
            Self::DataQuality => "P3: Data Quality",
            Self::OperationalEfficiency => "P4: Operational Efficiency",
            Self::CostOptimization => "P5: Cost Optimization",
        }
    }
}

impl fmt::Display for DataPrivacyPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patient_safety_outranks_privacy() {
        assert!(DataPrivacyPriority::PatientSafety.outranks(&DataPrivacyPriority::DataPrivacy));
    }

    #[test]
    fn privacy_outranks_data_quality() {
        assert!(DataPrivacyPriority::DataPrivacy.outranks(&DataPrivacyPriority::DataQuality));
    }

    #[test]
    fn privacy_outranks_cost() {
        assert!(DataPrivacyPriority::DataPrivacy.outranks(&DataPrivacyPriority::CostOptimization));
    }

    #[test]
    fn regulatory_outranks_privacy() {
        assert!(
            DataPrivacyPriority::RegulatoryCompliance.outranks(&DataPrivacyPriority::DataPrivacy)
        );
    }

    #[test]
    fn privacy_does_not_outrank_patient_safety() {
        assert!(!DataPrivacyPriority::DataPrivacy.outranks(&DataPrivacyPriority::PatientSafety));
    }
}
