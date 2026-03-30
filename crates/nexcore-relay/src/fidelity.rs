//! # FidelityBound — T2: N (Quantity) + ∂ (Boundary) + κ (Comparison)
//!
//! Trait and metrics for relay components that can report fidelity.
//! Grounds to `∂ + N + κ` — a bounded quantity with a threshold comparison.
//!
//! ## Relay Axiom Alignment
//!
//! | Construct | Axiom | Role |
//! |-----------|-------|------|
//! | `fidelity()` | A3 | Reports preservation ratio |
//! | `meets_minimum()` | A3 gate | Verifies A3 is satisfied |
//! | `FidelityMetrics` | A3/A4 | Structured measurement record |
//!
//! ## Fidelity Law
//!
//! For chained relays, total fidelity is multiplicative:
//! `F_total = ∏ Fᵢ` — each hop multiplies, never adds.
//! This law is enforced by [`RelayChain`](crate::chain::RelayChain).

use nexcore_primitives::relay::Fidelity;

// ============================================================================
// FidelityBound — trait: N + ∂ + κ
// ============================================================================

/// A relay component that can report its fidelity score.
///
/// Any type implementing `FidelityBound` declares that it can measure how
/// much information it preserves during processing. This enables pipeline-wide
/// fidelity tracking and Axiom A3 verification.
///
/// ## Implementation Contract
///
/// - `fidelity()` must return a value in `[0.0, 1.0]`
/// - `1.0` means no information loss (perfect relay)
/// - `0.0` means complete information loss (dead relay)
/// - The returned value should reflect the *actual* measured fidelity,
///   not an optimistic estimate
pub trait FidelityBound {
    /// Return the fidelity score for this component: [0.0, 1.0].
    ///
    /// `1.0` = perfect preservation. `0.0` = complete loss.
    fn fidelity(&self) -> Fidelity;

    /// Return the minimum acceptable fidelity for this component (Axiom A3).
    ///
    /// Default: `0.80` (safety-critical threshold per relay theory).
    fn min_fidelity(&self) -> f64 {
        0.80
    }

    /// Verify Axiom A3: measured fidelity ≥ minimum acceptable fidelity.
    fn meets_minimum(&self) -> bool {
        self.fidelity().meets_minimum(self.min_fidelity())
    }
}

// ============================================================================
// FidelityMetrics — structured measurement record
// ============================================================================

/// Structured fidelity measurement for a single relay or relay chain.
///
/// Captures the current fidelity score alongside context fields needed
/// for pipeline diagnostics, dead relay detection, and A3 verification.
///
/// ## Dead Relay Detection
///
/// A relay is classified as **dead** when:
/// - `fidelity == 1.0` (identity — passes everything unchanged), OR
/// - `fidelity == 0.0` (blocks everything — complete loss)
///
/// Both cases indicate the relay is not performing useful work (Cluster 3,
/// fragment 11: Dead Relay Detection).
#[derive(Debug, Clone, PartialEq)]
pub struct FidelityMetrics {
    /// Measured fidelity of this component.
    pub fidelity: Fidelity,
    /// Minimum acceptable fidelity threshold (Axiom A3).
    pub f_min: f64,
    /// Name or identifier of the relay stage being measured.
    pub stage: String,
    /// Whether the relay activated for this measurement (Axiom A4).
    pub activated: bool,
}

impl FidelityMetrics {
    /// Create a new fidelity measurement record.
    pub fn new(stage: impl Into<String>, fidelity: f64, f_min: f64, activated: bool) -> Self {
        Self {
            fidelity: Fidelity::new(fidelity),
            f_min: f_min.clamp(0.0, 1.0),
            stage: stage.into(),
            activated,
        }
    }

    /// Create a measurement for an active relay with default safety-critical threshold.
    pub fn active(stage: impl Into<String>, fidelity: f64) -> Self {
        Self::new(stage, fidelity, 0.80, true)
    }

    /// Create a measurement for an inactive relay (threshold not met).
    pub fn inactive(stage: impl Into<String>) -> Self {
        Self::new(stage, 0.0, 0.80, false)
    }

    /// Verify Axiom A3: fidelity ≥ f_min.
    #[must_use]
    pub fn passes_a3(&self) -> bool {
        self.fidelity.meets_minimum(self.f_min)
    }

    /// Detect a dead relay: fidelity is exactly 0.0 or 1.0 (no transformation).
    ///
    /// A dead relay either passes everything unchanged (identity, f=1.0)
    /// or blocks everything (f=0.0). Neither is performing useful relay work.
    #[must_use]
    pub fn is_dead_relay(&self) -> bool {
        let v = self.fidelity.value();
        v <= f64::EPSILON || (v - 1.0).abs() <= f64::EPSILON
    }

    /// Signal loss as a fraction (1.0 - fidelity).
    #[must_use]
    pub fn signal_loss(&self) -> f64 {
        self.fidelity.loss()
    }
}

impl FidelityBound for FidelityMetrics {
    fn fidelity(&self) -> Fidelity {
        self.fidelity
    }

    fn min_fidelity(&self) -> f64 {
        self.f_min
    }
}

impl std::fmt::Display for FidelityMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.activated { "active" } else { "inactive" };
        write!(
            f,
            "{} [{}]: F={}, loss={:.1}%, A3={}",
            self.stage,
            status,
            self.fidelity,
            self.signal_loss() * 100.0,
            if self.passes_a3() { "pass" } else { "fail" }
        )
    }
}
