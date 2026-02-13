// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # STOS Layers
//!
//! The 15-layer State Operating System following the Quindecet pattern.
//! Each layer has a unique dominant T1 primitive.
//!
//! ## Layer Summary
//!
//! | Layer | Module | Dominant | Engine Type |
//! |-------|--------|----------|-------------|
//! | 1 | `state_registry` | ς | `StateRegistry` |
//! | 2 | `transition_engine` | → | `TransitionEngine` |
//! | 3 | `boundary_manager` | ∂ | `BoundaryManager` |
//! | 4 | `guard_evaluator` | κ | `GuardEvaluator` |
//! | 5 | `count_metrics` | N | `CountMetrics` |
//! | 6 | `sequence_controller` | σ | `SequenceController` |
//! | 7 | `recursion_detector` | ρ | `RecursionDetector` |
//! | 8 | `void_cleaner` | ∅ | `VoidCleaner` |
//! | 9 | `persist_store` | π | `PersistStore` |
//! | 10 | `existence_validator` | ∃ | `ExistenceValidator` |
//! | 11 | `aggregate_coordinator` | Σ | `AggregateCoordinator` |
//! | 12 | `temporal_scheduler` | ν | `TemporalScheduler` |
//! | 13 | `location_router` | λ | `LocationRouter` |
//! | 14 | `irreversibility_auditor` | ∝ | `IrreversibilityAuditor` |
//! | 15 | `mapping_transformer` | μ | `MappingTransformer` |

// Layer 1: State Registry (ς State)
pub mod state_registry;

// Layer 2: Transition Engine (→ Causality)
pub mod transition_engine;

// Layer 3: Boundary Manager (∂ Boundary)
pub mod boundary_manager;

// Layer 4: Guard Evaluator (κ Comparison)
pub mod guard_evaluator;

// Layer 5: Count Metrics (N Quantity)
pub mod count_metrics;

// Layer 6: Sequence Controller (σ Sequence)
pub mod sequence_controller;

// Layer 7: Recursion Detector (ρ Recursion)
pub mod recursion_detector;

// Layer 8: Void Cleaner (∅ Void)
pub mod void_cleaner;

// Layer 9: Persist Store (π Persistence)
pub mod persist_store;

// Layer 10: Existence Validator (∃ Existence)
pub mod existence_validator;

// Layer 11: Aggregate Coordinator (Σ Sum)
pub mod aggregate_coordinator;

// Layer 12: Temporal Scheduler (ν Frequency)
pub mod temporal_scheduler;

// Layer 13: Location Router (λ Location)
pub mod location_router;

// Layer 14: Irreversibility Auditor (∝ Irreversibility)
pub mod irreversibility_auditor;

// Layer 15: Mapping Transformer (μ Mapping)
pub mod mapping_transformer;
