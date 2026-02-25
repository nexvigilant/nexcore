// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Value Intelligence System
//!
//! T3 composite that synthesizes multiple value signals into actionable intelligence.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    ValueIntelligence (T3)                    │
//! │         Σ dominant — multi-signal aggregation                │
//! ├──────────────┬──────────────┬──────────────┬───────────────┤
//! │ Convergence  │  Source      │  Temporal    │  Confidence    │
//! │ κ (compare)  │  λ (locate)  │  σ (sequence)│  ∂ (boundary)  │
//! ├──────────────┴──────────────┴──────────────┴───────────────┤
//! │  IntelligenceState (ς)  →  ActionRecommendation (∃)         │
//! │  Emerging → Developing → Confirmed → Actionable → Stale     │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## State Machine
//!
//! ```text
//! Emerging ──→ Developing ──→ Confirmed ──→ Actionable
//!     │             │              │             │
//!     └─────────────┴──────────────┴─────────────┘
//!                         │
//!                    Stale ──→ Expired
//! ```
//!
//! ## Primitive Grounding
//!
//! | Symbol | Primitive | Role |
//! |:------:|:----------|:-----|
//! | **Σ** | Sum | Multi-signal aggregation (dominant) |
//! | **κ** | Comparison | Cross-signal convergence |
//! | **σ** | Sequence | Temporal alignment |
//! | **∂** | Boundary | Confidence synthesis |
//! | **∃** | Existence | Actionability determination |
//! | **λ** | Location | Source diversity |
//! | **ς** | State | Intelligence lifecycle |
//! | **∝** | Irreversibility | Forward-only state transitions |
//! | **N** | Quantity | Measurable metrics |
//!
//! ## Cross-Domain Transfer
//!
//! | Domain | Instantiation |
//! |--------|---------------|
//! | Pharmacovigilance | Signal triage → case assessment → regulatory action |
//! | Finance | Multi-indicator convergence → trade recommendation |
//! | Cybersecurity | Multi-sensor alert → incident assessment → response |
//! | Epidemiology | Surveillance signals → outbreak confirmation → intervention |

use nexcore_chrono::{DateTime, Duration};
use serde::{Deserialize, Serialize};

use crate::types::{SignalStrength, SignalType};

// ============================================================================
// Signal Convergence — κ (Comparison)
// ============================================================================

/// Measures how multiple signals reinforce each other.
///
/// Convergence = (concordant_pairs / total_pairs) × direction_agreement
///
/// High convergence (>0.7) means multiple independent signals point the same
/// direction. This is the cross-signal reinforcement primitive.
///
/// ## Tier: T2-P (κ dominant)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SignalConvergence {
    /// Convergence score [0.0, 1.0]
    score: f64,
    /// Number of concordant signal pairs
    concordant_pairs: u32,
    /// Total signal pairs compared
    total_pairs: u32,
    /// Direction agreement factor [0.0, 1.0]
    direction_agreement: f64,
}

impl SignalConvergence {
    /// Create from signal pair analysis.
    ///
    /// Returns `None` if total_pairs is 0 or direction_agreement is out of range.
    pub fn new(concordant_pairs: u32, total_pairs: u32, direction_agreement: f64) -> Option<Self> {
        if total_pairs == 0 || !(0.0..=1.0).contains(&direction_agreement) {
            return None;
        }

        let pair_ratio = concordant_pairs as f64 / total_pairs as f64;
        let score = (pair_ratio * direction_agreement).clamp(0.0, 1.0);

        Some(Self {
            score,
            concordant_pairs,
            total_pairs,
            direction_agreement,
        })
    }

    /// Compute convergence from a set of signal scores.
    ///
    /// Signals with same-sign scores are concordant. Direction agreement
    /// is the proportion of signals in the majority direction.
    pub fn from_scores(scores: &[f64]) -> Option<Self> {
        if scores.len() < 2 {
            return None;
        }

        let n = scores.len();
        let total_pairs = (n * (n - 1) / 2) as u32;

        // Count concordant pairs (same sign)
        let mut concordant = 0u32;
        for i in 0..n {
            for j in (i + 1)..n {
                if scores[i].signum() == scores[j].signum() {
                    concordant += 1;
                }
            }
        }

        // Direction agreement: proportion in majority direction
        let positive = scores.iter().filter(|&&s| s > 0.0).count();
        let negative = scores.iter().filter(|&&s| s < 0.0).count();
        let majority = positive.max(negative) as f64;
        let total_nonzero = (positive + negative).max(1) as f64;
        let direction_agreement = majority / total_nonzero;

        Self::new(concordant, total_pairs, direction_agreement)
    }

    /// Convergence score [0.0, 1.0].
    pub fn score(&self) -> f64 {
        self.score
    }

    /// Number of concordant signal pairs.
    pub fn concordant_pairs(&self) -> u32 {
        self.concordant_pairs
    }

    /// Total signal pairs.
    pub fn total_pairs(&self) -> u32 {
        self.total_pairs
    }

    /// Direction agreement factor.
    pub fn direction_agreement(&self) -> f64 {
        self.direction_agreement
    }

    /// Whether convergence indicates strong reinforcement (>0.7).
    pub fn is_strong(&self) -> bool {
        self.score > 0.7
    }

    /// Convergence classification.
    pub fn classification(&self) -> &'static str {
        if self.score >= 0.85 {
            "decisive"
        } else if self.score >= 0.7 {
            "strong"
        } else if self.score >= 0.5 {
            "moderate"
        } else if self.score >= 0.3 {
            "weak"
        } else {
            "divergent"
        }
    }
}

impl std::fmt::Display for SignalConvergence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Convergence: {:.2} ({}, {}/{} concordant)",
            self.score,
            self.classification(),
            self.concordant_pairs,
            self.total_pairs
        )
    }
}

// ============================================================================
// Source Diversity — λ (Location)
// ============================================================================

/// Measures multi-source attribution for intelligence robustness.
///
/// Diversity = 1 - HHI (Herfindahl-Hirschman Index)
///
/// HHI = Σ(share_i²) where share_i is each source's contribution.
/// Single source → HHI=1.0 → diversity=0.0
/// Perfectly distributed → HHI=1/n → diversity=1-1/n
///
/// ## Tier: T2-P (λ dominant)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceDiversity {
    /// Diversity score [0.0, 1.0]
    score: f64,
    /// Number of unique sources
    source_count: u32,
    /// Herfindahl-Hirschman Index [0.0, 1.0]
    hhi: f64,
    /// Source names with signal counts
    sources: Vec<(String, u32)>,
}

impl SourceDiversity {
    /// Compute diversity from source signal counts.
    ///
    /// Returns `None` if sources is empty.
    pub fn from_sources(sources: Vec<(String, u32)>) -> Option<Self> {
        if sources.is_empty() {
            return None;
        }

        let total: u32 = sources.iter().map(|(_, c)| c).sum();
        if total == 0 {
            return None;
        }

        let total_f = total as f64;
        let hhi: f64 = sources
            .iter()
            .map(|(_, c)| {
                let share = *c as f64 / total_f;
                share * share
            })
            .sum();

        let score = (1.0 - hhi).clamp(0.0, 1.0);
        let source_count = sources.len() as u32;

        Some(Self {
            score,
            source_count,
            hhi,
            sources,
        })
    }

    /// Diversity score [0.0, 1.0].
    pub fn score(&self) -> f64 {
        self.score
    }

    /// Number of unique sources.
    pub fn source_count(&self) -> u32 {
        self.source_count
    }

    /// Herfindahl-Hirschman Index.
    pub fn hhi(&self) -> f64 {
        self.hhi
    }

    /// Source attribution list.
    pub fn sources(&self) -> &[(String, u32)] {
        &self.sources
    }

    /// Whether diversity is sufficient for robust intelligence (score > 0.5, sources >= 3).
    pub fn is_robust(&self) -> bool {
        self.score > 0.5 && self.source_count >= 3
    }

    /// Diversity classification.
    pub fn classification(&self) -> &'static str {
        match (self.source_count, self.score > 0.5) {
            (1, _) => "single_source",
            (2, _) => "dual_source",
            (_, true) => "diversified",
            (_, false) => "concentrated",
        }
    }
}

impl std::fmt::Display for SourceDiversity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Sources: {} ({}, diversity={:.2}, HHI={:.3})",
            self.source_count,
            self.classification(),
            self.score,
            self.hhi
        )
    }
}

// ============================================================================
// Temporal Alignment — σ (Sequence)
// ============================================================================

/// Measures how closely signals align in time.
///
/// Alignment = 1 / (1 + normalized_time_spread)
///
/// Where normalized_time_spread = stddev(timestamps) / reference_window.
/// Perfect alignment (simultaneous) → score = 1.0.
/// Spread across entire window → score → 0.
///
/// ## Tier: T2-P (σ dominant)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TemporalAlignment {
    /// Alignment score [0.0, 1.0]
    score: f64,
    /// Time span of signals in seconds
    span_seconds: f64,
    /// Reference window in seconds
    window_seconds: f64,
    /// Number of signals in alignment
    signal_count: u32,
}

impl TemporalAlignment {
    /// Create from signal timestamps and a reference window.
    ///
    /// `window_seconds` is the expected observation window.
    /// Returns `None` if fewer than 2 timestamps or window <= 0.
    pub fn from_timestamps(timestamps: &[f64], window_seconds: f64) -> Option<Self> {
        if timestamps.len() < 2 || window_seconds <= 0.0 {
            return None;
        }

        let min_t = timestamps.iter().copied().fold(f64::INFINITY, f64::min);
        let max_t = timestamps.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let span = max_t - min_t;

        let normalized_spread = span / window_seconds;
        let score = (1.0 / (1.0 + normalized_spread)).clamp(0.0, 1.0);

        Some(Self {
            score,
            span_seconds: span,
            window_seconds,
            signal_count: timestamps.len() as u32,
        })
    }

    /// Alignment score [0.0, 1.0].
    pub fn score(&self) -> f64 {
        self.score
    }

    /// Time span of signals in seconds.
    pub fn span_seconds(&self) -> f64 {
        self.span_seconds
    }

    /// Reference window in seconds.
    pub fn window_seconds(&self) -> f64 {
        self.window_seconds
    }

    /// Signal count.
    pub fn signal_count(&self) -> u32 {
        self.signal_count
    }

    /// Whether signals are tightly clustered (score > 0.7).
    pub fn is_clustered(&self) -> bool {
        self.score > 0.7
    }

    /// Temporal classification.
    pub fn classification(&self) -> &'static str {
        if self.score >= 0.8 {
            "synchronous"
        } else if self.score >= 0.5 {
            "clustered"
        } else if self.score >= 0.3 {
            "distributed"
        } else {
            "dispersed"
        }
    }
}

impl std::fmt::Display for TemporalAlignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Temporal: {:.2} ({}, span={:.0}s in {:.0}s window)",
            self.score,
            self.classification(),
            self.span_seconds,
            self.window_seconds
        )
    }
}

// ============================================================================
// Intelligence Confidence — ∂ (Boundary)
// ============================================================================

/// Synthesized confidence from multiple signal confidences.
///
/// Uses geometric mean of individual confidences weighted by convergence,
/// then bounded by source diversity penalty.
///
/// confidence = geometric_mean(individual_confidences)
///            × convergence_factor
///            × diversity_factor
///
/// ## Tier: T2-P (∂ dominant)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IntelligenceConfidence {
    /// Synthesized confidence [0.0, 1.0]
    score: f64,
    /// Geometric mean of individual signal confidences
    geometric_mean: f64,
    /// Convergence contribution factor
    convergence_factor: f64,
    /// Source diversity contribution factor
    diversity_factor: f64,
    /// Lower bound of 95% CI (conservative estimate)
    lower_bound: f64,
}

impl IntelligenceConfidence {
    /// Compute from individual confidences and weighting factors.
    ///
    /// Returns `None` if confidences is empty or factors out of range.
    pub fn compute(
        confidences: &[f64],
        convergence_factor: f64,
        diversity_factor: f64,
    ) -> Option<Self> {
        if confidences.is_empty() {
            return None;
        }

        // Geometric mean
        let log_sum: f64 = confidences
            .iter()
            .filter(|&&c| c > 0.0)
            .map(|c| c.ln())
            .sum();
        let n = confidences.iter().filter(|&&c| c > 0.0).count();
        if n == 0 {
            return None;
        }
        let geometric_mean = (log_sum / n as f64).exp();

        // Composite score
        let score = (geometric_mean
            * convergence_factor.clamp(0.0, 1.0)
            * diversity_factor.clamp(0.0, 1.0))
        .clamp(0.0, 1.0);

        // Conservative lower bound (Wilson interval approximation)
        let z = 1.96; // 95% CI
        let n_f = n as f64;
        let lower_bound = ((score + z * z / (2.0 * n_f)
            - z * ((score * (1.0 - score) / n_f + z * z / (4.0 * n_f * n_f)).sqrt()))
            / (1.0 + z * z / n_f))
            .clamp(0.0, score);

        Some(Self {
            score,
            geometric_mean,
            convergence_factor: convergence_factor.clamp(0.0, 1.0),
            diversity_factor: diversity_factor.clamp(0.0, 1.0),
            lower_bound,
        })
    }

    /// Synthesized confidence score [0.0, 1.0].
    pub fn score(&self) -> f64 {
        self.score
    }

    /// Geometric mean of individual confidences.
    pub fn geometric_mean(&self) -> f64 {
        self.geometric_mean
    }

    /// Convergence contribution factor.
    pub fn convergence_factor(&self) -> f64 {
        self.convergence_factor
    }

    /// Diversity contribution factor.
    pub fn diversity_factor(&self) -> f64 {
        self.diversity_factor
    }

    /// Lower bound of confidence interval (conservative estimate).
    pub fn lower_bound(&self) -> f64 {
        self.lower_bound
    }

    /// Confidence classification.
    pub fn classification(&self) -> &'static str {
        if self.score >= 0.9 {
            "very_high"
        } else if self.score >= 0.7 {
            "high"
        } else if self.score >= 0.5 {
            "moderate"
        } else if self.score >= 0.3 {
            "low"
        } else {
            "insufficient"
        }
    }
}

impl std::fmt::Display for IntelligenceConfidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Confidence: {:.2} ({}, lower_bound={:.2})",
            self.score,
            self.classification(),
            self.lower_bound
        )
    }
}

// ============================================================================
// Intelligence State — ς (State) + ∝ (Irreversibility)
// ============================================================================

/// Intelligence lifecycle state machine.
///
/// Forward transitions are driven by signal accumulation:
/// ```text
/// Emerging ──→ Developing ──→ Confirmed ──→ Actionable
///     │             │              │             │
///     └─────────────┴──────────────┴─────────────┘
///                         │
///                    Stale ──→ Expired
/// ```
///
/// ## Transition Rules
///
/// | From | To | Condition |
/// |------|----|-----------|
/// | Emerging | Developing | signals >= 2 AND convergence > 0.3 |
/// | Developing | Confirmed | signals >= 3 AND convergence > 0.5 AND confidence > 0.6 |
/// | Confirmed | Actionable | confidence > 0.7 AND source_diversity.is_robust() |
/// | Any active | Stale | time_since_last_signal > staleness_threshold |
/// | Stale | Expired | time_since_stale > expiry_threshold |
///
/// ## Tier: T2-C (ς + ∝)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntelligenceState {
    /// Initial detection — fewer than 2 converging signals.
    Emerging,
    /// 2+ converging signals with moderate alignment.
    Developing,
    /// 3+ signals with high convergence and confidence.
    Confirmed,
    /// Confirmed with action threshold met.
    Actionable,
    /// Signal decay below freshness threshold.
    Stale,
    /// TTL exceeded — no longer relevant.
    Expired,
}

impl IntelligenceState {
    /// Whether this state represents active intelligence.
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Emerging | Self::Developing | Self::Confirmed | Self::Actionable
        )
    }

    /// Whether this state permits action.
    pub fn is_actionable(&self) -> bool {
        matches!(self, Self::Actionable)
    }

    /// Whether this state indicates decay.
    pub fn is_decayed(&self) -> bool {
        matches!(self, Self::Stale | Self::Expired)
    }

    /// Ordinal position in the lifecycle (for ordering).
    pub fn ordinal(&self) -> u8 {
        match self {
            Self::Emerging => 0,
            Self::Developing => 1,
            Self::Confirmed => 2,
            Self::Actionable => 3,
            Self::Stale => 4,
            Self::Expired => 5,
        }
    }

    /// Valid forward transitions from this state.
    pub fn valid_transitions(&self) -> &[IntelligenceState] {
        match self {
            Self::Emerging => &[Self::Developing, Self::Stale],
            Self::Developing => &[Self::Confirmed, Self::Stale],
            Self::Confirmed => &[Self::Actionable, Self::Stale],
            Self::Actionable => &[Self::Stale],
            Self::Stale => &[Self::Expired],
            Self::Expired => &[],
        }
    }

    /// Check whether a transition to `target` is valid.
    pub fn can_transition_to(&self, target: Self) -> bool {
        self.valid_transitions().contains(&target)
    }
}

impl std::fmt::Display for IntelligenceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Emerging => write!(f, "Emerging"),
            Self::Developing => write!(f, "Developing"),
            Self::Confirmed => write!(f, "Confirmed"),
            Self::Actionable => write!(f, "Actionable"),
            Self::Stale => write!(f, "Stale"),
            Self::Expired => write!(f, "Expired"),
        }
    }
}

// ============================================================================
// Action Recommendation — ∃ (Existence)
// ============================================================================

/// Actionable recommendation synthesized from intelligence assessment.
///
/// Each recommendation has a conviction level (how strongly to act)
/// and a rationale derived from the underlying signals.
///
/// ## Tier: T2-P (∃ dominant)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionRecommendation {
    /// The recommended action.
    pub action: ResponseAction,
    /// Conviction level [0.0, 1.0] — how strongly to follow the recommendation.
    pub conviction: f64,
    /// Human-readable rationale.
    pub rationale: String,
    /// Signal types that contribute to this recommendation.
    pub contributing_signals: Vec<SignalType>,
}

/// Value signal response action (Observe/Monitor/Investigate/Act/Hedge/Exit).
///
/// Tier: T2-P (ς + κ — state with comparison)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseAction {
    /// Insufficient data — continue collecting.
    Observe,
    /// Signal detected — increase monitoring frequency.
    Monitor,
    /// Developing intelligence — deep-dive investigation warranted.
    Investigate,
    /// Confirmed intelligence — take the recommended action.
    Act,
    /// Protective action — reduce exposure.
    Hedge,
    /// Exit position / cease activity.
    Exit,
}

/// Backward-compatible alias.
#[deprecated(note = "use ResponseAction — F2 equivocation fix")]
pub type Action = ResponseAction;

impl ResponseAction {
    /// Intensity level [0, 5].
    pub fn intensity(&self) -> u8 {
        match self {
            Self::Observe => 0,
            Self::Monitor => 1,
            Self::Investigate => 2,
            Self::Act => 3,
            Self::Hedge => 4,
            Self::Exit => 5,
        }
    }

    /// Derive action from intelligence state and confidence.
    pub fn from_intelligence(state: IntelligenceState, confidence: f64) -> Self {
        match state {
            IntelligenceState::Emerging => Self::Observe,
            IntelligenceState::Developing => {
                if confidence > 0.6 {
                    Self::Monitor
                } else {
                    Self::Observe
                }
            }
            IntelligenceState::Confirmed => {
                if confidence > 0.8 {
                    Self::Investigate
                } else {
                    Self::Monitor
                }
            }
            IntelligenceState::Actionable => {
                if confidence > 0.9 {
                    Self::Act
                } else {
                    Self::Investigate
                }
            }
            IntelligenceState::Stale => Self::Monitor,
            IntelligenceState::Expired => Self::Observe,
        }
    }
}

impl std::fmt::Display for ResponseAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Observe => write!(f, "OBSERVE"),
            Self::Monitor => write!(f, "MONITOR"),
            Self::Investigate => write!(f, "INVESTIGATE"),
            Self::Act => write!(f, "ACT"),
            Self::Hedge => write!(f, "HEDGE"),
            Self::Exit => write!(f, "EXIT"),
        }
    }
}

impl ActionRecommendation {
    /// Create a new recommendation.
    pub fn new(
        action: ResponseAction,
        conviction: f64,
        rationale: impl Into<String>,
        contributing_signals: Vec<SignalType>,
    ) -> Self {
        Self {
            action,
            conviction: conviction.clamp(0.0, 1.0),
            rationale: rationale.into(),
            contributing_signals,
        }
    }

    /// Derive recommendation from a ValueIntelligence assessment.
    pub fn derive(
        state: IntelligenceState,
        confidence: &IntelligenceConfidence,
        convergence: &SignalConvergence,
        active_signals: &[SignalType],
    ) -> Self {
        let action = ResponseAction::from_intelligence(state, confidence.score());
        let conviction = (confidence.score() * convergence.score()).clamp(0.0, 1.0);

        let rationale = format!(
            "{} state, {} confidence ({:.0}%), {} convergence across {} signal types",
            state,
            confidence.classification(),
            confidence.score() * 100.0,
            convergence.classification(),
            active_signals.len()
        );

        Self::new(action, conviction, rationale, active_signals.to_vec())
    }
}

impl std::fmt::Display for ActionRecommendation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (conviction={:.0}%): {}",
            self.action,
            self.conviction * 100.0,
            self.rationale
        )
    }
}

// ============================================================================
// Monitoring Parameters — N (Quantity)
// ============================================================================

/// Real-time monitoring dashboard for intelligence health.
///
/// Tracks all measurable state parameters that an operator monitors
/// to assess intelligence system performance.
///
/// ## Tier: T2-C (N + ν + π)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringDashboard {
    /// Number of active signals feeding intelligence.
    pub active_signal_count: u32,
    /// Number of unique signal types active.
    pub active_signal_types: u32,
    /// Cross-signal convergence score.
    pub convergence_score: f64,
    /// Source diversity score.
    pub source_diversity_score: f64,
    /// Temporal alignment score.
    pub temporal_alignment_score: f64,
    /// Synthesized confidence score.
    pub confidence_score: f64,
    /// Confidence lower bound (conservative).
    pub confidence_lower_bound: f64,
    /// Current intelligence state.
    pub state: IntelligenceState,
    /// Current action recommendation.
    pub recommended_action: ResponseAction,
    /// Conviction level for recommendation.
    pub conviction: f64,
    /// Time since last signal received (seconds).
    pub seconds_since_last_signal: f64,
    /// Decay rate (signals going stale per hour).
    pub decay_rate: f64,
    /// Historical false positive rate.
    pub false_positive_rate: f64,
    /// Ratio of signals reaching actionable state.
    pub actionable_ratio: f64,
    /// Time from first signal to current state (seconds).
    pub time_to_current_state: f64,
    /// Snapshot timestamp.
    pub snapshot_at: DateTime,
}

impl MonitoringDashboard {
    /// Overall system health score [0.0, 1.0].
    ///
    /// Weighted: confidence 30%, convergence 25%, diversity 20%,
    /// temporal 15%, freshness 10%.
    pub fn health_score(&self) -> f64 {
        let freshness = if self.seconds_since_last_signal < 3600.0 {
            1.0 - (self.seconds_since_last_signal / 3600.0)
        } else {
            0.0
        };

        (self.confidence_score * 0.30
            + self.convergence_score * 0.25
            + self.source_diversity_score * 0.20
            + self.temporal_alignment_score * 0.15
            + freshness * 0.10)
            .clamp(0.0, 1.0)
    }

    /// Whether the system is in a degraded state.
    pub fn is_degraded(&self) -> bool {
        self.health_score() < 0.5
            || self.state.is_decayed()
            || self.seconds_since_last_signal > 7200.0
    }

    /// Summary classification.
    pub fn classification(&self) -> &'static str {
        let h = self.health_score();
        if h >= 0.8 {
            "optimal"
        } else if h >= 0.6 {
            "healthy"
        } else if h >= 0.4 {
            "degraded"
        } else if h >= 0.2 {
            "critical"
        } else {
            "offline"
        }
    }
}

impl std::fmt::Display for MonitoringDashboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} | {} signals, {} types | conv={:.2} div={:.2} conf={:.2} | {} ({}%)",
            self.classification().to_uppercase(),
            self.state,
            self.active_signal_count,
            self.active_signal_types,
            self.convergence_score,
            self.source_diversity_score,
            self.confidence_score,
            self.recommended_action,
            (self.conviction * 100.0) as u32
        )
    }
}

// ============================================================================
// Value Intelligence — Σ (Sum) dominant, T3 composite
// ============================================================================

/// Top-level intelligence product synthesizing all value signals.
///
/// This is the T3 composite that composes raw signal detection (Layer 1-3)
/// with signal analytics (Layer 5) into a single actionable assessment.
///
/// ## Construction
///
/// ```text
/// SignalDetectors → raw Signals
///      ↓
/// SignalConvergence (κ) ← cross-signal comparison
/// SourceDiversity (λ)  ← multi-source attribution
/// TemporalAlignment (σ) ← time clustering
///      ↓
/// IntelligenceConfidence (∂) ← synthesized boundary
///      ↓
/// IntelligenceState (ς) ← lifecycle determination
///      ↓
/// ActionRecommendation (∃) ← existence of actionable intelligence
///      ↓
/// ValueIntelligence (Σ) ← aggregated product
/// ```
///
/// ## Tier: T3 (Σ dominant — multi-signal aggregation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueIntelligence {
    /// Unique intelligence ID.
    pub id: String,
    /// Entity this intelligence concerns.
    pub entity: String,
    /// Current lifecycle state.
    pub state: IntelligenceState,
    /// Cross-signal convergence measurement.
    pub convergence: SignalConvergence,
    /// Source diversity measurement.
    pub source_diversity: SourceDiversity,
    /// Temporal alignment measurement.
    pub temporal_alignment: TemporalAlignment,
    /// Synthesized confidence.
    pub confidence: IntelligenceConfidence,
    /// Action recommendation.
    pub recommendation: ActionRecommendation,
    /// Contributing signal types.
    pub signal_types: Vec<SignalType>,
    /// Total signal count.
    pub signal_count: u32,
    /// Best signal strength observed.
    pub peak_strength: SignalStrength,
    /// When intelligence was first created.
    pub created_at: DateTime,
    /// When intelligence was last updated.
    pub updated_at: DateTime,
    /// When intelligence expires if not refreshed.
    pub expires_at: DateTime,
    /// Version counter (increments on each update).
    pub version: u32,
}

impl ValueIntelligence {
    /// Create a new intelligence assessment from component measurements.
    ///
    /// Returns `None` if any required component is missing.
    #[allow(clippy::too_many_arguments)]
    pub fn synthesize(
        entity: impl Into<String>,
        signal_scores: &[f64],
        signal_types: Vec<SignalType>,
        source_contributions: Vec<(String, u32)>,
        signal_timestamps: &[f64],
        individual_confidences: &[f64],
        observation_window_seconds: f64,
        ttl_hours: i64,
    ) -> Option<Self> {
        let entity = entity.into();

        // Build components
        let convergence = SignalConvergence::from_scores(signal_scores)?;
        let source_diversity = SourceDiversity::from_sources(source_contributions)?;
        let temporal_alignment =
            TemporalAlignment::from_timestamps(signal_timestamps, observation_window_seconds)?;

        // Synthesize confidence
        let convergence_factor = convergence.score();
        let diversity_factor = if source_diversity.is_robust() {
            1.0
        } else {
            0.5 + source_diversity.score() * 0.5
        };
        let confidence = IntelligenceConfidence::compute(
            individual_confidences,
            convergence_factor,
            diversity_factor,
        )?;

        // Determine state
        let signal_count = signal_scores.len() as u32;
        let state =
            Self::determine_state(signal_count, &convergence, &confidence, &source_diversity);

        // Derive recommendation
        let recommendation =
            ActionRecommendation::derive(state, &confidence, &convergence, &signal_types);

        // Peak strength from individual confidences
        let peak_conf = individual_confidences
            .iter()
            .copied()
            .fold(0.0f64, f64::max);
        let peak_strength = SignalStrength::from_confidence(peak_conf);

        let now = DateTime::now();
        let expires_at = now + Duration::hours(ttl_hours);

        let id = format!("VI-{}-{}", entity.replace(' ', "_"), now.timestamp_millis());

        Some(Self {
            id,
            entity,
            state,
            convergence,
            source_diversity,
            temporal_alignment,
            confidence,
            recommendation,
            signal_types,
            signal_count,
            peak_strength,
            created_at: now,
            updated_at: now,
            expires_at,
            version: 1,
        })
    }

    /// Determine intelligence state from measurements.
    fn determine_state(
        signal_count: u32,
        convergence: &SignalConvergence,
        confidence: &IntelligenceConfidence,
        source_diversity: &SourceDiversity,
    ) -> IntelligenceState {
        if signal_count >= 3
            && convergence.score() > 0.5
            && confidence.score() > 0.7
            && source_diversity.is_robust()
        {
            IntelligenceState::Actionable
        } else if signal_count >= 3 && convergence.score() > 0.5 && confidence.score() > 0.6 {
            IntelligenceState::Confirmed
        } else if signal_count >= 2 && convergence.score() > 0.3 {
            IntelligenceState::Developing
        } else {
            IntelligenceState::Emerging
        }
    }

    /// Mark as stale (signal freshness degraded).
    pub fn mark_stale(&mut self) {
        if self.state.can_transition_to(IntelligenceState::Stale) {
            self.state = IntelligenceState::Stale;
            self.updated_at = DateTime::now();
            self.version += 1;
        }
    }

    /// Mark as expired (TTL exceeded).
    pub fn mark_expired(&mut self) {
        if self.state.can_transition_to(IntelligenceState::Expired) {
            self.state = IntelligenceState::Expired;
            self.updated_at = DateTime::now();
            self.version += 1;
        }
    }

    /// Check whether intelligence has expired based on TTL.
    pub fn is_expired(&self) -> bool {
        DateTime::now() > self.expires_at || self.state == IntelligenceState::Expired
    }

    /// Generate a monitoring dashboard snapshot.
    pub fn dashboard(&self, seconds_since_last_signal: f64) -> MonitoringDashboard {
        let time_to_current_state = (self.updated_at - self.created_at).num_seconds() as f64;

        MonitoringDashboard {
            active_signal_count: self.signal_count,
            active_signal_types: self.signal_types.len() as u32,
            convergence_score: self.convergence.score(),
            source_diversity_score: self.source_diversity.score(),
            temporal_alignment_score: self.temporal_alignment.score(),
            confidence_score: self.confidence.score(),
            confidence_lower_bound: self.confidence.lower_bound(),
            state: self.state,
            recommended_action: self.recommendation.action,
            conviction: self.recommendation.conviction,
            seconds_since_last_signal,
            decay_rate: 0.0,          // Computed from historical data
            false_positive_rate: 0.0, // Computed from historical data
            actionable_ratio: if self.state.is_actionable() { 1.0 } else { 0.0 },
            time_to_current_state,
            snapshot_at: DateTime::now(),
        }
    }
}

impl std::fmt::Display for ValueIntelligence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} | {} | {} signals ({} types) | {} | v{}",
            self.id,
            self.entity,
            self.state,
            self.signal_count,
            self.signal_types.len(),
            self.recommendation,
            self.version
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── SignalConvergence ──

    #[test]
    fn convergence_from_scores_all_positive() {
        let scores = vec![2.0, 3.0, 1.5, 2.5];
        let conv = SignalConvergence::from_scores(&scores);
        assert!(conv.is_some());
        let c = conv.unwrap_or_else(|| panic!("expected Some"));
        // All same sign → all concordant
        assert_eq!(c.concordant_pairs(), 6); // C(4,2) = 6
        assert_eq!(c.total_pairs(), 6);
        assert!((c.direction_agreement() - 1.0).abs() < f64::EPSILON);
        assert!(c.is_strong());
        assert_eq!(c.classification(), "decisive");
    }

    #[test]
    fn convergence_from_scores_mixed() {
        let scores = vec![2.0, -1.0, 3.0, -2.0];
        let conv = SignalConvergence::from_scores(&scores);
        assert!(conv.is_some());
        let c = conv.unwrap_or_else(|| panic!("expected Some"));
        // 2 positive, 2 negative → 2 concordant pairs (pos-pos, neg-neg) + 4 discordant
        assert_eq!(c.concordant_pairs(), 2);
        assert_eq!(c.total_pairs(), 6);
        assert!((c.direction_agreement() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn convergence_insufficient_scores() {
        assert!(SignalConvergence::from_scores(&[1.0]).is_none());
        assert!(SignalConvergence::from_scores(&[]).is_none());
    }

    #[test]
    fn convergence_zero_pairs_rejected() {
        assert!(SignalConvergence::new(0, 0, 0.5).is_none());
    }

    // ── SourceDiversity ──

    #[test]
    fn diversity_single_source() {
        let sd = SourceDiversity::from_sources(vec![("reddit".into(), 10)]);
        assert!(sd.is_some());
        let d = sd.unwrap_or_else(|| panic!("expected Some"));
        assert!((d.hhi() - 1.0).abs() < f64::EPSILON); // Monopoly
        assert!((d.score() - 0.0).abs() < f64::EPSILON);
        assert_eq!(d.classification(), "single_source");
        assert!(!d.is_robust());
    }

    #[test]
    fn diversity_three_equal_sources() {
        let sd = SourceDiversity::from_sources(vec![
            ("reddit".into(), 10),
            ("twitter".into(), 10),
            ("news".into(), 10),
        ]);
        assert!(sd.is_some());
        let d = sd.unwrap_or_else(|| panic!("expected Some"));
        // HHI = 3 × (1/3)² = 1/3 ≈ 0.333
        assert!((d.hhi() - 1.0 / 3.0).abs() < 0.01);
        // Diversity = 1 - 1/3 ≈ 0.667
        assert!((d.score() - 2.0 / 3.0).abs() < 0.01);
        assert_eq!(d.classification(), "diversified");
        assert!(d.is_robust());
    }

    #[test]
    fn diversity_empty_rejected() {
        assert!(SourceDiversity::from_sources(vec![]).is_none());
    }

    #[test]
    fn diversity_zero_counts_rejected() {
        assert!(SourceDiversity::from_sources(vec![("reddit".into(), 0)]).is_none());
    }

    // ── TemporalAlignment ──

    #[test]
    fn temporal_synchronous() {
        let timestamps = vec![100.0, 100.5, 101.0];
        let ta = TemporalAlignment::from_timestamps(&timestamps, 3600.0);
        assert!(ta.is_some());
        let t = ta.unwrap_or_else(|| panic!("expected Some"));
        assert!(t.score() > 0.99); // Very tight cluster
        assert!(t.is_clustered());
        assert_eq!(t.classification(), "synchronous");
    }

    #[test]
    fn temporal_dispersed() {
        let timestamps = vec![0.0, 3600.0, 7200.0, 10800.0];
        let ta = TemporalAlignment::from_timestamps(&timestamps, 3600.0);
        assert!(ta.is_some());
        let t = ta.unwrap_or_else(|| panic!("expected Some"));
        assert!(t.score() < 0.3); // Spread across 3x the window
        assert_eq!(t.classification(), "dispersed");
    }

    #[test]
    fn temporal_insufficient_timestamps() {
        assert!(TemporalAlignment::from_timestamps(&[1.0], 3600.0).is_none());
        assert!(TemporalAlignment::from_timestamps(&[], 3600.0).is_none());
    }

    // ── IntelligenceConfidence ──

    #[test]
    fn confidence_high_inputs() {
        let conf = IntelligenceConfidence::compute(&[0.9, 0.85, 0.88], 0.9, 0.8);
        assert!(conf.is_some());
        let c = conf.unwrap_or_else(|| panic!("expected Some"));
        assert!(c.score() > 0.5);
        assert!(c.geometric_mean() > 0.85);
        assert!(c.lower_bound() <= c.score());
    }

    #[test]
    fn confidence_empty_rejected() {
        assert!(IntelligenceConfidence::compute(&[], 1.0, 1.0).is_none());
    }

    #[test]
    fn confidence_zero_inputs_rejected() {
        assert!(IntelligenceConfidence::compute(&[0.0, 0.0], 1.0, 1.0).is_none());
    }

    #[test]
    fn confidence_diversity_penalty() {
        let high_div = IntelligenceConfidence::compute(&[0.8, 0.8], 0.8, 1.0);
        let low_div = IntelligenceConfidence::compute(&[0.8, 0.8], 0.8, 0.3);
        assert!(high_div.is_some());
        assert!(low_div.is_some());
        let h = high_div.unwrap_or_else(|| panic!("expected Some"));
        let l = low_div.unwrap_or_else(|| panic!("expected Some"));
        assert!(h.score() > l.score());
    }

    // ── IntelligenceState ──

    #[test]
    fn state_transitions() {
        assert!(IntelligenceState::Emerging.can_transition_to(IntelligenceState::Developing));
        assert!(IntelligenceState::Emerging.can_transition_to(IntelligenceState::Stale));
        assert!(!IntelligenceState::Emerging.can_transition_to(IntelligenceState::Actionable));
        assert!(!IntelligenceState::Expired.can_transition_to(IntelligenceState::Emerging));
    }

    #[test]
    fn state_properties() {
        assert!(IntelligenceState::Emerging.is_active());
        assert!(IntelligenceState::Actionable.is_actionable());
        assert!(!IntelligenceState::Confirmed.is_actionable());
        assert!(IntelligenceState::Stale.is_decayed());
        assert!(IntelligenceState::Expired.is_decayed());
    }

    #[test]
    fn state_ordinals() {
        assert!(IntelligenceState::Emerging.ordinal() < IntelligenceState::Developing.ordinal());
        assert!(IntelligenceState::Actionable.ordinal() < IntelligenceState::Stale.ordinal());
    }

    // ── Action ──

    #[test]
    fn action_from_state_and_confidence() {
        assert_eq!(
            ResponseAction::from_intelligence(IntelligenceState::Emerging, 0.9),
            ResponseAction::Observe
        );
        assert_eq!(
            ResponseAction::from_intelligence(IntelligenceState::Developing, 0.7),
            ResponseAction::Monitor
        );
        assert_eq!(
            ResponseAction::from_intelligence(IntelligenceState::Confirmed, 0.85),
            ResponseAction::Investigate
        );
        assert_eq!(
            ResponseAction::from_intelligence(IntelligenceState::Actionable, 0.95),
            ResponseAction::Act
        );
    }

    #[test]
    fn action_intensity_ordering() {
        assert!(ResponseAction::Observe.intensity() < ResponseAction::Monitor.intensity());
        assert!(ResponseAction::Monitor.intensity() < ResponseAction::Investigate.intensity());
        assert!(ResponseAction::Investigate.intensity() < ResponseAction::Act.intensity());
        assert!(ResponseAction::Act.intensity() < ResponseAction::Hedge.intensity());
        assert!(ResponseAction::Hedge.intensity() < ResponseAction::Exit.intensity());
    }

    // ── ValueIntelligence ──

    #[test]
    fn synthesize_actionable_intelligence() {
        let vi = ValueIntelligence::synthesize(
            "TSLA",
            &[3.5, 2.1, 1.8, 2.5],
            vec![
                SignalType::Sentiment,
                SignalType::Trend,
                SignalType::Engagement,
                SignalType::Virality,
            ],
            vec![
                ("reddit".into(), 5),
                ("twitter".into(), 4),
                ("news".into(), 3),
            ],
            &[100.0, 110.0, 120.0, 105.0],
            &[0.85, 0.78, 0.72, 0.80],
            3600.0,
            24,
        );

        assert!(vi.is_some());
        let intel = vi.unwrap_or_else(|| panic!("expected Some"));
        assert_eq!(intel.entity, "TSLA");
        assert_eq!(intel.signal_count, 4);
        assert_eq!(intel.signal_types.len(), 4);
        assert!(intel.state.is_active());
        assert!(!intel.is_expired());
        assert_eq!(intel.version, 1);

        // 4 signals, all positive, diverse sources → should be at least Confirmed
        assert!(intel.state.ordinal() >= IntelligenceState::Confirmed.ordinal());
    }

    #[test]
    fn synthesize_emerging_intelligence() {
        let vi = ValueIntelligence::synthesize(
            "BTC",
            &[1.2, -0.5], // Divergent
            vec![SignalType::Sentiment, SignalType::Trend],
            vec![("reddit".into(), 2)],
            &[100.0, 200.0],
            &[0.55, 0.45],
            3600.0,
            12,
        );

        assert!(vi.is_some());
        let intel = vi.unwrap_or_else(|| panic!("expected Some"));
        assert_eq!(intel.state, IntelligenceState::Emerging);
        assert_eq!(intel.recommendation.action, ResponseAction::Observe);
    }

    #[test]
    fn synthesize_insufficient_data() {
        // Only 1 signal → convergence fails
        assert!(
            ValueIntelligence::synthesize(
                "ETH",
                &[2.0],
                vec![SignalType::Sentiment],
                vec![("reddit".into(), 1)],
                &[100.0],
                &[0.8],
                3600.0,
                24,
            )
            .is_none()
        );
    }

    #[test]
    fn stale_and_expire_transitions() {
        let mut vi = ValueIntelligence::synthesize(
            "TEST",
            &[2.0, 3.0],
            vec![SignalType::Sentiment, SignalType::Trend],
            vec![("reddit".into(), 3), ("twitter".into(), 2)],
            &[100.0, 105.0],
            &[0.8, 0.75],
            3600.0,
            1,
        );

        assert!(vi.is_some());
        let intel = vi.as_mut().unwrap_or_else(|| panic!("expected Some"));
        let initial_version = intel.version;

        intel.mark_stale();
        assert_eq!(intel.state, IntelligenceState::Stale);
        assert_eq!(intel.version, initial_version + 1);

        intel.mark_expired();
        assert_eq!(intel.state, IntelligenceState::Expired);
        assert_eq!(intel.version, initial_version + 2);
    }

    #[test]
    fn dashboard_generation() {
        let vi = ValueIntelligence::synthesize(
            "DASH",
            &[2.5, 1.8, 3.0],
            vec![
                SignalType::Sentiment,
                SignalType::Engagement,
                SignalType::Virality,
            ],
            vec![
                ("reddit".into(), 4),
                ("twitter".into(), 3),
                ("news".into(), 2),
            ],
            &[100.0, 110.0, 120.0],
            &[0.8, 0.75, 0.82],
            3600.0,
            24,
        );

        assert!(vi.is_some());
        let intel = vi.unwrap_or_else(|| panic!("expected Some"));
        let dash = intel.dashboard(300.0);

        assert_eq!(dash.active_signal_count, 3);
        assert_eq!(dash.active_signal_types, 3);
        assert!(dash.convergence_score > 0.0);
        assert!(dash.source_diversity_score > 0.0);
        assert!(dash.health_score() > 0.0);
        assert!(!dash.is_degraded());
    }

    // ── Display traits ──

    #[test]
    fn display_formats() {
        let conv =
            SignalConvergence::from_scores(&[2.0, 3.0]).unwrap_or_else(|| panic!("expected Some"));
        let display = format!("{}", conv);
        assert!(display.contains("Convergence"));

        let state = IntelligenceState::Actionable;
        assert_eq!(format!("{}", state), "Actionable");

        let action = ResponseAction::Investigate;
        assert_eq!(format!("{}", action), "INVESTIGATE");
    }
}
