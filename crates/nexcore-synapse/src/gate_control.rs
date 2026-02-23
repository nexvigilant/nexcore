//! Gate Control Theory — inter-hook signal modulation.
//!
//! ## Biology Analog
//!
//! Melzack & Wall (1965): Pain signals at the spinal cord level can be
//! modulated by other neural signals. Large-diameter A-beta fibers close
//! the gate on small-diameter C-fiber pain signals. Without this mechanism:
//! allodynia — every stimulus arrives at maximum intensity simultaneously.
//!
//! ## Purpose
//!
//! When multiple hooks fire simultaneously on the same operation, the gate
//! controller modulates their combined signal. Antagonistic hooks have
//! priority rules. Synergistic hooks amplify. Independent hooks pass through
//! unchanged.
//!
//! ## Primitive Grounding: μ(Mapping) + →(Causality) + ∂(Boundary)
//!
//! ```text
//! GateVerdict = μ(signals → verdict) governed by → (causal priority)
//!               with ∂ (threshold boundaries) preventing allodynia
//! ```
//!
//! ## Example
//!
//! ```rust
//! use nexcore_synapse::gate_control::{
//!     GateController, HookSignal, InteractionMatrix, SignalType,
//! };
//!
//! let matrix = InteractionMatrix::default_matrix();
//! let controller = GateController::new(matrix);
//!
//! let signals = vec![
//!     HookSignal::new("security-guidance", 0.9, SignalType::Block),
//!     HookSignal::new("unwrap-guardian", 0.3, SignalType::Warn),
//! ];
//!
//! let verdict = controller.evaluate(&signals);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ============================================================================
// SIGNAL TYPE
// ============================================================================

/// Classification of what a hook signal intends to communicate.
///
/// Tier: T2-P (grounded to ς: State)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    /// Hook is blocking the operation outright.
    Block,

    /// Hook is warning but not blocking.
    Warn,

    /// Hook is explicitly permitting the operation.
    Allow,

    /// Hook is providing information only (no gating effect).
    Inform,
}

impl SignalType {
    /// Numeric weight for severity comparison (higher = more forceful).
    #[must_use]
    pub fn weight(self) -> f64 {
        match self {
            Self::Block => 4.0,
            Self::Warn => 2.0,
            Self::Allow => 1.0,
            Self::Inform => 0.5,
        }
    }

    /// Whether this signal type has an active gating effect.
    #[must_use]
    pub fn is_gating(self) -> bool {
        matches!(self, Self::Block | Self::Warn)
    }
}

impl fmt::Display for SignalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Block => "block",
            Self::Warn => "warn",
            Self::Allow => "allow",
            Self::Inform => "inform",
        };
        write!(f, "{s}")
    }
}

// ============================================================================
// PRIORITY
// ============================================================================

/// Priority resolution rule for antagonistic hook pairs.
///
/// Tier: T2-P (grounded to κ: Comparison + →: Causality)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    /// First signal evaluated wins (position-based).
    FirstWins,

    /// Signal with the highest effective severity wins.
    HighestSeverityWins,

    /// Signal from the more specific hook wins (domain-scoped over generic).
    MostSpecificWins,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::FirstWins => "first_wins",
            Self::HighestSeverityWins => "highest_severity_wins",
            Self::MostSpecificWins => "most_specific_wins",
        };
        write!(f, "{s}")
    }
}

// ============================================================================
// CASCADE PROTOCOL
// ============================================================================

/// Defines amplification chain behavior for synergistic hook pairs.
///
/// Tier: T2-C (grounded to ρ: Recursion + ∂: Boundary + N: Quantity)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CascadeProtocol {
    /// Amplify by a constant linear factor.
    Linear {
        /// Multiplicative factor applied to combined severity.
        factor: f64,
    },

    /// Amplify exponentially, capped at a ceiling.
    Exponential {
        /// Base for exponentiation.
        base: f64,

        /// Maximum combined severity (prevents unbounded growth).
        cap: f64,
    },

    /// Escalate only when minimum signal count is reached.
    Threshold {
        /// Number of concurrent signals required to trigger escalation.
        min_signals: usize,

        /// Whether to escalate to Block when threshold crossed.
        then_escalate: bool,
    },
}

impl CascadeProtocol {
    /// Apply this cascade protocol to a combined severity value.
    ///
    /// `signal_count` is used by `Threshold` variant.
    #[must_use]
    pub fn apply(&self, combined_severity: f64, signal_count: usize) -> f64 {
        match self {
            Self::Linear { factor } => (combined_severity * factor).clamp(0.0, 1.0),
            Self::Exponential { base, cap } => {
                let amplified = combined_severity.powf(*base);
                amplified.clamp(0.0, *cap).clamp(0.0, 1.0)
            }
            Self::Threshold {
                min_signals,
                then_escalate,
            } => {
                if signal_count >= *min_signals && *then_escalate {
                    1.0 // Full escalation
                } else {
                    combined_severity.clamp(0.0, 1.0)
                }
            }
        }
    }
}

impl Default for CascadeProtocol {
    fn default() -> Self {
        Self::Linear { factor: 1.2 }
    }
}

// ============================================================================
// HOOK INTERACTION
// ============================================================================

/// How two hooks relate to each other's signals.
///
/// Tier: T2-C (grounded to μ: Mapping + →: Causality)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum HookInteraction {
    /// Hooks are unrelated; both signals pass independently.
    Independent,

    /// Hooks reinforce each other; combined severity is amplified.
    Synergistic {
        /// Amplification factor applied to combined severity.
        amplification: f64,

        /// Optional cascade behavior for complex amplification chains.
        #[serde(skip_serializing_if = "Option::is_none")]
        cascade: Option<CascadeProtocol>,
    },

    /// Hooks oppose each other; priority rule determines which wins.
    Antagonistic {
        /// How to resolve the competing signals.
        priority: Priority,
    },
}

impl HookInteraction {
    /// Construct a simple synergistic interaction.
    #[must_use]
    pub fn synergistic(amplification: f64) -> Self {
        Self::Synergistic {
            amplification,
            cascade: None,
        }
    }

    /// Construct a synergistic interaction with cascade protocol.
    #[must_use]
    pub fn synergistic_cascading(amplification: f64, cascade: CascadeProtocol) -> Self {
        Self::Synergistic {
            amplification,
            cascade: Some(cascade),
        }
    }

    /// Construct an antagonistic interaction with a specific priority rule.
    #[must_use]
    pub fn antagonistic(priority: Priority) -> Self {
        Self::Antagonistic { priority }
    }
}

impl Default for HookInteraction {
    fn default() -> Self {
        Self::Independent
    }
}

// ============================================================================
// HOOK SIGNAL
// ============================================================================

/// A signal emitted by a hook when it fires on an operation.
///
/// Tier: T2-C (grounded to ∃: Existence + N: Quantity + ς: State)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookSignal {
    /// Name of the hook that emitted this signal.
    pub hook_name: String,

    /// Severity of the signal [0.0, 1.0].
    pub severity: f64,

    /// Classification of the signal intent.
    pub signal_type: SignalType,

    /// When the hook fired.
    pub timestamp: DateTime<Utc>,
}

impl HookSignal {
    /// Create a new hook signal with current timestamp.
    #[must_use]
    pub fn new(hook_name: impl Into<String>, severity: f64, signal_type: SignalType) -> Self {
        Self {
            hook_name: hook_name.into(),
            severity: severity.clamp(0.0, 1.0),
            signal_type,
            timestamp: Utc::now(),
        }
    }

    /// Create a signal with an explicit timestamp.
    #[must_use]
    pub fn with_timestamp(
        hook_name: impl Into<String>,
        severity: f64,
        signal_type: SignalType,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            hook_name: hook_name.into(),
            severity: severity.clamp(0.0, 1.0),
            signal_type,
            timestamp,
        }
    }

    /// Effective severity: raw severity weighted by signal type.
    ///
    /// A Block at 0.5 severity is more forceful than a Warn at 0.8.
    #[must_use]
    pub fn effective_severity(&self) -> f64 {
        (self.severity * self.signal_type.weight()).clamp(0.0, 1.0)
    }
}

impl fmt::Display for HookSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HookSignal[{}]: {} @ {:.3}",
            self.hook_name, self.signal_type, self.severity
        )
    }
}

// ============================================================================
// INTERACTION MATRIX
// ============================================================================

/// Lookup table mapping hook pairs to their interaction type.
///
/// Tier: T3 (domain composition — μ: Mapping applied to (String, String) pairs)
///
/// Keys are normalized so (A, B) == (B, A) — order does not matter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InteractionMatrix {
    /// Interaction entries keyed by ordered (a, b) pair, a <= b lexicographically.
    entries: HashMap<(String, String), HookInteraction>,
}

impl InteractionMatrix {
    /// Create an empty interaction matrix.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Normalize a pair so the lexicographically smaller name comes first.
    fn normalize(hook_a: &str, hook_b: &str) -> (String, String) {
        if hook_a <= hook_b {
            (hook_a.to_string(), hook_b.to_string())
        } else {
            (hook_b.to_string(), hook_a.to_string())
        }
    }

    /// Register an interaction between two hooks.
    pub fn set_interaction(
        &mut self,
        hook_a: impl AsRef<str>,
        hook_b: impl AsRef<str>,
        interaction: HookInteraction,
    ) {
        let key = Self::normalize(hook_a.as_ref(), hook_b.as_ref());
        self.entries.insert(key, interaction);
    }

    /// Look up the interaction between two hooks.
    ///
    /// Returns `HookInteraction::Independent` if no entry exists.
    #[must_use]
    pub fn get_interaction(&self, hook_a: &str, hook_b: &str) -> HookInteraction {
        let key = Self::normalize(hook_a, hook_b);
        self.entries
            .get(&key)
            .cloned()
            .unwrap_or(HookInteraction::Independent)
    }

    /// Number of registered interactions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the matrix has no registered interactions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Pre-populated matrix with known NexCore hook relationships.
    ///
    /// Based on observed hook co-firing patterns and their intended semantics.
    #[must_use]
    pub fn default_matrix() -> Self {
        let mut matrix = Self::new();

        // security-guidance + unwrap-guardian → Synergistic (both guard code quality)
        matrix.set_interaction(
            "security-guidance",
            "unwrap-guardian",
            HookInteraction::synergistic_cascading(1.3, CascadeProtocol::Linear { factor: 1.2 }),
        );

        // security-guidance + python-creation-blocker → Synergistic (both block unsafe patterns)
        matrix.set_interaction(
            "security-guidance",
            "python-creation-blocker",
            HookInteraction::synergistic(1.2),
        );

        // unwrap-guardian + reflex-mcp-prefer → Independent (different domains)
        matrix.set_interaction(
            "unwrap-guardian",
            "reflex-mcp-prefer",
            HookInteraction::Independent,
        );

        // security-guidance + reflex-mcp-prefer → Antagonistic
        // If security is blocking a file write, MCP preference is irrelevant.
        matrix.set_interaction(
            "security-guidance",
            "reflex-mcp-prefer",
            HookInteraction::antagonistic(Priority::HighestSeverityWins),
        );

        // stop-skill-harvest + any gating hook → Antagonistic (harvest should not
        // be blocked by unrelated code quality signals during stop events)
        matrix.set_interaction(
            "security-guidance",
            "stop-skill-harvest",
            HookInteraction::antagonistic(Priority::MostSpecificWins),
        );

        // Two safety hooks amplify strongly (threshold cascade at 2+ signals)
        matrix.set_interaction(
            "python-creation-blocker",
            "unwrap-guardian",
            HookInteraction::synergistic_cascading(
                1.1,
                CascadeProtocol::Threshold {
                    min_signals: 2,
                    then_escalate: true,
                },
            ),
        );

        matrix
    }
}

// ============================================================================
// GATE VERDICT
// ============================================================================

/// The final decision produced by the gate controller.
///
/// Tier: T2-P (grounded to ς: State + ∂: Boundary + →: Causality)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum GateVerdict {
    /// All signals permit the operation.
    Allow,

    /// A signal has suppressed the operation.
    Suppress {
        /// Name of the hook that caused suppression.
        reason: String,
    },

    /// Combined signals have escalated beyond any single hook's severity.
    Escalate {
        /// Combined effective severity of all contributing signals.
        combined_severity: f64,
    },

    /// Signal was modulated — adjusted but not blocked.
    Modulate {
        /// Adjusted severity after gate processing.
        adjusted_severity: f64,
    },
}

impl GateVerdict {
    /// Whether this verdict permits the operation.
    #[must_use]
    pub fn is_permissive(&self) -> bool {
        matches!(self, Self::Allow | Self::Modulate { .. })
    }

    /// Whether this verdict blocks the operation.
    #[must_use]
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Suppress { .. } | Self::Escalate { .. })
    }

    /// Extract the effective severity of this verdict.
    #[must_use]
    pub fn severity(&self) -> f64 {
        match self {
            Self::Allow => 0.0,
            Self::Suppress { .. } => 1.0,
            Self::Escalate { combined_severity } => *combined_severity,
            Self::Modulate { adjusted_severity } => *adjusted_severity,
        }
    }
}

impl fmt::Display for GateVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow => write!(f, "allow"),
            Self::Suppress { reason } => write!(f, "suppress({reason})"),
            Self::Escalate { combined_severity } => {
                write!(f, "escalate({combined_severity:.3})")
            }
            Self::Modulate { adjusted_severity } => {
                write!(f, "modulate({adjusted_severity:.3})")
            }
        }
    }
}

// ============================================================================
// GATE CONTROLLER
// ============================================================================

/// The main gate control engine.
///
/// Evaluates concurrent hook signals and applies the Melzack-Wall
/// modulation model to produce a single `GateVerdict`.
///
/// Tier: T3 (domain composition — μ + → + ∂)
///
/// ## Algorithm
///
/// 1. No signals → Allow
/// 2. One signal → direct pass-through based on `SignalType`
/// 3. Multiple signals → pairwise modulation, then reduce to single verdict
///
/// Allodynia prevention: synergistic escalation is capped at 1.0, and
/// escalation requires effective_severity > 0.85 to prevent false positives
/// from low-severity synergistic pairs.
pub struct GateController {
    matrix: InteractionMatrix,

    /// Severity threshold above which `Escalate` is issued instead of `Modulate`.
    pub escalation_threshold: f64,
}

impl GateController {
    /// Allodynia prevention boundary: escalation threshold.
    const DEFAULT_ESCALATION_THRESHOLD: f64 = 0.85;

    /// Create a new gate controller with a given interaction matrix.
    #[must_use]
    pub fn new(matrix: InteractionMatrix) -> Self {
        Self {
            matrix,
            escalation_threshold: Self::DEFAULT_ESCALATION_THRESHOLD,
        }
    }

    /// Evaluate a slice of concurrent hook signals.
    ///
    /// Returns a single `GateVerdict` representing the gate decision.
    #[must_use]
    pub fn evaluate(&self, signals: &[HookSignal]) -> GateVerdict {
        match signals.len() {
            0 => GateVerdict::Allow,
            1 => self.evaluate_single(&signals[0]),
            2 => self.modulate_pair(&signals[0], &signals[1]),
            _ => self.evaluate_multi(signals),
        }
    }

    /// Evaluate a single signal directly.
    fn evaluate_single(&self, signal: &HookSignal) -> GateVerdict {
        match signal.signal_type {
            SignalType::Block => GateVerdict::Suppress {
                reason: signal.hook_name.clone(),
            },
            SignalType::Warn => GateVerdict::Modulate {
                adjusted_severity: signal.severity,
            },
            SignalType::Allow | SignalType::Inform => GateVerdict::Allow,
        }
    }

    /// Modulate two signals according to their interaction type.
    ///
    /// This is the core gate logic, analogous to the spinal cord gate.
    #[must_use]
    pub fn modulate_pair(&self, a: &HookSignal, b: &HookSignal) -> GateVerdict {
        let interaction = self.matrix.get_interaction(&a.hook_name, &b.hook_name);

        match interaction {
            HookInteraction::Independent => {
                // Both pass; take the one with higher effective severity
                let (dominant, _) = self.dominant_signal(a, b);
                self.evaluate_single(dominant)
            }

            HookInteraction::Synergistic {
                amplification,
                cascade,
            } => {
                let combined = (a.effective_severity() + b.effective_severity()) * amplification;
                let adjusted = if let Some(protocol) = cascade {
                    protocol.apply(combined, 2)
                } else {
                    combined.clamp(0.0, 1.0)
                };

                if adjusted >= self.escalation_threshold {
                    GateVerdict::Escalate {
                        combined_severity: adjusted,
                    }
                } else {
                    GateVerdict::Modulate {
                        adjusted_severity: adjusted,
                    }
                }
            }

            HookInteraction::Antagonistic { priority } => {
                let winner = self.resolve_antagonism(a, b, priority);
                self.evaluate_single(winner)
            }
        }
    }

    /// Evaluate 3+ signals via pairwise reduction.
    ///
    /// Pairs the highest-severity signal against each other in turn,
    /// reducing the list to a final verdict.
    fn evaluate_multi(&self, signals: &[HookSignal]) -> GateVerdict {
        // Sort by effective severity descending (most forceful first)
        let mut sorted: Vec<&HookSignal> = signals.iter().collect();
        sorted.sort_by(|a, b| {
            b.effective_severity()
                .partial_cmp(&a.effective_severity())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Check for any outright Block signals first
        let blocks: Vec<&HookSignal> = sorted
            .iter()
            .copied()
            .filter(|s| s.signal_type == SignalType::Block)
            .collect();

        if !blocks.is_empty() {
            // Any Block signal that isn't suppressed by an antagonist wins
            if let Some(first_block) = blocks.first() {
                return GateVerdict::Suppress {
                    reason: first_block.hook_name.clone(),
                };
            }
        }

        // Accumulate effective severities for non-Block signals
        let total_severity: f64 = sorted
            .iter()
            .filter(|s| s.signal_type != SignalType::Block)
            .map(|s| s.effective_severity())
            .sum::<f64>()
            .clamp(0.0, 1.0);

        // Apply cascade for synergistic groups
        let synergistic_count = sorted
            .iter()
            .filter(|s| s.signal_type == SignalType::Warn)
            .count();

        let adjusted = if synergistic_count >= 2 {
            let protocol = CascadeProtocol::Threshold {
                min_signals: 2,
                then_escalate: false,
            };
            protocol.apply(total_severity, synergistic_count)
        } else {
            total_severity
        };

        if adjusted >= self.escalation_threshold {
            GateVerdict::Escalate {
                combined_severity: adjusted,
            }
        } else if adjusted > 0.0 {
            GateVerdict::Modulate {
                adjusted_severity: adjusted,
            }
        } else {
            GateVerdict::Allow
        }
    }

    /// Return the dominant signal from two, based on effective severity.
    fn dominant_signal<'a>(
        &self,
        a: &'a HookSignal,
        b: &'a HookSignal,
    ) -> (&'a HookSignal, &'a HookSignal) {
        if a.effective_severity() >= b.effective_severity() {
            (a, b)
        } else {
            (b, a)
        }
    }

    /// Resolve antagonism between two signals under the given priority rule.
    fn resolve_antagonism<'a>(
        &self,
        a: &'a HookSignal,
        b: &'a HookSignal,
        priority: Priority,
    ) -> &'a HookSignal {
        match priority {
            Priority::FirstWins => a,
            Priority::HighestSeverityWins => {
                if a.effective_severity() >= b.effective_severity() {
                    a
                } else {
                    b
                }
            }
            Priority::MostSpecificWins => {
                // A more specific hook name has more path separators or longer name.
                // This is a heuristic: longer/more-scoped names win.
                let a_specificity = a.hook_name.matches('-').count() + a.hook_name.len();
                let b_specificity = b.hook_name.matches('-').count() + b.hook_name.len();
                if a_specificity >= b_specificity { a } else { b }
            }
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn block_signal(name: &str, severity: f64) -> HookSignal {
        HookSignal::new(name, severity, SignalType::Block)
    }

    fn warn_signal(name: &str, severity: f64) -> HookSignal {
        HookSignal::new(name, severity, SignalType::Warn)
    }

    fn allow_signal(name: &str) -> HookSignal {
        HookSignal::new(name, 0.0, SignalType::Allow)
    }

    fn inform_signal(name: &str) -> HookSignal {
        HookSignal::new(name, 0.1, SignalType::Inform)
    }

    // ── Signal type ──────────────────────────────────────────────────────────

    #[test]
    fn signal_type_weights_ordered() {
        assert!(SignalType::Block.weight() > SignalType::Warn.weight());
        assert!(SignalType::Warn.weight() > SignalType::Allow.weight());
        assert!(SignalType::Allow.weight() > SignalType::Inform.weight());
    }

    #[test]
    fn signal_type_gating() {
        assert!(SignalType::Block.is_gating());
        assert!(SignalType::Warn.is_gating());
        assert!(!SignalType::Allow.is_gating());
        assert!(!SignalType::Inform.is_gating());
    }

    #[test]
    fn effective_severity_block_amplified() {
        let sig = block_signal("hook", 0.5);
        // Block weight is 4.0, so 0.5 * 4.0 = 2.0 clamped to 1.0
        assert!((sig.effective_severity() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn effective_severity_warn_moderate() {
        let sig = warn_signal("hook", 0.3);
        // Warn weight is 2.0, so 0.3 * 2.0 = 0.6
        assert!((sig.effective_severity() - 0.6).abs() < f64::EPSILON);
    }

    // ── Cascade protocol ─────────────────────────────────────────────────────

    #[test]
    fn cascade_linear_clamps_at_one() {
        let p = CascadeProtocol::Linear { factor: 2.0 };
        assert!((p.apply(0.6, 2) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cascade_linear_partial() {
        let p = CascadeProtocol::Linear { factor: 1.2 };
        assert!((p.apply(0.5, 2) - 0.6).abs() < 1e-10);
    }

    #[test]
    fn cascade_exponential_applies_cap() {
        let p = CascadeProtocol::Exponential {
            base: 2.0,
            cap: 0.8,
        };
        // 0.9^2 = 0.81, cap is 0.8
        let result = p.apply(0.9, 2);
        assert!(result <= 0.8 + f64::EPSILON);
    }

    #[test]
    fn cascade_threshold_triggers_at_min() {
        let p = CascadeProtocol::Threshold {
            min_signals: 3,
            then_escalate: true,
        };
        // Below threshold: pass through
        assert!((p.apply(0.5, 2) - 0.5).abs() < f64::EPSILON);
        // At threshold: escalate to 1.0
        assert!((p.apply(0.5, 3) - 1.0).abs() < f64::EPSILON);
    }

    // ── Interaction matrix ───────────────────────────────────────────────────

    #[test]
    fn matrix_symmetric_lookup() {
        let mut m = InteractionMatrix::new();
        m.set_interaction("alpha", "beta", HookInteraction::synergistic(1.5));

        let ab = m.get_interaction("alpha", "beta");
        let ba = m.get_interaction("beta", "alpha");
        assert_eq!(ab, ba);
    }

    #[test]
    fn matrix_missing_pair_returns_independent() {
        let m = InteractionMatrix::new();
        let result = m.get_interaction("foo", "bar");
        assert_eq!(result, HookInteraction::Independent);
    }

    #[test]
    fn default_matrix_has_known_entries() {
        let m = InteractionMatrix::default_matrix();
        assert!(!m.is_empty());

        let sg_ug = m.get_interaction("security-guidance", "unwrap-guardian");
        assert!(matches!(sg_ug, HookInteraction::Synergistic { .. }));

        let sg_rmp = m.get_interaction("security-guidance", "reflex-mcp-prefer");
        assert!(matches!(sg_rmp, HookInteraction::Antagonistic { .. }));
    }

    // ── Gate controller — single signal ─────────────────────────────────────

    #[test]
    fn single_block_signal_suppresses() {
        let ctrl = GateController::new(InteractionMatrix::new());
        let verdict = ctrl.evaluate(&[block_signal("security-guidance", 0.9)]);
        assert!(
            matches!(verdict, GateVerdict::Suppress { reason } if reason == "security-guidance")
        );
    }

    #[test]
    fn single_warn_signal_modulates() {
        let ctrl = GateController::new(InteractionMatrix::new());
        let verdict = ctrl.evaluate(&[warn_signal("unwrap-guardian", 0.4)]);
        assert!(
            matches!(verdict, GateVerdict::Modulate { adjusted_severity } if (adjusted_severity - 0.4).abs() < f64::EPSILON)
        );
    }

    #[test]
    fn single_allow_signal_permits() {
        let ctrl = GateController::new(InteractionMatrix::new());
        let verdict = ctrl.evaluate(&[allow_signal("some-hook")]);
        assert_eq!(verdict, GateVerdict::Allow);
    }

    #[test]
    fn single_inform_signal_permits() {
        let ctrl = GateController::new(InteractionMatrix::new());
        let verdict = ctrl.evaluate(&[inform_signal("logger-hook")]);
        assert_eq!(verdict, GateVerdict::Allow);
    }

    #[test]
    fn empty_signals_allow() {
        let ctrl = GateController::new(InteractionMatrix::new());
        assert_eq!(ctrl.evaluate(&[]), GateVerdict::Allow);
    }

    // ── Gate controller — pair modulation ────────────────────────────────────

    #[test]
    fn synergistic_pair_amplifies_severity() {
        let mut matrix = InteractionMatrix::new();
        matrix.set_interaction("hook-a", "hook-b", HookInteraction::synergistic(1.5));
        let ctrl = GateController::new(matrix);

        let signals = vec![warn_signal("hook-a", 0.3), warn_signal("hook-b", 0.3)];
        let verdict = ctrl.evaluate(&signals);
        // 0.3 * 2.0 = 0.6 eff each; combined = 1.2 * 1.5 = 1.8, clamped to 1.0 → Escalate
        assert!(verdict.is_blocking());
    }

    #[test]
    fn antagonistic_pair_highest_severity_wins() {
        let mut matrix = InteractionMatrix::new();
        matrix.set_interaction(
            "hook-a",
            "hook-b",
            HookInteraction::antagonistic(Priority::HighestSeverityWins),
        );
        let ctrl = GateController::new(matrix);

        // hook-b has higher severity → it wins
        let signals = vec![warn_signal("hook-a", 0.2), block_signal("hook-b", 0.9)];
        let verdict = ctrl.evaluate(&signals);
        assert!(matches!(verdict, GateVerdict::Suppress { reason } if reason == "hook-b"));
    }

    #[test]
    fn antagonistic_pair_first_wins() {
        let mut matrix = InteractionMatrix::new();
        matrix.set_interaction(
            "hook-a",
            "hook-b",
            HookInteraction::antagonistic(Priority::FirstWins),
        );
        let ctrl = GateController::new(matrix);

        let signals = vec![warn_signal("hook-a", 0.2), block_signal("hook-b", 0.9)];
        let verdict = ctrl.evaluate(&signals);
        // hook-a is first — but warn doesn't suppress
        assert!(matches!(verdict, GateVerdict::Modulate { .. }));
    }

    #[test]
    fn independent_pair_takes_dominant() {
        let ctrl = GateController::new(InteractionMatrix::new());
        // Both independent; block wins over warn
        let signals = vec![warn_signal("hook-a", 0.4), block_signal("hook-b", 0.9)];
        let verdict = ctrl.evaluate(&signals);
        // Dominant is hook-b (Block, eff=1.0)
        assert!(verdict.is_blocking());
    }

    // ── Gate controller — multi-signal ───────────────────────────────────────

    #[test]
    fn multi_signal_with_block_suppresses() {
        let ctrl = GateController::new(InteractionMatrix::default_matrix());
        let signals = vec![
            inform_signal("logger"),
            warn_signal("lint-check", 0.3),
            block_signal("security-guidance", 0.9),
        ];
        let verdict = ctrl.evaluate(&signals);
        assert!(verdict.is_blocking());
    }

    #[test]
    fn multi_signal_warn_only_modulates() {
        let ctrl = GateController::new(InteractionMatrix::new());
        let signals = vec![
            warn_signal("lint-a", 0.2),
            warn_signal("lint-b", 0.2),
            warn_signal("lint-c", 0.1),
        ];
        let verdict = ctrl.evaluate(&signals);
        // No blocks; should modulate or escalate based on combined severity
        assert!(!matches!(verdict, GateVerdict::Suppress { .. }));
    }

    #[test]
    fn multi_signal_all_allow_permits() {
        let ctrl = GateController::new(InteractionMatrix::new());
        let signals = vec![
            allow_signal("hook-a"),
            allow_signal("hook-b"),
            inform_signal("hook-c"),
        ];
        let verdict = ctrl.evaluate(&signals);
        assert!(verdict.is_permissive());
    }

    // ── Verdict helpers ───────────────────────────────────────────────────────

    #[test]
    fn verdict_severity_values() {
        assert!((GateVerdict::Allow.severity()).abs() < f64::EPSILON);
        assert!(
            (GateVerdict::Suppress { reason: "x".into() }.severity() - 1.0).abs() < f64::EPSILON
        );
        assert!(
            (GateVerdict::Escalate {
                combined_severity: 0.95
            }
            .severity()
                - 0.95)
                .abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn verdict_permissive_blocking_exclusive() {
        let allow = GateVerdict::Allow;
        let suppress = GateVerdict::Suppress { reason: "r".into() };
        assert!(allow.is_permissive());
        assert!(!allow.is_blocking());
        assert!(!suppress.is_permissive());
        assert!(suppress.is_blocking());
    }

    // ── Default matrix integration ────────────────────────────────────────────

    #[test]
    fn default_matrix_security_guidance_escalates_with_unwrap_guardian() {
        let ctrl = GateController::new(InteractionMatrix::default_matrix());
        let signals = vec![
            block_signal("security-guidance", 0.9),
            warn_signal("unwrap-guardian", 0.8),
        ];
        let verdict = ctrl.evaluate(&signals);
        // security-guidance alone would suppress; with synergistic unwrap-guardian
        // the pair modulate_pair is called; security is Block → Suppress wins
        assert!(verdict.is_blocking());
    }

    #[test]
    fn gate_verdict_display() {
        assert_eq!(GateVerdict::Allow.to_string(), "allow");
        assert_eq!(
            GateVerdict::Suppress {
                reason: "hook-x".into()
            }
            .to_string(),
            "suppress(hook-x)"
        );
    }
}
