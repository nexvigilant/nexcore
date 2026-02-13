//! T2-P cross-domain primitives for pharmacovigilance.
//!
//! 22 atomic concepts that appear across multiple domains (epidemiology,
//! clinical trials, health economics, quality control). Each grounds to
//! 1-3 unique Lex Primitiva symbols.

use crate::lex::{LexSymbol, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 22 cross-domain primitives active in pharmacovigilance.
///
/// Tier: T2-P | Each concept grounds to 1-3 unique symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PvPrimitive {
    /// Boundary value separating decision regions.
    Threshold,
    /// Quotient comparing two commensurable quantities.
    Ratio,
    /// Rate of occurrence per unit observation.
    ReportingRate,
    /// Point in ordered time.
    Timestamp,
    /// Elapsed interval between two time points.
    Duration,
    /// Directed pairing of exposure to outcome.
    Association,
    /// Identified origin of data.
    Source,
    /// Negative change to a valued state.
    Harm,
    /// Magnitude of deviation along a harm axis.
    Severity,
    /// Membership of observation in known baseline set.
    Expectedness,
    /// Confirmation that data satisfies acceptance criteria.
    Validation,
    /// Binding requirement to perform a specified action.
    Obligation,
    /// Assignment to mutually exclusive categories.
    Classification,
    /// Proportion of true positives correctly identified.
    Sensitivity,
    /// Proportion of true negatives correctly identified.
    Specificity,
    /// New cases per unit person-time at risk.
    IncidenceRate,
    /// Proportion with condition at a point in time.
    Prevalence,
    /// Absolute risk difference between exposed and unexposed.
    AttributableRisk,
    /// Ratio of signal power to noise power.
    SignalToNoise,
    /// Proportion of positive results that are true positives.
    PositivePredictiveValue,
    /// Proportion of negative results that are true negatives.
    NegativePredictiveValue,
    /// Range within which true parameter falls with specified probability.
    ConfidenceInterval,
}

impl PvPrimitive {
    /// All 22 primitives.
    pub const ALL: &'static [Self] = &[
        Self::Threshold,
        Self::Ratio,
        Self::ReportingRate,
        Self::Timestamp,
        Self::Duration,
        Self::Association,
        Self::Source,
        Self::Harm,
        Self::Severity,
        Self::Expectedness,
        Self::Validation,
        Self::Obligation,
        Self::Classification,
        Self::Sensitivity,
        Self::Specificity,
        Self::IncidenceRate,
        Self::Prevalence,
        Self::AttributableRisk,
        Self::SignalToNoise,
        Self::PositivePredictiveValue,
        Self::NegativePredictiveValue,
        Self::ConfidenceInterval,
    ];

    /// Lex Primitiva grounding for this primitive.
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        use LexSymbol::*;
        PrimitiveComposition::new(match self {
            Self::Threshold => &[Boundary],
            Self::Ratio => &[Comparison],
            Self::ReportingRate => &[Frequency],
            Self::Timestamp => &[Sequence],
            Self::Duration => &[Sequence, Quantity],
            Self::Association => &[Causality],
            Self::Source => &[Location],
            Self::Harm => &[Irreversibility, State],
            Self::Severity => &[Quantity, Comparison],
            Self::Expectedness => &[Comparison, Existence],
            Self::Validation => &[Comparison, Boundary],
            Self::Obligation => &[Boundary, Causality],
            Self::Classification => &[Sum, Mapping],
            Self::Sensitivity => &[Comparison],
            Self::Specificity => &[Comparison],
            Self::IncidenceRate => &[Frequency],
            Self::Prevalence => &[Frequency],
            Self::AttributableRisk => &[Comparison],
            Self::SignalToNoise => &[Comparison],
            Self::PositivePredictiveValue => &[Comparison],
            Self::NegativePredictiveValue => &[Comparison],
            Self::ConfidenceInterval => &[Boundary, Quantity],
        })
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Threshold => "Boundary value that separates one decision region from another",
            Self::Ratio => "Quotient comparing two commensurable quantities",
            Self::ReportingRate => "Rate of AE occurrence per unit observation time",
            Self::Timestamp => "Point in ordered time (onset, report date, receipt date)",
            Self::Duration => "Elapsed interval (time-to-onset, exposure duration, latency)",
            Self::Association => "Directed pairing of drug exposure to clinical outcome",
            Self::Source => {
                "Identified origin of safety data (spontaneous, clinical trial, literature)"
            }
            Self::Harm => "Negative change to a valued state of a patient",
            Self::Severity => {
                "Magnitude of deviation from normal along a harm axis (mild/moderate/severe)"
            }
            Self::Expectedness => "Whether an AE is listed in the reference safety information",
            Self::Validation => "Confirmation that case data satisfies minimum acceptance criteria",
            Self::Obligation => "Binding regulatory requirement to report or act within a deadline",
            Self::Classification => {
                "Assignment of entity to mutually exclusive category (MedDRA, WHO-ART)"
            }
            Self::Sensitivity => "Proportion of true safety signals correctly detected",
            Self::Specificity => "Proportion of non-signals correctly excluded",
            Self::IncidenceRate => "New AE cases per unit person-time at risk",
            Self::Prevalence => "Proportion of population affected at a point in time",
            Self::AttributableRisk => "Absolute risk difference: exposed minus unexposed rate",
            Self::SignalToNoise => "Ratio of true signal power to background noise in AE data",
            Self::PositivePredictiveValue => "Proportion of detected signals that are true signals",
            Self::NegativePredictiveValue => {
                "Proportion of non-detections that are true non-signals"
            }
            Self::ConfidenceInterval => {
                "Range within which true parameter falls with specified probability"
            }
        }
    }

    /// Domains where this primitive appears.
    #[must_use]
    pub fn domains(&self) -> &'static [&'static str] {
        match self {
            Self::Threshold => &["PV", "statistics", "control theory", "decision theory"],
            Self::Ratio => &["PV", "mathematics", "epidemiology", "finance"],
            Self::ReportingRate => &["PV", "epidemiology", "reliability engineering"],
            Self::Timestamp => &["PV", "physics", "databases", "logging"],
            Self::Duration => &["PV", "physics", "project management"],
            Self::Association => &["PV", "epidemiology", "cybersecurity", "finance"],
            Self::Source => &["PV", "journalism", "library science", "provenance"],
            Self::Harm => &["PV", "ethics", "economics", "insurance"],
            Self::Severity => &["PV", "oncology (CTCAE)", "seismology", "weather"],
            Self::Expectedness => &["PV", "statistics", "control theory"],
            Self::Validation => &["PV", "QA", "manufacturing", "logic"],
            Self::Obligation => &["PV", "law", "deontic logic", "contract theory"],
            Self::Classification => &["PV", "taxonomy", "ML", "set theory"],
            Self::Sensitivity => &["PV", "diagnostics", "information retrieval", "electronics"],
            Self::Specificity => &["PV", "diagnostics", "information retrieval"],
            Self::IncidenceRate => &["PV", "epidemiology", "actuarial science", "reliability"],
            Self::Prevalence => &["PV", "epidemiology", "market research", "ecology"],
            Self::AttributableRisk => &["PV", "epidemiology", "insurance", "public health"],
            Self::SignalToNoise => &["PV", "electronics", "finance", "communications"],
            Self::PositivePredictiveValue => &["PV", "diagnostics", "ML", "screening"],
            Self::NegativePredictiveValue => &["PV", "diagnostics", "ML", "screening"],
            Self::ConfidenceInterval => &["PV", "statistics", "engineering", "quality control"],
        }
    }

    /// Tier for this primitive.
    #[must_use]
    pub fn tier(&self) -> crate::lex::Tier {
        self.grounding().tier()
    }
}

impl fmt::Display for PvPrimitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} [{}]", self, self.grounding().tier())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_is_22() {
        assert_eq!(PvPrimitive::ALL.len(), 22);
    }

    #[test]
    fn all_are_t1_or_t2p() {
        for p in PvPrimitive::ALL {
            let tier = p.tier();
            assert!(
                tier == crate::lex::Tier::T1 || tier == crate::lex::Tier::T2P,
                "{:?} is {:?}, expected T1 or T2-P",
                p,
                tier
            );
        }
    }

    #[test]
    fn all_have_descriptions() {
        for p in PvPrimitive::ALL {
            assert!(!p.description().is_empty(), "{:?} has empty description", p);
        }
    }

    #[test]
    fn all_have_domains() {
        for p in PvPrimitive::ALL {
            assert!(!p.domains().is_empty(), "{:?} has no domains", p);
            assert!(p.domains().contains(&"PV"), "{:?} missing PV domain", p);
        }
    }

    #[test]
    fn harm_is_t2p() {
        assert_eq!(PvPrimitive::Harm.tier(), crate::lex::Tier::T2P);
    }

    #[test]
    fn threshold_is_t1() {
        assert_eq!(PvPrimitive::Threshold.tier(), crate::lex::Tier::T1);
    }
}
