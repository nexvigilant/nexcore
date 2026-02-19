// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Lex Primitiva Grounding for STOS Types
//!
//! Implements `GroundsTo` for all 15 STOS layer engines plus kernel types.
//! Each grounding reflects both **structural** (data fields) and **functional**
//! (method behavior) primitive composition.
//!
//! ## Tier Summary
//!
//! Classification: 1=T1, 2-3=T2-P, 4-5=T2-C, 6+=T3
//!
//! | Type | Tier | Count | Primitives |
//! |------|------|-------|------------|
//! | StateRegistry | T2-P | 3 | ς, μ, N |
//! | TransitionEngine | T2-C | 5 | →, ς, κ, μ, σ |
//! | BoundaryManager | T2-P | 3 | ∂, ς, σ |
//! | GuardEvaluator | T2-P | 3 | κ, μ, → |
//! | CountMetrics | T2-P | 3 | N, ς, μ |
//! | SequenceController | T2-P | 3 | σ, →, N |
//! | RecursionDetector | T2-P | 3 | ρ, σ, μ |
//! | VoidCleaner | T2-P | 3 | ∅, ∃, ς |
//! | PersistStore | T2-P | 3 | π, ς, σ |
//! | ExistenceValidator | T2-P | 3 | ∃, ∅, ς |
//! | AggregateCoordinator | T2-P | 3 | Σ, ς, μ |
//! | TemporalScheduler | T2-P | 3 | ν, μ, σ |
//! | LocationRouter | T2-P | 3 | λ, μ, Σ |
//! | IrreversibilityAuditor | T2-P | 3 | ∝, σ, π |
//! | MappingTransformer | T2-C | 4 | μ, →, ς, σ |
//! | MachineSpec | T2-C | 4 | ς, →, σ, ∂ |
//! | MachineInstance | T2-C | 4 | ς, →, σ, ∂ |
//! | StateKernel | T3 | 7 | ς, →, ∂, π, Σ, N, μ |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::stos::{
    aggregate_coordinator::AggregateCoordinator, boundary_manager::BoundaryManager,
    count_metrics::CountMetrics, existence_validator::ExistenceValidator,
    guard_evaluator::GuardEvaluator, irreversibility_auditor::IrreversibilityAuditor,
    location_router::LocationRouter, mapping_transformer::MappingTransformer,
    persist_store::PersistStore, recursion_detector::RecursionDetector,
    sequence_controller::SequenceController, state_registry::StateRegistry,
    temporal_scheduler::TemporalScheduler, transition_engine::TransitionEngine,
    void_cleaner::VoidCleaner,
};

use crate::kernel::StateKernel;
use crate::machine::{MachineInstance, MachineSpec};

// ═══════════════════════════════════════════════════════════
// LAYER 1: STATE REGISTRY (ς State)
// ═══════════════════════════════════════════════════════════

/// StateRegistry grounds to ς + μ + N
///
/// **Structural**: BTreeMap (μ), counters/limits (N), StateEntry (ς)
/// **Functional**: register (ς), lookup (μ), len/capacity (N)
impl GroundsTo for StateRegistry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // Core state entries
            LexPrimitiva::Mapping,  // BTreeMap lookups
            LexPrimitiva::Quantity, // Counters and limits
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// LAYER 2: TRANSITION ENGINE (→ Causality)
// ═══════════════════════════════════════════════════════════

/// TransitionEngine grounds to → + ς + κ + μ + σ
///
/// **Structural**: BTreeMaps (μ), history Vec (σ), TransitionSpec (→, ς)
/// **Functional**: execute (→), find (μ), source check (κ), history (σ)
impl GroundsTo for TransitionEngine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // Transition execution
            LexPrimitiva::State,      // State references
            LexPrimitiva::Comparison, // Source state validation
            LexPrimitiva::Mapping,    // Transition lookups
            LexPrimitiva::Sequence,   // Execution history
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

// ═══════════════════════════════════════════════════════════
// LAYER 3: BOUNDARY MANAGER (∂ Boundary)
// ═══════════════════════════════════════════════════════════

/// BoundaryManager grounds to ∂ + ς + σ
///
/// **Structural**: BTreeSet of initial/terminal/error states (∂, ς), crossings Vec (σ)
/// **Functional**: is_initial/is_terminal (∂), state tracking (ς), record_crossing (σ)
impl GroundsTo for BoundaryManager {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // Initial/terminal detection
            LexPrimitiva::State,    // State classification
            LexPrimitiva::Sequence, // Crossing history
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// LAYER 4: GUARD EVALUATOR (κ Comparison)
// ═══════════════════════════════════════════════════════════

/// GuardEvaluator grounds to κ + μ + →
///
/// **Structural**: BTreeMaps (μ), GuardSpec with conditions (κ)
/// **Functional**: evaluate (κ), variable lookup (μ), conditional logic (→)
impl GroundsTo for GuardEvaluator {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // Boolean expression evaluation
            LexPrimitiva::Mapping,    // Variable/guard lookups
            LexPrimitiva::Causality,  // If-then conditional logic
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// LAYER 5: COUNT METRICS (N Quantity)
// ═══════════════════════════════════════════════════════════

/// CountMetrics grounds to N + ς + μ
///
/// **Structural**: BTreeMaps of counts (μ, N), StateId keys (ς)
/// **Functional**: record (N), lookup (μ), state visits (ς)
impl GroundsTo for CountMetrics {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // All count operations
            LexPrimitiva::State,    // State-keyed metrics
            LexPrimitiva::Mapping,  // Count storage maps
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// LAYER 6: SEQUENCE CONTROLLER (σ Sequence)
// ═══════════════════════════════════════════════════════════

/// SequenceController grounds to σ + → + N
///
/// **Structural**: VecDeque queue (σ), counter/max_queue_size (N)
/// **Functional**: enqueue/dequeue (σ), transition ordering (→), priority (N)
impl GroundsTo for SequenceController {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // Queue operations
            LexPrimitiva::Causality, // Transition scheduling
            LexPrimitiva::Quantity,  // Counter, priority, limits
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// LAYER 7: RECURSION DETECTOR (ρ Recursion)
// ═══════════════════════════════════════════════════════════

/// RecursionDetector grounds to ρ + σ + μ
///
/// **Structural**: BTreeMap adjacency graph (μ), visit path Vec (σ)
/// **Functional**: DFS cycle detection (ρ), path tracking (σ), graph lookup (μ)
impl GroundsTo for RecursionDetector {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // Cycle detection via DFS
            LexPrimitiva::Sequence,  // Visit path tracking
            LexPrimitiva::Mapping,   // Adjacency graph storage
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// LAYER 8: VOID CLEANER (∅ Void)
// ═══════════════════════════════════════════════════════════

/// VoidCleaner grounds to ∅ + ∃ + ς
///
/// **Structural**: Unreachable state collections (∅), state sets (∃), StateId tracking (ς)
/// **Functional**: detect unreachable (∅), reachability check (∃), state classification (ς)
impl GroundsTo for VoidCleaner {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,      // Unreachable/dead state detection
            LexPrimitiva::Existence, // Reachability validation
            LexPrimitiva::State,     // State entity tracking
        ])
        .with_dominant(LexPrimitiva::Void, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// LAYER 9: PERSIST STORE (π Persistence)
// ═══════════════════════════════════════════════════════════

/// PersistStore grounds to π + ς + σ
///
/// **Structural**: Snapshot Vec (σ, π), state capture (ς)
/// **Functional**: snapshot/restore (π), state capture (ς), history (σ)
impl GroundsTo for PersistStore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // Snapshot storage
            LexPrimitiva::State,       // State capture
            LexPrimitiva::Sequence,    // Snapshot history
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// LAYER 10: EXISTENCE VALIDATOR (∃ Existence)
// ═══════════════════════════════════════════════════════════

/// ExistenceValidator grounds to ∃ + ∅ + ς
///
/// **Structural**: Known/deleted state sets (∃, ∅), StateId tracking (ς)
/// **Functional**: exists check (∃), deleted tracking (∅), state entity validation (ς)
impl GroundsTo for ExistenceValidator {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // Existence validation
            LexPrimitiva::Void,      // Deleted/non-existent tracking
            LexPrimitiva::State,     // State entity classification
        ])
        .with_dominant(LexPrimitiva::Existence, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// LAYER 11: AGGREGATE COORDINATOR (Σ Sum)
// ═══════════════════════════════════════════════════════════

/// AggregateCoordinator grounds to Σ + ς + μ
///
/// **Structural**: BTreeMaps of machines (μ), machine summaries (ς)
/// **Functional**: stats aggregation (Σ), state tracking (ς), lookup (μ)
impl GroundsTo for AggregateCoordinator {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,     // Multi-machine aggregation
            LexPrimitiva::State,   // Machine state summaries
            LexPrimitiva::Mapping, // Machine registry
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// LAYER 12: TEMPORAL SCHEDULER (ν Frequency)
// ═══════════════════════════════════════════════════════════

/// TemporalScheduler grounds to ν + μ + σ
///
/// **Structural**: BTreeMaps of schedules (μ), time tracking (ν)
/// **Functional**: schedule (ν, σ), lookup (μ), time advancement (ν)
impl GroundsTo for TemporalScheduler {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // Time-based scheduling
            LexPrimitiva::Mapping,   // Schedule storage
            LexPrimitiva::Sequence,  // Scheduled order
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// LAYER 13: LOCATION ROUTER (λ Location)
// ═══════════════════════════════════════════════════════════

/// LocationRouter grounds to λ + μ + Σ
///
/// **Structural**: Location BTreeMaps (μ, λ), machine assignments
/// **Functional**: route (λ), lookup (μ), multi-location (Σ)
impl GroundsTo for LocationRouter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // Location management
            LexPrimitiva::Mapping,  // Assignment maps
            LexPrimitiva::Sum,      // Multi-location aggregation
        ])
        .with_dominant(LexPrimitiva::Location, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// LAYER 14: IRREVERSIBILITY AUDITOR (∝ Irreversibility)
// ═══════════════════════════════════════════════════════════

/// IrreversibilityAuditor grounds to ∝ + σ + π
///
/// **Structural**: Audit trail Vec (σ, π), irreversibility maps (∝)
/// **Functional**: record (π, σ), verify integrity (∝)
impl GroundsTo for IrreversibilityAuditor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility, // Tamper detection
            LexPrimitiva::Sequence,        // Audit trail order
            LexPrimitiva::Persistence,     // Trail storage
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// LAYER 15: MAPPING TRANSFORMER (μ Mapping)
// ═══════════════════════════════════════════════════════════

/// MappingTransformer grounds to μ + → + ς + σ
///
/// **Structural**: All BTreeMaps (μ), StateMapping/state_to_name (ς), transition Vec (σ)
/// **Functional**: map transformations (μ), event→transition (→), state mapping (ς), ordered pairs (σ)
impl GroundsTo for MappingTransformer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // State/transition mappings
            LexPrimitiva::Causality, // Event-triggered transitions
            LexPrimitiva::State,     // State identity transformation
            LexPrimitiva::Sequence,  // Ordered transition pairs
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ═══════════════════════════════════════════════════════════
// KERNEL: STATE KERNEL (T3 Orchestrator)
// ═══════════════════════════════════════════════════════════

/// StateKernel grounds to ς + → + ∂ + π + Σ + N + μ (T3: 7 primitives)
///
/// **Structural**: BTreeMap machines (μ), config limits (N), subsystems
/// **Functional**: orchestrates all 15 layers
impl GroundsTo for StateKernel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,       // Core state management
            LexPrimitiva::Causality,   // Transition execution
            LexPrimitiva::Boundary,    // Initial/terminal handling
            LexPrimitiva::Persistence, // Snapshots
            LexPrimitiva::Sum,         // Aggregation
            LexPrimitiva::Quantity,    // Metrics
            LexPrimitiva::Mapping,     // Machine registry
        ])
        .with_dominant(LexPrimitiva::State, 0.70)
    }
}

// ═══════════════════════════════════════════════════════════
// MACHINE TYPES
// ═══════════════════════════════════════════════════════════

/// MachineSpec grounds to ς + → + σ + ∂ (T2-C: 4 primitives)
///
/// **Structural**: State/transition Vecs (σ), state_ids BTreeMap (μ)
/// **Functional**: FSM definition with initial/terminal (∂)
impl GroundsTo for MachineSpec {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // State definitions
            LexPrimitiva::Causality, // Transition definitions
            LexPrimitiva::Sequence,  // State/transition ordering
            LexPrimitiva::Boundary,  // Initial/terminal markers
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// MachineInstance grounds to ς + → + σ + ∂ (T2-C: 4 primitives)
///
/// **Structural**: Current state (ς), history Vec (σ), terminated (∂)
/// **Functional**: handle events (→), track history (σ)
impl GroundsTo for MachineInstance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // Current state
            LexPrimitiva::Causality, // Event handling
            LexPrimitiva::Sequence,  // Transition history
            LexPrimitiva::Boundary,  // Terminal detection
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn test_layer_dominants() {
        // Each layer has its designated dominant primitive
        assert_eq!(
            StateRegistry::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
        assert_eq!(
            TransitionEngine::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
        assert_eq!(
            BoundaryManager::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(
            GuardEvaluator::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
        assert_eq!(
            CountMetrics::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(
            SequenceController::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(
            RecursionDetector::dominant_primitive(),
            Some(LexPrimitiva::Recursion)
        );
        assert_eq!(VoidCleaner::dominant_primitive(), Some(LexPrimitiva::Void));
        assert_eq!(
            PersistStore::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
        assert_eq!(
            ExistenceValidator::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
        assert_eq!(
            AggregateCoordinator::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(
            TemporalScheduler::dominant_primitive(),
            Some(LexPrimitiva::Frequency)
        );
        assert_eq!(
            LocationRouter::dominant_primitive(),
            Some(LexPrimitiva::Location)
        );
        assert_eq!(
            IrreversibilityAuditor::dominant_primitive(),
            Some(LexPrimitiva::Irreversibility)
        );
        assert_eq!(
            MappingTransformer::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn test_tier_classifications() {
        // T2-P: 2-3 primitives
        assert_eq!(StateRegistry::tier(), Tier::T2Primitive); // 3
        assert_eq!(BoundaryManager::tier(), Tier::T2Primitive); // 2
        assert_eq!(GuardEvaluator::tier(), Tier::T2Primitive); // 3
        assert_eq!(CountMetrics::tier(), Tier::T2Primitive); // 3
        assert_eq!(SequenceController::tier(), Tier::T2Primitive); // 2
        assert_eq!(RecursionDetector::tier(), Tier::T2Primitive); // 2
        assert_eq!(VoidCleaner::tier(), Tier::T2Primitive); // 2
        assert_eq!(PersistStore::tier(), Tier::T2Primitive); // 3
        assert_eq!(ExistenceValidator::tier(), Tier::T2Primitive); // 2
        // T2-C: 4-5 primitives
        assert_eq!(MappingTransformer::tier(), Tier::T2Composite); // 4
        assert_eq!(AggregateCoordinator::tier(), Tier::T2Primitive); // 3
        assert_eq!(TemporalScheduler::tier(), Tier::T2Primitive); // 3
        assert_eq!(LocationRouter::tier(), Tier::T2Primitive); // 3
        assert_eq!(IrreversibilityAuditor::tier(), Tier::T2Primitive); // 3

        // T2-C: 4-5 primitives
        assert_eq!(TransitionEngine::tier(), Tier::T2Composite); // 5
        assert_eq!(MachineSpec::tier(), Tier::T2Composite); // 4
        assert_eq!(MachineInstance::tier(), Tier::T2Composite); // 4

        // T3: 6+ primitives
        assert_eq!(StateKernel::tier(), Tier::T3DomainSpecific); // 7
    }

    #[test]
    fn test_quindecet_coverage() {
        // All 15 primitives are represented as dominants
        let primitives: Vec<LexPrimitiva> = vec![
            StateRegistry::dominant_primitive().unwrap(),
            TransitionEngine::dominant_primitive().unwrap(),
            BoundaryManager::dominant_primitive().unwrap(),
            GuardEvaluator::dominant_primitive().unwrap(),
            CountMetrics::dominant_primitive().unwrap(),
            SequenceController::dominant_primitive().unwrap(),
            RecursionDetector::dominant_primitive().unwrap(),
            VoidCleaner::dominant_primitive().unwrap(),
            PersistStore::dominant_primitive().unwrap(),
            ExistenceValidator::dominant_primitive().unwrap(),
            AggregateCoordinator::dominant_primitive().unwrap(),
            TemporalScheduler::dominant_primitive().unwrap(),
            LocationRouter::dominant_primitive().unwrap(),
            IrreversibilityAuditor::dominant_primitive().unwrap(),
            MappingTransformer::dominant_primitive().unwrap(),
        ];

        // Verify all 15 unique primitives
        let unique: std::collections::HashSet<_> = primitives.into_iter().collect();
        assert_eq!(unique.len(), 15, "Not all 15 primitives represented");
    }

    #[test]
    fn test_structural_functional_alignment() {
        // Verify key types have structurally-justified primitives

        // TransitionEngine: 5 primitives (T2-C)
        let te = TransitionEngine::primitive_composition();
        assert_eq!(te.unique().len(), 5);
        assert!(te.unique().contains(&LexPrimitiva::Mapping)); // BTreeMaps
        assert!(te.unique().contains(&LexPrimitiva::Sequence)); // history Vec

        // StateKernel: 7 primitives (T3)
        let sk = StateKernel::primitive_composition();
        assert_eq!(sk.unique().len(), 7);
        assert!(sk.unique().contains(&LexPrimitiva::Mapping)); // machines BTreeMap

        // ExistenceValidator: ∃ + ∅ (not ∂)
        let ev = ExistenceValidator::primitive_composition();
        assert!(ev.unique().contains(&LexPrimitiva::Void)); // deleted tracking
        assert!(!ev.unique().contains(&LexPrimitiva::Boundary)); // not boundary

        // GuardEvaluator: κ + μ + → (not ∂)
        let ge = GuardEvaluator::primitive_composition();
        assert!(ge.unique().contains(&LexPrimitiva::Causality)); // conditional logic
        assert!(!ge.unique().contains(&LexPrimitiva::Boundary)); // not boundary
    }
}
