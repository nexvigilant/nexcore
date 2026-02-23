//! # Composite Type Inventory
//!
//! Documents the T2-C and T3 composite types in `nexcore-preemptive-pv`.
//! Composites are types that combine multiple unique T1 primitives into
//! domain-meaningful structures: T2-C requires 4-5 unique primitives; T3 requires 6+.
//!
//! ## Composite Types
//!
//! | Type | Tier | Unique Primitives | Description |
//! |------|------|-------------------|-------------|
//! | `PredictiveResult` | T2-C | N, ->, exists, nu | Multi-metric signal evaluation |
//! | `PreemptiveConfig` | T2-C | varsigma, N, ->, partial | Full decision configuration |
//! | `PreemptiveResult` | T3 | N, ->, partial, prop, exists, kappa | Complete 3-tier decision |
//!
//! ## Primitives at Play
//!
//! - `Sigma` (Sum): Composites aggregate sub-results
//! - `times` (Product): Composites ARE products of their component fields
//! - `kappa` (Comparison): Tier classification compares primitive counts

use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use nexcore_lex_primitiva::tier::Tier;
use serde::{Deserialize, Serialize};

/// Describes a composite type and its primitive decomposition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeDescriptor {
    /// Type name (e.g., "PredictiveResult")
    pub name: &'static str,
    /// Tier classification
    pub tier: Tier,
    /// The T1 primitives that compose this type
    pub component_primitives: Vec<LexPrimitiva>,
    /// Human-readable description of the composite's role
    pub description: &'static str,
}

/// Returns the full inventory of composite types in this crate.
///
/// Only includes T2-C and T3 types (those with 4+ unique primitives).
/// T2-P types (2-3 primitives) are simpler compositions documented
/// in the `grounding` module.
#[must_use]
pub fn composite_inventory() -> Vec<CompositeDescriptor> {
    vec![
        CompositeDescriptor {
            name: "PredictiveResult",
            tier: Tier::T2Composite,
            component_primitives: vec![
                LexPrimitiva::Quantity,
                LexPrimitiva::Causality,
                LexPrimitiva::Existence,
                LexPrimitiva::Frequency,
            ],
            description: "Multi-component result from Tier 2 predictive signal evaluation. \
                Composes Gibbs feasibility (N + ->), trajectory (nu), noise correction (N), \
                and signal existence test (exists) into the Preemptive Signal Potential Psi.",
        },
        CompositeDescriptor {
            name: "PreemptiveConfig",
            tier: Tier::T2Composite,
            component_primitives: vec![
                LexPrimitiva::State,
                LexPrimitiva::Quantity,
                LexPrimitiva::Causality,
                LexPrimitiva::Boundary,
            ],
            description: "Full domain configuration for the three-tier preemptive decision engine. \
                Nests PredictiveConfig (varsigma), adds intervention cost (N), causal model \
                parameters (->), and detection boundary thresholds (partial).",
        },
        CompositeDescriptor {
            name: "PreemptiveResult",
            tier: Tier::T3DomainSpecific,
            component_primitives: vec![
                LexPrimitiva::Quantity,
                LexPrimitiva::Causality,
                LexPrimitiva::Boundary,
                LexPrimitiva::Irreversibility,
                LexPrimitiva::Existence,
                LexPrimitiva::Comparison,
            ],
            description: "The complete three-tier preemptive evaluation result. THE domain type \
                of this crate. Integrates predictive metrics (N), causal assessment (->), \
                safety boundary enforcement (partial), irreversibility weighting via Omega (prop), \
                signal existence determination (exists), and threshold comparison for tier \
                escalation (kappa). This is the only T3 type -- patient safety at stake.",
        },
    ]
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use nexcore_lex_primitiva::grounding::GroundsTo;

    #[test]
    fn inventory_has_3_composites() {
        let inv = composite_inventory();
        assert_eq!(inv.len(), 3);
    }

    #[test]
    fn predictive_result_is_t2c() {
        let inv = composite_inventory();
        let pred = inv.iter().find(|c| c.name == "PredictiveResult");
        assert!(pred.is_some());
        if let Some(p) = pred {
            assert_eq!(p.tier, Tier::T2Composite);
            assert_eq!(p.component_primitives.len(), 4);
        }
    }

    #[test]
    fn preemptive_config_is_t2c() {
        let inv = composite_inventory();
        let cfg = inv.iter().find(|c| c.name == "PreemptiveConfig");
        assert!(cfg.is_some());
        if let Some(c) = cfg {
            assert_eq!(c.tier, Tier::T2Composite);
            assert_eq!(c.component_primitives.len(), 4);
        }
    }

    #[test]
    fn preemptive_result_is_t3() {
        let inv = composite_inventory();
        let result = inv.iter().find(|c| c.name == "PreemptiveResult");
        assert!(result.is_some());
        if let Some(r) = result {
            assert_eq!(r.tier, Tier::T3DomainSpecific);
            assert_eq!(r.component_primitives.len(), 6);
        }
    }

    #[test]
    fn only_one_t3_type() {
        let inv = composite_inventory();
        let t3_count = inv
            .iter()
            .filter(|c| c.tier == Tier::T3DomainSpecific)
            .count();
        assert_eq!(t3_count, 1, "Only PreemptiveResult should be T3");
    }

    #[test]
    fn tiers_match_grounding_impls() {
        // Verify that the tiers declared here match the actual GroundsTo impls
        use crate::predictive::PredictiveResult;
        use crate::preemptive::{PreemptiveConfig, PreemptiveResult};

        let inv = composite_inventory();

        let pred = inv.iter().find(|c| c.name == "PredictiveResult");
        assert!(pred.is_some());
        if let Some(p) = pred {
            assert_eq!(p.tier, PredictiveResult::tier());
        }

        let cfg = inv.iter().find(|c| c.name == "PreemptiveConfig");
        assert!(cfg.is_some());
        if let Some(c) = cfg {
            assert_eq!(c.tier, PreemptiveConfig::tier());
        }

        let result = inv.iter().find(|c| c.name == "PreemptiveResult");
        assert!(result.is_some());
        if let Some(r) = result {
            assert_eq!(r.tier, PreemptiveResult::tier());
        }
    }

    #[test]
    fn all_composites_have_4_plus_primitives() {
        let inv = composite_inventory();
        for composite in &inv {
            assert!(
                composite.component_primitives.len() >= 4,
                "{} has only {} primitives (need 4+ for composite)",
                composite.name,
                composite.component_primitives.len()
            );
        }
    }

    #[test]
    fn no_duplicate_primitives_per_composite() {
        let inv = composite_inventory();
        for composite in &inv {
            let unique: std::collections::HashSet<LexPrimitiva> =
                composite.component_primitives.iter().copied().collect();
            assert_eq!(
                unique.len(),
                composite.component_primitives.len(),
                "{} has duplicate primitives",
                composite.name
            );
        }
    }

    #[test]
    fn all_composites_have_descriptions() {
        let inv = composite_inventory();
        for composite in &inv {
            assert!(
                !composite.description.is_empty(),
                "{} has empty description",
                composite.name
            );
            assert!(
                composite.description.len() > 50,
                "{} has suspiciously short description",
                composite.name
            );
        }
    }

    #[test]
    fn preemptive_result_contains_irreversibility() {
        let inv = composite_inventory();
        let result = inv.iter().find(|c| c.name == "PreemptiveResult");
        assert!(result.is_some());
        if let Some(r) = result {
            assert!(
                r.component_primitives
                    .contains(&LexPrimitiva::Irreversibility),
                "PreemptiveResult must contain Irreversibility -- it's the defining primitive"
            );
        }
    }

    #[test]
    fn tier_ordering_monotonic() {
        let inv = composite_inventory();
        // T2-C types should appear before T3 types in the inventory
        let tiers: Vec<Tier> = inv.iter().map(|c| c.tier).collect();
        for window in tiers.windows(2) {
            assert!(
                window[0] <= window[1],
                "Inventory should be ordered by tier (got {:?} before {:?})",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn serde_roundtrip() {
        let inv = composite_inventory();
        let json = serde_json::to_string(&inv);
        assert!(json.is_ok());

        // CompositeDescriptor uses &'static str fields, so full deserialization
        // requires a 'static borrow. Verify via serde_json::Value instead.
        if let Ok(json_str) = json {
            let value: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
            assert!(value.is_ok());

            if let Ok(serde_json::Value::Array(arr)) = value {
                assert_eq!(arr.len(), 3);
            }
        }
    }
}
