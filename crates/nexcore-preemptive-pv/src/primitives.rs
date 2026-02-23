//! # T1 Primitive Inventory
//!
//! Documents the 11 operational Lex Primitiva that manifest in the preemptive
//! PV domain. Each primitive's role is catalogued with a description of how
//! it appears in the crate's types and equations.
//!
//! ## The 11 Operational Primitives
//!
//! | Symbol | Name | Manifestation |
//! |--------|------|---------------|
//! | `->` | Causality | Drug causes adverse event; intervention causes rate reduction |
//! | `kappa` | Comparison | Threshold comparisons; tier escalation |
//! | `prop` | Irreversibility | Omega weighting; fatal outcomes cannot be undone |
//! | `sigma` | Sequence | Temporal trajectory of reporting data |
//! | `partial` | Boundary | Detection thresholds; noise/signal boundary |
//! | `N` | Quantity | Numeric counts, rates, scores throughout |
//! | `Σ` | Sum | Decision variants (Monitor/Predict/Intervene); Seriousness enum |
//! | `times` | Product | Drug-event pairs; parameter tuples |
//! | `exists` | Existence | Signal existence test (does Psi > 0?) |
//! | `nu` | Frequency | Reporting rate frequency; temporal trajectory |
//! | `varsigma` | State | Configuration state (PredictiveConfig, PreemptiveConfig) |
//!
//! ## Primitives NOT Present
//!
//! `mu` (Mapping), `rho` (Recursion), `empty` (Void), `lambda` (Location),
//! `pi` (Persistence) are not operationally present in this crate.

use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};

/// Describes how a T1 primitive manifests in the preemptive PV domain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrimitiveManifestation {
    /// The Lex Primitiva symbol
    pub primitive: LexPrimitiva,
    /// Unicode symbol for display
    pub symbol: &'static str,
    /// Human-readable name
    pub name: &'static str,
    /// How this primitive manifests in the preemptive PV domain
    pub manifestation: &'static str,
    /// Which crate types exhibit this primitive
    pub exhibited_by: Vec<&'static str>,
}

/// Complete manifest of T1 primitives present in this crate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CratePrimitiveManifest {
    /// Crate name
    pub crate_name: &'static str,
    /// Total count of unique T1 primitives used
    pub primitive_count: usize,
    /// Total count of T1 primitives in the Lex Primitiva system
    pub total_primitives: usize,
    /// Coverage ratio (primitive_count / total operational primitives)
    pub coverage: f64,
    /// Individual primitive manifestations
    pub manifestations: Vec<PrimitiveManifestation>,
}

/// Returns the complete T1 primitive manifest for `nexcore-preemptive-pv`.
///
/// Documents all 11 operational primitives that appear in this crate's
/// types and equations, along with their manifestation descriptions.
#[must_use]
pub fn manifest() -> CratePrimitiveManifest {
    let manifestations = vec![
        PrimitiveManifestation {
            primitive: LexPrimitiva::Causality,
            symbol: "\u{2192}",
            name: "Causality",
            manifestation: "Drug-event causal relationship; intervention causes rate reduction; \
                GibbsParams determine signal feasibility; Decision causes downstream action",
            exhibited_by: vec![
                "Decision",
                "InterventionResult",
                "GibbsParams",
                "PredictiveResult",
                "PreemptiveConfig",
                "PreemptiveResult",
            ],
        },
        PrimitiveManifestation {
            primitive: LexPrimitiva::Comparison,
            symbol: "\u{03BA}",
            name: "Comparison",
            manifestation: "Threshold comparison (Psi vs theta_preemptive); severity ordering \
                in Seriousness; tier escalation logic (Monitor < Predict < Intervene)",
            exhibited_by: vec!["Seriousness", "Decision", "PreemptiveResult"],
        },
        PrimitiveManifestation {
            primitive: LexPrimitiva::Irreversibility,
            symbol: "\u{221D}",
            name: "Irreversibility",
            manifestation: "Omega weighting makes irreversible outcomes (Fatal, LifeThreatening) \
                dominate the decision. Once a patient dies, no intervention can reverse it. \
                The irreversibility_factor() on Seriousness encodes this directly.",
            exhibited_by: vec!["PreemptiveResult", "Seriousness"],
        },
        PrimitiveManifestation {
            primitive: LexPrimitiva::Sequence,
            symbol: "\u{03C3}",
            name: "Sequence",
            manifestation: "Temporal ordering of ReportingDataPoints; trajectory computation \
                requires sequential time-series data; Gamma measures change over ordered intervals",
            exhibited_by: vec!["ReportingDataPoint"],
        },
        PrimitiveManifestation {
            primitive: LexPrimitiva::Boundary,
            symbol: "\u{2202}",
            name: "Boundary",
            manifestation: "Detection threshold (theta_preemptive); noise/signal boundary (eta); \
                safety lambda adjusts boundary sensitivity; intervention creates new safety boundary",
            exhibited_by: vec![
                "Seriousness",
                "SafetyLambda",
                "InterventionResult",
                "NoiseParams",
                "PreemptiveConfig",
                "PreemptiveResult",
            ],
        },
        PrimitiveManifestation {
            primitive: LexPrimitiva::Quantity,
            symbol: "N",
            name: "Quantity",
            manifestation: "Numeric values pervade: reporting counts (a,b,c,d), rates, scores, \
                thresholds, Psi, Pi, Omega, DeltaG, Gamma, eta -- all are f64 quantities",
            exhibited_by: vec![
                "ReportingCounts",
                "ReportingDataPoint",
                "GibbsParams",
                "NoiseParams",
                "SafetyLambda",
                "PredictiveConfig",
                "PredictiveResult",
                "PreemptiveConfig",
                "PreemptiveResult",
            ],
        },
        PrimitiveManifestation {
            primitive: LexPrimitiva::Sum,
            symbol: "\u{03A3}",
            name: "Sum",
            manifestation: "Seriousness is one-of-5 (sum type); Decision is one-of-3 (sum type); \
                both represent exclusive variant selection via Rust enum dispatch",
            exhibited_by: vec!["Seriousness", "Decision"],
        },
        PrimitiveManifestation {
            primitive: LexPrimitiva::Product,
            symbol: "\u{00D7}",
            name: "Product",
            manifestation: "DrugEventPair = drug_id x event_id; ReportingCounts = a x b x c x d; \
                GibbsParams = deltaH x T x deltaS; all struct fields form product types",
            exhibited_by: vec![
                "DrugEventPair",
                "ReportingCounts",
                "ReportingDataPoint",
                "GibbsParams",
            ],
        },
        PrimitiveManifestation {
            primitive: LexPrimitiva::Existence,
            symbol: "\u{2203}",
            name: "Existence",
            manifestation: "Signal existence test: does Psi > 0? Does the drug-event pair exist \
                in the reporting database? The Optional<InterventionResult> tests existence of intervention.",
            exhibited_by: vec!["DrugEventPair", "PredictiveResult", "PreemptiveResult"],
        },
        PrimitiveManifestation {
            primitive: LexPrimitiva::Frequency,
            symbol: "\u{03BD}",
            name: "Frequency",
            manifestation: "Reporting rate is a frequency (events per time unit); noise floor \
                distinguishes organic reporting frequency from stimulated frequency; \
                trajectory measures change in frequency over time",
            exhibited_by: vec!["NoiseParams", "PredictiveResult"],
        },
        PrimitiveManifestation {
            primitive: LexPrimitiva::State,
            symbol: "\u{03C2}",
            name: "State",
            manifestation: "PredictiveConfig and PreemptiveConfig are configuration states; \
                they parameterize the system behavior without being consumed. \
                The three-tier decision is also a state transition (Monitor -> Predict -> Intervene).",
            exhibited_by: vec!["PredictiveConfig", "PreemptiveConfig"],
        },
    ];

    let primitive_count = manifestations.len();

    CratePrimitiveManifest {
        crate_name: "nexcore-preemptive-pv",
        primitive_count,
        total_primitives: 15,
        // 11 out of 15 operational (excluding axiomatic Product which is structurally
        // present but counted here since it's explicitly used)
        coverage: primitive_count as f64 / 15.0,
        manifestations,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn manifest_has_11_primitives() {
        let m = manifest();
        assert_eq!(m.primitive_count, 11);
        assert_eq!(m.manifestations.len(), 11);
    }

    #[test]
    fn no_duplicate_primitives() {
        let m = manifest();
        let primitives: HashSet<LexPrimitiva> =
            m.manifestations.iter().map(|p| p.primitive).collect();
        assert_eq!(primitives.len(), 11, "Duplicate primitives in manifest");
    }

    #[test]
    fn coverage_is_correct() {
        let m = manifest();
        let expected = 11.0 / 15.0;
        assert!(
            (m.coverage - expected).abs() < f64::EPSILON,
            "Coverage should be 11/15 = {}, got {}",
            expected,
            m.coverage
        );
    }

    #[test]
    fn root_primitives_present() {
        let m = manifest();
        let primitives: HashSet<LexPrimitiva> =
            m.manifestations.iter().map(|p| p.primitive).collect();

        // Causality and Quantity are root primitives -- both must be present
        assert!(
            primitives.contains(&LexPrimitiva::Causality),
            "Root primitive Causality missing"
        );
        assert!(
            primitives.contains(&LexPrimitiva::Quantity),
            "Root primitive Quantity missing"
        );
    }

    #[test]
    fn irreversibility_present() {
        let m = manifest();
        let has_irreversibility = m
            .manifestations
            .iter()
            .any(|p| p.primitive == LexPrimitiva::Irreversibility);
        assert!(
            has_irreversibility,
            "Irreversibility is central to preemptive PV and must be present"
        );
    }

    #[test]
    fn all_manifestations_have_exhibited_types() {
        let m = manifest();
        for manifestation in &m.manifestations {
            assert!(
                !manifestation.exhibited_by.is_empty(),
                "Primitive {:?} has no exhibiting types",
                manifestation.primitive
            );
        }
    }

    #[test]
    fn all_manifestations_have_descriptions() {
        let m = manifest();
        for manifestation in &m.manifestations {
            assert!(
                !manifestation.manifestation.is_empty(),
                "Primitive {:?} has empty manifestation description",
                manifestation.primitive
            );
            assert!(
                manifestation.manifestation.len() > 20,
                "Primitive {:?} has suspiciously short description",
                manifestation.primitive
            );
        }
    }

    #[test]
    fn boundary_is_most_exhibited() {
        let m = manifest();
        let boundary = m
            .manifestations
            .iter()
            .find(|p| p.primitive == LexPrimitiva::Boundary);
        assert!(boundary.is_some());

        if let Some(b) = boundary {
            // Boundary should appear in 6 types (highest count)
            assert!(
                b.exhibited_by.len() >= 6,
                "Boundary should be exhibited by at least 6 types, got {}",
                b.exhibited_by.len()
            );
        }
    }

    #[test]
    fn quantity_pervades() {
        let m = manifest();
        let quantity = m
            .manifestations
            .iter()
            .find(|p| p.primitive == LexPrimitiva::Quantity);
        assert!(quantity.is_some());

        if let Some(q) = quantity {
            // Quantity should appear in many types (numeric values everywhere)
            assert!(
                q.exhibited_by.len() >= 8,
                "Quantity should be exhibited by at least 8 types, got {}",
                q.exhibited_by.len()
            );
        }
    }

    #[test]
    fn crate_name_correct() {
        let m = manifest();
        assert_eq!(m.crate_name, "nexcore-preemptive-pv");
    }

    #[test]
    fn total_primitives_is_15() {
        let m = manifest();
        assert_eq!(m.total_primitives, 15);
    }

    #[test]
    fn excluded_primitives_not_present() {
        let m = manifest();
        let primitives: HashSet<LexPrimitiva> =
            m.manifestations.iter().map(|p| p.primitive).collect();

        // These 5 primitives should NOT be in the manifest
        assert!(
            !primitives.contains(&LexPrimitiva::Mapping),
            "Mapping should not be in manifest"
        );
        assert!(
            !primitives.contains(&LexPrimitiva::Recursion),
            "Recursion should not be in manifest"
        );
        assert!(
            !primitives.contains(&LexPrimitiva::Void),
            "Void should not be in manifest"
        );
        assert!(
            !primitives.contains(&LexPrimitiva::Location),
            "Location should not be in manifest"
        );
        assert!(
            !primitives.contains(&LexPrimitiva::Persistence),
            "Persistence should not be in manifest"
        );
    }

    #[test]
    fn serde_roundtrip() {
        let m = manifest();
        let json = serde_json::to_string(&m);
        assert!(json.is_ok());

        if let Ok(json_str) = json {
            let value: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
            assert!(value.is_ok());
        }
    }
}
