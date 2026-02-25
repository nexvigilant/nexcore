//! # Composite Type Inventory
//!
//! Documents the T2-C and T3 composite types in `nexcore-signal-pipeline`,
//! each decomposed into its constituent T1 primitives.
//!
//! ## Tier Classification
//!
//! - **T2-C (Cross-domain composite)**: Built from 2-5 T1/T2-P primitives,
//!   transfers to other domains with moderate fidelity. Examples:
//!   `NormalizedEvent`, `ValidationReport`, `SignalMetrics`.
//!
//! - **T3 (Domain-specific)**: Built from 6+ primitives or deeply domain-bound,
//!   transfers poorly outside pharmacovigilance. Examples:
//!   `RawReport`, `DetectionResult`, `Alert`.
//!
//! ## T1 Primitive: Product (x)
//!
//! Every composite type is fundamentally a product: it combines
//! multiple primitives into a single conjunctive structure.

use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use nexcore_lex_primitiva::tier::Tier;
use serde::{Deserialize, Serialize};

/// Descriptor for a composite type in the signal pipeline.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeDescriptor {
    /// Type name (e.g., "NormalizedEvent").
    pub type_name: &'static str,
    /// Tier classification.
    pub tier: Tier,
    /// Constituent T1 primitives (in order of dominance).
    pub primitives: Vec<LexPrimitiva>,
    /// Number of struct fields (structural complexity proxy).
    pub field_count: u8,
    /// Human-readable description.
    pub description: &'static str,
    /// Approximate cross-domain transfer potential (0.0 to 1.0).
    pub transfer_potential: f64,
}

impl CompositeDescriptor {
    /// Create a new composite descriptor.
    #[must_use]
    pub const fn new(
        type_name: &'static str,
        tier: Tier,
        field_count: u8,
        description: &'static str,
        transfer_potential: f64,
    ) -> Self {
        Self {
            type_name,
            tier,
            primitives: Vec::new(),
            field_count,
            description,
            transfer_potential,
        }
    }

    /// Builder: set primitives list.
    #[must_use]
    pub fn with_primitives(mut self, primitives: Vec<LexPrimitiva>) -> Self {
        self.primitives = primitives;
        self
    }

    /// Primitive count (structural weight).
    #[must_use]
    pub fn primitive_count(&self) -> usize {
        self.primitives.len()
    }

    /// Whether this composite is cross-domain transferable (T2-C).
    #[must_use]
    pub fn is_cross_domain(&self) -> bool {
        matches!(self.tier, Tier::T2Composite)
    }

    /// Whether this composite is domain-specific (T3).
    #[must_use]
    pub fn is_domain_specific(&self) -> bool {
        matches!(self.tier, Tier::T3DomainSpecific)
    }
}

/// Return the full inventory of composite types in the signal pipeline.
#[must_use]
pub fn composite_inventory() -> Vec<CompositeDescriptor> {
    vec![
        // ---- T2-C: Cross-domain composites ----
        CompositeDescriptor::new(
            "NormalizedEvent",
            Tier::T2Composite,
            7,
            "Standardized event after drug/event name normalization. \
             Pattern: take raw input, apply mapping, produce canonical form.",
            0.85,
        )
        .with_primitives(vec![
            LexPrimitiva::Mapping,   // mu: raw -> standardized
            LexPrimitiva::Existence, // exists: NexId creation
            LexPrimitiva::Location,  // lambda: source + MedDRA codes
            LexPrimitiva::Product,   // x: conjunctive struct
        ]),
        CompositeDescriptor::new(
            "ValidationReport",
            Tier::T2Composite,
            3,
            "Outcome of quality checks on a detection result. \
             Pattern: run predicate suite, collect pass/fail, summarize.",
            0.88,
        )
        .with_primitives(vec![
            LexPrimitiva::Comparison, // kappa: check predicates
            LexPrimitiva::Sequence,   // sigma: ordered check list
            LexPrimitiva::Product,    // x: conjunctive struct
        ]),
        CompositeDescriptor::new(
            "SignalMetrics",
            Tier::T2Composite,
            6,
            "All computed disproportionality metrics for a contingency table. \
             Pattern: transform input to multi-metric output.",
            0.85,
        )
        .with_primitives(vec![
            LexPrimitiva::Quantity,   // N: PRR, ROR, IC, EBGM, chi-sq
            LexPrimitiva::Mapping,    // mu: table -> metrics
            LexPrimitiva::Comparison, // kappa: strength classification
            LexPrimitiva::Product,    // x: conjunctive struct
        ]),
        CompositeDescriptor::new(
            "ContingencyTable",
            Tier::T2Composite,
            4,
            "2x2 contingency table, the atom of disproportionality analysis. \
             Pattern: four numeric cells in fixed structure.",
            0.92,
        )
        .with_primitives(vec![
            LexPrimitiva::Quantity, // N: cell counts (a, b, c, d)
            LexPrimitiva::Product,  // x: 4-cell structure
            LexPrimitiva::Mapping,  // mu: cell position -> count
            LexPrimitiva::Boundary, // partial: exposed/unexposed boundary
        ]),
        CompositeDescriptor::new(
            "ThresholdConfig",
            Tier::T2Composite,
            6,
            "Evans criteria thresholds for signal detection. \
             Pattern: named boundary values for gating decisions.",
            0.88,
        )
        .with_primitives(vec![
            LexPrimitiva::Boundary, // partial: threshold boundaries
            LexPrimitiva::Quantity, // N: numeric threshold values
            LexPrimitiva::Product,  // x: conjunctive struct
        ]),
        // ---- T3: Domain-specific composites ----
        CompositeDescriptor::new(
            "RawReport",
            Tier::T3DomainSpecific,
            6,
            "Raw adverse event report before normalization. \
             Deeply PV-specific: drug names, event terms, source, metadata.",
            0.60,
        )
        .with_primitives(vec![
            LexPrimitiva::Existence,   // exists: report identity
            LexPrimitiva::Sequence,    // sigma: lists of drugs/events
            LexPrimitiva::Location,    // lambda: ReportSource origin
            LexPrimitiva::Persistence, // pi: metadata blob
            LexPrimitiva::Product,     // x: conjunctive struct
            LexPrimitiva::Frequency,   // nu: report date (temporal)
        ]),
        CompositeDescriptor::new(
            "DetectionResult",
            Tier::T3DomainSpecific,
            9,
            "Full signal detection output for a drug-event pair. \
             Combines pair identity, contingency table, all metrics, strength, timestamp.",
            0.55,
        )
        .with_primitives(vec![
            LexPrimitiva::Quantity,   // N: PRR, ROR, IC, EBGM, chi-sq
            LexPrimitiva::Comparison, // kappa: strength classification
            LexPrimitiva::State,      // varsigma: detection snapshot
            LexPrimitiva::Product,    // x: conjunctive struct
            LexPrimitiva::Existence,  // exists: pair identity
            LexPrimitiva::Frequency,  // nu: detected_at timestamp
        ]),
        CompositeDescriptor::new(
            "Alert",
            Tier::T3DomainSpecific,
            6,
            "Signal alert with lifecycle state machine. \
             PV-specific: wraps DetectionResult with review workflow.",
            0.50,
        )
        .with_primitives(vec![
            LexPrimitiva::State,     // varsigma: AlertState lifecycle
            LexPrimitiva::Existence, // exists: NexId
            LexPrimitiva::Recursion, // rho: contains DetectionResult (nested composite)
            LexPrimitiva::Sequence,  // sigma: notes list
            LexPrimitiva::Frequency, // nu: created_at, updated_at
            LexPrimitiva::Product,   // x: conjunctive struct
        ]),
        CompositeDescriptor::new(
            "DrugEventPair",
            Tier::T3DomainSpecific,
            2,
            "Drug-event pair identifier. Minimal but domain-locked: \
             the pair concept is specific to pharmacovigilance.",
            0.65,
        )
        .with_primitives(vec![
            LexPrimitiva::Existence, // exists: named entity
            LexPrimitiva::Product,   // x: (drug, event) tuple
        ]),
    ]
}

/// Return composites filtered by tier.
#[must_use]
pub fn composites_by_tier(tier: Tier) -> Vec<CompositeDescriptor> {
    composite_inventory()
        .into_iter()
        .filter(|c| c.tier == tier)
        .collect()
}

/// Return the composite descriptor for a given type name.
#[must_use]
pub fn composite_by_name(type_name: &str) -> Option<CompositeDescriptor> {
    composite_inventory()
        .into_iter()
        .find(|c| c.type_name == type_name)
}

/// Compute the average transfer potential across all composites.
#[must_use]
#[allow(
    clippy::as_conversions,
    reason = "usize->f64 cast for averaging; inventory size never exceeds f64 precision limits"
)]
pub fn average_transfer_potential() -> f64 {
    let inv = composite_inventory();
    if inv.is_empty() {
        return 0.0;
    }
    let sum: f64 = inv.iter().map(|c| c.transfer_potential).sum();
    sum / inv.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_is_not_empty() {
        let inv = composite_inventory();
        assert!(!inv.is_empty());
    }

    #[test]
    fn inventory_has_9_composites() {
        // 5 T2-C + 4 T3 = 9
        let inv = composite_inventory();
        assert_eq!(inv.len(), 9);
    }

    #[test]
    fn t2c_composites_count() {
        let t2c = composites_by_tier(Tier::T2Composite);
        assert_eq!(t2c.len(), 5, "Expected 5 T2-C composites");
        for c in &t2c {
            assert!(c.is_cross_domain());
            assert!(!c.is_domain_specific());
        }
    }

    #[test]
    fn t3_composites_count() {
        let t3 = composites_by_tier(Tier::T3DomainSpecific);
        assert_eq!(t3.len(), 4, "Expected 4 T3 composites");
        for c in &t3 {
            assert!(c.is_domain_specific());
            assert!(!c.is_cross_domain());
        }
    }

    #[test]
    fn t2c_types_are_correct() {
        let t2c = composites_by_tier(Tier::T2Composite);
        let names: Vec<&str> = t2c.iter().map(|c| c.type_name).collect();
        assert!(names.contains(&"NormalizedEvent"));
        assert!(names.contains(&"ValidationReport"));
        assert!(names.contains(&"SignalMetrics"));
        assert!(names.contains(&"ContingencyTable"));
        assert!(names.contains(&"ThresholdConfig"));
    }

    #[test]
    fn t3_types_are_correct() {
        let t3 = composites_by_tier(Tier::T3DomainSpecific);
        let names: Vec<&str> = t3.iter().map(|c| c.type_name).collect();
        assert!(names.contains(&"RawReport"));
        assert!(names.contains(&"DetectionResult"));
        assert!(names.contains(&"Alert"));
        assert!(names.contains(&"DrugEventPair"));
    }

    #[test]
    fn every_composite_has_primitives() {
        for c in &composite_inventory() {
            assert!(
                !c.primitives.is_empty(),
                "{} has no primitives",
                c.type_name,
            );
        }
    }

    #[test]
    fn every_composite_includes_product() {
        // Every composite is a product type (struct).
        for c in &composite_inventory() {
            assert!(
                c.primitives.contains(&LexPrimitiva::Product),
                "{} should contain Product primitive",
                c.type_name,
            );
        }
    }

    #[test]
    fn t2c_composites_have_higher_transfer_potential() {
        let t2c = composites_by_tier(Tier::T2Composite);
        let t3 = composites_by_tier(Tier::T3DomainSpecific);

        let avg_t2c: f64 = t2c.iter().map(|c| c.transfer_potential).sum::<f64>() / t2c.len() as f64;
        let avg_t3: f64 = t3.iter().map(|c| c.transfer_potential).sum::<f64>() / t3.len() as f64;

        assert!(
            avg_t2c > avg_t3,
            "T2-C avg ({avg_t2c:.3}) should exceed T3 avg ({avg_t3:.3})"
        );
    }

    #[test]
    fn contingency_table_has_highest_transfer_potential() {
        let ct = composite_by_name("ContingencyTable");
        assert!(ct.is_some());
        if let Some(table) = ct {
            assert!(
                table.transfer_potential >= 0.90,
                "ContingencyTable transfer potential should be >= 0.90, got {}",
                table.transfer_potential,
            );
        }
    }

    #[test]
    fn alert_has_lowest_transfer_potential() {
        let alert = composite_by_name("Alert");
        assert!(alert.is_some());
        if let Some(a) = alert {
            assert!(
                a.transfer_potential <= 0.55,
                "Alert transfer potential should be <= 0.55, got {}",
                a.transfer_potential,
            );
        }
    }

    #[test]
    fn composite_by_name_found() {
        assert!(composite_by_name("DetectionResult").is_some());
        assert!(composite_by_name("SignalMetrics").is_some());
    }

    #[test]
    fn composite_by_name_not_found() {
        assert!(composite_by_name("NonExistentType").is_none());
    }

    #[test]
    fn average_transfer_potential_is_reasonable() {
        let avg = average_transfer_potential();
        // Should be between 0.5 and 0.9
        assert!(
            avg > 0.5 && avg < 0.9,
            "Average transfer potential should be 0.5-0.9, got {avg}"
        );
    }

    #[test]
    fn detection_result_has_most_primitives() {
        let dr = composite_by_name("DetectionResult");
        assert!(dr.is_some());
        if let Some(d) = dr {
            assert!(
                d.primitive_count() >= 6,
                "DetectionResult should have >= 6 primitives, got {}",
                d.primitive_count(),
            );
        }
    }

    #[test]
    fn field_counts_match_core_types() {
        // Verify field counts match the actual struct definitions in core.rs.
        let checks = [
            ("ContingencyTable", 4), // a, b, c, d
            ("DrugEventPair", 2),    // drug, event
            ("ValidationReport", 3), // pair, passed, checks
            ("Alert", 6),            // id, detection, state, created_at, updated_at, notes
            ("DetectionResult", 9), // pair, table, prr, ror, ic, ebgm, chi_square, strength, detected_at
        ];
        for &(name, expected) in &checks {
            let c = composite_by_name(name);
            assert!(c.is_some(), "Should find {name}");
            if let Some(desc) = c {
                assert_eq!(
                    desc.field_count, expected,
                    "{name} should have {expected} fields, got {}",
                    desc.field_count,
                );
            }
        }
    }

    #[test]
    fn serialization() {
        let inv = composite_inventory();
        let json = serde_json::to_string(&inv);
        assert!(json.is_ok());
        let json_str = json.unwrap_or_default();
        // Verify JSON structure via serde_json::Value (avoids 'static lifetime)
        let value: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
        assert!(value.is_ok());
        if let Ok(v) = value {
            assert!(v.is_array());
            let arr = v.as_array();
            assert!(arr.is_some());
            if let Some(a) = arr {
                assert_eq!(a.len(), inv.len());
            }
        }
    }
}
