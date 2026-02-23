//! # Composite Type Inventory
//!
//! Documents the T2 and T3 composite types in this crate -- types built from
//! multiple T1 Lex Primitiva that form meaningful structural patterns.
//!
//! ## Composite Types
//!
//! | Type | Tier | Primitives | Pattern |
//! |------|------|-----------|---------|
//! | `Incident` | T2-C | pi + arrow + sigma + varsigma + kappa | Recorded event with lifecycle |
//! | `Playbook` | T2-C | sigma + mu + pi + kappa | Reusable response sequence |
//! | `MemoryStore` | T2-P | pi + mu + varsigma | Persistent pattern-indexed store |

use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use nexcore_lex_primitiva::tier::Tier;
use serde::Serialize;

/// Descriptor for a composite type in this crate.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompositeDescriptor {
    /// Type name.
    pub name: &'static str,
    /// Tier classification.
    pub tier: Tier,
    /// The primitives composing this type.
    pub primitives: Vec<LexPrimitiva>,
    /// Short description of the composite pattern.
    pub pattern: &'static str,
    /// Whether this type serves as a primary data container.
    pub is_aggregate: bool,
}

/// Returns descriptors for all composite types in this crate.
///
/// Composite types are those classified at T2-P or higher that combine
/// multiple T1 primitives into meaningful domain structures.
#[must_use]
pub fn composite_inventory() -> Vec<CompositeDescriptor> {
    vec![
        CompositeDescriptor {
            name: "Incident",
            tier: Tier::T2Composite,
            primitives: vec![
                LexPrimitiva::Persistence,
                LexPrimitiva::Causality,
                LexPrimitiva::Sequence,
                LexPrimitiva::State,
                LexPrimitiva::Comparison,
            ],
            pattern: "Recorded event with detection-to-resolution lifecycle, \
                combining persistence (memory), causality (trigger chain), \
                sequence (temporal ordering), state (health transitions), \
                and comparison (similarity matching)",
            is_aggregate: true,
        },
        CompositeDescriptor {
            name: "Playbook",
            tier: Tier::T2Composite,
            primitives: vec![
                LexPrimitiva::Sequence,
                LexPrimitiva::Mapping,
                LexPrimitiva::Persistence,
                LexPrimitiva::Comparison,
            ],
            pattern: "Reusable response sequence that maps incident patterns \
                to ordered action steps, persisted and refined over time \
                through success-rate tracking",
            is_aggregate: true,
        },
        CompositeDescriptor {
            name: "MemoryStore",
            tier: Tier::T2Primitive,
            primitives: vec![
                LexPrimitiva::Persistence,
                LexPrimitiva::Mapping,
                LexPrimitiva::State,
            ],
            pattern: "Persistent pattern-indexed store — the immune memory itself. \
                Maps incident signatures to historical incidents and playbooks, \
                maintaining mutable state with capacity enforcement",
            is_aggregate: true,
        },
    ]
}

/// Returns the composite descriptor for a named type, if it exists.
#[must_use]
pub fn composite_by_name(name: &str) -> Option<CompositeDescriptor> {
    composite_inventory().into_iter().find(|d| d.name == name)
}

/// Returns only the T2-C (cross-domain composite) types.
#[must_use]
pub fn t2c_composites() -> Vec<CompositeDescriptor> {
    composite_inventory()
        .into_iter()
        .filter(|d| d.tier == Tier::T2Composite)
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn inventory_has_3_composites() {
        let inv = composite_inventory();
        assert_eq!(inv.len(), 3);
    }

    #[test]
    fn incident_is_t2c() {
        let desc = composite_by_name("Incident");
        assert!(desc.is_some());
        let desc = desc.unwrap_or_else(|| CompositeDescriptor {
            name: "",
            tier: Tier::T1Universal,
            primitives: vec![],
            pattern: "",
            is_aggregate: false,
        });
        assert_eq!(desc.tier, Tier::T2Composite);
        assert_eq!(desc.primitives.len(), 5);
        assert!(desc.is_aggregate);
    }

    #[test]
    fn playbook_is_t2c() {
        let desc = composite_by_name("Playbook");
        assert!(desc.is_some());
        let desc = desc.unwrap_or_else(|| CompositeDescriptor {
            name: "",
            tier: Tier::T1Universal,
            primitives: vec![],
            pattern: "",
            is_aggregate: false,
        });
        assert_eq!(desc.tier, Tier::T2Composite);
        assert_eq!(desc.primitives.len(), 4);
    }

    #[test]
    fn memory_store_is_t2p() {
        let desc = composite_by_name("MemoryStore");
        assert!(desc.is_some());
        let desc = desc.unwrap_or_else(|| CompositeDescriptor {
            name: "",
            tier: Tier::T1Universal,
            primitives: vec![],
            pattern: "",
            is_aggregate: false,
        });
        assert_eq!(desc.tier, Tier::T2Primitive);
        assert_eq!(desc.primitives.len(), 3);
    }

    #[test]
    fn t2c_filter_returns_2() {
        let t2c = t2c_composites();
        assert_eq!(t2c.len(), 2);
        let names: Vec<&str> = t2c.iter().map(|d| d.name).collect();
        assert!(names.contains(&"Incident"));
        assert!(names.contains(&"Playbook"));
    }

    #[test]
    fn unknown_type_returns_none() {
        let desc = composite_by_name("NonExistent");
        assert!(desc.is_none());
    }

    #[test]
    fn all_composites_have_patterns() {
        for desc in composite_inventory() {
            assert!(!desc.pattern.is_empty(), "{} has empty pattern", desc.name);
        }
    }

    #[test]
    fn all_composites_have_primitives() {
        for desc in composite_inventory() {
            assert!(
                !desc.primitives.is_empty(),
                "{} has no primitives",
                desc.name,
            );
        }
    }

    #[test]
    fn all_composites_are_aggregates() {
        for desc in composite_inventory() {
            assert!(desc.is_aggregate, "{} should be an aggregate", desc.name);
        }
    }

    #[test]
    fn serde_round_trip() {
        for desc in composite_inventory() {
            let json = serde_json::to_string(&desc);
            assert!(json.is_ok(), "Serialization failed for {}", desc.name);
        }
    }
}
