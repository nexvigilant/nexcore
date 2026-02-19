//! Cross-domain transfer confidence from pharmacovigilance.
//!
//! Formula: `TC = structural × 0.4 + functional × 0.4 + contextual × 0.2`
//! Four target domains: Clinical Trials, Regulatory Affairs, Epidemiology, Health Economics.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Target domains for PV knowledge transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(clippy::exhaustive_enums)]
pub enum TransferDomain {
    /// Phase I-III human clinical trials.
    ClinicalTrials,
    /// Regulatory submission, approval, post-market compliance.
    RegulatoryAffairs,
    /// Population-level disease surveillance and causal inference.
    Epidemiology,
    /// Cost-effectiveness, QALY, health technology assessment.
    HealthEconomics,
}

/// Three-dimensional transfer confidence score.
#[derive(Debug, Clone)]
pub struct TransferConfidence {
    /// Target domain.
    pub domain: TransferDomain,
    /// Structural similarity (type system, data model overlap).
    pub structural: f64,
    /// Functional similarity (same operations, same algorithms).
    pub functional: f64,
    /// Contextual similarity (same use cases, same stakeholders).
    pub contextual: f64,
    /// Limiting factor description.
    pub limiting_factor: &'static str,
    /// Key concept mappings.
    pub mappings: &'static [(&'static str, &'static str)],
    /// Important caveats about transfer.
    pub caveat: &'static str,
}

impl TransferConfidence {
    /// Weighted composite score: 0.4·structural + 0.4·functional + 0.2·contextual.
    #[must_use]
    pub fn score(&self) -> f64 {
        self.structural * 0.4 + self.functional * 0.4 + self.contextual * 0.2
    }

    /// Human-readable label for the score.
    #[must_use]
    pub fn label(&self) -> &'static str {
        let s = self.score();
        if s >= 0.85 {
            "Very High"
        } else if s >= 0.70 {
            "High"
        } else if s >= 0.55 {
            "Moderate"
        } else if s >= 0.40 {
            "Low"
        } else {
            "Very Low"
        }
    }
}

impl fmt::Display for TransferConfidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PV → {:?}: {:.2} ({}) [S={:.2} F={:.2} C={:.2}] limited by: {}",
            self.domain,
            self.score(),
            self.label(),
            self.structural,
            self.functional,
            self.contextual,
            self.limiting_factor
        )
    }
}

/// Get transfer confidence for a specific domain.
#[must_use]
pub fn lookup_transfer(domain: TransferDomain) -> TransferConfidence {
    match domain {
        TransferDomain::ClinicalTrials => TransferConfidence {
            domain,
            structural: 0.92,
            functional: 0.88,
            contextual: 0.68,
            limiting_factor: "contextual: controlled vs uncontrolled exposure",
            mappings: &[
                ("AE", "AE (identical, ICH E2A)"),
                ("SAE", "SAE (identical)"),
                ("SUSAR", "SUSAR (defined FOR clinical trials)"),
                ("Signal Detection", "DSMB alert trigger"),
                ("ICSR", "CRF safety section"),
                ("Causality Assessment", "Investigator assessment"),
                ("PSUR", "DSUR (direct analog)"),
                ("RMP", "Safety Monitoring Plan"),
            ],
            caveat: "Clinical trials have controlled exposure (randomization, blinding) that fundamentally changes causal inference. PV operates in uncontrolled real-world settings where confounding is pervasive.",
        },
        TransferDomain::RegulatoryAffairs => TransferConfidence {
            domain,
            structural: 0.95,
            functional: 0.92,
            contextual: 0.78,
            limiting_factor: "contextual: regulatory scope broader than safety",
            mappings: &[
                ("ICSR", "Regulatory submission unit"),
                ("PSUR/PBRER", "Periodic regulatory report"),
                ("Expedited Report", "Regulatory alert submission"),
                ("RMP", "Regulatory risk mitigation commitment"),
                ("Label Update", "Regulatory variation"),
                ("Signal", "Regulatory concern trigger"),
                ("Benefit-Risk", "Regulatory decision basis"),
            ],
            caveat: "Regulatory affairs encompasses CMC, clinical efficacy, pricing — broader scope. PV safety primitives are a strict subset of regulatory vocabulary.",
        },
        TransferDomain::Epidemiology => TransferConfidence {
            domain,
            structural: 0.88,
            functional: 0.85,
            contextual: 0.62,
            limiting_factor: "contextual: population-first vs case-first",
            mappings: &[
                ("Signal Detection", "Outbreak detection"),
                ("PRR/ROR", "Risk Ratio / Odds Ratio (identical math)"),
                ("Contingency Table", "2x2 table (identical)"),
                ("Incidence Rate", "Incidence rate (identical)"),
                (
                    "Bradford Hill",
                    "Bradford Hill (ORIGINATED in epidemiology)",
                ),
                ("Spontaneous Reporting", "Passive surveillance"),
                ("Active Surveillance", "Active surveillance (identical)"),
                ("Causality Assessment", "Causal inference"),
            ],
            caveat: "Epidemiology handles population-level inference without individual case assessment. PV inverts focus: individual cases first, populations second. Mathematical primitives transfer perfectly; case-first workflow does not.",
        },
        TransferDomain::HealthEconomics => TransferConfidence {
            domain,
            structural: 0.65,
            functional: 0.68,
            contextual: 0.38,
            limiting_factor: "contextual: safety priority vs cost optimization",
            mappings: &[
                ("Benefit-Risk", "Cost-effectiveness ratio"),
                ("NNH", "NNT (Number Needed to Treat)"),
                ("Incidence Rate", "Event rate in Markov model"),
                ("Risk", "QALY decrement"),
                ("SAE", "Hospitalization cost driver"),
                ("Signal", "Adverse budget impact trigger"),
                ("PAF", "Population burden attributable to drug"),
            ],
            caveat: "Health economics optimizes resource allocation; PV optimizes patient safety. PV has lexicographic priority (P0 safety overrides P5 cost); HE trades them. This philosophical gap limits transfer despite structural overlap.",
        },
    }
}

/// All 4 transfer confidences.
#[must_use]
pub fn transfer_matrix() -> Vec<TransferConfidence> {
    vec![
        lookup_transfer(TransferDomain::ClinicalTrials),
        lookup_transfer(TransferDomain::RegulatoryAffairs),
        lookup_transfer(TransferDomain::Epidemiology),
        lookup_transfer(TransferDomain::HealthEconomics),
    ]
}

/// Strongest transfer corridor.
#[must_use]
pub fn strongest_transfer() -> TransferConfidence {
    lookup_transfer(TransferDomain::RegulatoryAffairs)
}

/// Weakest transfer corridor.
#[must_use]
pub fn weakest_transfer() -> TransferConfidence {
    lookup_transfer(TransferDomain::HealthEconomics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formula_correct() {
        let tc = lookup_transfer(TransferDomain::ClinicalTrials);
        let expected = 0.92 * 0.4 + 0.88 * 0.4 + 0.68 * 0.2;
        assert!((tc.score() - expected).abs() < 1e-10);
    }

    #[test]
    fn regulatory_is_strongest() {
        let strongest = strongest_transfer();
        assert_eq!(strongest.domain, TransferDomain::RegulatoryAffairs);
    }

    #[test]
    fn health_economics_is_weakest() {
        let weakest = weakest_transfer();
        assert_eq!(weakest.domain, TransferDomain::HealthEconomics);
    }

    #[test]
    fn matrix_has_4_entries() {
        assert_eq!(transfer_matrix().len(), 4);
    }

    #[test]
    fn all_scores_between_0_and_1() {
        for tc in transfer_matrix() {
            let s = tc.score();
            assert!(
                s >= 0.0 && s <= 1.0,
                "{:?} score out of range: {}",
                tc.domain,
                s
            );
        }
    }

    #[test]
    fn label_ranges() {
        let reg = lookup_transfer(TransferDomain::RegulatoryAffairs);
        assert_eq!(reg.label(), "Very High");

        let he = lookup_transfer(TransferDomain::HealthEconomics);
        assert_eq!(he.label(), "Moderate");
    }

    #[test]
    fn all_have_mappings() {
        for tc in transfer_matrix() {
            assert!(
                !tc.mappings.is_empty(),
                "{:?} has no concept mappings",
                tc.domain
            );
        }
    }

    #[test]
    fn all_have_caveats() {
        for tc in transfer_matrix() {
            assert!(
                !tc.caveat.is_empty(),
                "{:?} has no transfer caveat",
                tc.domain
            );
        }
    }

    #[test]
    fn display_includes_all_dimensions() {
        let s = format!("{}", lookup_transfer(TransferDomain::Epidemiology));
        assert!(s.contains("S="));
        assert!(s.contains("F="));
        assert!(s.contains("C="));
    }
}
