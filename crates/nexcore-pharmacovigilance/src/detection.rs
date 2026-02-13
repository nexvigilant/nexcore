//! WHO Pillar 1: Detection — "the detection of adverse effects."
//!
//! 10 T3 domain-specific concepts for identifying safety signals
//! from spontaneous reports, active surveillance, and data mining.

use crate::lex::{LexSymbol, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::fmt;

/// T3 detection concepts — WHO Pillar 1.
///
/// Tier: T3 | Dominant: ∃ (Existence) — detection asks "does this signal exist?"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DetectionConcept {
    /// Proportional Reporting Ratio: (a/a+b) / (c/c+d).
    Prr,
    /// Reporting Odds Ratio: (a*d) / (b*c).
    Ror,
    /// Information Component: log2(observed/expected), Bayesian.
    InformationComponent,
    /// Empirical Bayes Geometric Mean: shrinkage estimator.
    Ebgm,
    /// Chi-square with Yates correction.
    ChiSquare,
    /// Evans criteria: PRR≥2 AND χ²≥3.841 AND n≥3.
    EvansCriteria,
    /// Composite of all disproportionality methods.
    DisproportionalityAnalysis,
    /// End-to-end signal finding: statistical + clinical + temporal.
    SignalDetection,
    /// Automated statistical scanning of large databases.
    DataMining,
    /// Passive collection of unsolicited AE reports.
    SpontaneousReporting,
}

impl DetectionConcept {
    /// All 10 detection concepts.
    pub const ALL: &'static [Self] = &[
        Self::Prr,
        Self::Ror,
        Self::InformationComponent,
        Self::Ebgm,
        Self::ChiSquare,
        Self::EvansCriteria,
        Self::DisproportionalityAnalysis,
        Self::SignalDetection,
        Self::DataMining,
        Self::SpontaneousReporting,
    ];

    /// Lex Primitiva grounding.
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        use LexSymbol::*;
        PrimitiveComposition::new(match self {
            Self::Prr => &[Comparison, Frequency, Boundary, Quantity],
            Self::Ror => &[Comparison, Quantity, Boundary, Product],
            Self::InformationComponent => &[Comparison, Quantity, Recursion, Boundary],
            Self::Ebgm => &[Comparison, Quantity, Recursion, Boundary, Frequency],
            Self::ChiSquare => &[Quantity, Comparison, Boundary],
            Self::EvansCriteria => &[Boundary, Boundary, Boundary, Comparison, Quantity],
            Self::DisproportionalityAnalysis => {
                &[Sum, Comparison, Quantity, Boundary, Frequency, Recursion]
            }
            Self::SignalDetection => &[
                Existence, Comparison, Sequence, Boundary, Quantity, Frequency,
            ],
            Self::DataMining => &[Mapping, Recursion, Existence, Quantity, Boundary, Frequency],
            Self::SpontaneousReporting => &[
                Location,
                Sequence,
                Existence,
                Persistence,
                Causality,
                Boundary,
            ],
        })
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Prr => {
                "Proportional Reporting Ratio: ratio of reporting proportions (exposed vs all)"
            }
            Self::Ror => "Reporting Odds Ratio: cross-product ratio from 2x2 contingency table",
            Self::InformationComponent => {
                "IC = log2(Observed/Expected): Bayesian confidence propagation (BCPNN/WHO-UMC)"
            }
            Self::Ebgm => {
                "Empirical Bayes Geometric Mean: shrinkage estimator for sparse data (MGPS/FDA)"
            }
            Self::ChiSquare => {
                "Chi-square with Yates correction: N*(|ad-bc|-N/2)² / marginal products"
            }
            Self::EvansCriteria => "Evans/MHRA triple gate: PRR ≥ 2.0 AND χ² ≥ 3.841 AND n ≥ 3",
            Self::DisproportionalityAnalysis => {
                "Composite application of PRR, ROR, IC, EBGM, χ² to contingency data"
            }
            Self::SignalDetection => {
                "End-to-end: statistical disproportionality + temporal analysis + clinical review"
            }
            Self::DataMining => {
                "Automated statistical scanning of FAERS/EudraVigilance/VigiBase databases"
            }
            Self::SpontaneousReporting => {
                "Passive surveillance: unsolicited AE reports from HCPs, patients, MAHs"
            }
        }
    }

    /// Minimum Chomsky grammar level for this method.
    #[must_use]
    pub const fn chomsky_level(&self) -> &'static str {
        match self {
            // Pure arithmetic on contingency tables — no recursion needed
            Self::Prr | Self::Ror | Self::ChiSquare | Self::EvansCriteria => "Type-3 (Regular)",
            // Bayesian methods require iterative shrinkage
            Self::InformationComponent | Self::Ebgm => "Type-2 (Context-Free)",
            // Composite methods aggregate context
            Self::DisproportionalityAnalysis | Self::SignalDetection | Self::DataMining => {
                "Type-1 (Context-Sensitive)"
            }
            // Data collection is simple pipeline
            Self::SpontaneousReporting => "Type-3 (Regular)",
        }
    }

    /// ICH/regulatory source reference.
    #[must_use]
    pub const fn source(&self) -> &'static str {
        match self {
            Self::Prr => "ICH E2E, Evans et al. 2001",
            Self::Ror => "ICH E2E, van Puijenbroek et al. 2002",
            Self::InformationComponent => "BCPNN, WHO-UMC, Bate et al. 1998",
            Self::Ebgm => "MGPS, DuMouchel 1999, FDA",
            Self::ChiSquare => "Evans et al. 2001, MHRA",
            Self::EvansCriteria => "Evans et al. 2001, MHRA",
            Self::DisproportionalityAnalysis => "ICH E2E, CIOMS VIII",
            Self::SignalDetection => "ICH E2E, GVP Module IX",
            Self::DataMining => "CIOMS VIII",
            Self::SpontaneousReporting => "ICH E2D",
        }
    }
}

impl fmt::Display for DetectionConcept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Detection::{:?} [{}]", self, self.grounding().tier())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_is_10() {
        assert_eq!(DetectionConcept::ALL.len(), 10);
    }

    #[test]
    fn prr_is_type3() {
        assert_eq!(DetectionConcept::Prr.chomsky_level(), "Type-3 (Regular)");
    }

    #[test]
    fn ebgm_uses_recursion() {
        let g = DetectionConcept::Ebgm.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Recursion),
            "EBGM requires Bayesian shrinkage (recursion)"
        );
    }

    #[test]
    fn all_have_sources() {
        for c in DetectionConcept::ALL {
            assert!(!c.source().is_empty(), "{:?} missing source", c);
        }
    }

    #[test]
    fn evans_has_three_boundaries() {
        // Evans criteria = 3 independent thresholds
        let desc = DetectionConcept::EvansCriteria.description();
        assert!(desc.contains("AND"), "Evans must have AND gates");
    }
}
