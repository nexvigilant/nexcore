//! # T1 Primitive Inventory for pv-core
//!
//! Documents the 12 operational T1 primitives manifested in this crate,
//! making nexcore-pv-core the richest primitive coverage in the workspace.
//!
//! ## Why 12 of 15?
//!
//! Pharmacovigilance is a uniquely cross-cutting discipline: it spans molecular
//! biology (drug binding), clinical medicine (patient safety), epidemiology
//! (population signals), regulatory science (compliance boundaries), and
//! information science (case report databases). This breadth means PV naturally
//! exercises nearly every computational primitive:
//!
//! | Primitive | Symbol | PV Manifestation |
//! |-----------|--------|------------------|
//! | Existence | ∃ | Signal exists? Drug-event pair instantiation |
//! | Causality | → | Drug caused adverse event? Naranjo, WHO-UMC |
//! | Sequence | σ | Temporal onset, PV cycle ordering, E2B workflow |
//! | Comparison | κ | Threshold comparison (PRR>=2.0, ROR-LCI>1.0) |
//! | State | ς | Case lifecycle, REMS/RMP status, signal state |
//! | Boundary | ∂ | Safety margins, regulatory limits, ToV axioms |
//! | Quantity | N | Case counts, disproportionality ratios, doses |
//! | Sum | Σ | Signal classification (detected/noise), drug role enum |
//! | Recursion | ρ | Recursive signal deepening, conservation law validation |
//! | Mapping | μ | MedDRA coding, domain transfer, causality assessment mapping |
//! | Persistence | π | ICSR storage, periodic reporting, audit trails |
//! | Frequency | ν | Reporting rates, PRR numerator, periodic assessment schedules |
//!
//! The 3 absent primitives (+ Product which is axiomatic, not operational):
//! - **Void (∅)**: Rarely central — PV always works with present data
//! - **Location (λ)**: Geographic data exists but is not structurally central
//! - **Irreversibility (∝)**: Drug withdrawal is one-way, but not a primary structural concern
//! - **Product (×)**: Axiomatic — structurally present in every struct, excluded from operational tracking
//!
//! ## Tier Distribution
//!
//! ```text
//! T1 (Universal): 12 primitives manifested
//! T2-P (Cross-domain Primitive): CaseId, DrugRole, Route, ReactionOutcome
//! T2-C (Cross-domain Composite): CompleteSignalResult, DetectionResult, AssessmentResult,
//!                                  PvCycleResult, LandscapeAnalysis, CollectionResult
//! T3 (Domain-specific): Icsr, RiskManagementProgram, PeriodicReport, CSPGrid
//! ```

use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use serde::Serialize;

// ═══════════════════════════════════════════════════════════════════════════
// PRIMITIVE MANIFEST
// ═══════════════════════════════════════════════════════════════════════════

/// A single primitive manifestation in pv-core.
///
/// Serialize-only: these are static inventory records constructed in code,
/// not deserialized from external input.
#[derive(Debug, Clone, Serialize)]
pub struct PrimitiveManifest {
    /// The T1 primitive
    pub primitive: LexPrimitiva,
    /// How this primitive manifests in PV
    pub pv_manifestation: &'static str,
    /// Key types in pv-core that exercise this primitive
    pub exemplar_types: &'static [&'static str],
    /// Key modules where this primitive is structurally central
    pub exemplar_modules: &'static [&'static str],
}

/// The complete primitive inventory for nexcore-pv-core.
///
/// Documents all 12 operational T1 primitives with their PV manifestations,
/// exemplar types, and key modules.
///
/// Serialize-only: constructed via `crate_primitive_manifest()`, never deserialized.
#[derive(Debug, Clone, Serialize)]
pub struct CratePrimitiveManifest {
    /// Crate name
    pub crate_name: &'static str,
    /// Total T1 primitives manifested
    pub primitive_count: usize,
    /// Total operational T1 primitives (15 in Lex Primitiva; Product is axiomatic)
    pub total_possible: usize,
    /// Coverage ratio [0.0, 1.0]
    pub coverage: f64,
    /// Individual primitive manifestations
    pub manifests: Vec<PrimitiveManifest>,
    /// Absent primitives with explanations
    pub absent: Vec<AbsentPrimitive>,
}

/// A T1 primitive absent from this crate, with rationale.
///
/// Serialize-only: static inventory record.
#[derive(Debug, Clone, Serialize)]
pub struct AbsentPrimitive {
    /// The absent T1 primitive
    pub primitive: LexPrimitiva,
    /// Why this primitive is not structurally central
    pub rationale: &'static str,
}

// ═══════════════════════════════════════════════════════════════════════════
// STATIC INVENTORY
// ═══════════════════════════════════════════════════════════════════════════

/// Returns the complete T1 primitive manifest for pv-core.
///
/// This is the richest primitive coverage of any crate in the nexcore workspace
/// (12 of 15 operational primitives).
#[must_use]
pub fn crate_primitive_manifest() -> CratePrimitiveManifest {
    let manifests = vec![
        PrimitiveManifest {
            primitive: LexPrimitiva::Existence,
            pv_manifestation: "Signal existence detection — does a drug-event pair exceed noise?",
            exemplar_types: &["DetectionResult", "SignalResult", "CaseId"],
            exemplar_modules: &["signals", "definition", "icsr"],
        },
        PrimitiveManifest {
            primitive: LexPrimitiva::Causality,
            pv_manifestation: "Drug-event causality assessment — Naranjo, WHO-UMC, RUCAM scores",
            exemplar_types: &["CausalityAssessment", "AssessmentResult", "CausalityLevel"],
            exemplar_modules: &["causality", "definition", "icsr"],
        },
        PrimitiveManifest {
            primitive: LexPrimitiva::Sequence,
            pv_manifestation: "Temporal ordering — onset sequence, PV cycle stages, E2B workflow",
            exemplar_types: &["PvCycleResult", "CollectionResult", "TemporalWindow"],
            exemplar_modules: &["definition", "temporal", "minesweeper"],
        },
        PrimitiveManifest {
            primitive: LexPrimitiva::Comparison,
            pv_manifestation: "Threshold comparison — PRR>=2.0, ROR-LCI>1.0, IC025>0, EB05>=2.0",
            exemplar_types: &["SignalCriteria", "ThresholdRegistry", "SafetyMargin"],
            exemplar_modules: &["thresholds", "signals"],
        },
        PrimitiveManifest {
            primitive: LexPrimitiva::State,
            pv_manifestation: "Lifecycle state — case status, REMS/RMP state, signal investigation state",
            exemplar_types: &["RiskManagementProgram", "CellStatus", "BeliefState"],
            exemplar_modules: &["risk_management", "minesweeper", "icsr"],
        },
        PrimitiveManifest {
            primitive: LexPrimitiva::Boundary,
            pv_manifestation: "Safety boundaries — ToV axioms, regulatory limits, d(s) safety margin",
            exemplar_types: &["SafetyMargin", "RegulatoryViolation", "ComplianceReport"],
            exemplar_modules: &["regulatory", "comppv", "ivf"],
        },
        PrimitiveManifest {
            primitive: LexPrimitiva::Quantity,
            pv_manifestation: "Numerical magnitude — case counts, PRR/ROR ratios, AUC, doses",
            exemplar_types: &["ContingencyTable", "CompleteSignalResult", "Dosage"],
            exemplar_modules: &["types", "signals", "pk"],
        },
        PrimitiveManifest {
            primitive: LexPrimitiva::Sum,
            pv_manifestation: "Exclusive classification — signal detected/noise, drug role variants",
            exemplar_types: &["DrugRole", "ReactionOutcome", "SeverityLevel"],
            exemplar_modules: &["icsr", "classification"],
        },
        PrimitiveManifest {
            primitive: LexPrimitiva::Recursion,
            pv_manifestation: "Recursive deepening — conservation law validation, signal refinement",
            exemplar_types: &["UnderstandingResult", "ConservationLaw"],
            exemplar_modules: &["definition", "comppv"],
        },
        PrimitiveManifest {
            primitive: LexPrimitiva::Mapping,
            pv_manifestation: "Domain transformation — MedDRA coding, causality method mapping, transfer",
            exemplar_types: &["CausalityMethod", "IchCategory", "LandscapeAnalysis"],
            exemplar_modules: &["coding", "regulatory", "landscape"],
        },
        PrimitiveManifest {
            primitive: LexPrimitiva::Persistence,
            pv_manifestation: "Data continuity — ICSR storage, periodic reporting, regulatory audit trails",
            exemplar_types: &["PeriodicReport", "Icsr", "RiskManagementProgram"],
            exemplar_modules: &["periodic_reporting", "icsr", "risk_management"],
        },
        PrimitiveManifest {
            primitive: LexPrimitiva::Frequency,
            pv_manifestation: "Rate of occurrence — reporting rates, PRR numerator, assessment schedules",
            exemplar_types: &["SignalResult", "ThresholdRegistry"],
            exemplar_modules: &["signals", "thresholds", "periodic_reporting"],
        },
    ];

    let absent = vec![
        AbsentPrimitive {
            primitive: LexPrimitiva::Void,
            rationale: "PV operates on present data — absence is handled via Option<T> but \
                        is not structurally central to any PV algorithm",
        },
        AbsentPrimitive {
            primitive: LexPrimitiva::Location,
            rationale: "Geographic data exists in ICSR reports (country of occurrence) but \
                        is metadata, not a structural primitive in signal detection or causality",
        },
        AbsentPrimitive {
            primitive: LexPrimitiva::Irreversibility,
            rationale: "Drug withdrawal and market removal are one-way transitions, but the \
                        irreversibility primitive is not a primary driver of PV computation — \
                        it manifests as a consequence of Boundary and State transitions",
        },
        AbsentPrimitive {
            primitive: LexPrimitiva::Product,
            rationale: "Axiomatic — Product (×) is structurally present in every Rust struct \
                        as field conjunction, but excluded from operational tracking per Lex \
                        Primitiva convention (trivially everywhere, not a distinguishing primitive)",
        },
    ];

    let count = manifests.len();

    CratePrimitiveManifest {
        crate_name: "nexcore-pv-core",
        primitive_count: count,
        total_possible: 15,
        coverage: count as f64 / 15.0,
        manifests,
        absent,
    }
}

/// Returns the primitives exercised by a given module name.
///
/// # Arguments
///
/// * `module_name` — Module name (e.g., "signals", "icsr", "causality")
#[must_use]
pub fn primitives_for_module(module_name: &str) -> Vec<LexPrimitiva> {
    let manifest = crate_primitive_manifest();
    manifest
        .manifests
        .iter()
        .filter(|m| m.exemplar_modules.contains(&module_name))
        .map(|m| m.primitive)
        .collect()
}

/// Returns the modules that exercise a given T1 primitive.
///
/// # Arguments
///
/// * `primitive` — The T1 primitive to look up
#[must_use]
pub fn modules_for_primitive(primitive: LexPrimitiva) -> Vec<&'static str> {
    let manifest = crate_primitive_manifest();
    manifest
        .manifests
        .iter()
        .find(|m| m.primitive == primitive)
        .map(|m| m.exemplar_modules.to_vec())
        .unwrap_or_default()
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_has_12_primitives() {
        let manifest = crate_primitive_manifest();
        assert_eq!(manifest.primitive_count, 12);
        assert_eq!(manifest.manifests.len(), 12);
    }

    #[test]
    fn coverage_is_correct() {
        let manifest = crate_primitive_manifest();
        let expected = 12.0 / 15.0;
        assert!((manifest.coverage - expected).abs() < 1e-10);
    }

    #[test]
    fn has_four_absent_primitives() {
        let manifest = crate_primitive_manifest();
        assert_eq!(manifest.absent.len(), 4);
    }

    #[test]
    fn absent_primitives_are_void_location_irreversibility_product() {
        let manifest = crate_primitive_manifest();
        let absent_prims: Vec<LexPrimitiva> = manifest
            .absent
            .iter()
            .map(|a| a.primitive)
            .collect();
        assert!(absent_prims.contains(&LexPrimitiva::Void));
        assert!(absent_prims.contains(&LexPrimitiva::Location));
        assert!(absent_prims.contains(&LexPrimitiva::Irreversibility));
        assert!(absent_prims.contains(&LexPrimitiva::Product));
    }

    #[test]
    fn no_overlap_between_present_and_absent() {
        let manifest = crate_primitive_manifest();
        let present: Vec<LexPrimitiva> = manifest
            .manifests
            .iter()
            .map(|m| m.primitive)
            .collect();
        for absent in &manifest.absent {
            assert!(
                !present.contains(&absent.primitive),
                "Primitive {:?} is both present and absent",
                absent.primitive
            );
        }
    }

    #[test]
    fn all_manifests_have_exemplar_types() {
        let manifest = crate_primitive_manifest();
        for m in &manifest.manifests {
            assert!(
                !m.exemplar_types.is_empty(),
                "Primitive {:?} has no exemplar types",
                m.primitive
            );
        }
    }

    #[test]
    fn all_manifests_have_exemplar_modules() {
        let manifest = crate_primitive_manifest();
        for m in &manifest.manifests {
            assert!(
                !m.exemplar_modules.is_empty(),
                "Primitive {:?} has no exemplar modules",
                m.primitive
            );
        }
    }

    #[test]
    fn root_primitives_present() {
        let manifest = crate_primitive_manifest();
        let prims: Vec<LexPrimitiva> = manifest
            .manifests
            .iter()
            .map(|m| m.primitive)
            .collect();
        // The two root primitives (N + →) must be present
        assert!(prims.contains(&LexPrimitiva::Quantity), "Root primitive N missing");
        assert!(prims.contains(&LexPrimitiva::Causality), "Root primitive → missing");
    }

    #[test]
    fn signals_module_exercises_key_primitives() {
        let prims = primitives_for_module("signals");
        assert!(prims.contains(&LexPrimitiva::Existence));
        assert!(prims.contains(&LexPrimitiva::Comparison));
        assert!(prims.contains(&LexPrimitiva::Quantity));
    }

    #[test]
    fn causality_primitive_maps_to_expected_modules() {
        let modules = modules_for_primitive(LexPrimitiva::Causality);
        assert!(modules.contains(&"causality"));
        assert!(modules.contains(&"definition"));
    }

    #[test]
    fn unknown_module_returns_empty() {
        let prims = primitives_for_module("nonexistent_module");
        assert!(prims.is_empty());
    }

    #[test]
    fn crate_name_is_correct() {
        let manifest = crate_primitive_manifest();
        assert_eq!(manifest.crate_name, "nexcore-pv-core");
    }
}
