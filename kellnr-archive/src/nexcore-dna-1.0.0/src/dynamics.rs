//! Dynamical Systems — Guardian Homeostasis as Discrete Control Theory.
//!
//! Models Guardian homeostasis as a formal discrete dynamical system `(S, U, Y, f, g)`:
//!
//! - **S** = {0, 1, ..., 7} -- 8 risk states (Safe -> Critical)
//! - **U** = {Signal, NoSignal} -- binary input
//! - **Y** = (AlertLevel, ResponseAction, amplified_severity) -- observable output
//! - **f(s, Signal)** = min(s+1, 7) -- escalate on signal
//! - **f(s, NoSignal)** = max(s-1, 0) -- decay toward safe
//! - **g(s)** = (alert_level(s), response_action(s), severity(s)) -- output function
//!
//! ## Lyapunov Stability
//!
//! V(s) = s^2 is a Lyapunov function. Under NoSignal:
//! - Delta_V = (s-1)^2 - s^2 = -2s + 1 < 0 for all s >= 1
//! - V(0) = 0 (equilibrium), V(s) > 0 for s > 0
//! - Globally asymptotically stable under decay. QED.
//!
//! ## Type Taxonomy
//!
//! | Type | Primitives | Tier | Dominant |
//! |------|-----------|------|----------|
//! | `RiskState` | `ς, N` | T2-P | ς State |
//! | `HomeostasisAlert` | `ς` | T1 | ς State |
//! | `SystemInput` | `Σ, ∂` | T2-P | Σ Sum |
//! | `ResponseAction` | `→, ς` | T2-P | → Causality |
//! | `SystemOutput` | `ς, N, →` | T2-P | → Causality |
//! | `HomeostaticSystem` | `ς, σ, μ, κ, N, →, ∂, ν` | T3 | ς State |
//! | `LyapunovResult` | `N, κ, →` | T2-P | N Quantity |
//! | `ControllabilityResult` | `μ, σ, κ, ∂` | T2-C | μ Mapping |
//! | `ObservabilityResult` | `κ, μ, ∃, ∂` | T2-C | κ Comparison |
//! | `Trajectory` | `σ, ς, →` | T2-P | σ Sequence |
//! | `PhasePortrait` | `σ, ς, μ, →, ∂, κ, N` | T3 | σ Sequence |
//! | `BifurcationResult` | `N, ∂, κ, ς` | T2-C | ∂ Boundary |

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of discrete risk states in the Guardian homeostasis model.
pub const STATE_COUNT: usize = 8;

/// Minimum severity amplifier (state 0 = safe).
pub const AMP_MIN: f64 = 0.0;

/// Maximum severity amplifier (state 7 = critical).
pub const AMP_MAX: f64 = 10.0;

/// Default decay rate: each NoSignal step reduces state by 1.
pub const DECAY_RATE: usize = 1;

/// Default escalation rate: each Signal step increases state by 1.
pub const ESCALATION_RATE: usize = 1;

/// Default amplification threshold for bifurcation analysis.
pub const DEFAULT_THRESHOLD: f64 = 3.0;

/// Number of inputs in the binary input alphabet.
pub const INPUT_COUNT: usize = 2;

// ---------------------------------------------------------------------------
// RiskState — newtype over usize, bounded [0, STATE_COUNT-1]
// ---------------------------------------------------------------------------

/// Discrete risk state in range [0, 7].
///
/// - 0 = Safe (no active risk)
/// - 7 = Critical (maximum risk, immediate action required)
///
/// Tier: T2-P (ς State + N Quantity)
/// Dominant: ς State (represents system state at a point in time)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RiskState(usize);

impl RiskState {
    /// Create a new `RiskState`, clamped to [0, STATE_COUNT-1].
    #[must_use]
    pub const fn new(value: usize) -> Self {
        if value >= STATE_COUNT {
            Self(STATE_COUNT - 1)
        } else {
            Self(value)
        }
    }

    /// The safe equilibrium state (0).
    #[must_use]
    pub const fn safe() -> Self {
        Self(0)
    }

    /// The critical state (STATE_COUNT - 1).
    #[must_use]
    pub const fn critical() -> Self {
        Self(STATE_COUNT - 1)
    }

    /// Inner value.
    #[must_use]
    pub const fn value(self) -> usize {
        self.0
    }

    /// Lyapunov function: V(s) = s^2.
    #[must_use]
    pub fn lyapunov(self) -> f64 {
        (self.0 as f64) * (self.0 as f64)
    }

    /// Escalate: min(s+1, 7).
    #[must_use]
    pub const fn escalate(self) -> Self {
        if self.0 >= STATE_COUNT - 1 {
            Self(STATE_COUNT - 1)
        } else {
            Self(self.0 + 1)
        }
    }

    /// Decay: max(s-1, 0).
    #[must_use]
    pub const fn decay(self) -> Self {
        if self.0 == 0 {
            Self(0)
        } else {
            Self(self.0 - 1)
        }
    }
}

impl fmt::Display for RiskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "S{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// HomeostasisAlert — 4 alert levels mapped from risk state
// ---------------------------------------------------------------------------

/// Alert level derived from the current risk state.
///
/// Tier: T1 (ς State)
/// Dominant: ς State (discrete alert classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HomeostasisAlert {
    /// States 0-1: system is in safe equilibrium.
    Green,
    /// States 2-3: deviation detected, monitoring.
    Yellow,
    /// States 4-5: significant deviation, active response.
    Orange,
    /// States 6-7: critical deviation, immediate intervention.
    Red,
}

impl HomeostasisAlert {
    /// Derive alert level from risk state.
    #[must_use]
    pub fn from_risk_state(state: RiskState) -> Self {
        match state.value() {
            0..=1 => Self::Green,
            2..=3 => Self::Yellow,
            4..=5 => Self::Orange,
            _ => Self::Red,
        }
    }

    /// Numeric severity weight (0.0 = safe, 1.0 = critical).
    #[must_use]
    pub fn severity_weight(self) -> f64 {
        match self {
            Self::Green => 0.0,
            Self::Yellow => 0.33,
            Self::Orange => 0.66,
            Self::Red => 1.0,
        }
    }
}

impl fmt::Display for HomeostasisAlert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Green => write!(f, "GREEN"),
            Self::Yellow => write!(f, "YELLOW"),
            Self::Orange => write!(f, "ORANGE"),
            Self::Red => write!(f, "RED"),
        }
    }
}

// ---------------------------------------------------------------------------
// SystemInput — binary input alphabet
// ---------------------------------------------------------------------------

/// Binary input to the Guardian dynamical system.
///
/// Tier: T2-P (Σ Sum + ∂ Boundary)
/// Dominant: Σ Sum (exclusive disjunction: signal or no-signal)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemInput {
    /// A safety signal has been detected.
    Signal,
    /// No new signal — system decays toward equilibrium.
    NoSignal,
}

impl SystemInput {
    /// All possible inputs in enumeration order.
    pub const ALL: [SystemInput; INPUT_COUNT] = [SystemInput::Signal, SystemInput::NoSignal];

    /// Index for transition table lookup.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Signal => 0,
            Self::NoSignal => 1,
        }
    }
}

impl fmt::Display for SystemInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signal => write!(f, "Signal"),
            Self::NoSignal => write!(f, "NoSignal"),
        }
    }
}

// ---------------------------------------------------------------------------
// ResponseAction — 9 graduated response variants
// ---------------------------------------------------------------------------

/// Response action prescribed by the Guardian at each risk state.
///
/// Tier: T2-P (→ Causality + ς State)
/// Dominant: → Causality (each action causes a downstream effect)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResponseAction {
    /// State 0: No action needed — system is at equilibrium.
    NoAction,
    /// State 1: Passive observation, log only.
    Observe,
    /// State 2: Enhanced monitoring frequency.
    Monitor,
    /// State 3: Active investigation initiated.
    Investigate,
    /// State 4: Alert stakeholders, flag for review.
    Alert,
    /// State 5: Restrict operations, limit exposure.
    Restrict,
    /// State 6: Suspend operations pending review.
    Suspend,
    /// State 7: Emergency shutdown, escalate to human authority.
    Terminate,
}

impl ResponseAction {
    /// Derive response from risk state.
    #[must_use]
    pub fn from_risk_state(state: RiskState) -> Self {
        match state.value() {
            0 => Self::NoAction,
            1 => Self::Observe,
            2 => Self::Monitor,
            3 => Self::Investigate,
            4 => Self::Alert,
            5 => Self::Restrict,
            6 => Self::Suspend,
            _ => Self::Terminate,
        }
    }

    /// Numeric urgency weight (0.0 = none, 1.0 = emergency).
    #[must_use]
    pub fn urgency(self) -> f64 {
        match self {
            Self::NoAction => 0.0,
            Self::Observe => 1.0 / 7.0,
            Self::Monitor => 2.0 / 7.0,
            Self::Investigate => 3.0 / 7.0,
            Self::Alert => 4.0 / 7.0,
            Self::Restrict => 5.0 / 7.0,
            Self::Suspend => 6.0 / 7.0,
            Self::Terminate => 1.0,
        }
    }
}

impl fmt::Display for ResponseAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAction => write!(f, "NoAction"),
            Self::Observe => write!(f, "Observe"),
            Self::Monitor => write!(f, "Monitor"),
            Self::Investigate => write!(f, "Investigate"),
            Self::Alert => write!(f, "Alert"),
            Self::Restrict => write!(f, "Restrict"),
            Self::Suspend => write!(f, "Suspend"),
            Self::Terminate => write!(f, "Terminate"),
        }
    }
}

// ---------------------------------------------------------------------------
// SystemOutput — observable output triple
// ---------------------------------------------------------------------------

/// Observable output of the Guardian dynamical system at a given state.
///
/// Tier: T2-P (ς State + N Quantity + → Causality)
/// Dominant: → Causality (output drives downstream actions)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemOutput {
    /// Alert level classification.
    pub alert: HomeostasisAlert,
    /// Prescribed response action.
    pub action: ResponseAction,
    /// Amplified severity value: state * (threshold / STATE_COUNT).
    pub amplified_severity: f64,
}

impl fmt::Display for SystemOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Output(alert={}, action={}, severity={:.3})",
            self.alert, self.action, self.amplified_severity
        )
    }
}

// ---------------------------------------------------------------------------
// DiscreteSystem trait
// ---------------------------------------------------------------------------

/// Trait for discrete dynamical systems `(S, U, Y, f, g)`.
///
/// Provides the algebraic interface for state-space analysis:
/// controllability, observability, Lyapunov stability, and phase portraits.
pub trait DiscreteSystem {
    /// Number of states in S.
    fn state_count(&self) -> usize;

    /// The full input alphabet U.
    fn inputs(&self) -> &[SystemInput];

    /// State transition function f: S x U -> S.
    fn transition(&self, state: RiskState, input: SystemInput) -> RiskState;

    /// Output function g: S -> Y.
    fn output(&self, state: RiskState) -> SystemOutput;

    /// Lyapunov candidate function V: S -> R+.
    /// Default: V(s) = s^2.
    fn lyapunov(&self, state: RiskState) -> f64 {
        state.lyapunov()
    }
}

// ---------------------------------------------------------------------------
// HomeostaticSystem — concrete implementation
// ---------------------------------------------------------------------------

/// Guardian homeostasis modeled as a discrete dynamical system.
///
/// Pre-computes the 8x2 transition table for O(1) lookups.
///
/// Tier: T3 (ς + σ + μ + κ + N + → + ∂ + ν)
/// Dominant: ς State (system's core behavior is state evolution)
pub struct HomeostaticSystem {
    /// Pre-computed transition table: transitions[state][input_index].
    transitions: [[RiskState; INPUT_COUNT]; STATE_COUNT],
    /// Severity amplification threshold.
    threshold: f64,
}

impl HomeostaticSystem {
    /// Create with default threshold.
    #[must_use]
    pub fn new() -> Self {
        Self::with_threshold(DEFAULT_THRESHOLD)
    }

    /// Create with a custom severity threshold.
    #[must_use]
    pub fn with_threshold(threshold: f64) -> Self {
        let mut transitions = [[RiskState::safe(); INPUT_COUNT]; STATE_COUNT];

        for s in 0..STATE_COUNT {
            let state = RiskState::new(s);
            // Signal -> escalate
            transitions[s][SystemInput::Signal.index()] = state.escalate();
            // NoSignal -> decay
            transitions[s][SystemInput::NoSignal.index()] = state.decay();
        }

        Self {
            transitions,
            threshold,
        }
    }

    /// Get the severity threshold.
    #[must_use]
    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    /// Compute amplified severity for a state.
    #[must_use]
    pub fn amplified_severity(&self, state: RiskState) -> f64 {
        (state.value() as f64) * (self.threshold / STATE_COUNT as f64)
    }
}

impl Default for HomeostaticSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscreteSystem for HomeostaticSystem {
    fn state_count(&self) -> usize {
        STATE_COUNT
    }

    fn inputs(&self) -> &[SystemInput] {
        &SystemInput::ALL
    }

    fn transition(&self, state: RiskState, input: SystemInput) -> RiskState {
        self.transitions[state.value()][input.index()]
    }

    fn output(&self, state: RiskState) -> SystemOutput {
        SystemOutput {
            alert: HomeostasisAlert::from_risk_state(state),
            action: ResponseAction::from_risk_state(state),
            amplified_severity: self.amplified_severity(state),
        }
    }
}

impl fmt::Display for HomeostaticSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HomeostaticSystem(states={}, inputs={}, threshold={:.2})",
            STATE_COUNT, INPUT_COUNT, self.threshold
        )
    }
}

// ---------------------------------------------------------------------------
// LyapunovResult — stability analysis output
// ---------------------------------------------------------------------------

/// Result of Lyapunov stability analysis over all states.
///
/// Tier: T2-P (N Quantity + κ Comparison + → Causality)
/// Dominant: N Quantity (V(s) values and Delta_V are numeric)
#[derive(Debug, Clone, PartialEq)]
pub struct LyapunovResult {
    /// V(s) for each state.
    pub values: Vec<f64>,
    /// Delta_V(s) = V(f(s, NoSignal)) - V(s) for each state.
    pub deltas: Vec<f64>,
    /// True if the system is stable under decay (Delta_V < 0 for all s >= 1).
    pub stable: bool,
    /// The equilibrium state (V(s) = 0).
    pub equilibrium: RiskState,
}

impl fmt::Display for LyapunovResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lyapunov(stable={}, equilibrium={}, V_max={:.1})",
            self.stable,
            self.equilibrium,
            self.values.iter().copied().fold(0.0_f64, f64::max)
        )
    }
}

// ---------------------------------------------------------------------------
// ControllabilityResult — reachability analysis
// ---------------------------------------------------------------------------

/// Result of controllability analysis: can every state reach every other state?
///
/// Tier: T2-C (μ Mapping + σ Sequence + κ Comparison + ∂ Boundary)
/// Dominant: μ Mapping (reachability matrix maps state pairs to steps)
#[derive(Debug, Clone, PartialEq)]
pub struct ControllabilityResult {
    /// reachability[i][j] = minimum steps from state i to state j.
    /// None if unreachable.
    pub reachability: Vec<Vec<Option<usize>>>,
    /// True if the system is fully controllable (all pairs reachable).
    pub controllable: bool,
    /// Maximum steps needed between any pair (diameter of the reachability graph).
    pub max_steps: usize,
}

impl fmt::Display for ControllabilityResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Controllability(controllable={}, diameter={})",
            self.controllable, self.max_steps
        )
    }
}

// ---------------------------------------------------------------------------
// ObservabilityResult — output distinguishability
// ---------------------------------------------------------------------------

/// Result of observability analysis: are all states distinguishable by output?
///
/// Tier: T2-C (κ Comparison + μ Mapping + ∃ Existence + ∂ Boundary)
/// Dominant: κ Comparison (comparing outputs for distinctness)
#[derive(Debug, Clone, PartialEq)]
pub struct ObservabilityResult {
    /// True if all states produce distinct outputs (g is injective).
    pub observable: bool,
    /// Number of distinct output equivalence classes.
    pub equivalence_classes: usize,
    /// Partition: maps each state to its equivalence class index.
    pub partition: Vec<usize>,
}

impl fmt::Display for ObservabilityResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Observability(observable={}, classes={})",
            self.observable, self.equivalence_classes
        )
    }
}

// ---------------------------------------------------------------------------
// Trajectory — state sequence under a fixed input
// ---------------------------------------------------------------------------

/// A trajectory through state space under a fixed input.
///
/// Tier: T2-P (σ Sequence + ς State + → Causality)
/// Dominant: σ Sequence (ordered progression of states)
#[derive(Debug, Clone, PartialEq)]
pub struct Trajectory {
    /// The starting state.
    pub initial: RiskState,
    /// The input applied at each step.
    pub input: SystemInput,
    /// Sequence of states visited (including initial).
    pub states: Vec<RiskState>,
    /// The terminal state (fixed point or cycle entry).
    pub terminal: RiskState,
    /// Steps to reach the terminal state.
    pub steps_to_terminal: usize,
    /// Cycle length at the terminal (1 = fixed point).
    pub cycle_length: usize,
}

impl fmt::Display for Trajectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path: Vec<String> = self.states.iter().map(|s| format!("{s}")).collect();
        write!(
            f,
            "Trajectory({}, {}: [{}] -> {} in {} steps, cycle={})",
            self.initial,
            self.input,
            path.join(" -> "),
            self.terminal,
            self.steps_to_terminal,
            self.cycle_length
        )
    }
}

// ---------------------------------------------------------------------------
// PhasePortrait — complete trajectory map
// ---------------------------------------------------------------------------

/// Phase portrait: all trajectories from all initial states under all inputs.
///
/// Tier: T3 (σ + ς + μ + → + ∂ + κ + N)
/// Dominant: σ Sequence (collection of ordered trajectories)
#[derive(Debug, Clone, PartialEq)]
pub struct PhasePortrait {
    /// Trajectories indexed by (initial_state, input).
    pub trajectories: Vec<Trajectory>,
    /// Number of distinct fixed points found.
    pub fixed_points: Vec<RiskState>,
    /// True if all trajectories converge to a fixed point (cycle_length = 1).
    pub all_converge: bool,
}

impl fmt::Display for PhasePortrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PhasePortrait(trajectories={}, fixed_points={}, all_converge={})",
            self.trajectories.len(),
            self.fixed_points.len(),
            self.all_converge
        )
    }
}

// ---------------------------------------------------------------------------
// BifurcationResult — stability as threshold varies
// ---------------------------------------------------------------------------

/// Result of bifurcation analysis: how stability changes with the threshold parameter.
///
/// Tier: T2-C (N Quantity + ∂ Boundary + κ Comparison + ς State)
/// Dominant: ∂ Boundary (tracks where stability boundaries shift)
#[derive(Debug, Clone, PartialEq)]
pub struct BifurcationResult {
    /// (threshold, stable) pairs.
    pub points: Vec<(f64, bool)>,
    /// True if the system remains stable across the entire range.
    pub uniformly_stable: bool,
    /// Threshold values at which stability changes (bifurcation points).
    pub bifurcation_points: Vec<f64>,
}

impl fmt::Display for BifurcationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bifurcation(points={}, uniform={}, bifurcations={})",
            self.points.len(),
            self.uniformly_stable,
            self.bifurcation_points.len()
        )
    }
}

// ---------------------------------------------------------------------------
// Analysis Functions
// ---------------------------------------------------------------------------

/// Lyapunov stability analysis for a discrete system.
///
/// Computes V(s) = s^2 for all states and verifies Delta_V < 0 under NoSignal
/// for all s >= 1, proving global asymptotic stability toward the safe state.
#[must_use]
pub fn lyapunov_analysis(system: &dyn DiscreteSystem) -> LyapunovResult {
    let n = system.state_count();
    let mut values = Vec::with_capacity(n);
    let mut deltas = Vec::with_capacity(n);

    for s in 0..n {
        let state = RiskState::new(s);
        let v_s = system.lyapunov(state);
        values.push(v_s);

        let next = system.transition(state, SystemInput::NoSignal);
        let v_next = system.lyapunov(next);
        deltas.push(v_next - v_s);
    }

    // Stable if: V(0) = 0 (equilibrium) and Delta_V < 0 for all s >= 1
    let stable = (values[0].abs() < f64::EPSILON)
        && deltas.iter().skip(1).all(|&d| d < 0.0);

    LyapunovResult {
        values,
        deltas,
        stable,
        equilibrium: RiskState::safe(),
    }
}

/// Controllability analysis via BFS reachability.
///
/// For each pair (i, j), computes the minimum number of input steps to
/// reach state j from state i. The system is fully controllable if all
/// pairs are reachable.
#[must_use]
pub fn controllability(system: &dyn DiscreteSystem) -> ControllabilityResult {
    let n = system.state_count();
    let inputs = system.inputs();
    let mut reachability = vec![vec![None; n]; n];

    for start in 0..n {
        // BFS from `start`
        reachability[start][start] = Some(0);
        let mut frontier = vec![RiskState::new(start)];
        let mut step = 0usize;

        while !frontier.is_empty() {
            step += 1;
            let mut next_frontier = Vec::new();

            for &s in &frontier {
                for &input in inputs {
                    let t = system.transition(s, input);
                    let tv = t.value();
                    if reachability[start][tv].is_none() {
                        reachability[start][tv] = Some(step);
                        next_frontier.push(t);
                    }
                }
            }

            frontier = next_frontier;
        }
    }

    let controllable = reachability
        .iter()
        .all(|row| row.iter().all(|cell| cell.is_some()));

    let max_steps = reachability
        .iter()
        .flat_map(|row| row.iter())
        .filter_map(|&cell| cell)
        .max()
        .unwrap_or(0);

    ControllabilityResult {
        reachability,
        controllable,
        max_steps,
    }
}

/// Observability analysis via partition refinement.
///
/// Groups states by their output, then checks if all states are distinguishable.
/// The system is observable if g is injective (each state has a unique output).
#[must_use]
pub fn observability(system: &dyn DiscreteSystem) -> ObservabilityResult {
    let n = system.state_count();

    // Compute outputs for all states
    let outputs: Vec<SystemOutput> = (0..n)
        .map(|s| system.output(RiskState::new(s)))
        .collect();

    // Group states by output equivalence
    // Two outputs are equivalent if alert, action, and severity match
    let mut classes: Vec<Vec<usize>> = Vec::new();
    let mut partition = vec![0usize; n];

    for s in 0..n {
        let mut found = false;
        for (class_idx, class) in classes.iter().enumerate() {
            // Compare with first member of the class
            let representative = class[0];
            if outputs[s].alert == outputs[representative].alert
                && outputs[s].action == outputs[representative].action
                && (outputs[s].amplified_severity - outputs[representative].amplified_severity).abs()
                    < f64::EPSILON
            {
                partition[s] = class_idx;
                found = true;
                break;
            }
        }
        if !found {
            partition[s] = classes.len();
            classes.push(vec![s]);
        } else {
            classes[partition[s]].push(s);
        }
    }

    let equivalence_classes = classes.len();
    let observable = equivalence_classes == n;

    ObservabilityResult {
        observable,
        equivalence_classes,
        partition,
    }
}

/// Trace a single trajectory from a given initial state under a fixed input.
///
/// Stops when a cycle is detected (state revisited).
#[must_use]
pub fn trace_trajectory(
    system: &dyn DiscreteSystem,
    initial: RiskState,
    input: SystemInput,
) -> Trajectory {
    let mut states = vec![initial];
    let mut visited: HashMap<usize, usize> = HashMap::new();
    visited.insert(initial.value(), 0);

    let mut current = initial;

    loop {
        let next = system.transition(current, input);
        let next_val = next.value();

        if let Some(&cycle_start) = visited.get(&next_val) {
            // Cycle detected: terminal is `next`, cycle length = current_step - cycle_start + 1
            let steps = states.len() - 1; // steps taken so far
            let cycle_length = states.len() - cycle_start;

            return Trajectory {
                initial,
                input,
                states,
                terminal: next,
                steps_to_terminal: steps,
                cycle_length,
            };
        }

        visited.insert(next_val, states.len());
        states.push(next);
        current = next;

        // Safety: with STATE_COUNT=8, we cannot loop more than 8 times
        if states.len() > STATE_COUNT + 1 {
            break;
        }
    }

    // Fallback (should not reach here for finite state systems)
    let terminal = *states.last().unwrap_or(&initial);
    Trajectory {
        initial,
        input,
        states,
        terminal,
        steps_to_terminal: 0,
        cycle_length: 1,
    }
}

/// Compute the complete phase portrait: all trajectories from all states under all inputs.
#[must_use]
pub fn phase_portrait(system: &dyn DiscreteSystem) -> PhasePortrait {
    let n = system.state_count();
    let inputs = system.inputs();
    let mut trajectories = Vec::with_capacity(n * inputs.len());
    let mut fixed_points_set: Vec<RiskState> = Vec::new();

    for s in 0..n {
        for &input in inputs {
            let traj = trace_trajectory(system, RiskState::new(s), input);

            // Track fixed points (cycle_length == 1 means terminal is a fixed point)
            if traj.cycle_length == 1
                && !fixed_points_set.contains(&traj.terminal)
            {
                fixed_points_set.push(traj.terminal);
            }

            trajectories.push(traj);
        }
    }

    fixed_points_set.sort();

    let all_converge = trajectories.iter().all(|t| t.cycle_length == 1);

    PhasePortrait {
        trajectories,
        fixed_points: fixed_points_set,
        all_converge,
    }
}

/// Bifurcation scan: sweep the threshold parameter and check Lyapunov stability at each point.
///
/// The dynamical structure (transition function) doesn't change with threshold,
/// so stability is invariant. This verifies that invariance empirically.
#[must_use]
pub fn bifurcation_scan(low: f64, high: f64, steps: usize) -> BifurcationResult {
    let mut points = Vec::with_capacity(steps + 1);
    let mut bifurcation_points = Vec::new();
    let mut prev_stable: Option<bool> = None;

    let step_size = if steps > 0 {
        (high - low) / steps as f64
    } else {
        0.0
    };

    for i in 0..=steps {
        let threshold = low + step_size * i as f64;
        let system = HomeostaticSystem::with_threshold(threshold);
        let result = lyapunov_analysis(&system);
        let stable = result.stable;

        if let Some(was_stable) = prev_stable {
            if was_stable != stable {
                bifurcation_points.push(threshold);
            }
        }
        prev_stable = Some(stable);
        points.push((threshold, stable));
    }

    let uniformly_stable = points.iter().all(|(_, s)| *s);

    BifurcationResult {
        points,
        uniformly_stable,
        bifurcation_points,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- RiskState ---

    #[test]
    fn risk_state_clamp() {
        assert_eq!(RiskState::new(0).value(), 0);
        assert_eq!(RiskState::new(7).value(), 7);
        assert_eq!(RiskState::new(100).value(), 7); // clamped
    }

    #[test]
    fn risk_state_safe_and_critical() {
        assert_eq!(RiskState::safe().value(), 0);
        assert_eq!(RiskState::critical().value(), STATE_COUNT - 1);
    }

    #[test]
    fn risk_state_escalate_decay() {
        assert_eq!(RiskState::new(0).escalate().value(), 1);
        assert_eq!(RiskState::new(7).escalate().value(), 7); // clamped
        assert_eq!(RiskState::new(0).decay().value(), 0); // clamped
        assert_eq!(RiskState::new(5).decay().value(), 4);
    }

    #[test]
    fn risk_state_lyapunov_values() {
        assert!((RiskState::new(0).lyapunov() - 0.0).abs() < f64::EPSILON);
        assert!((RiskState::new(3).lyapunov() - 9.0).abs() < f64::EPSILON);
        assert!((RiskState::new(7).lyapunov() - 49.0).abs() < f64::EPSILON);
    }

    #[test]
    fn risk_state_display() {
        assert_eq!(format!("{}", RiskState::new(3)), "S3");
    }

    // --- HomeostasisAlert ---

    #[test]
    fn alert_from_risk_state_mapping() {
        assert_eq!(
            HomeostasisAlert::from_risk_state(RiskState::new(0)),
            HomeostasisAlert::Green
        );
        assert_eq!(
            HomeostasisAlert::from_risk_state(RiskState::new(1)),
            HomeostasisAlert::Green
        );
        assert_eq!(
            HomeostasisAlert::from_risk_state(RiskState::new(2)),
            HomeostasisAlert::Yellow
        );
        assert_eq!(
            HomeostasisAlert::from_risk_state(RiskState::new(5)),
            HomeostasisAlert::Orange
        );
        assert_eq!(
            HomeostasisAlert::from_risk_state(RiskState::new(7)),
            HomeostasisAlert::Red
        );
    }

    #[test]
    fn alert_severity_weight_monotonic() {
        let weights = [
            HomeostasisAlert::Green.severity_weight(),
            HomeostasisAlert::Yellow.severity_weight(),
            HomeostasisAlert::Orange.severity_weight(),
            HomeostasisAlert::Red.severity_weight(),
        ];
        for i in 0..3 {
            assert!(weights[i] < weights[i + 1]);
        }
    }

    #[test]
    fn alert_display() {
        assert_eq!(format!("{}", HomeostasisAlert::Green), "GREEN");
        assert_eq!(format!("{}", HomeostasisAlert::Red), "RED");
    }

    // --- SystemInput ---

    #[test]
    fn system_input_all_count() {
        assert_eq!(SystemInput::ALL.len(), INPUT_COUNT);
    }

    #[test]
    fn system_input_indices_distinct() {
        assert_ne!(
            SystemInput::Signal.index(),
            SystemInput::NoSignal.index()
        );
    }

    #[test]
    fn system_input_display() {
        assert_eq!(format!("{}", SystemInput::Signal), "Signal");
        assert_eq!(format!("{}", SystemInput::NoSignal), "NoSignal");
    }

    // --- ResponseAction ---

    #[test]
    fn response_action_from_all_states() {
        let actions: Vec<ResponseAction> = (0..STATE_COUNT)
            .map(|s| ResponseAction::from_risk_state(RiskState::new(s)))
            .collect();
        // All distinct
        for i in 0..actions.len() {
            for j in (i + 1)..actions.len() {
                assert_ne!(actions[i], actions[j], "actions [{i}] == [{j}]");
            }
        }
    }

    #[test]
    fn response_action_urgency_monotonic() {
        let urgencies: Vec<f64> = (0..STATE_COUNT)
            .map(|s| ResponseAction::from_risk_state(RiskState::new(s)).urgency())
            .collect();
        for i in 0..urgencies.len() - 1 {
            assert!(urgencies[i] < urgencies[i + 1]);
        }
    }

    #[test]
    fn response_action_display() {
        assert_eq!(format!("{}", ResponseAction::NoAction), "NoAction");
        assert_eq!(format!("{}", ResponseAction::Terminate), "Terminate");
    }

    // --- HomeostaticSystem ---

    #[test]
    fn homeostatic_system_default_threshold() {
        let sys = HomeostaticSystem::new();
        assert!((sys.threshold() - DEFAULT_THRESHOLD).abs() < f64::EPSILON);
    }

    #[test]
    fn homeostatic_system_transition_signal() {
        let sys = HomeostaticSystem::new();
        for s in 0..STATE_COUNT {
            let state = RiskState::new(s);
            let next = sys.transition(state, SystemInput::Signal);
            if s < STATE_COUNT - 1 {
                assert_eq!(next.value(), s + 1);
            } else {
                assert_eq!(next.value(), STATE_COUNT - 1);
            }
        }
    }

    #[test]
    fn homeostatic_system_transition_nosignal() {
        let sys = HomeostaticSystem::new();
        for s in 0..STATE_COUNT {
            let state = RiskState::new(s);
            let next = sys.transition(state, SystemInput::NoSignal);
            if s > 0 {
                assert_eq!(next.value(), s - 1);
            } else {
                assert_eq!(next.value(), 0);
            }
        }
    }

    #[test]
    fn homeostatic_system_output_distinct() {
        let sys = HomeostaticSystem::new();
        let outputs: Vec<SystemOutput> = (0..STATE_COUNT)
            .map(|s| sys.output(RiskState::new(s)))
            .collect();
        // All outputs should be distinct (each state maps to unique action)
        for i in 0..outputs.len() {
            for j in (i + 1)..outputs.len() {
                assert_ne!(
                    outputs[i].action, outputs[j].action,
                    "output actions [{i}] == [{j}]"
                );
            }
        }
    }

    #[test]
    fn homeostatic_system_display() {
        let sys = HomeostaticSystem::new();
        let s = format!("{sys}");
        assert!(s.contains("HomeostaticSystem"));
        assert!(s.contains("states=8"));
    }

    // --- Lyapunov Analysis ---

    #[test]
    fn lyapunov_is_stable() {
        let sys = HomeostaticSystem::new();
        let result = lyapunov_analysis(&sys);
        assert!(result.stable, "system should be Lyapunov stable");
        assert_eq!(result.equilibrium, RiskState::safe());
    }

    #[test]
    fn lyapunov_values_correct() {
        let sys = HomeostaticSystem::new();
        let result = lyapunov_analysis(&sys);
        for s in 0..STATE_COUNT {
            let expected = (s as f64) * (s as f64);
            assert!(
                (result.values[s] - expected).abs() < f64::EPSILON,
                "V({s}) = {} != {expected}",
                result.values[s]
            );
        }
    }

    #[test]
    fn lyapunov_deltas_formula() {
        let sys = HomeostaticSystem::new();
        let result = lyapunov_analysis(&sys);
        // Delta_V(s) under NoSignal = (s-1)^2 - s^2 = -2s + 1
        for s in 0..STATE_COUNT {
            let expected = if s == 0 {
                0.0 // max(s-1, 0) = 0, so Delta_V = 0 - 0 = 0
            } else {
                -2.0 * (s as f64) + 1.0
            };
            assert!(
                (result.deltas[s] - expected).abs() < f64::EPSILON,
                "Delta_V({s}) = {} != {expected}",
                result.deltas[s]
            );
        }
    }

    #[test]
    fn lyapunov_display() {
        let sys = HomeostaticSystem::new();
        let result = lyapunov_analysis(&sys);
        let s = format!("{result}");
        assert!(s.contains("Lyapunov"));
        assert!(s.contains("stable=true"));
    }

    // --- Controllability ---

    #[test]
    fn controllability_fully_controllable() {
        let sys = HomeostaticSystem::new();
        let result = controllability(&sys);
        assert!(result.controllable, "system should be fully controllable");
    }

    #[test]
    fn controllability_diagonal_zero() {
        let sys = HomeostaticSystem::new();
        let result = controllability(&sys);
        for s in 0..STATE_COUNT {
            assert_eq!(result.reachability[s][s], Some(0));
        }
    }

    #[test]
    fn controllability_min_steps() {
        let sys = HomeostaticSystem::new();
        let result = controllability(&sys);
        // From state 0 to state 7: need exactly 7 Signal inputs
        assert_eq!(result.reachability[0][7], Some(7));
        // From state 7 to state 0: need exactly 7 NoSignal inputs
        assert_eq!(result.reachability[7][0], Some(7));
    }

    #[test]
    fn controllability_diameter() {
        let sys = HomeostaticSystem::new();
        let result = controllability(&sys);
        // Maximum distance is from 0->7 or 7->0 = 7 steps
        assert_eq!(result.max_steps, 7);
    }

    #[test]
    fn controllability_display() {
        let sys = HomeostaticSystem::new();
        let result = controllability(&sys);
        let s = format!("{result}");
        assert!(s.contains("Controllability"));
        assert!(s.contains("controllable=true"));
    }

    // --- Observability ---

    #[test]
    fn observability_fully_observable() {
        let sys = HomeostaticSystem::new();
        let result = observability(&sys);
        assert!(result.observable, "system should be fully observable");
        assert_eq!(result.equivalence_classes, STATE_COUNT);
    }

    #[test]
    fn observability_partition_identity() {
        let sys = HomeostaticSystem::new();
        let result = observability(&sys);
        // Each state in its own class -> partition should be [0, 1, 2, ..., 7]
        for (i, &class) in result.partition.iter().enumerate() {
            assert_eq!(class, i, "state {i} in wrong class");
        }
    }

    #[test]
    fn observability_display() {
        let sys = HomeostaticSystem::new();
        let result = observability(&sys);
        let s = format!("{result}");
        assert!(s.contains("Observability"));
        assert!(s.contains("observable=true"));
    }

    // --- Phase Portrait ---

    #[test]
    fn phase_portrait_trajectory_count() {
        let sys = HomeostaticSystem::new();
        let portrait = phase_portrait(&sys);
        // 8 states x 2 inputs = 16 trajectories
        assert_eq!(portrait.trajectories.len(), STATE_COUNT * INPUT_COUNT);
    }

    #[test]
    fn phase_portrait_all_converge() {
        let sys = HomeostaticSystem::new();
        let portrait = phase_portrait(&sys);
        assert!(
            portrait.all_converge,
            "all trajectories should converge to fixed points"
        );
    }

    #[test]
    fn phase_portrait_fixed_points() {
        let sys = HomeostaticSystem::new();
        let portrait = phase_portrait(&sys);
        // Under NoSignal: fixed point at S0
        // Under Signal: fixed point at S7
        assert!(portrait.fixed_points.contains(&RiskState::safe()));
        assert!(portrait.fixed_points.contains(&RiskState::critical()));
    }

    #[test]
    fn phase_portrait_signal_convergence() {
        let sys = HomeostaticSystem::new();
        for s in 0..STATE_COUNT {
            let traj = trace_trajectory(&sys, RiskState::new(s), SystemInput::Signal);
            assert_eq!(traj.terminal, RiskState::critical());
        }
    }

    #[test]
    fn phase_portrait_decay_convergence() {
        let sys = HomeostaticSystem::new();
        for s in 0..STATE_COUNT {
            let traj = trace_trajectory(&sys, RiskState::new(s), SystemInput::NoSignal);
            assert_eq!(traj.terminal, RiskState::safe());
        }
    }

    #[test]
    fn trajectory_display() {
        let sys = HomeostaticSystem::new();
        let traj = trace_trajectory(&sys, RiskState::new(3), SystemInput::NoSignal);
        let s = format!("{traj}");
        assert!(s.contains("Trajectory"));
        assert!(s.contains("S3"));
    }

    #[test]
    fn phase_portrait_display() {
        let sys = HomeostaticSystem::new();
        let portrait = phase_portrait(&sys);
        let s = format!("{portrait}");
        assert!(s.contains("PhasePortrait"));
        assert!(s.contains("trajectories=16"));
    }

    // --- Bifurcation ---

    #[test]
    fn bifurcation_uniformly_stable() {
        let result = bifurcation_scan(0.5, 10.0, 200);
        assert!(
            result.uniformly_stable,
            "system should be stable across all thresholds"
        );
        assert!(
            result.bifurcation_points.is_empty(),
            "no bifurcation points expected"
        );
    }

    #[test]
    fn bifurcation_point_count() {
        let result = bifurcation_scan(0.5, 10.0, 200);
        assert_eq!(result.points.len(), 201);
    }

    #[test]
    fn bifurcation_display() {
        let result = bifurcation_scan(1.0, 5.0, 10);
        let s = format!("{result}");
        assert!(s.contains("Bifurcation"));
        assert!(s.contains("uniform=true"));
    }

    // --- SystemOutput ---

    #[test]
    fn system_output_display() {
        let output = SystemOutput {
            alert: HomeostasisAlert::Orange,
            action: ResponseAction::Restrict,
            amplified_severity: 2.5,
        };
        let s = format!("{output}");
        assert!(s.contains("Output"));
        assert!(s.contains("ORANGE"));
        assert!(s.contains("Restrict"));
    }
}
