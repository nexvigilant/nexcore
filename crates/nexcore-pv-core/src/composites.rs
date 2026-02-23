//! # T2/T3 Composite Inventory
//!
//! Documents the key composite types in nexcore-pv-core with their tier
//! classifications, constituent primitives, and structural descriptors.
//!
//! Composites are types built from multiple T1 primitives. They live at:
//! - **T2-C** (Cross-domain composite): Structurally transferable across domains
//! - **T3** (Domain-specific): Meaningful only within pharmacovigilance
//!
//! ## Design Rationale
//!
//! A composite's tier is determined by its _structural transferability_:
//! - If the structure maps cleanly to 2+ domains (e.g., a 2x2 table), it's T2-C
//! - If the structure requires PV-specific semantics (e.g., ICH E2B fields), it's T3
//!
//! The boundary between T2-C and T3 is the _domain erasure test_: can you strip
//! all PV field names and still recognize the structure? If yes → T2-C.
//!
//! ## T1 Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|-------------|--------|
//! | Composite structure | Product | × |
//! | Tier classification | Comparison | κ |
//! | Constituent listing | Sequence | σ |

use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use nexcore_lex_primitiva::tier::Tier;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// COMPOSITE DESCRIPTOR
// ═══════════════════════════════════════════════════════════════════════════

/// Describes a composite type: its tier, constituent primitives, and structural role.
///
/// # Tier: T2-C (self-referentially — this descriptor is itself a cross-domain composite)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeDescriptor {
    /// Type name (e.g., "CompleteSignalResult")
    pub type_name: &'static str,
    /// Tier classification
    pub tier: Tier,
    /// Constituent T1 primitives
    pub primitives: Vec<LexPrimitiva>,
    /// Dominant primitive (drives the type's primary behavior)
    pub dominant: LexPrimitiva,
    /// Structural role description
    pub role: &'static str,
    /// Source module in pv-core
    pub module: &'static str,
    /// Number of fields / structural complexity indicator
    pub field_count: usize,
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPOSITE INVENTORY
// ═══════════════════════════════════════════════════════════════════════════

/// Returns the complete inventory of key composite types in pv-core.
///
/// Covers the 6 most structurally significant composites:
/// - `CompleteSignalResult` (T2-C): Full disproportionality analysis output
/// - `Icsr` (T3): Individual Case Safety Report (the atomic PV unit)
/// - `PvCycleResult` (T2-C): End-to-end PV cycle output
/// - `LandscapeAnalysis` (T2-C): Competitive benefit-risk landscape
/// - `RiskManagementProgram` (T3): REMS/RMP lifecycle
/// - `PeriodicReport` (T3): PSUR/PBRER/PADER periodic safety report
#[must_use]
pub fn composite_inventory() -> Vec<CompositeDescriptor> {
    vec![
        // ── CompleteSignalResult (T2-C) ─────────────────────────────────
        CompositeDescriptor {
            type_name: "CompleteSignalResult",
            tier: Tier::T2Composite,
            primitives: vec![
                LexPrimitiva::Existence,
                LexPrimitiva::Quantity,
                LexPrimitiva::Comparison,
                LexPrimitiva::Product,
            ],
            dominant: LexPrimitiva::Existence,
            role: "Aggregates PRR, ROR, IC, EBGM disproportionality results into a \
                   single analysis output. Structurally equivalent to any multi-algorithm \
                   detection ensemble — domain-erasable to 'multi-test result'.",
            module: "types",
            field_count: 6,
        },
        // ── Icsr (T3) ──────────────────────────────────────────────────
        CompositeDescriptor {
            type_name: "Icsr",
            tier: Tier::T3DomainSpecific,
            primitives: vec![
                LexPrimitiva::Existence,
                LexPrimitiva::Causality,
                LexPrimitiva::Sequence,
                LexPrimitiva::Comparison,
                LexPrimitiva::State,
                LexPrimitiva::Product,
            ],
            dominant: LexPrimitiva::Existence,
            role: "The atomic unit of pharmacovigilance. Conforms to ICH E2B(R3) with \
                   patient, drugs, reactions, causality, and report metadata. T3 because \
                   the field semantics are PV-specific (drug roles, MedDRA terms, seriousness \
                   criteria) even though the structure (entity + relationships + metadata) \
                   is universal.",
            module: "icsr",
            field_count: 7,
        },
        // ── PvCycleResult (T2-C) ───────────────────────────────────────
        CompositeDescriptor {
            type_name: "PvCycleResult",
            tier: Tier::T2Composite,
            primitives: vec![
                LexPrimitiva::Sequence,
                LexPrimitiva::Existence,
                LexPrimitiva::Causality,
                LexPrimitiva::Recursion,
                LexPrimitiva::Boundary,
                LexPrimitiva::Product,
            ],
            dominant: LexPrimitiva::Sequence,
            role: "End-to-end output of the PV cycle: detect -> assess -> understand -> prevent. \
                   Structurally equivalent to any pipeline result (stages with typed outputs). \
                   T2-C because the 4-stage pipeline pattern transfers to incident response, \
                   immune cascades, and audit cycles.",
            module: "definition",
            field_count: 5,
        },
        // ── LandscapeAnalysis (T2-C) ───────────────────────────────────
        CompositeDescriptor {
            type_name: "LandscapeAnalysis",
            tier: Tier::T2Composite,
            primitives: vec![
                LexPrimitiva::Comparison,
                LexPrimitiva::Quantity,
                LexPrimitiva::Mapping,
                LexPrimitiva::Product,
            ],
            dominant: LexPrimitiva::Comparison,
            role: "Competitive landscape comparing a target drug's benefit-risk profile \
                   against competitors. Structurally: ranked entity list with aggregate \
                   metrics — transfers to market analysis, ecosystem biodiversity indices, \
                   and service reliability comparisons.",
            module: "landscape",
            field_count: 4,
        },
        // ── RiskManagementProgram (T3) ──────────────────────────────────
        CompositeDescriptor {
            type_name: "RiskManagementProgram",
            tier: Tier::T3DomainSpecific,
            primitives: vec![
                LexPrimitiva::State,
                LexPrimitiva::Causality,
                LexPrimitiva::Frequency,
                LexPrimitiva::Boundary,
                LexPrimitiva::Persistence,
                LexPrimitiva::Product,
            ],
            dominant: LexPrimitiva::State,
            role: "REMS/RMP lifecycle management with state transitions \
                   (Draft -> Submitted -> Approved -> Active -> Modified -> Closed). \
                   T3 because the measures (Medication Guide, ETASU, Communication Plan) \
                   and regulatory framework references are PV-specific.",
            module: "risk_management",
            field_count: 8,
        },
        // ── PeriodicReport (T3) ─────────────────────────────────────────
        CompositeDescriptor {
            type_name: "PeriodicReport",
            tier: Tier::T3DomainSpecific,
            primitives: vec![
                LexPrimitiva::Sequence,
                LexPrimitiva::Quantity,
                LexPrimitiva::Comparison,
                LexPrimitiva::State,
                LexPrimitiva::Persistence,
                LexPrimitiva::Product,
            ],
            dominant: LexPrimitiva::Sequence,
            role: "Periodic safety update reports (PSUR/PBRER/PADER/DSUR) per ICH E2C(R2). \
                   Aggregates safety data over a reporting period with benefit-risk conclusions. \
                   T3 because the 12-section structure and regulatory terminology are \
                   PV-specific, even though 'periodic aggregated report' is universal.",
            module: "periodic_reporting",
            field_count: 10,
        },
    ]
}

/// Returns composites at a specific tier.
///
/// # Arguments
///
/// * `tier` — The tier to filter by
#[must_use]
pub fn composites_at_tier(tier: Tier) -> Vec<CompositeDescriptor> {
    composite_inventory()
        .into_iter()
        .filter(|c| c.tier == tier)
        .collect()
}

/// Returns composites where a given primitive is dominant.
///
/// # Arguments
///
/// * `primitive` — The T1 primitive to search for as dominant
#[must_use]
pub fn composites_with_dominant(primitive: LexPrimitiva) -> Vec<CompositeDescriptor> {
    composite_inventory()
        .into_iter()
        .filter(|c| c.dominant == primitive)
        .collect()
}

/// Returns the average structural complexity (field count) across all composites.
#[must_use]
pub fn average_complexity() -> f64 {
    let inv = composite_inventory();
    if inv.is_empty() {
        return 0.0;
    }
    let total: usize = inv.iter().map(|c| c.field_count).sum();
    total as f64 / inv.len() as f64
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_has_six_composites() {
        let inv = composite_inventory();
        assert_eq!(inv.len(), 6);
    }

    #[test]
    fn three_t2c_and_three_t3() {
        let t2c = composites_at_tier(Tier::T2Composite);
        let t3 = composites_at_tier(Tier::T3DomainSpecific);
        assert_eq!(t2c.len(), 3, "Expected 3 T2-C composites");
        assert_eq!(t3.len(), 3, "Expected 3 T3 composites");
    }

    #[test]
    fn all_composites_have_product_primitive() {
        // Product (×) is axiomatic — present in every multi-field struct
        for c in composite_inventory() {
            assert!(
                c.primitives.contains(&LexPrimitiva::Product),
                "Composite '{}' missing Product primitive",
                c.type_name
            );
        }
    }

    #[test]
    fn all_composites_have_non_empty_role() {
        for c in composite_inventory() {
            assert!(
                !c.role.is_empty(),
                "Composite '{}' has empty role",
                c.type_name
            );
        }
    }

    #[test]
    fn all_composites_have_at_least_two_primitives() {
        for c in composite_inventory() {
            assert!(
                c.primitives.len() >= 2,
                "Composite '{}' has only {} primitive(s)",
                c.type_name,
                c.primitives.len()
            );
        }
    }

    #[test]
    fn dominant_is_in_primitives_list() {
        for c in composite_inventory() {
            assert!(
                c.primitives.contains(&c.dominant),
                "Composite '{}' dominant {:?} not in primitives list",
                c.type_name,
                c.dominant
            );
        }
    }

    #[test]
    fn complete_signal_result_is_t2c() {
        let inv = composite_inventory();
        let csr = inv.iter().find(|c| c.type_name == "CompleteSignalResult");
        assert!(csr.is_some());
        assert_eq!(csr.map(|c| c.tier), Some(Tier::T2Composite));
    }

    #[test]
    fn icsr_is_t3() {
        let inv = composite_inventory();
        let icsr = inv.iter().find(|c| c.type_name == "Icsr");
        assert!(icsr.is_some());
        assert_eq!(icsr.map(|c| c.tier), Some(Tier::T3DomainSpecific));
    }

    #[test]
    fn sequence_dominant_composites() {
        let seq = composites_with_dominant(LexPrimitiva::Sequence);
        let names: Vec<&str> = seq.iter().map(|c| c.type_name).collect();
        assert!(names.contains(&"PvCycleResult"));
        assert!(names.contains(&"PeriodicReport"));
    }

    #[test]
    fn average_complexity_is_reasonable() {
        let avg = average_complexity();
        // All composites have 4-10 fields; average should be in that range
        assert!(avg >= 4.0 && avg <= 10.0, "Average complexity {} out of expected range", avg);
    }

    #[test]
    fn field_counts_are_positive() {
        for c in composite_inventory() {
            assert!(
                c.field_count > 0,
                "Composite '{}' has zero fields",
                c.type_name
            );
        }
    }

    #[test]
    fn no_t1_or_t2p_composites() {
        // composite_inventory should only contain T2-C and T3
        for c in composite_inventory() {
            assert!(
                c.tier == Tier::T2Composite || c.tier == Tier::T3DomainSpecific,
                "Composite '{}' has unexpected tier {:?}",
                c.type_name,
                c.tier
            );
        }
    }

    #[test]
    fn type_names_are_unique() {
        let inv = composite_inventory();
        let mut names: Vec<&str> = inv.iter().map(|c| c.type_name).collect();
        let original_len = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), original_len, "Duplicate type names in composite inventory");
    }
}
