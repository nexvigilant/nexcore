//! # NexVigilant Core — lex-primitiva
//!
//! The 16 irreducible Lex Primitiva symbols with mathematical constants grounding.
//!
//! ## Overview
//!
//! This crate provides the foundational primitive system for nexcore:
//!
//! - **16 Lex Primitiva** (σ, μ, ς, ρ, ∅, ∂, f, ∃, π, →, κ, N, λ, ∝, Σ, ×)
//! - **80 Bedrock Atoms** (5 per primitive)
//! - **10 Mathematical Foundations** (Peano, Category Theory, etc.)
//! - **10 Terminal Constants** (0, 1, π, e, φ, ∞, ω, ln(2), kᵦ, ε)
//!
//! ## The Two Root Constants
//!
//! All computation ultimately grounds to:
//! - **ZERO (0)**: Absence, identity, origin
//! - **ONE (1)**: Existence, unit, witness
//!
//! ## Example
//!
//! ```rust
//! use nexcore_lex_primitiva::prelude::*;
//!
//! // Get the symbol for a primitive
//! assert_eq!(LexPrimitiva::Sequence.symbol(), "σ");
//!
//! // Check if a primitive is a root
//! assert!(LexPrimitiva::Quantity.is_root());
//! assert!(LexPrimitiva::Causality.is_root());
//!
//! // Get the primary constant
//! let constant = LexPrimitiva::Frequency.primary_constant();
//! assert_eq!(constant.symbol, "π");
//!
//! // Trace primitives through the dependency graph
//! let traces = DependencyGraph::trace(LexPrimitiva::Void);
//! for trace in traces {
//!     println!("{}", trace);
//! }
//! ```
//!
//! ## Tier System
//!
//! | Tier | Description | Transfer Multiplier |
//! |------|-------------|---------------------|
//! | T1 | Universal primitive | 1.0 |
//! | T2-P | Cross-domain primitive | 0.9 |
//! | T2-C | Cross-domain composite | 0.7 |
//! | T3 | Domain-specific | 0.4 |

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![warn(missing_docs)]

pub mod bedrock;
pub mod cli;
pub mod complexity;
pub mod composition;
pub mod composition_proofs;
pub mod compound;
pub mod compound_detector;
pub mod constants;
#[cfg(test)]
mod core_true_proofs;
pub mod dossier;
pub mod external_grounding;
pub mod extraction;
pub mod grammar;
pub mod graph;
pub mod grounding;
pub mod molecular_weight;
pub mod primitiva;
pub mod semantic_path;
// spatial_bridge moved to stem-math (avoids circular dep for crates.io publish)
pub mod state_mode;
pub mod symbols;
pub mod synthesizer;
pub mod tier;
pub mod transfer;
pub mod validate;
pub mod vocabulary;
pub mod weighted;

/// Prelude for convenient imports.
pub mod prelude;

// Re-export main types at crate root
pub use bedrock::BedrockAtom;
pub use composition::{CompositionAlgebra, CompositionBuilder, CompositionScore};
pub use compound::{BasisSnapshot, CompoundTracker};
pub use compound_detector::{CompoundDetector, DetectionResult};
pub use constants::MathConstant;
pub use dossier::{Dossier, DossierGenerator};
pub use extraction::{ExtractionResult, PrimitiveExtractor};
pub use grammar::{InteractionGraph, InteractionType, PatternRegistry};
pub use graph::DependencyGraph;
pub use grounding::GroundsTo;
pub use molecular_weight::{
    AtomicMass, MolecularFormula, MolecularWeight, TierPrediction, TransferClass,
};
pub use primitiva::{LexPrimitiva, PrimitiveComposition};
pub use state_mode::StateMode;
pub use synthesizer::RevSynthesizer;
pub use tier::Tier;
pub use transfer::{Domain, TransferCalculator};
pub use validate::{PrimitivaValidator, ValidationReport};
pub use weighted::WeightedComposition;

/// Backward-compatible alias for `Tier`.
///
/// `nexcore-vigilance` originally defined `GroundingTier` inline.
/// After extraction to this standalone crate, this alias preserves
/// all existing `GroundingTier::classify()` and variant usage.
pub type GroundingTier = Tier;

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use std::collections::{HashMap, HashSet};

    // ═══════════════════════════════════════════════════════════════════════════
    // BASIC STRUCTURAL TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_prelude_imports() {
        let _ = (
            LexPrimitiva::Sequence,
            BedrockAtom::Count,
            MathConstant::ZERO,
        );
        let _ = (Tier::T1Universal, MathFoundation::PeanoAxioms);
    }

    #[test]
    fn test_16_primitives() {
        assert_eq!(LexPrimitiva::all().len(), 16);
    }

    #[test]
    fn test_80_bedrock_atoms() {
        let count: usize = LexPrimitiva::all()
            .iter()
            .map(|p| BedrockAtom::for_primitive(*p).len())
            .sum();
        assert_eq!(count, 80);
    }

    #[test]
    fn test_10_foundations() {
        assert_eq!(MathFoundation::all().len(), 10);
    }

    #[test]
    fn test_10_constants() {
        assert_eq!(MathConstant::all().len(), 10);
    }

    #[test]
    fn test_root_primitives() {
        let roots = LexPrimitiva::roots();
        assert_eq!(roots.len(), 2);
        assert!(roots.contains(&LexPrimitiva::Quantity));
        assert!(roots.contains(&LexPrimitiva::Causality));
    }

    #[test]
    fn test_grounding_chain() {
        let traces = DependencyGraph::trace(LexPrimitiva::Void);
        let has_zero = traces.iter().any(|t| t.constant.symbol == "0");
        assert!(has_zero);
    }

    #[test]
    fn test_tier_classification() {
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CTVP GAP #1: CYCLE DETECTION (Extracted to module for clarity)
    // ═══════════════════════════════════════════════════════════════════════════

    mod cycle_detection {
        use super::*;

        /// DFS cycle detector - returns true if cycle found
        fn detect_cycle(
            node: LexPrimitiva,
            visited: &mut HashSet<LexPrimitiva>,
            stack: &mut HashSet<LexPrimitiva>,
        ) -> bool {
            if stack.contains(&node) {
                return true;
            }
            if visited.contains(&node) {
                return false;
            }
            visited.insert(node);
            stack.insert(node);
            let has_cycle = node
                .derives_from()
                .into_iter()
                .any(|dep| detect_cycle(dep, visited, stack));
            stack.remove(&node);
            has_cycle
        }

        #[test]
        fn test_derives_from_is_dag() {
            for start in LexPrimitiva::all() {
                let mut visited = HashSet::new();
                let mut stack = HashSet::new();
                assert!(
                    !detect_cycle(start, &mut visited, &mut stack),
                    "Cycle from {:?}",
                    start
                );
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CTVP GAP #1b: REACHABILITY FROM ROOTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_all_primitives_reachable_from_roots() {
        let all: HashSet<_> = LexPrimitiva::all().into_iter().collect();
        let mut derived_by: HashMap<LexPrimitiva, Vec<LexPrimitiva>> = HashMap::new();

        for p in LexPrimitiva::all() {
            for dep in p.derives_from() {
                derived_by.entry(dep).or_default().push(p);
            }
        }

        let mut reached: HashSet<_> = LexPrimitiva::roots().into_iter().collect();
        let mut frontier: Vec<_> = reached.iter().copied().collect();

        while let Some(node) = frontier.pop() {
            let dependents = derived_by.get(&node).map(|v| v.as_slice()).unwrap_or(&[]);
            for &dep in dependents {
                if reached.insert(dep) {
                    frontier.push(dep);
                }
            }
        }

        let unreached: Vec<_> = all.difference(&reached).collect();
        assert!(
            unreached.is_empty(),
            "Unreachable from roots: {:?}",
            unreached
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CTVP GAP #2: REACHABILITY INVARIANT
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_all_primitives_reach_constants() {
        for p in LexPrimitiva::all() {
            let constants = DependencyGraph::constants_for_primitive(p);
            assert!(!constants.is_empty(), "{:?} reaches no constants", p);
        }
    }

    #[test]
    fn test_all_primitives_reach_root_constants() {
        for p in LexPrimitiva::all() {
            let constants = DependencyGraph::constants_for_primitive(p);
            let has_root = constants.contains("0") || constants.contains("1");
            assert!(
                has_root,
                "{:?} misses root constants. Got: {:?}",
                p, constants
            );
        }
    }

    #[test]
    fn test_trace_exactly_5_per_primitive() {
        for p in LexPrimitiva::all() {
            assert_eq!(DependencyGraph::trace(p).len(), 5, "{:?} trace != 5", p);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CTVP GAP #3: BIDIRECTIONAL CONSISTENCY
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_atom_primitive_bidirectional() {
        let mut count = 0;
        for p in LexPrimitiva::all() {
            for atom in BedrockAtom::for_primitive(p) {
                count += 1;
                assert_eq!(atom.parent_primitive(), p, "{:?} parent mismatch", atom);
            }
        }
        assert_eq!(count, 80);
    }

    #[test]
    fn test_no_duplicate_atoms() {
        let mut seen = HashSet::new();
        for p in LexPrimitiva::all() {
            for atom in BedrockAtom::for_primitive(p) {
                assert!(seen.insert(*atom), "{:?} appears twice", atom);
            }
        }
        assert_eq!(seen.len(), 80);
    }

    #[test]
    fn test_atoms_map_to_known_constants() {
        let known: HashSet<_> = MathConstant::all().iter().map(|c| c.symbol).collect();
        for p in LexPrimitiva::all() {
            for atom in BedrockAtom::for_primitive(p) {
                let sym = atom.primary_constant().symbol;
                assert!(known.contains(sym), "{:?} → unknown {:?}", atom, sym);
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CTVP GAP #4: FOUNDATION ARRAY CONSISTENCY
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_foundation_arrays_length_5() {
        use crate::graph::foundations_for_primitive;
        for p in LexPrimitiva::all() {
            assert_eq!(
                foundations_for_primitive(p).len(),
                5,
                "{:?} foundations != 5",
                p
            );
        }
    }

    #[test]
    fn test_foundations_in_known_set() {
        use crate::graph::foundations_for_primitive;
        let known: HashSet<_> = MathFoundation::all().into_iter().collect();
        for p in LexPrimitiva::all() {
            for f in foundations_for_primitive(p) {
                assert!(known.contains(f), "Unknown foundation {:?}", f);
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CTVP GAP #5: TIER EDGE CASES
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_tier_empty_composition() {
        let empty = PrimitiveComposition::new(vec![]);
        assert_eq!(Tier::classify(&empty), Tier::T1Universal);
    }

    #[test]
    fn test_tier_all_16_primitives() {
        let all = PrimitiveComposition::new(LexPrimitiva::all().to_vec());
        assert_eq!(Tier::classify(&all), Tier::T3DomainSpecific);
    }

    #[test]
    fn test_tier_boundaries() {
        let cases = [
            (1, Tier::T1Universal),
            (2, Tier::T2Primitive),
            (3, Tier::T2Primitive),
            (4, Tier::T2Composite),
            (5, Tier::T2Composite),
            (6, Tier::T3DomainSpecific),
        ];
        let all_prims = LexPrimitiva::all();
        for (count, expected) in cases {
            let comp = PrimitiveComposition::new(all_prims[..count].to_vec());
            assert_eq!(Tier::classify(&comp), expected, "count={}", count);
        }
    }

    #[test]
    fn test_tier_deduplicates() {
        let dupes = PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Quantity,
            LexPrimitiva::Quantity,
        ]);
        assert_eq!(Tier::classify(&dupes), Tier::T1Universal);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MATHEMATICAL INVARIANTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_transfer_multipliers_monotonic() {
        let tiers = Tier::all();
        for window in tiers.windows(2) {
            assert!(window[0].transfer_multiplier() >= window[1].transfer_multiplier());
        }
    }

    #[test]
    fn test_constants_finite_or_special() {
        for c in MathConstant::all() {
            let valid = match c.value {
                Some(v) => v.is_finite(),
                None => c.symbol == "∞" || c.symbol == "ω", // Infinite constants have None
            };
            assert!(valid, "{:?} invalid value: {:?}", c.symbol, c.value);
        }
    }

    #[test]
    fn test_primary_constants_cover_roots() {
        let seen: HashSet<_> = LexPrimitiva::all()
            .iter()
            .map(|p| p.primary_constant().symbol)
            .collect();
        assert!(seen.contains("0") && seen.contains("1"));
    }

    #[test]
    fn test_symbol_uniqueness() {
        let symbols: HashSet<_> = LexPrimitiva::all().iter().map(|p| p.symbol()).collect();
        assert_eq!(symbols.len(), 16);
    }

    #[test]
    fn test_constant_symbol_uniqueness() {
        let symbols: HashSet<_> = MathConstant::all().iter().map(|c| c.symbol).collect();
        assert_eq!(symbols.len(), 10);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ADDITIONAL PRESSURE TESTS: SERDE, CONFIDENCE, DISPLAY, ROUND-TRIPS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_confidence_clamping_out_of_range() {
        // Test with_dominant clamps confidence to [0.0, 1.0]
        let over = PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 5.0);
        assert!(
            (over.confidence - 1.0).abs() < f64::EPSILON,
            "Should clamp to 1.0"
        );

        let under = PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, -3.0);
        assert!(
            (under.confidence - 0.0).abs() < f64::EPSILON,
            "Should clamp to 0.0"
        );

        let normal = PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 0.75);
        assert!((normal.confidence - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_is_finite_matches_value() {
        for c in MathConstant::all() {
            let has_value = c.value.is_some();
            assert_eq!(c.is_finite, has_value, "{} is_finite mismatch", c.symbol);
        }
    }

    #[test]
    fn test_symbol_round_trip() {
        for p in LexPrimitiva::all() {
            let symbol = p.symbol();
            let parsed = LexPrimitiva::from_symbol(symbol);
            assert_eq!(parsed, Some(p), "Round-trip failed for {}", symbol);
        }
    }

    #[test]
    fn test_serde_lex_primitiva() {
        for p in LexPrimitiva::all() {
            let json = serde_json::to_string(&p).expect("serialize");
            let back: LexPrimitiva = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, p, "Serde round-trip failed for {:?}", p);
        }
    }

    #[test]
    fn test_serde_bedrock_atom() {
        for p in LexPrimitiva::all() {
            for atom in BedrockAtom::for_primitive(p) {
                let json = serde_json::to_string(atom).expect("serialize");
                let back: BedrockAtom = serde_json::from_str(&json).expect("deserialize");
                assert_eq!(back, *atom, "Serde failed for {:?}", atom);
            }
        }
    }

    #[test]
    fn test_serde_tier() {
        for t in Tier::all() {
            let json = serde_json::to_string(&t).expect("serialize");
            let back: Tier = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, t);
        }
    }

    #[test]
    fn test_serde_trichotomy() {
        use crate::constants::Trichotomy;
        for t in [Trichotomy::Less, Trichotomy::Equal, Trichotomy::Greater] {
            let json = serde_json::to_string(&t).expect("serialize");
            let back: Trichotomy = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, t);
        }
    }

    #[test]
    fn test_display_all_types() {
        // LexPrimitiva Display
        for p in LexPrimitiva::all() {
            let s = format!("{}", p);
            assert!(!s.is_empty());
            assert!(s.contains(p.symbol()));
        }

        // BedrockAtom Display
        for p in LexPrimitiva::all() {
            for atom in BedrockAtom::for_primitive(p) {
                let s = format!("{}", atom);
                assert!(!s.is_empty());
            }
        }

        // Tier Display
        for t in Tier::all() {
            let s = format!("{}", t);
            assert!(s.starts_with("T"));
        }

        // PrimitiveComposition Display
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Mapping]);
        let s = format!("{}", comp);
        assert!(s.contains("σ") && s.contains("μ"));

        // MathConstant Display
        for c in MathConstant::all() {
            let s = format!("{}", c);
            assert!(s.contains(c.symbol));
        }
    }

    #[test]
    fn test_grounding_trace_display() {
        let traces = DependencyGraph::trace(LexPrimitiva::Quantity);
        for trace in traces {
            let s = format!("{}", trace);
            assert!(s.contains("→"), "Trace display should contain arrows");
        }
    }

    #[test]
    fn test_all_numeric_types_ground_to_quantity() {
        use crate::grounding::GroundsTo;

        // All numeric types should have Quantity as dominant
        assert_eq!(u8::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(u16::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(u32::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(u64::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(u128::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(usize::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(i8::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(i16::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(i32::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(i64::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(i128::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(isize::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(f32::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(f64::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(char::dominant_primitive(), Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn test_all_numeric_types_are_pure() {
        use crate::grounding::GroundsTo;
        assert!(u8::is_pure_primitive());
        assert!(i64::is_pure_primitive());
        assert!(f64::is_pure_primitive());
    }

    #[test]
    fn test_container_types_not_pure() {
        use crate::grounding::GroundsTo;
        assert!(!<Vec<i32>>::is_pure_primitive()); // Sequence + State + Quantity
        assert!(!<Option<i32>>::is_pure_primitive()); // Void + Sum
        assert!(!String::is_pure_primitive()); // Sequence + State
    }

    #[test]
    fn test_invalid_symbol_parsing() {
        assert_eq!(LexPrimitiva::from_symbol(""), None);
        assert_eq!(LexPrimitiva::from_symbol("xyz"), None);
        assert_eq!(LexPrimitiva::from_symbol("🔥"), None);
        assert_eq!(LexPrimitiva::from_symbol("SEQUENCE"), None); // case sensitive
    }

    #[test]
    fn test_invalid_tier_from_u8() {
        assert_eq!(Tier::from_u8(0), None);
        assert_eq!(Tier::from_u8(5), None);
        assert_eq!(Tier::from_u8(255), None);
    }

    #[test]
    fn test_trichotomy_invalid_i8() {
        use crate::constants::Trichotomy;
        assert_eq!(Trichotomy::from_i8(-2), None);
        assert_eq!(Trichotomy::from_i8(2), None);
        assert_eq!(Trichotomy::from_i8(127), None);
        assert_eq!(Trichotomy::from_i8(-128), None);
    }
}
