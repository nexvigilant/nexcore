//! # Sentinel Constraint Propagation (ToV §33)
//!
//! Codex-compliant implementation of the Sentinel CSP framework for PV signal investigation.
//!
//! ## ToV Sections Implemented
//!
//! - §20: Signal Equation S = U × R × T
//! - §21: Unrepeatable Pattern (U) with U_NR threshold
//! - §22: Recognition Presence (R)
//! - §23: Temporal Window (T)
//! - §33: Sentinel Constraint Propagation
//!
//! ## Codex Compliance
//!
//! All types follow the Primitive Codex:
//! - **WRAP**: Domain values wrapped in newtypes
//! - **CLASSIFY**: Explicit tier annotations
//! - **GROUND**: Traces to T1 primitives via signal_equation.rs

use crate::signals::signal_equation::{
    RecognitionPresence, SignalStrength, TemporalWindow as TovTemporalWindow, UnrepeatablePattern,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// CONSTANTS - ToV §33 Signal Thresholds
// =============================================================================

/// Signal confirmation threshold (S ≥ 15.0 = confirmed signal)
/// Tier: T1 (universal constant)
pub const S_CONFIRM: f64 = 15.0;

/// High priority signal threshold (S ≥ 8.0 = high priority)
/// Tier: T1 (universal constant)
pub const S_HIGH: f64 = 8.0;

/// Moderate signal threshold (S ≥ 3.0 = moderate priority)
/// Tier: T1 (universal constant)
pub const S_MODERATE: f64 = 3.0;

/// Signal clearance threshold (S < 0.5 = cleared)
/// Tier: T1 (universal constant)
pub const S_CLEAR: f64 = 0.5;

/// Non-recurrence threshold in bits (U_NR ≈ 63 bits)
/// From ToV §21: "Beyond this threshold, pattern is considered non-recurring"
/// Tier: T1 (universal constant)
pub const U_NON_RECURRENCE_BITS: f64 = 63.0;

/// Safety margin threshold for silent failure detection (ToV §21.7)
/// If d(s) < D_SAFE, SILENT_RISK status applies
/// Tier: T1 (universal constant)
pub const D_SAFE_THRESHOLD: f64 = 0.1;

/// Universal Epistemic Prior for signal strength (S_min)
/// Tier: T1 (Axiomatic)
pub const EPISTEMIC_PRIOR_S: f64 = 0.001;

// =============================================================================
// ADJACENCY WEIGHTS - ToV §33 Specification
// =============================================================================

/// Adjacency weights per relationship type (ToV §33)
/// Tier: T1 (universal constants)
pub mod adjacency_weights {
    /// Mechanistic adjacency weight (same drug class/MoA)
    pub const MECHANISTIC: f64 = 0.35;
    /// Phenotypic adjacency weight (similar clinical presentation)
    pub const PHENOTYPIC: f64 = 0.25;
    /// Temporal adjacency weight (similar time-to-onset)
    pub const TEMPORAL: f64 = 0.20;
    /// Demographic adjacency weight (similar patient demographics)
    pub const DEMOGRAPHIC: f64 = 0.10;
    /// Concomitant adjacency weight (concomitant medication)
    pub const CONCOMITANT: f64 = 0.10;
}

// =============================================================================
// ENUMS - Tier: T2-C (Cross-Domain Composite)
// =============================================================================

/// Vigilance status for a Sentinel cell (ToV §33)
///
/// ## Tier: T2-C (Cross-Domain Composite)
///
/// Extends CellStatus to include propagated states and silent failure detection.
///
/// ## Status Transitions
///
/// ```text
/// UNKNOWN → INVESTIGATED → { FLAGGED → CONFIRMED
///                          { CLEARED
///                          { PROPAGATED_FLAG → CONFIRMED
///
/// UNKNOWN → SILENT_RISK (if unobservable and S > threshold)
/// FLAGGED → PROPAGATED_CLEAR (cleared via propagation)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SentinelStatus {
    /// Not yet investigated - initial state
    #[default]
    Unknown,
    /// Actively being investigated
    Investigated,
    /// Flagged for review (S ≥ S_MODERATE)
    Flagged,
    /// Confirmed safety signal (S ≥ S_CONFIRM)
    Confirmed,
    /// Flagged via propagation from neighbor (not direct evidence)
    PropagatedFlag,
    /// Cleared - evidence shows no signal (S < S_CLEAR)
    Cleared,
    /// Cleared via propagation from neighbor
    PropagatedClear,
    /// Silent risk - unobservable violation detected (ToV §21.7)
    /// Requires d(s) < D_SAFE and !observable
    SilentRisk,
}

impl SentinelStatus {
    /// Check if status requires attention
    #[must_use]
    pub const fn needs_attention(&self) -> bool {
        matches!(
            self,
            Self::Flagged | Self::Confirmed | Self::PropagatedFlag | Self::SilentRisk
        )
    }

    /// Check if status indicates cleared state
    #[must_use]
    pub const fn is_cleared(&self) -> bool {
        matches!(self, Self::Cleared | Self::PropagatedClear)
    }

    /// Check if status is terminal (no further transitions)
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Confirmed | Self::SilentRisk)
    }

    /// Display name for reporting
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Investigated => "Investigated",
            Self::Flagged => "Flagged",
            Self::Confirmed => "Confirmed",
            Self::PropagatedFlag => "Propagated-Flag",
            Self::Cleared => "Cleared",
            Self::PropagatedClear => "Propagated-Clear",
            Self::SilentRisk => "SILENT-RISK",
        }
    }
}

/// Adjacency type for cell relationships
///
/// ## Tier: T2-P (Cross-Domain Primitive)
///
/// Matches ToV §33 adjacency taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SentinelAdjacency {
    /// Same drug class or mechanism of action
    Mechanistic,
    /// Similar clinical presentation/phenotype
    Phenotypic,
    /// Similar time-to-onset pattern
    Temporal,
    /// Similar patient demographics
    Demographic,
    /// Concomitant medication relationship
    Concomitant,
}

impl SentinelAdjacency {
    /// Get default weight for this adjacency type
    #[must_use]
    pub const fn default_weight(&self) -> f64 {
        match self {
            Self::Mechanistic => adjacency_weights::MECHANISTIC,
            Self::Phenotypic => adjacency_weights::PHENOTYPIC,
            Self::Temporal => adjacency_weights::TEMPORAL,
            Self::Demographic => adjacency_weights::DEMOGRAPHIC,
            Self::Concomitant => adjacency_weights::CONCOMITANT,
        }
    }

    /// Get all adjacency types
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::Mechanistic,
            Self::Phenotypic,
            Self::Temporal,
            Self::Demographic,
            Self::Concomitant,
        ]
    }
}

// =============================================================================
// NEWTYPES - Tier: T2-P (Cross-Domain Primitive)
// =============================================================================

/// Safety margin d(s) for silent failure detection (ToV §21.7)
///
/// ## Tier: T2-P (Cross-Domain Primitive)
///
/// Measures distance from safety threshold. If d(s) < D_SAFE and !observable,
/// the cell enters SILENT_RISK status.
///
/// ## Formula
///
/// d(s) = |S - S_threshold| / S_threshold
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SafetyMargin(f64);

impl SafetyMargin {
    /// Create from signal strength and threshold
    #[must_use]
    pub fn calculate(signal: SignalStrength, threshold: f64) -> Self {
        let s = signal.value();
        let d = (threshold - s) / threshold.max(f64::EPSILON);
        Self(d)
    }

    /// Get the raw value
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.0
    }

    /// Check if safety margin is below safe threshold
    #[must_use]
    pub fn is_unsafe(&self) -> bool {
        self.0 < D_SAFE_THRESHOLD
    }
}

impl From<f64> for SafetyMargin {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

/// Observable flag indicating whether a signal can be directly measured
///
/// ## Tier: T2-P (Cross-Domain Primitive)
///
/// A cell is observable if:
/// - Direct case reports exist
/// - Signal can be validated through investigation
///
/// Unobservable cells with high signal strength enter SILENT_RISK status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Observable(bool);

impl Observable {
    /// Create observable cell
    #[must_use]
    pub const fn yes() -> Self {
        Self(true)
    }

    /// Create unobservable cell
    #[must_use]
    pub const fn no() -> Self {
        Self(false)
    }

    /// Check if observable
    #[must_use]
    pub const fn is_observable(&self) -> bool {
        self.0
    }
}

impl From<bool> for Observable {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

// =============================================================================
// SENTINEL CELL STATE - Tier: T3 (Domain-Specific)
// =============================================================================

/// Disproportionality scores for a cell (PRR, ROR, IC, EBGM)
///
/// ## Tier: T2-C (Cross-Domain Composite)
///
/// Wrapped scores to avoid naked primitives. Each field uses the appropriate
/// newtype from the signals module.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisproportionalityScores {
    /// PRR value (0 if not calculated)
    pub prr: f64,
    /// ROR value (0 if not calculated)
    pub ror: f64,
    /// IC value (can be negative)
    pub ic: f64,
    /// EBGM value (0 if not calculated)
    pub ebgm: f64,
}

/// Complete state for a Sentinel CSP cell (ToV §33)
///
/// ## Tier: T3 (Domain-Specific)
///
/// Contains all vigilance state fields:
/// - U (unrepeatable pattern measure)
/// - R (recognition presence)
/// - T (temporal window)
/// - S (signal strength = U × R × T)
/// - d (disproportionality scores)
/// - σ (uncertainty)
/// - status (vigilance status)
/// - observable (observability flag)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelCellState {
    /// Cell identifier (drug|event|population)
    pub id: String,
    /// Unrepeatable pattern measure (U) - ToV §21
    pub unrepeatable: UnrepeatablePattern,
    /// Recognition presence (R) - ToV §22
    pub recognition: RecognitionPresence,
    /// Temporal window (T) - ToV §23
    pub temporal: TovTemporalWindow,
    /// Signal strength (S = U × R × T) - ToV §20
    pub signal: SignalStrength,
    /// Disproportionality scores (PRR, ROR, IC, EBGM)
    pub disproportionality: DisproportionalityScores,
    /// Uncertainty measure (σ) - higher = more uncertain
    pub uncertainty: f64,
    /// Vigilance status
    pub status: SentinelStatus,
    /// Whether signal is directly observable
    pub observable: Observable,
    /// Safety margin d(s) for silent failure detection
    pub safety_margin: SafetyMargin,
    /// Evidence count
    pub evidence_count: u32,
}

impl SentinelCellState {
    /// Create a new cell state with default values
    #[must_use]
    pub fn new(drug: &str, event: &str, population: &str) -> Self {
        let id = format!("{}|{}|{}", drug, event, population);

        // Initialize with base values
        let unrepeatable = UnrepeatablePattern::from_ln1p_ratio(0, 1.0);
        let recognition = RecognitionPresence::from_dme_status(false, false);
        let temporal = TovTemporalWindow::from_elapsed(0, 30);
        let signal = SignalStrength::from_value(EPISTEMIC_PRIOR_S);
        let safety_margin = SafetyMargin::calculate(signal, S_MODERATE);

        Self {
            id,
            unrepeatable,
            recognition,
            temporal,
            signal,
            disproportionality: DisproportionalityScores::default(),
            uncertainty: 1.0,
            status: SentinelStatus::Unknown,
            observable: Observable::yes(),
            safety_margin,
            evidence_count: 0,
        }
    }

    /// Update signal components and recalculate S
    pub fn update_signal(
        &mut self,
        observed: u64,
        expected: f64,
        is_dme: bool,
        reported_previously: bool,
        days_since_first: u32,
        total_days: u32,
    ) {
        self.unrepeatable = UnrepeatablePattern::from_ln1p_ratio(observed, expected);
        self.recognition = RecognitionPresence::from_dme_status(is_dme, reported_previously);
        self.temporal = TovTemporalWindow::from_elapsed(days_since_first, total_days);
        self.signal = SignalStrength::calculate(self.unrepeatable, self.recognition, self.temporal);
        self.safety_margin = SafetyMargin::calculate(self.signal, S_MODERATE);
        self.evidence_count += 1;

        // Update status based on signal strength
        self.update_status();
    }

    /// Update disproportionality scores
    pub fn update_disproportionality(&mut self, prr: f64, ror: f64, ic: f64, ebgm: f64) {
        self.disproportionality = DisproportionalityScores { prr, ror, ic, ebgm };
    }

    /// Update status based on current signal strength and observability
    fn update_status(&mut self) {
        if self.status.is_terminal() {
            return;
        }

        let s = self.signal.value();

        // Check for silent risk (unobservable + crossed threshold)
        // Axiom 21.7: If d(s) < D_SAFE and !observable, SILENT_RISK applies.
        if !self.observable.is_observable()
            && self.safety_margin.value() < D_SAFE_THRESHOLD
            && s >= S_MODERATE
        {
            self.status = SentinelStatus::SilentRisk;
            return;
        }

        // Check for confirmation
        if s >= S_CONFIRM && self.uncertainty < 0.3 {
            self.status = SentinelStatus::Confirmed;
            return;
        }

        // Check for flagging
        if s >= S_MODERATE {
            match self.status {
                SentinelStatus::Unknown => self.status = SentinelStatus::PropagatedFlag,
                SentinelStatus::PropagatedFlag => {
                    // INVARIANT: Indirect evidence cannot promote to standard Flagged
                    // It stays PropagatedFlag until direct evidence arrives.
                }
                SentinelStatus::Investigated
                | SentinelStatus::Flagged
                | SentinelStatus::Cleared
                | SentinelStatus::PropagatedClear => {
                    self.status = SentinelStatus::Flagged;
                }
                _ => {}
            }
            return;
        }

        // Check for clearance
        if s < S_CLEAR {
            if self.status == SentinelStatus::Flagged {
                self.status = SentinelStatus::PropagatedClear;
            } else {
                self.status = SentinelStatus::Cleared;
            }
        }
    }

    /// Apply propagated signal update (log-linear propagation)
    ///
    /// ## Formula (ToV §33)
    ///
    /// ln(S') = ln(S_current) + w × ln(S_source)
    pub fn apply_propagated_signal(&mut self, source_signal: SignalStrength, weight: f64) {
        let current_ln_s = self.signal.value().ln();
        let source_ln_s = source_signal.value().ln();

        // Log-linear propagation
        let new_ln_s = current_ln_s + weight * source_ln_s;
        let new_s = new_ln_s.exp().max(0.0);

        // Create new signal strength
        self.signal = SignalStrength::from_value(new_s);
        self.safety_margin = SafetyMargin::calculate(self.signal, S_MODERATE);

        // Harmonized Update Logic: Always trigger status evaluation
        self.update_status();
    }

    /// Check if U exceeds non-recurrence threshold
    #[must_use]
    pub fn exceeds_non_recurrence(&self) -> bool {
        self.unrepeatable.value() >= U_NON_RECURRENCE_BITS
    }
}

// =============================================================================
// SENTINEL GRID - Tier: T3 (Domain-Specific)
// =============================================================================

/// Edge in adjacency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelEdge {
    /// Target cell ID
    pub target: String,
    /// Edge weight
    pub weight: f64,
    /// Adjacency type
    pub adjacency_type: SentinelAdjacency,
}

/// Sentinel CSP Grid for constraint propagation (ToV §33)
///
/// ## Tier: T3 (Domain-Specific)
///
/// Implements the Sentinel investigation framework with:
/// - Log-linear belief propagation
/// - Silent failure detection
/// - Codex-compliant state management
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SentinelGrid {
    /// Cells indexed by ID
    cells: HashMap<String, SentinelCellState>,
    /// Adjacency graph: cell_id -> edges
    adjacency: HashMap<String, Vec<SentinelEdge>>,
}

impl SentinelGrid {
    /// Create new empty grid
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a cell to the grid
    pub fn add_cell(
        &mut self,
        drug: &str,
        event: &str,
        population: &str,
    ) -> &mut SentinelCellState {
        let cell_id = format!("{}|{}|{}", drug, event, population);
        self.cells
            .entry(cell_id.clone())
            .or_insert_with(|| SentinelCellState::new(drug, event, population))
    }

    /// Get cell by ID
    #[must_use]
    pub fn get_cell(&self, cell_id: &str) -> Option<&SentinelCellState> {
        self.cells.get(cell_id)
    }

    /// Get mutable cell by ID
    #[must_use]
    pub fn get_cell_mut(&mut self, cell_id: &str) -> Option<&mut SentinelCellState> {
        self.cells.get_mut(cell_id)
    }

    /// Set adjacency between cells (bidirectional)
    pub fn set_adjacency(
        &mut self,
        cell_a: &str,
        cell_b: &str,
        adjacency_type: SentinelAdjacency,
        weight: Option<f64>,
    ) {
        let w = weight.unwrap_or_else(|| adjacency_type.default_weight());

        // Add edge A -> B
        self.adjacency
            .entry(cell_a.to_string())
            .or_default()
            .push(SentinelEdge {
                target: cell_b.to_string(),
                weight: w,
                adjacency_type,
            });

        // Add edge B -> A (symmetric)
        self.adjacency
            .entry(cell_b.to_string())
            .or_default()
            .push(SentinelEdge {
                target: cell_a.to_string(),
                weight: w,
                adjacency_type,
            });
    }

    /// Get neighbors of a cell
    #[must_use]
    pub fn get_neighbors(&self, cell_id: &str) -> &[SentinelEdge] {
        self.adjacency.get(cell_id).map_or(&[], |v| v.as_slice())
    }

    /// Propagate signal from source cell to neighbors (log-linear)
    ///
    /// ## Algorithm (ToV §33)
    ///
    /// 1. Get source signal S_source
    /// 2. For each neighbor with edge weight w:
    ///    ln(S_neighbor') = ln(S_neighbor) + w × ln(S_source)
    /// 3. Update neighbor status
    ///
    /// Returns list of newly flagged cell IDs
    pub fn propagate_signal(&mut self, source_id: &str) -> Vec<String> {
        // Get source signal
        let source_signal = match self.cells.get(source_id) {
            Some(cell) => cell.signal.clone(),
            None => return Vec::new(),
        };

        // Get neighbors (clone to avoid borrow issues)
        let neighbors: Vec<SentinelEdge> =
            self.adjacency.get(source_id).cloned().unwrap_or_default();

        let mut newly_flagged = Vec::new();

        for edge in neighbors {
            if let Some(neighbor) = self.cells.get_mut(&edge.target) {
                let was_flagged = neighbor.status.needs_attention();

                neighbor.apply_propagated_signal(source_signal, edge.weight);

                if neighbor.status.needs_attention() && !was_flagged {
                    newly_flagged.push(edge.target.clone());
                }
            }
        }

        newly_flagged
    }

    /// Get all cells with SILENT_RISK status
    #[must_use]
    pub fn silent_risk_cells(&self) -> Vec<&SentinelCellState> {
        self.cells
            .values()
            .filter(|c| c.status == SentinelStatus::SilentRisk)
            .collect()
    }

    /// Get all cells requiring attention
    #[must_use]
    pub fn attention_required(&self) -> Vec<&SentinelCellState> {
        self.cells
            .values()
            .filter(|c| c.status.needs_attention())
            .collect()
    }

    /// Get status summary
    #[must_use]
    pub fn status_summary(&self) -> HashMap<SentinelStatus, usize> {
        let mut summary = HashMap::new();
        for cell in self.cells.values() {
            *summary.entry(cell.status).or_insert(0) += 1;
        }
        summary
    }

    /// Get cell count
    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentinel_status_transitions() {
        assert!(!SentinelStatus::Unknown.needs_attention());
        assert!(SentinelStatus::Flagged.needs_attention());
        assert!(SentinelStatus::SilentRisk.needs_attention());
        assert!(SentinelStatus::Confirmed.is_terminal());
        assert!(SentinelStatus::SilentRisk.is_terminal());
        assert!(SentinelStatus::Cleared.is_cleared());
        assert!(SentinelStatus::PropagatedClear.is_cleared());
    }

    #[test]
    fn test_adjacency_weights() {
        assert!((SentinelAdjacency::Mechanistic.default_weight() - 0.35).abs() < f64::EPSILON);
        assert!((SentinelAdjacency::Temporal.default_weight() - 0.20).abs() < f64::EPSILON);
    }

    #[test]
    fn test_safety_margin() {
        // Signal well BELOW threshold: safe margin (haven't triggered yet)
        let low_signal = SignalStrength::from_value(1.5); // 50% of threshold
        let safe_margin = SafetyMargin::calculate(low_signal, S_MODERATE);
        // d = (3.0 - 1.5) / 3.0 = 0.5 > 0.1 = safe
        assert!(!safe_margin.is_unsafe());

        // Signal just past threshold: unsafe (triggered)
        let edge_signal = SignalStrength::from_value(S_MODERATE + 0.01);
        let edge_margin = SafetyMargin::calculate(edge_signal, S_MODERATE);
        // d = (3.0 - 3.01) / 3.0 = -0.003 < 0.1 = unsafe
        assert!(edge_margin.is_unsafe());

        // Signal well ABOVE threshold: unsafe (deep into triggered zone)
        let high_signal = SignalStrength::from_value(10.0);
        let high_margin = SafetyMargin::calculate(high_signal, S_MODERATE);
        // d = (3.0 - 10.0) / 3.0 = -2.33 < 0.1 = unsafe
        assert!(high_margin.is_unsafe());
    }

    #[test]
    fn test_sentinel_cell_creation() {
        let cell = SentinelCellState::new("aspirin", "gi_bleeding", "all");
        assert_eq!(cell.id, "aspirin|gi_bleeding|all");
        assert_eq!(cell.status, SentinelStatus::Unknown);
        assert!(cell.observable.is_observable());
    }

    #[test]
    fn test_signal_update() {
        let mut cell = SentinelCellState::new("drug_x", "event_y", "all");

        // Update with strong signal
        cell.update_signal(100, 10.0, true, false, 0, 30);

        assert!(cell.signal.value() > 0.0);
        assert_eq!(cell.evidence_count, 1);
    }

    #[test]
    fn test_log_linear_propagation() {
        let mut cell = SentinelCellState::new("drug_a", "event_x", "all");
        cell.signal = SignalStrength::from_value(1.0);

        let source_signal = SignalStrength::from_value(10.0);
        cell.apply_propagated_signal(source_signal, 0.5);

        // ln(S') = ln(1.0) + 0.5 * ln(10.0) = 0 + 0.5 * 2.303 = 1.15
        // S' = e^1.15 ≈ 3.16
        assert!(cell.signal.value() > 1.0);
        assert!(cell.signal.value() < 10.0);
    }

    #[test]
    fn test_sentinel_grid() {
        let mut grid = SentinelGrid::new();

        grid.add_cell("canagliflozin", "ketoacidosis", "all");
        grid.add_cell("dapagliflozin", "ketoacidosis", "all");

        grid.set_adjacency(
            "canagliflozin|ketoacidosis|all",
            "dapagliflozin|ketoacidosis|all",
            SentinelAdjacency::Mechanistic,
            None,
        );

        let neighbors = grid.get_neighbors("canagliflozin|ketoacidosis|all");
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].target, "dapagliflozin|ketoacidosis|all");
    }

    #[test]
    fn test_non_recurrence_threshold() {
        let mut cell = SentinelCellState::new("drug", "event", "all");

        // Very rare pattern: observed=1000, expected=1
        cell.unrepeatable = UnrepeatablePattern::from_ln1p_ratio(1000, 1.0);

        // U = ln(1 + 1000/1) = ln(1001) ≈ 6.9 (not exceeding 63 bits)
        assert!(!cell.exceeds_non_recurrence());
    }

    #[test]
    fn test_silent_risk_detection() {
        let mut cell = SentinelCellState::new("drug", "event", "all");

        // Set up: moderate signal, unobservable, unsafe margin
        cell.signal = SignalStrength::from_value(S_MODERATE + 0.05);
        cell.observable = Observable::no();
        cell.safety_margin = SafetyMargin::from(0.01); // Very unsafe
        cell.update_status();

        assert_eq!(cell.status, SentinelStatus::SilentRisk);
    }
}
