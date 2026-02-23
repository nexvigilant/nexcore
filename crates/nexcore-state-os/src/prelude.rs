//! # State OS Prelude
//!
//! Convenience re-exports for the most common STOS types.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexcore_state_os::prelude::*;
//!
//! let mut kernel = StateKernel::new();
//! let machine_id = kernel.create_machine(0).unwrap();
//! let s0 = kernel.register_state(machine_id, "pending", StateKind::Initial).unwrap();
//! let s1 = kernel.register_state(machine_id, "confirmed", StateKind::Normal).unwrap();
//! ```

// Theory foundations
pub use crate::theory::prelude::*;

// Kernel
pub use crate::{KernelConfig, KernelError, MachineId, StateKernel};

// Machine
pub use crate::{MachineBuilder, MachineInstance, MachineSpec};

// Layer 1: State Registry
pub use crate::stos::state_registry::{StateEntry, StateKind, StateRegistry};

// Layer 2: Transition Engine
pub use crate::stos::transition_engine::{TransitionEngine, TransitionResult, TransitionSpec};

// Layer 3: Boundary Manager
pub use crate::stos::boundary_manager::{BoundaryKind, BoundaryManager};

// Layer 4: Guard Evaluator
pub use crate::stos::guard_evaluator::{
    GuardContext, GuardEvaluator, GuardResult, GuardSpec, GuardValue,
};

// Layer 5: Count Metrics
pub use crate::stos::count_metrics::{CountMetrics, MachineMetrics};

// Layer 6: Sequence Controller
pub use crate::stos::sequence_controller::{ExecutionOrder, SequenceController};

// Layer 7: Recursion Detector
pub use crate::stos::recursion_detector::{CycleInfo, RecursionDetector};

// Layer 8: Void Cleaner
pub use crate::stos::void_cleaner::{UnreachableState, VoidCleaner};

// Layer 9: Persist Store
pub use crate::stos::persist_store::{PersistStore, Snapshot};

// Layer 10: Existence Validator
pub use crate::stos::existence_validator::{ExistenceResult, ExistenceValidator};

// Layer 11: Aggregate Coordinator
pub use crate::stos::aggregate_coordinator::{
    AggregateCoordinator, AggregateStats, MachineStatus, MachineSummary,
};

// Layer 12: Temporal Scheduler
pub use crate::stos::temporal_scheduler::{ScheduledTransition, TemporalScheduler};

// Layer 13: Location Router
pub use crate::stos::location_router::{LocationId, LocationRouter};

// Layer 14: Irreversibility Auditor
pub use crate::stos::irreversibility_auditor::{
    AuditEntry, IrreversibilityAuditor, IrreversibilityLevel,
};

// Layer 15: Mapping Transformer
pub use crate::stos::mapping_transformer::{
    EventStateMapping, MappingTransformer, StateMapping,
};

// Kernel new types
pub use crate::TickResult;
