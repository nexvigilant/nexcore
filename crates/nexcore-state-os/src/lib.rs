// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # State Operating System (STOS)
//!
//! A 15-layer runtime operating system for state machine orchestration,
//! built on the Universal Theory of State foundations.
//!
//! ## Architecture
//!
//! STOS follows the Quindecet pattern: 15 layers, each with a unique
//! dominant T1 primitive from the Lex Primitiva.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     STATE OPERATING SYSTEM                       │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Layer  │ Prefix      │ Dominant │ Purpose                      │
//! ├─────────┼─────────────┼──────────┼──────────────────────────────┤
//! │ STOS-ST │ state_*     │ ς State  │ Core state registry          │
//! │ STOS-TR │ transition_*│ → Cause  │ Transition execution         │
//! │ STOS-BD │ boundary_*  │ ∂ Bound  │ Initial/terminal handling    │
//! │ STOS-GD │ guard_*     │ κ Compare│ Guard evaluation             │
//! │ STOS-CT │ count_*     │ N Quant  │ Metrics & cardinality        │
//! │ STOS-SQ │ sequence_*  │ σ Seq    │ Transition ordering          │
//! │ STOS-RC │ recursion_* │ ρ Recur  │ Cycle detection              │
//! │ STOS-VD │ void_*      │ ∅ Void   │ Unreachable cleanup          │
//! │ STOS-PR │ persist_*   │ π Persist│ Snapshots & storage          │
//! │ STOS-EX │ existence_* │ ∃ Exist  │ State validation             │
//! │ STOS-AG │ aggregate_* │ Σ Sum    │ Multi-machine aggregation    │
//! │ STOS-TM │ temporal_*  │ ν Freq   │ Time-based scheduling        │
//! │ STOS-LC │ location_*  │ λ Loc    │ Distributed state            │
//! │ STOS-IR │ irreverse_* │ ∝ Irrev  │ Audit trails                 │
//! │ STOS-MP │ mapping_*   │ μ Map    │ State transformations        │
//! └─────────┴─────────────┴──────────┴──────────────────────────────┘
//! ```
//!
//! ## Layer Dependencies
//!
//! ```text
//!                    ┌──────────────┐
//!                    │   STOS-ST    │ (State Registry - Foundation)
//!                    └──────┬───────┘
//!           ┌───────────────┼───────────────┐
//!           ▼               ▼               ▼
//!     ┌──────────┐   ┌──────────┐   ┌──────────┐
//!     │ STOS-TR  │   │ STOS-BD  │   │ STOS-GD  │
//!     └────┬─────┘   └────┬─────┘   └────┬─────┘
//!          │              │              │
//!          └──────────────┼──────────────┘
//!                         ▼
//!     ┌──────────┬──────────────┬──────────┐
//!     │ STOS-CT  │   STOS-SQ    │ STOS-RC  │
//!     └────┬─────┴──────┬───────┴────┬─────┘
//!          │            │            │
//!          └────────────┼────────────┘
//!                       ▼
//!     ┌──────────┬──────────────┬──────────┐
//!     │ STOS-VD  │   STOS-PR    │ STOS-EX  │
//!     └────┬─────┴──────┬───────┴────┬─────┘
//!          │            │            │
//!          └────────────┼────────────┘
//!                       ▼
//!     ┌──────────┬──────────────┬──────────┐
//!     │ STOS-AG  │   STOS-TM    │ STOS-LC  │
//!     └────┬─────┴──────┬───────┴────┬─────┘
//!          │            │            │
//!          └────────────┼────────────┘
//!                       ▼
//!              ┌──────────────┐
//!              │   STOS-IR    │ (Audit)
//!              └──────┬───────┘
//!                     ▼
//!              ┌──────────────┐
//!              │   STOS-MP    │ (Transformations - Top)
//!              └──────────────┘
//! ```
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use nexcore_state_os::prelude::*;
//!
//! // Create the State OS kernel
//! let mut kernel = StateKernel::new();
//!
//! // Create a machine with initial state 0
//! let machine_id = kernel.create_machine(0).unwrap();
//!
//! // Register states
//! let s0 = kernel.register_state(machine_id, "pending", StateKind::Initial).unwrap();
//! let s1 = kernel.register_state(machine_id, "confirmed", StateKind::Normal).unwrap();
//! let s2 = kernel.register_state(machine_id, "delivered", StateKind::Terminal).unwrap();
//!
//! // Register transitions
//! let t0 = kernel.register_transition(machine_id, "confirm", s0, s1).unwrap();
//! let t1 = kernel.register_transition(machine_id, "deliver", s1, s2).unwrap();
//!
//! // Execute transitions by ID
//! kernel.transition(machine_id, t0).unwrap();
//! kernel.transition(machine_id, t1).unwrap();
//!
//! // Check terminal state
//! assert!(kernel.is_terminal(machine_id).unwrap());
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![warn(missing_docs)]

extern crate alloc;

// Re-export the theory crate
pub use nexcore_state_theory as theory;

// ═══════════════════════════════════════════════════════════
// STOS LAYERS (15 modules)
// ═══════════════════════════════════════════════════════════

/// STOS Layer modules following the Quindecet pattern.
pub mod stos;

// Lex Primitiva grounding (optional)
#[cfg(feature = "grounding")]
pub mod grounding;

// Re-export layer engines
pub use stos::{
    // Layer 11: Aggregate Coordinator (Σ)
    aggregate_coordinator::AggregateCoordinator,
    aggregate_coordinator::AggregateStats,
    aggregate_coordinator::MachineStatus,
    aggregate_coordinator::MachineSummary,
    boundary_manager::BoundaryKind,
    // Layer 3: Boundary Manager (∂)
    boundary_manager::BoundaryManager,
    // Layer 5: Count Metrics (N)
    count_metrics::CountMetrics,
    count_metrics::MachineMetrics,
    existence_validator::ExistenceResult,
    // Layer 10: Existence Validator (∃)
    existence_validator::ExistenceValidator,
    guard_evaluator::GuardContext,
    // Layer 4: Guard Evaluator (κ)
    guard_evaluator::GuardEvaluator,
    guard_evaluator::GuardResult,
    guard_evaluator::GuardSpec,
    guard_evaluator::GuardValue,
    irreversibility_auditor::AuditEntry,
    // Layer 14: Irreversibility Auditor (∝)
    irreversibility_auditor::IrreversibilityAuditor,
    irreversibility_auditor::IrreversibilityLevel,
    location_router::LocationId,
    // Layer 13: Location Router (λ)
    location_router::LocationRouter,
    mapping_transformer::EventStateMapping,
    // Layer 15: Mapping Transformer (μ)
    mapping_transformer::MappingTransformer,
    mapping_transformer::StateMapping,
    // Layer 9: Persist Store (π)
    persist_store::PersistStore,
    persist_store::Snapshot,
    recursion_detector::CycleInfo,
    // Layer 7: Recursion Detector (ρ)
    recursion_detector::RecursionDetector,
    sequence_controller::ExecutionOrder,
    // Layer 6: Sequence Controller (σ)
    sequence_controller::SequenceController,
    state_registry::StateEntry,
    state_registry::StateKind,
    // Layer 1: State Registry (ς)
    state_registry::StateRegistry,
    temporal_scheduler::ScheduledTransition,
    // Layer 12: Temporal Scheduler (ν)
    temporal_scheduler::TemporalScheduler,
    // Layer 2: Transition Engine (→)
    transition_engine::TransitionEngine,
    transition_engine::TransitionResult,
    transition_engine::TransitionSpec,
    void_cleaner::UnreachableState,
    // Layer 8: Void Cleaner (∅)
    void_cleaner::VoidCleaner,
};

// ═══════════════════════════════════════════════════════════
// KERNEL
// ═══════════════════════════════════════════════════════════

mod kernel;
pub use kernel::{KernelConfig, KernelError, MachineId, StateKernel, TickResult};

// ═══════════════════════════════════════════════════════════
// MACHINE SPECIFICATION
// ═══════════════════════════════════════════════════════════

mod machine;
pub use machine::{MachineBuilder, MachineInstance, MachineSpec};

// ═══════════════════════════════════════════════════════════
// GOLD-STANDARD MODULES
// ═══════════════════════════════════════════════════════════

pub mod composites;
pub mod transfer;

// Lex Primitiva primitives (optional — requires `grounding` feature)
#[cfg(feature = "grounding")]
pub mod primitives;

// ═══════════════════════════════════════════════════════════
// PRELUDE
// ═══════════════════════════════════════════════════════════

/// Prelude for convenient imports.
pub mod prelude;

// ═══════════════════════════════════════════════════════════
// PRIMITIVE SYMBOLS
// ═══════════════════════════════════════════════════════════

/// The 15 primitives used across STOS layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StosPrimitive {
    /// ς — State (Layer 1: Registry)
    State,
    /// → — Causality (Layer 2: Transitions)
    Causality,
    /// ∂ — Boundary (Layer 3: Initial/Terminal)
    Boundary,
    /// κ — Comparison (Layer 4: Guards)
    Comparison,
    /// N — Quantity (Layer 5: Metrics)
    Quantity,
    /// σ — Sequence (Layer 6: Ordering)
    Sequence,
    /// ρ — Recursion (Layer 7: Cycles)
    Recursion,
    /// ∅ — Void (Layer 8: Unreachable)
    Void,
    /// π — Persistence (Layer 9: Storage)
    Persistence,
    /// ∃ — Existence (Layer 10: Validation)
    Existence,
    /// Σ — Sum (Layer 11: Aggregation)
    Sum,
    /// ν — Frequency (Layer 12: Scheduling)
    Frequency,
    /// λ — Location (Layer 13: Distribution)
    Location,
    /// ∝ — Irreversibility (Layer 14: Audit)
    Irreversibility,
    /// μ — Mapping (Layer 15: Transformation)
    Mapping,
}

impl StosPrimitive {
    /// Unicode symbol for this primitive.
    #[must_use]
    pub const fn symbol(&self) -> char {
        match self {
            Self::State => 'ς',
            Self::Causality => '→',
            Self::Boundary => '∂',
            Self::Comparison => 'κ',
            Self::Quantity => 'N',
            Self::Sequence => 'σ',
            Self::Recursion => 'ρ',
            Self::Void => '∅',
            Self::Persistence => 'π',
            Self::Existence => '∃',
            Self::Sum => 'Σ',
            Self::Frequency => 'ν',
            Self::Location => 'λ',
            Self::Irreversibility => '∝',
            Self::Mapping => 'μ',
        }
    }

    /// Layer number (1-15).
    #[must_use]
    pub const fn layer(&self) -> u8 {
        match self {
            Self::State => 1,
            Self::Causality => 2,
            Self::Boundary => 3,
            Self::Comparison => 4,
            Self::Quantity => 5,
            Self::Sequence => 6,
            Self::Recursion => 7,
            Self::Void => 8,
            Self::Persistence => 9,
            Self::Existence => 10,
            Self::Sum => 11,
            Self::Frequency => 12,
            Self::Location => 13,
            Self::Irreversibility => 14,
            Self::Mapping => 15,
        }
    }

    /// Layer prefix for module naming.
    #[must_use]
    pub const fn prefix(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::Causality => "transition",
            Self::Boundary => "boundary",
            Self::Comparison => "guard",
            Self::Quantity => "count",
            Self::Sequence => "sequence",
            Self::Recursion => "recursion",
            Self::Void => "void",
            Self::Persistence => "persist",
            Self::Existence => "existence",
            Self::Sum => "aggregate",
            Self::Frequency => "temporal",
            Self::Location => "location",
            Self::Irreversibility => "irreversible",
            Self::Mapping => "mapping",
        }
    }

    /// All primitives in layer order.
    #[must_use]
    pub const fn all() -> [Self; 15] {
        [
            Self::State,
            Self::Causality,
            Self::Boundary,
            Self::Comparison,
            Self::Quantity,
            Self::Sequence,
            Self::Recursion,
            Self::Void,
            Self::Persistence,
            Self::Existence,
            Self::Sum,
            Self::Frequency,
            Self::Location,
            Self::Irreversibility,
            Self::Mapping,
        ]
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_symbols() {
        assert_eq!(StosPrimitive::State.symbol(), 'ς');
        assert_eq!(StosPrimitive::Causality.symbol(), '→');
        assert_eq!(StosPrimitive::Mapping.symbol(), 'μ');
    }

    #[test]
    fn test_primitive_layers() {
        assert_eq!(StosPrimitive::State.layer(), 1);
        assert_eq!(StosPrimitive::Mapping.layer(), 15);
        assert_eq!(StosPrimitive::all().len(), 15);
    }

    #[test]
    fn test_layer_coverage() {
        // Verify all 15 layers are covered
        let layers: Vec<u8> = StosPrimitive::all().iter().map(|p| p.layer()).collect();
        assert_eq!(layers, (1..=15).collect::<Vec<u8>>());
    }
}
