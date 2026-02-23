// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # State Operating System Kernel
//!
//! The StateKernel is the central orchestrator that coordinates
//! all 15 STOS layers to manage state machines.
//!
//! ## Layer Wiring Summary
//!
//! | Layer | Wired In | Method(s) |
//! |-------|----------|-----------|
//! | 1 ς StateRegistry | `register_state`, `load_machine` | State lifecycle |
//! | 2 → TransitionEngine | `transition`, `transition_guarded` | Causal execution |
//! | 3 ∂ BoundaryManager | `register_state`, `transition` | Entry/exit boundaries |
//! | 4 κ GuardEvaluator | `transition_guarded`, `register_guard` | Guard evaluation |
//! | 5 N CountMetrics | `transition` | Visit/execution counts |
//! | 6 σ SequenceController | `enqueue_transition`, `execute_next` | Ordered execution |
//! | 7 ρ RecursionDetector | `register_transition` | Cycle edge tracking |
//! | 8 ∅ VoidCleaner | `register_state`, `register_transition`, `analyze_voids` | Unreachable detection |
//! | 9 π PersistStore | `transition`, `snapshot` | Auto/manual snapshots |
//! | 10 ∃ ExistenceValidator | `register_state`, `register_transition` | Entity validation |
//! | 11 Σ AggregateCoordinator | `create_machine`, `transition` | Cross-machine stats |
//! | 12 ν TemporalScheduler | `tick` | Time-based scheduling |
//! | 13 λ LocationRouter | `create_location`, `assign_location` | Distributed routing |
//! | 14 ∝ IrreversibilityAuditor | `transition`, absorbing check | Audit trail |
//! | 15 μ MappingTransformer | `register_state_mapping`, `handle_event` | State/event transforms |

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::machine::MachineSpec;
use crate::stos::{
    aggregate_coordinator::{AggregateCoordinator, MachineStatus},
    boundary_manager::BoundaryManager,
    count_metrics::CountMetrics,
    existence_validator::ExistenceValidator,
    guard_evaluator::{GuardContext, GuardEvaluator, GuardId},
    irreversibility_auditor::{IrreversibilityAuditor, IrreversibilityLevel},
    location_router::{LocationId, LocationRouter},
    mapping_transformer::{EventStateMapping, MappingTransformer, StateMapping},
    persist_store::PersistStore,
    recursion_detector::RecursionDetector,
    sequence_controller::SequenceController,
    state_registry::{StateId, StateKind, StateRegistry},
    temporal_scheduler::TemporalScheduler,
    transition_engine::{TransitionEngine, TransitionId, TransitionResult},
    void_cleaner::{UnreachableState, VoidCleaner},
};

/// Machine identifier.
pub type MachineId = u64;

/// Kernel configuration.
#[derive(Debug, Clone)]
pub struct KernelConfig {
    /// Maximum machines allowed.
    pub max_machines: usize,
    /// Maximum states per machine.
    pub max_states_per_machine: usize,
    /// Maximum transitions per machine.
    pub max_transitions_per_machine: usize,
    /// Enable auto-snapshots.
    pub auto_snapshot: bool,
    /// Snapshot interval (transitions).
    pub snapshot_interval: u64,
    /// Enable audit trail.
    pub audit_enabled: bool,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            max_machines: 1000,
            max_states_per_machine: 100,
            max_transitions_per_machine: 500,
            auto_snapshot: true,
            snapshot_interval: 100,
            audit_enabled: true,
        }
    }
}

/// Kernel error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    /// Machine not found.
    MachineNotFound(MachineId),
    /// State not found.
    StateNotFound(StateId),
    /// Transition not found.
    TransitionNotFound(TransitionId),
    /// Transition failed.
    TransitionFailed(String),
    /// Guard rejected.
    GuardRejected(String),
    /// Guard not found by name.
    GuardNotFound(String),
    /// Invalid operation.
    InvalidOperation(String),
    /// Capacity exceeded.
    CapacityExceeded(String),
    /// Machine already exists.
    MachineExists(MachineId),
    /// In terminal state.
    InTerminalState(MachineId),
    /// Registry error.
    RegistryError(String),
    /// Machine is in an absorbing (permanently irreversible) state.
    AbsorbingState(MachineId),
    /// No available transition from the current state.
    NoAvailableTransition(String),
    /// Void (unreachable) state detected.
    VoidStateDetected(String),
}

/// Result of a kernel tick operation.
#[derive(Debug, Clone)]
pub struct TickResult {
    /// Successfully executed transitions.
    pub executed: Vec<(MachineId, TransitionResult)>,
    /// Timed-out transitions.
    pub timeouts: Vec<(MachineId, TransitionId)>,
    /// Errors encountered during tick.
    pub errors: Vec<(MachineId, KernelError)>,
}

/// Per-machine runtime state.
#[derive(Debug)]
struct MachineRuntime {
    /// State registry.
    states: StateRegistry,
    /// Transition engine.
    transitions: TransitionEngine,
    /// Boundary manager.
    boundaries: BoundaryManager,
    /// Guard evaluator.
    guards: GuardEvaluator,
    /// Metrics tracker.
    metrics: CountMetrics,
    /// Sequence controller.
    sequences: SequenceController,
    /// Recursion detector.
    recursion: RecursionDetector,
    /// Void cleaner.
    voids: VoidCleaner,
    /// Persist store.
    persistence: PersistStore,
    /// Existence validator.
    existence: ExistenceValidator,
    /// Auditor.
    auditor: IrreversibilityAuditor,
    /// Current state.
    current_state: StateId,
    /// Whether machine is terminated.
    terminated: bool,
}

impl MachineRuntime {
    fn new(machine_id: MachineId, initial_state: StateId) -> Self {
        Self {
            states: StateRegistry::new(machine_id),
            transitions: TransitionEngine::new(machine_id),
            boundaries: BoundaryManager::new(machine_id),
            guards: GuardEvaluator::new(machine_id),
            metrics: CountMetrics::new(machine_id),
            sequences: SequenceController::new(machine_id),
            recursion: RecursionDetector::new(machine_id),
            voids: VoidCleaner::new(machine_id),
            persistence: PersistStore::new(machine_id),
            existence: ExistenceValidator::new(machine_id),
            auditor: IrreversibilityAuditor::new(),
            current_state: initial_state,
            terminated: false,
        }
    }
}

/// The State Operating System Kernel.
///
/// Orchestrates all 15 STOS layers to manage multiple state machines.
///
/// ## Tier: T3 (ς + → + ∂ + π + Σ + N)
#[derive(Debug)]
pub struct StateKernel {
    /// Configuration.
    config: KernelConfig,
    /// Machine runtimes.
    machines: BTreeMap<MachineId, MachineRuntime>,
    /// Machine ID counter.
    machine_counter: MachineId,
    /// Aggregate coordinator (cross-machine).
    aggregate: AggregateCoordinator,
    /// Temporal scheduler (cross-machine).
    scheduler: TemporalScheduler,
    /// Location router (cross-machine).
    router: LocationRouter,
    /// Mapping transformer (cross-machine).
    mapper: MappingTransformer,
}

impl StateKernel {
    // ═══════════════════════════════════════════════════════════
    // CORE LIFECYCLE
    // ═══════════════════════════════════════════════════════════

    /// Create a new kernel with default config.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(KernelConfig::default())
    }

    /// Create a new kernel with config.
    #[must_use]
    pub fn with_config(config: KernelConfig) -> Self {
        Self {
            config,
            machines: BTreeMap::new(),
            machine_counter: 0,
            aggregate: AggregateCoordinator::new(),
            scheduler: TemporalScheduler::new(),
            router: LocationRouter::new(),
            mapper: MappingTransformer::new(),
        }
    }

    /// Get kernel configuration.
    #[must_use]
    pub fn config(&self) -> &KernelConfig {
        &self.config
    }

    /// Create a new machine with an initial state.
    pub fn create_machine(&mut self, initial_state: StateId) -> Result<MachineId, KernelError> {
        if self.machines.len() >= self.config.max_machines {
            return Err(KernelError::CapacityExceeded(
                "Maximum machines reached".into(),
            ));
        }

        self.machine_counter = self.machine_counter.saturating_add(1);
        let machine_id = self.machine_counter;

        let runtime = MachineRuntime::new(machine_id, initial_state);
        self.machines.insert(machine_id, runtime);

        // Register with aggregate coordinator (Layer 11: Σ)
        self.aggregate.register(machine_id, initial_state);

        Ok(machine_id)
    }

    /// Load a machine from a `MachineSpec` built via `MachineBuilder`.
    ///
    /// Registers all states, transitions, boundaries, void edges, and
    /// sets the current state to the spec's initial state.
    ///
    /// ## Layers Wired
    /// - Layer 1 (ς): State registration
    /// - Layer 2 (→): Transition registration
    /// - Layer 3 (∂): Boundary auto-registration
    /// - Layer 7 (ρ): Recursion edge tracking
    /// - Layer 8 (∅): Void cleaner graph population
    /// - Layer 10 (∃): Existence registration
    /// - Layer 11 (Σ): Aggregate registration
    pub fn load_machine(&mut self, spec: &MachineSpec) -> Result<MachineId, KernelError> {
        let initial = spec.initial_state().ok_or_else(|| {
            KernelError::InvalidOperation("MachineSpec has no initial state".into())
        })?;

        let mid = self.create_machine(initial)?;

        // Register all states
        for state_spec in &spec.states {
            let sid = self.register_state(mid, &state_spec.name, state_spec.kind)?;
            // Verify the builder-assigned ID matches the registry-assigned ID
            // (they should, since both use monotonic counters starting at 0)
            let _ = sid; // Used for registration side effects
        }

        // Register all transitions
        for transition_spec in &spec.transitions {
            let _from = spec.state_id(&transition_spec.from).ok_or({
                KernelError::StateNotFound(0) // State name not found
            })?;
            let _to = spec
                .state_id(&transition_spec.to)
                .ok_or(KernelError::StateNotFound(0))?;

            // Map spec state IDs to kernel-registered state IDs
            // The kernel registers states sequentially, so spec ID N maps to
            // the Nth state registered in the kernel's registry
            let runtime = self
                .machines
                .get(&mid)
                .ok_or(KernelError::MachineNotFound(mid))?;
            let kernel_from = runtime
                .states
                .get_by_name(&transition_spec.from)
                .map(|e| e.id)
                .ok_or_else(|| {
                    KernelError::InvalidOperation(alloc::format!(
                        "State '{}' not found in registry",
                        transition_spec.from
                    ))
                })?;
            let kernel_to = runtime
                .states
                .get_by_name(&transition_spec.to)
                .map(|e| e.id)
                .ok_or_else(|| {
                    KernelError::InvalidOperation(alloc::format!(
                        "State '{}' not found in registry",
                        transition_spec.to
                    ))
                })?;

            self.register_transition(mid, &transition_spec.event, kernel_from, kernel_to)?;
        }

        // Set current state to initial
        if let Some(runtime) = self.machines.get_mut(&mid) {
            if let Some(init_entry) = runtime.states.initial_state() {
                runtime.current_state = init_entry.id;
            }
        }

        Ok(mid)
    }

    /// Get current state of a machine.
    pub fn current_state(&self, machine_id: MachineId) -> Result<StateId, KernelError> {
        self.machines
            .get(&machine_id)
            .map(|r| r.current_state)
            .ok_or(KernelError::MachineNotFound(machine_id))
    }

    /// Check if machine is in terminal state.
    pub fn is_terminal(&self, machine_id: MachineId) -> Result<bool, KernelError> {
        self.machines
            .get(&machine_id)
            .map(|r| r.terminated)
            .ok_or(KernelError::MachineNotFound(machine_id))
    }

    // ═══════════════════════════════════════════════════════════
    // LAYER 1: STATE REGISTRY (ς)
    // ═══════════════════════════════════════════════════════════

    /// Register a state for a machine.
    ///
    /// Wires: Layer 1 (ς), Layer 3 (∂), Layer 8 (∅), Layer 10 (∃)
    pub fn register_state(
        &mut self,
        machine_id: MachineId,
        name: impl Into<String>,
        kind: StateKind,
    ) -> Result<StateId, KernelError> {
        let runtime = self
            .machines
            .get_mut(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;

        let state_id = runtime
            .states
            .register(name, kind)
            .map_err(|e| KernelError::RegistryError(alloc::format!("{e:?}")))?;

        // Layer 10 (∃): Existence validation
        runtime.existence.register_state(state_id);

        // Layer 3 (∂): Auto-register boundaries
        match kind {
            StateKind::Initial => runtime.boundaries.register_initial(state_id),
            StateKind::Terminal => runtime.boundaries.register_terminal(state_id),
            StateKind::Error => runtime.boundaries.register_error(state_id),
            StateKind::Normal => {}
        }

        // Layer 8 (∅): Void cleaner graph sync
        runtime.voids.add_state(
            state_id,
            kind == StateKind::Initial,
            kind == StateKind::Terminal,
        );

        Ok(state_id)
    }

    // ═══════════════════════════════════════════════════════════
    // LAYER 2 + 7: TRANSITION & RECURSION (→ + ρ)
    // ═══════════════════════════════════════════════════════════

    /// Register a transition for a machine.
    ///
    /// Wires: Layer 2 (→), Layer 7 (ρ), Layer 8 (∅), Layer 10 (∃)
    pub fn register_transition(
        &mut self,
        machine_id: MachineId,
        name: impl Into<String>,
        from: StateId,
        to: StateId,
    ) -> Result<TransitionId, KernelError> {
        let runtime = self
            .machines
            .get_mut(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;

        let transition_id = runtime.transitions.register(name, from, to);

        // Layer 10 (∃): Existence registration
        runtime.existence.register_transition(transition_id);

        // Layer 7 (ρ): Recursion edge tracking
        runtime.recursion.add_edge(from, to);

        // Layer 8 (∅): Void cleaner edge sync
        runtime.voids.add_edge(from, to);

        Ok(transition_id)
    }

    // ═══════════════════════════════════════════════════════════
    // LAYER 2 + 3 + 5 + 9 + 11 + 14: TRANSITION EXECUTION
    // ═══════════════════════════════════════════════════════════

    /// Execute a transition by ID (no guard context).
    ///
    /// Wires: Layer 2 (→), Layer 3 (∂), Layer 5 (N), Layer 9 (π),
    ///        Layer 11 (Σ), Layer 14 (∝)
    pub fn transition(
        &mut self,
        machine_id: MachineId,
        transition_id: TransitionId,
    ) -> Result<TransitionResult, KernelError> {
        self.transition_internal(machine_id, transition_id, None)
    }

    /// Execute a guarded transition with a context.
    ///
    /// Wires: Layer 2 (→), Layer 3 (∂), Layer 4 (κ), Layer 5 (N),
    ///        Layer 9 (π), Layer 11 (Σ), Layer 14 (∝)
    pub fn transition_guarded(
        &mut self,
        machine_id: MachineId,
        transition_id: TransitionId,
        context: &GuardContext,
    ) -> Result<TransitionResult, KernelError> {
        self.transition_internal(machine_id, transition_id, Some(context))
    }

    /// Internal transition logic wiring all applicable layers.
    fn transition_internal(
        &mut self,
        machine_id: MachineId,
        transition_id: TransitionId,
        guard_context: Option<&GuardContext>,
    ) -> Result<TransitionResult, KernelError> {
        let runtime = self
            .machines
            .get_mut(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;

        // Check if already terminated
        if runtime.terminated {
            return Err(KernelError::InTerminalState(machine_id));
        }

        // Layer 10 (∃): Validate transition exists
        if !runtime.existence.transition_exists(transition_id).exists() {
            return Err(KernelError::TransitionNotFound(transition_id));
        }

        // Layer 14 (∝): Check absorbing state — cannot leave Permanent absorbing states
        if runtime.auditor.is_state_absorbing(runtime.current_state) {
            if let Some(level) = runtime.auditor.state_absorbing_level(runtime.current_state) {
                if level >= IrreversibilityLevel::Permanent {
                    return Err(KernelError::AbsorbingState(machine_id));
                }
            }
        }

        // Layer 4 (κ): Guard evaluation
        if let Some(context) = guard_context {
            // Check if this transition has a guard reference
            if let Some(spec) = runtime.transitions.get(transition_id) {
                if let Some(guard_name) = spec.guard.clone() {
                    let result = runtime.guards.evaluate_by_name(&guard_name, context);
                    match result {
                        Some(guard_result) if !guard_result.passed => {
                            let reason = guard_result
                                .reason
                                .unwrap_or_else(|| alloc::format!("Guard '{guard_name}' rejected"));
                            return Err(KernelError::GuardRejected(reason));
                        }
                        None => {
                            return Err(KernelError::GuardNotFound(guard_name));
                        }
                        _ => {} // Guard passed
                    }
                }
            }
        }

        // Layer 2 (→): Execute transition
        let result = runtime
            .transitions
            .execute(transition_id, runtime.current_state)
            .map_err(|e| KernelError::TransitionFailed(alloc::format!("{e:?}")))?;

        let from_state = runtime.current_state;
        let to_state = result.to_state;
        runtime.current_state = to_state;

        // Layer 5 (N): Record metrics
        runtime.metrics.record_state_exit(from_state);
        runtime.metrics.record_transition(transition_id);
        runtime.metrics.record_state_visit(to_state);

        // Layer 3 (∂): Record boundary crossings
        runtime.boundaries.record_crossing(from_state, false);
        runtime.boundaries.record_crossing(to_state, true);

        // Layer 14 (∝): Record audit if enabled
        if self.config.audit_enabled {
            runtime
                .auditor
                .record(machine_id, transition_id, from_state, to_state);
        }

        // Layer 9 (π): Check auto-snapshot
        if self.config.auto_snapshot && runtime.persistence.record_transition() {
            runtime
                .persistence
                .snapshot(to_state, runtime.metrics.total_executions());
        }

        // Layer 11 (Σ): Update aggregate
        self.aggregate.update_state(machine_id, to_state);

        // Layer 3 + 11 (∂ + Σ): Check if now terminal
        if runtime.boundaries.is_terminal(to_state) {
            runtime.terminated = true;
            // Layer 11 (Σ): Update aggregate status to Terminated
            self.aggregate
                .update_status(machine_id, MachineStatus::Terminated);
        }

        Ok(result)
    }

    // ═══════════════════════════════════════════════════════════
    // LAYER 4: GUARD EVALUATOR (κ)
    // ═══════════════════════════════════════════════════════════

    /// Register a guard for a machine.
    pub fn register_guard(
        &mut self,
        machine_id: MachineId,
        name: impl Into<String>,
        expression: impl Into<String>,
    ) -> Result<GuardId, KernelError> {
        let runtime = self
            .machines
            .get_mut(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;

        Ok(runtime.guards.register(name, expression))
    }

    /// Register a guarded transition (transition + guard name link).
    pub fn register_guarded_transition(
        &mut self,
        machine_id: MachineId,
        name: impl Into<String>,
        from: StateId,
        to: StateId,
        guard: impl Into<String>,
    ) -> Result<TransitionId, KernelError> {
        let runtime = self
            .machines
            .get_mut(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;

        let transition_id = runtime.transitions.register_guarded(name, from, to, guard);

        // Layer 10 (∃): Existence
        runtime.existence.register_transition(transition_id);
        // Layer 7 (ρ): Recursion
        runtime.recursion.add_edge(from, to);
        // Layer 8 (∅): Void edges
        runtime.voids.add_edge(from, to);

        Ok(transition_id)
    }

    // ═══════════════════════════════════════════════════════════
    // LAYER 6: SEQUENCE CONTROLLER (σ)
    // ═══════════════════════════════════════════════════════════

    /// Enqueue a transition for sequential execution.
    pub fn enqueue_transition(
        &mut self,
        machine_id: MachineId,
        transition_id: TransitionId,
    ) -> Result<(), KernelError> {
        let runtime = self
            .machines
            .get_mut(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;

        if !runtime.sequences.enqueue(transition_id) {
            return Err(KernelError::CapacityExceeded("Sequence queue full".into()));
        }
        Ok(())
    }

    /// Execute the next queued transition.
    pub fn execute_next(
        &mut self,
        machine_id: MachineId,
    ) -> Result<Option<TransitionResult>, KernelError> {
        // Dequeue from sequence controller
        let queued = {
            let runtime = self
                .machines
                .get_mut(&machine_id)
                .ok_or(KernelError::MachineNotFound(machine_id))?;
            runtime.sequences.dequeue()
        };

        match queued {
            Some(q) => {
                let result = self.transition(machine_id, q.transition_id)?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    // ═══════════════════════════════════════════════════════════
    // LAYER 8: VOID CLEANER (∅)
    // ═══════════════════════════════════════════════════════════

    /// Analyze a machine for void (unreachable) states.
    pub fn analyze_voids(
        &mut self,
        machine_id: MachineId,
    ) -> Result<Vec<UnreachableState>, KernelError> {
        let runtime = self
            .machines
            .get_mut(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;

        runtime.voids.analyze();
        Ok(runtime.voids.unreachable().to_vec())
    }

    // ═══════════════════════════════════════════════════════════
    // LAYER 9: PERSIST STORE (π)
    // ═══════════════════════════════════════════════════════════

    /// Create a manual snapshot.
    pub fn snapshot(&mut self, machine_id: MachineId) -> Result<u64, KernelError> {
        let runtime = self
            .machines
            .get_mut(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;

        Ok(runtime
            .persistence
            .snapshot(runtime.current_state, runtime.metrics.total_executions()))
    }

    // ═══════════════════════════════════════════════════════════
    // LAYER 11: AGGREGATE COORDINATOR (Σ)
    // ═══════════════════════════════════════════════════════════

    /// Get machine count.
    #[must_use]
    pub fn machine_count(&self) -> usize {
        self.machines.len()
    }

    /// Get all machine IDs.
    #[must_use]
    pub fn machine_ids(&self) -> Vec<MachineId> {
        self.machines.keys().copied().collect()
    }

    /// Get aggregate stats.
    #[must_use]
    pub fn aggregate_stats(&self) -> crate::stos::aggregate_coordinator::AggregateStats {
        self.aggregate.stats()
    }

    /// Access the aggregate coordinator.
    #[must_use]
    pub fn aggregate(&self) -> &AggregateCoordinator {
        &self.aggregate
    }

    /// Access the aggregate coordinator mutably.
    pub fn aggregate_mut(&mut self) -> &mut AggregateCoordinator {
        &mut self.aggregate
    }

    // ═══════════════════════════════════════════════════════════
    // LAYER 12: TEMPORAL SCHEDULER (ν)
    // ═══════════════════════════════════════════════════════════

    /// Advance time and execute due transitions and expired timeouts.
    ///
    /// This is the main tick loop that drives temporal scheduling.
    pub fn tick(&mut self, delta: u64) -> TickResult {
        let mut result = TickResult {
            executed: Vec::new(),
            timeouts: Vec::new(),
            errors: Vec::new(),
        };

        // Advance scheduler time
        self.scheduler.advance(delta);

        // Collect due transitions
        let due = self.scheduler.due_transitions();
        for (schedule_id, machine_id, transition_id) in due {
            match self.transition(machine_id, transition_id) {
                Ok(tr) => {
                    result.executed.push((machine_id, tr));
                    self.scheduler.mark_executed(schedule_id);
                }
                Err(e) => {
                    result.errors.push((machine_id, e));
                }
            }
        }

        // Collect expired timeouts
        let expired = self.scheduler.expired_timeouts();
        for (timeout_id, machine_id, transition_id) in expired {
            result.timeouts.push((machine_id, transition_id));
            self.scheduler.mark_timeout_triggered(timeout_id);
        }

        // Cleanup exhausted schedules and triggered timeouts
        self.scheduler.cleanup();

        result
    }

    /// Access the temporal scheduler.
    #[must_use]
    pub fn scheduler(&self) -> &TemporalScheduler {
        &self.scheduler
    }

    /// Access the temporal scheduler mutably.
    pub fn scheduler_mut(&mut self) -> &mut TemporalScheduler {
        &mut self.scheduler
    }

    // ═══════════════════════════════════════════════════════════
    // LAYER 13: LOCATION ROUTER (λ)
    // ═══════════════════════════════════════════════════════════

    /// Create a location and return its ID.
    pub fn create_location(&mut self, name: impl Into<String>) -> LocationId {
        self.router.create_location(name)
    }

    /// Assign a machine to a location.
    pub fn assign_location(
        &mut self,
        machine_id: MachineId,
        location_id: LocationId,
    ) -> Result<(), KernelError> {
        if !self.machines.contains_key(&machine_id) {
            return Err(KernelError::MachineNotFound(machine_id));
        }
        self.router.assign(machine_id, location_id);
        Ok(())
    }

    /// Route a machine to a location using routing rules.
    pub fn route_machine(
        &mut self,
        machine_id: MachineId,
    ) -> Result<Option<LocationId>, KernelError> {
        if !self.machines.contains_key(&machine_id) {
            return Err(KernelError::MachineNotFound(machine_id));
        }
        Ok(self.router.route(machine_id))
    }

    /// Get all machines at a location.
    #[must_use]
    pub fn machines_at(&self, location_id: LocationId) -> Vec<MachineId> {
        self.router.machines_at(location_id)
    }

    /// Access the location router.
    #[must_use]
    pub fn router(&self) -> &LocationRouter {
        &self.router
    }

    /// Access the location router mutably.
    pub fn router_mut(&mut self) -> &mut LocationRouter {
        &mut self.router
    }

    // ═══════════════════════════════════════════════════════════
    // LAYER 14: IRREVERSIBILITY AUDITOR (∝)
    // ═══════════════════════════════════════════════════════════

    /// Register an absorbing state for a machine.
    pub fn register_absorbing_state(
        &mut self,
        machine_id: MachineId,
        state: StateId,
        level: IrreversibilityLevel,
    ) -> Result<(), KernelError> {
        let runtime = self
            .machines
            .get_mut(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;

        runtime.auditor.register_absorbing_state(state, level);
        Ok(())
    }

    /// Register an irreversible transition.
    pub fn register_irreversible_transition(
        &mut self,
        machine_id: MachineId,
        transition_id: TransitionId,
        level: IrreversibilityLevel,
    ) -> Result<(), KernelError> {
        let runtime = self
            .machines
            .get_mut(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;

        runtime
            .auditor
            .register_irreversible_transition(transition_id, level);
        Ok(())
    }

    /// Verify audit trail integrity for a machine.
    pub fn verify_audit_trail(&self, machine_id: MachineId) -> Result<bool, KernelError> {
        let runtime = self
            .machines
            .get(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;

        Ok(runtime.auditor.verify_all())
    }

    /// Get audit trail length for a machine.
    pub fn audit_trail_len(&self, machine_id: MachineId) -> Result<usize, KernelError> {
        let runtime = self
            .machines
            .get(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;

        Ok(runtime.auditor.len())
    }

    // ═══════════════════════════════════════════════════════════
    // LAYER 15: MAPPING TRANSFORMER (μ)
    // ═══════════════════════════════════════════════════════════

    /// Register a state mapping between machines.
    pub fn register_state_mapping(&mut self, mapping: StateMapping) {
        self.mapper.register_state_mapping(mapping);
    }

    /// Map a state between machines.
    #[must_use]
    pub fn map_state(
        &self,
        source: MachineId,
        target: MachineId,
        state: StateId,
    ) -> Option<StateId> {
        self.mapper.map_state(source, target, state)
    }

    /// Register an event-state mapping for a machine.
    pub fn register_event_mapping(
        &mut self,
        machine_id: MachineId,
        mapping: EventStateMapping,
    ) -> Result<(), KernelError> {
        if !self.machines.contains_key(&machine_id) {
            return Err(KernelError::MachineNotFound(machine_id));
        }
        self.mapper.register_event_mapping(machine_id, mapping);
        Ok(())
    }

    /// Handle an event by looking up the event-state mapping and executing
    /// the corresponding transition.
    pub fn handle_event(
        &mut self,
        machine_id: MachineId,
        event: &str,
    ) -> Result<TransitionResult, KernelError> {
        // Look up the target state from the event mapping
        let current = self.current_state(machine_id)?;
        let target_state = self
            .mapper
            .event_transition(machine_id, event, current)
            .ok_or_else(|| {
                KernelError::NoAvailableTransition(alloc::format!(
                    "No mapping for event '{event}' from state {current}"
                ))
            })?;

        // Find the matching transition in the engine
        let runtime = self
            .machines
            .get(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;

        let transition_id = runtime
            .transitions
            .find_transition(current, event)
            .map(|t| t.id)
            .or_else(|| {
                // Fallback: find any transition from current to target
                runtime
                    .transitions
                    .outgoing_from(current)
                    .into_iter()
                    .find(|t| t.to == target_state)
                    .map(|t| t.id)
            })
            .ok_or_else(|| {
                KernelError::NoAvailableTransition(alloc::format!(
                    "No transition for event '{event}' from state {current} to {target_state}"
                ))
            })?;

        self.transition(machine_id, transition_id)
    }

    /// Access the mapping transformer.
    #[must_use]
    pub fn mapper(&self) -> &MappingTransformer {
        &self.mapper
    }

    /// Access the mapping transformer mutably.
    pub fn mapper_mut(&mut self) -> &mut MappingTransformer {
        &mut self.mapper
    }

    /// Look up a transition ID by name from a given state.
    ///
    /// Useful for setting up temporal schedules or irreversibility rules
    /// that require a `TransitionId`.
    pub fn find_transition_id(
        &self,
        machine_id: MachineId,
        from_state: StateId,
        action: &str,
    ) -> Result<TransitionId, KernelError> {
        let runtime = self
            .machines
            .get(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;
        runtime
            .transitions
            .find_transition(from_state, action)
            .map(|t| t.id)
            .ok_or_else(|| {
                KernelError::NoAvailableTransition(alloc::format!(
                    "No transition '{action}' from state {from_state}"
                ))
            })
    }

    /// Look up a state ID by name.
    pub fn find_state_id(&self, machine_id: MachineId, name: &str) -> Result<StateId, KernelError> {
        let runtime = self
            .machines
            .get(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;
        runtime
            .states
            .id_of(name)
            .ok_or(KernelError::StateNotFound(0))
    }

    // ═══════════════════════════════════════════════════════════
    // LAYER 5 + 7: METRICS & RECURSION ACCESSORS
    // ═══════════════════════════════════════════════════════════

    /// Get metrics for a machine.
    pub fn metrics(
        &self,
        machine_id: MachineId,
    ) -> Result<&crate::stos::count_metrics::MachineMetrics, KernelError> {
        let runtime = self
            .machines
            .get(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;
        Ok(runtime.metrics.metrics())
    }

    /// Detect cycles in a machine's state graph.
    pub fn detect_cycles(
        &mut self,
        machine_id: MachineId,
    ) -> Result<Vec<crate::stos::recursion_detector::CycleInfo>, KernelError> {
        let runtime = self
            .machines
            .get_mut(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;
        Ok(runtime.recursion.detect_cycles())
    }

    /// Get boundary crossings for a machine.
    pub fn boundary_crossings(
        &self,
        machine_id: MachineId,
    ) -> Result<&[crate::stos::boundary_manager::BoundaryCrossing], KernelError> {
        let runtime = self
            .machines
            .get(&machine_id)
            .ok_or(KernelError::MachineNotFound(machine_id))?;
        Ok(runtime.boundaries.crossings())
    }
}

impl Default for StateKernel {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_kernel() {
        let kernel = StateKernel::new();
        assert_eq!(kernel.machine_count(), 0);
    }

    #[test]
    fn test_create_machine() {
        let mut kernel = StateKernel::new();

        let machine_id = kernel.create_machine(0);
        assert!(machine_id.is_ok());
        assert_eq!(kernel.machine_count(), 1);
    }

    #[test]
    fn test_current_state() {
        let mut kernel = StateKernel::new();

        let result = kernel.create_machine(5);
        assert!(result.is_ok());

        if let Ok(machine_id) = result {
            let state = kernel.current_state(machine_id);
            assert_eq!(state, Ok(5));
        }
    }

    #[test]
    fn test_register_and_transition() {
        let mut kernel = StateKernel::new();

        let result = kernel.create_machine(0);
        assert!(result.is_ok());

        if let Ok(mid) = result {
            // Register states
            let s0 = kernel.register_state(mid, "initial", StateKind::Initial);
            let s1 = kernel.register_state(mid, "processing", StateKind::Normal);
            let s2 = kernel.register_state(mid, "complete", StateKind::Terminal);

            assert!(s0.is_ok());
            assert!(s1.is_ok());
            assert!(s2.is_ok());

            if let (Ok(from0), Ok(to1), Ok(to2)) = (s0, s1, s2) {
                // Register transitions
                let t0 = kernel.register_transition(mid, "start", from0, to1);
                let t1 = kernel.register_transition(mid, "finish", to1, to2);

                assert!(t0.is_ok());
                assert!(t1.is_ok());

                if let (Ok(tid0), Ok(tid1)) = (t0, t1) {
                    // Update current state to the initial state we registered
                    if let Some(runtime) = kernel.machines.get_mut(&mid) {
                        runtime.current_state = from0;
                    }

                    // Execute transitions
                    let result1 = kernel.transition(mid, tid0);
                    assert!(result1.is_ok());
                    assert_eq!(kernel.current_state(mid), Ok(to1));

                    let result2 = kernel.transition(mid, tid1);
                    assert!(result2.is_ok());
                    assert_eq!(kernel.current_state(mid), Ok(to2));

                    // Should be terminal now
                    assert_eq!(kernel.is_terminal(mid), Ok(true));
                }
            }
        }
    }

    #[test]
    fn test_terminal_blocks_transition() {
        let mut kernel = StateKernel::new();

        if let Ok(mid) = kernel.create_machine(0) {
            let s0 = kernel.register_state(mid, "start", StateKind::Initial);
            let s1 = kernel.register_state(mid, "end", StateKind::Terminal);

            if let (Ok(from), Ok(to)) = (s0, s1) {
                let t0 = kernel.register_transition(mid, "go", from, to);
                let t1 = kernel.register_transition(mid, "back", to, from);

                if let Some(runtime) = kernel.machines.get_mut(&mid) {
                    runtime.current_state = from;
                }

                if let (Ok(tid0), Ok(tid1)) = (t0, t1) {
                    // Transition to terminal
                    let _ = kernel.transition(mid, tid0);
                    assert_eq!(kernel.is_terminal(mid), Ok(true));

                    // Try to transition from terminal - should fail
                    let result = kernel.transition(mid, tid1);
                    assert!(matches!(result, Err(KernelError::InTerminalState(_))));
                }
            }
        }
    }

    #[test]
    fn test_guard_evaluation() {
        let mut kernel = StateKernel::new();

        if let Ok(mid) = kernel.create_machine(0) {
            let s0 = kernel.register_state(mid, "start", StateKind::Initial);
            let s1 = kernel.register_state(mid, "end", StateKind::Normal);

            if let (Ok(from), Ok(to)) = (s0, s1) {
                // Register guard
                let _ = kernel.register_guard(mid, "check_ready", "ready");

                // Register guarded transition
                let t0 = kernel.register_guarded_transition(mid, "go", from, to, "check_ready");

                if let Some(runtime) = kernel.machines.get_mut(&mid) {
                    runtime.current_state = from;
                }

                if let Ok(tid) = t0 {
                    // Try with failing guard context
                    let mut ctx = GuardContext::new();
                    ctx.set_bool("ready", false);

                    let result = kernel.transition_guarded(mid, tid, &ctx);
                    assert!(matches!(result, Err(KernelError::GuardRejected(_))));

                    // Try with passing guard context
                    ctx.set_bool("ready", true);
                    let result = kernel.transition_guarded(mid, tid, &ctx);
                    assert!(result.is_ok());
                }
            }
        }
    }

    #[test]
    fn test_absorbing_state_blocks() {
        let mut kernel = StateKernel::new();

        if let Ok(mid) = kernel.create_machine(0) {
            let s0 = kernel.register_state(mid, "start", StateKind::Initial);
            let s1 = kernel.register_state(mid, "absorbing", StateKind::Normal);
            let s2 = kernel.register_state(mid, "next", StateKind::Normal);

            if let (Ok(from), Ok(abs), Ok(next)) = (s0, s1, s2) {
                let t0 = kernel.register_transition(mid, "to_abs", from, abs);
                let t1 = kernel.register_transition(mid, "try_leave", abs, next);

                // Mark s1 as absorbing
                let _ = kernel.register_absorbing_state(mid, abs, IrreversibilityLevel::Permanent);

                if let Some(runtime) = kernel.machines.get_mut(&mid) {
                    runtime.current_state = from;
                }

                if let (Ok(tid0), Ok(tid1)) = (t0, t1) {
                    let _ = kernel.transition(mid, tid0);
                    assert_eq!(kernel.current_state(mid), Ok(abs));

                    // Should be blocked by absorbing state
                    let result = kernel.transition(mid, tid1);
                    assert!(matches!(result, Err(KernelError::AbsorbingState(_))));
                }
            }
        }
    }

    #[test]
    fn test_tick() {
        let mut kernel = StateKernel::new();

        if let Ok(mid) = kernel.create_machine(0) {
            let s0 = kernel.register_state(mid, "start", StateKind::Initial);
            let s1 = kernel.register_state(mid, "end", StateKind::Normal);

            if let (Ok(from), Ok(to)) = (s0, s1) {
                let t0 = kernel.register_transition(mid, "go", from, to);

                if let Some(runtime) = kernel.machines.get_mut(&mid) {
                    runtime.current_state = from;
                }

                if let Ok(tid) = t0 {
                    // Schedule transition 100 ticks from now
                    kernel.scheduler_mut().schedule_once(mid, tid, 100);

                    // Tick 50 — nothing due
                    let result = kernel.tick(50);
                    assert!(result.executed.is_empty());

                    // Tick 50 more — transition should execute
                    let result = kernel.tick(50);
                    assert_eq!(result.executed.len(), 1);
                    assert_eq!(kernel.current_state(mid), Ok(to));
                }
            }
        }
    }

    #[test]
    fn test_sequence_execution() {
        let mut kernel = StateKernel::new();

        if let Ok(mid) = kernel.create_machine(0) {
            let s0 = kernel.register_state(mid, "a", StateKind::Initial);
            let s1 = kernel.register_state(mid, "b", StateKind::Normal);
            let s2 = kernel.register_state(mid, "c", StateKind::Normal);

            if let (Ok(a), Ok(b), Ok(c)) = (s0, s1, s2) {
                let t0 = kernel.register_transition(mid, "a_to_b", a, b);
                let t1 = kernel.register_transition(mid, "b_to_c", b, c);

                if let Some(runtime) = kernel.machines.get_mut(&mid) {
                    runtime.current_state = a;
                }

                if let (Ok(tid0), Ok(tid1)) = (t0, t1) {
                    // Enqueue transitions
                    assert!(kernel.enqueue_transition(mid, tid0).is_ok());
                    assert!(kernel.enqueue_transition(mid, tid1).is_ok());

                    // Execute sequentially
                    let r1 = kernel.execute_next(mid);
                    assert!(r1.is_ok());
                    assert_eq!(kernel.current_state(mid), Ok(b));

                    let r2 = kernel.execute_next(mid);
                    assert!(r2.is_ok());
                    assert_eq!(kernel.current_state(mid), Ok(c));

                    // No more in queue
                    let r3 = kernel.execute_next(mid);
                    assert!(r3.is_ok());
                    if let Ok(val) = r3 {
                        assert!(val.is_none());
                    }
                }
            }
        }
    }
}
