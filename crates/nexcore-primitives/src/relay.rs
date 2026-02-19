//! # Relay Primitives — T2-P: → (Causality) + ∂ (Boundary) + π (Persistence)
//!
//! Universal relay pattern: an intermediary that receives input, optionally
//! transforms it, and propagates output across a boundary while preserving
//! essential information.
//!
//! ## Verification Axioms
//!
//! | Axiom | Statement | What It Verifies |
//! |-------|-----------|------------------|
//! | A1 | Signal flows source → destination | Directionality |
//! | A2 | Intermediary is required for transit | Mediation |
//! | A3 | Fidelity ≥ F_min after relay | Preservation |
//! | A4 | Relay activates only when input ≥ threshold | Threshold |
//! | A5 | Relay bridges a boundary between regions | Boundedness |
//!
//! ## Relay Degradation Law
//!
//! For chained relays: `F_total = ∏ F_i` — each hop multiplies fidelity, never adds.
//! A 4-hop chain with 0.95 fidelity per hop yields 0.95⁴ ≈ 0.815 total fidelity,
//! meaning ~18.5% cumulative signal loss.

use std::fmt;

use nexcore_constants::Confidence;
use serde::{Deserialize, Serialize};

// ============================================================================
// Fidelity — T2-P: κ (Comparison) + N (Quantity)
// ============================================================================

/// Relay fidelity score — ratio of preserved essential information [0.0, 1.0].
///
/// Semantically distinct from [`Confidence`]: fidelity measures how much
/// information survives a relay hop, while confidence measures certainty
/// about a value. A relay's fidelity contributes multiplicatively to the
/// output confidence.
///
/// Transfers from: information theory (mutual information), signal processing
/// (SNR preservation), pharmacovigilance (signal strength through pipeline).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Fidelity(f64);

impl Fidelity {
    /// Perfect fidelity — no information loss.
    pub const PERFECT: Self = Self(1.0);
    /// Zero fidelity — complete information loss.
    pub const NONE: Self = Self(0.0);
    /// High fidelity (0.95) — typical for well-designed relay.
    pub const HIGH: Self = Self(0.95);
    /// Acceptable fidelity (0.80) — minimum for safety-critical relays.
    pub const ACCEPTABLE: Self = Self(0.80);

    /// Create a new fidelity score, clamping to [0.0, 1.0].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get the raw fidelity value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Check if fidelity meets a minimum threshold (Axiom A3).
    #[must_use]
    pub fn meets_minimum(self, f_min: f64) -> bool {
        self.0 >= f_min
    }

    /// Signal loss as a fraction (1.0 - fidelity).
    #[must_use]
    pub fn loss(self) -> f64 {
        1.0 - self.0
    }

    /// Compose two fidelity scores (product rule — multiplicative degradation).
    #[must_use]
    pub fn compose(self, other: Self) -> Self {
        Self::new(self.0 * other.0)
    }

    /// Convert to [`Confidence`] for interop with the measurement system.
    #[must_use]
    pub fn to_confidence(self) -> Confidence {
        Confidence::new(self.0)
    }

    /// Create from a [`Confidence`] value.
    #[must_use]
    pub fn from_confidence(c: Confidence) -> Self {
        Self::new(c.value())
    }
}

impl Default for Fidelity {
    fn default() -> Self {
        Self::PERFECT // Assume no loss until measured
    }
}

impl fmt::Display for Fidelity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}%", self.0 * 100.0)
    }
}

// ============================================================================
// RelayHop — T2-P: → (Causality) + ∂ (Boundary) + N (Quantity)
// ============================================================================

/// Record of a single relay hop — the measurement of one intermediary's
/// contribution to a relay chain.
///
/// Transfers from: network engineering (hop metrics), neuroscience (synaptic
/// relay), pharmacovigilance (pipeline stage fidelity).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelayHop {
    /// Name of this relay stage (e.g., "faers_ingest", "signal_detect").
    pub stage: String,
    /// Measured fidelity of this hop.
    pub fidelity: Fidelity,
    /// Minimum input magnitude for activation (Axiom A4).
    pub threshold: f64,
    /// Whether the relay activated (input met threshold).
    pub activated: bool,
}

impl RelayHop {
    /// Create a new relay hop record.
    pub fn new(stage: impl Into<String>, fidelity: Fidelity, threshold: f64) -> Self {
        Self {
            stage: stage.into(),
            fidelity,
            threshold,
            activated: true,
        }
    }

    /// Create an inactive hop (threshold not met — Axiom A4 blocked).
    pub fn inactive(stage: impl Into<String>, threshold: f64) -> Self {
        Self {
            stage: stage.into(),
            fidelity: Fidelity::NONE,
            threshold,
            activated: false,
        }
    }
}

impl fmt::Display for RelayHop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.activated {
            write!(
                f,
                "{}: {} (θ={})",
                self.stage, self.fidelity, self.threshold
            )
        } else {
            write!(f, "{}: INACTIVE (θ={})", self.stage, self.threshold)
        }
    }
}

// ============================================================================
// RelayChain — T2-C: σ (Sequence) + Σ (Sum) + π (Persistence)
// ============================================================================

/// A chain of relay hops with cumulative fidelity tracking.
///
/// Implements the **Relay Degradation Law**: `F_total = ∏ F_i`.
/// Each hop's fidelity multiplies into the total — the chain is only as
/// strong as the product of its links.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayChain {
    /// Ordered sequence of relay hops.
    hops: Vec<RelayHop>,
    /// Minimum acceptable total fidelity (Axiom A3 threshold).
    f_min: f64,
}

impl RelayChain {
    /// Create an empty relay chain with the given minimum fidelity.
    #[must_use]
    pub fn new(f_min: f64) -> Self {
        Self {
            hops: Vec::new(),
            f_min: f_min.clamp(0.0, 1.0),
        }
    }

    /// Create a relay chain with safety-critical minimum fidelity (0.80).
    #[must_use]
    pub fn safety_critical() -> Self {
        Self::new(Fidelity::ACCEPTABLE.value())
    }

    /// Add a hop to the chain.
    pub fn add_hop(&mut self, hop: RelayHop) {
        self.hops.push(hop);
    }

    /// Add a hop by parts (convenience method).
    pub fn record(
        &mut self,
        stage: impl Into<String>,
        fidelity: f64,
        threshold: f64,
        activated: bool,
    ) {
        if activated {
            self.hops
                .push(RelayHop::new(stage, Fidelity::new(fidelity), threshold));
        } else {
            self.hops.push(RelayHop::inactive(stage, threshold));
        }
    }

    /// Cumulative fidelity — product of all active hop fidelities.
    ///
    /// This is the **Relay Degradation Law**: `F_total = ∏ F_i`.
    #[must_use]
    pub fn total_fidelity(&self) -> Fidelity {
        let product = self
            .hops
            .iter()
            .filter(|h| h.activated)
            .map(|h| h.fidelity.value())
            .product::<f64>();
        Fidelity::new(product)
    }

    /// Cumulative signal loss as a fraction.
    #[must_use]
    pub fn signal_loss(&self) -> f64 {
        self.total_fidelity().loss()
    }

    /// Verify Axiom A3 (Preservation): total fidelity ≥ F_min.
    #[must_use]
    pub fn verify_preservation(&self) -> bool {
        self.total_fidelity().meets_minimum(self.f_min)
    }

    /// Number of active hops in the chain.
    #[must_use]
    pub fn active_hop_count(&self) -> usize {
        self.hops.iter().filter(|h| h.activated).count()
    }

    /// Total hop count (active + inactive).
    #[must_use]
    pub fn hop_count(&self) -> usize {
        self.hops.len()
    }

    /// Get all hops (read-only).
    #[must_use]
    pub fn hops(&self) -> &[RelayHop] {
        &self.hops
    }

    /// The minimum fidelity threshold.
    #[must_use]
    pub fn f_min(&self) -> f64 {
        self.f_min
    }

    /// Find the weakest hop (lowest fidelity among active hops).
    #[must_use]
    pub fn weakest_hop(&self) -> Option<&RelayHop> {
        self.hops.iter().filter(|h| h.activated).min_by(|a, b| {
            a.fidelity
                .value()
                .partial_cmp(&b.fidelity.value())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Run full axiom verification (A1-A5) on the chain.
    #[must_use]
    pub fn verify(&self) -> RelayVerification {
        let has_hops = !self.hops.is_empty();
        let has_active = self.active_hop_count() > 0;
        let total_fidelity = self.total_fidelity();
        let preserves = total_fidelity.meets_minimum(self.f_min);
        let all_thresholds_defined = self.hops.iter().all(|h| h.threshold.is_finite());

        RelayVerification {
            a1_directionality: has_hops, // Chain implies ordered sequence
            a2_mediation: has_active,    // At least one intermediary active
            a3_preservation: preserves,
            a4_threshold: all_thresholds_defined,
            a5_boundedness: has_hops, // Hops imply boundary crossings
            total_fidelity,
            signal_loss: total_fidelity.loss(),
            active_hops: self.active_hop_count(),
            weakest_stage: self
                .weakest_hop()
                .map(|h| h.stage.clone())
                .unwrap_or_default(),
        }
    }
}

impl Default for RelayChain {
    fn default() -> Self {
        Self::safety_critical()
    }
}

impl fmt::Display for RelayChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RelayChain({} hops, F={}, loss={:.1}%)",
            self.active_hop_count(),
            self.total_fidelity(),
            self.signal_loss() * 100.0
        )
    }
}

// ============================================================================
// RelayVerification — Axiom A1-A5 verification result
// ============================================================================

/// Result of verifying relay axioms A1-A5 on a chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayVerification {
    /// A1: Signal flows in one direction (source → destination).
    pub a1_directionality: bool,
    /// A2: At least one active intermediary exists.
    pub a2_mediation: bool,
    /// A3: Total fidelity meets minimum threshold.
    pub a3_preservation: bool,
    /// A4: All thresholds are well-defined.
    pub a4_threshold: bool,
    /// A5: Chain bridges boundaries (hops present).
    pub a5_boundedness: bool,
    /// Measured total fidelity across chain.
    pub total_fidelity: Fidelity,
    /// Signal loss fraction.
    pub signal_loss: f64,
    /// Number of active relay hops.
    pub active_hops: usize,
    /// Name of the weakest stage.
    pub weakest_stage: String,
}

impl RelayVerification {
    /// Whether all 5 axioms pass.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.a1_directionality
            && self.a2_mediation
            && self.a3_preservation
            && self.a4_threshold
            && self.a5_boundedness
    }

    /// Count of passing axioms (0-5).
    #[must_use]
    pub fn axioms_passing(&self) -> u8 {
        u8::from(self.a1_directionality)
            + u8::from(self.a2_mediation)
            + u8::from(self.a3_preservation)
            + u8::from(self.a4_threshold)
            + u8::from(self.a5_boundedness)
    }
}

impl fmt::Display for RelayVerification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.is_valid() { "VALID" } else { "FAILED" };
        write!(
            f,
            "Relay {status} ({}/5 axioms, F={}, loss={:.1}%)",
            self.axioms_passing(),
            self.total_fidelity,
            self.signal_loss * 100.0
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fidelity_clamps_to_range() {
        assert!((Fidelity::new(1.5).value() - 1.0).abs() < f64::EPSILON);
        assert!((Fidelity::new(-0.3).value() - 0.0).abs() < f64::EPSILON);
        assert!((Fidelity::new(0.85).value() - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn fidelity_compose_is_multiplicative() {
        let a = Fidelity::new(0.95);
        let b = Fidelity::new(0.90);
        let composed = a.compose(b);
        assert!((composed.value() - 0.855).abs() < f64::EPSILON);
    }

    #[test]
    fn fidelity_loss_is_complement() {
        let f = Fidelity::new(0.85);
        assert!((f.loss() - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn fidelity_confidence_interop() {
        let f = Fidelity::new(0.9);
        let c = f.to_confidence();
        assert!((c.value() - 0.9).abs() < f64::EPSILON);
        let back = Fidelity::from_confidence(c);
        assert!((back.value() - f.value()).abs() < f64::EPSILON);
    }

    #[test]
    fn fidelity_display() {
        assert_eq!(format!("{}", Fidelity::PERFECT), "100.0%");
        assert_eq!(format!("{}", Fidelity::NONE), "0.0%");
    }

    #[test]
    fn relay_hop_active() {
        let hop = RelayHop::new("detect", Fidelity::HIGH, 2.0);
        assert!(hop.activated);
        assert!((hop.fidelity.value() - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn relay_hop_inactive() {
        let hop = RelayHop::inactive("detect", 2.0);
        assert!(!hop.activated);
        assert!((hop.fidelity.value()).abs() < f64::EPSILON);
    }

    #[test]
    fn relay_chain_degradation_law() {
        // 4-hop chain at 0.95 per hop → 0.95⁴ ≈ 0.8145
        let mut chain = RelayChain::new(0.80);
        for i in 0..4 {
            chain.add_hop(RelayHop::new(
                format!("stage_{i}"),
                Fidelity::new(0.95),
                1.0,
            ));
        }
        let total = chain.total_fidelity().value();
        assert!((total - 0.81450625).abs() < 1e-6);
        assert!(chain.verify_preservation()); // 0.814 >= 0.80
    }

    #[test]
    fn relay_chain_fails_preservation_below_fmin() {
        let mut chain = RelayChain::new(0.90);
        chain.add_hop(RelayHop::new("stage_0", Fidelity::new(0.80), 1.0));
        chain.add_hop(RelayHop::new("stage_1", Fidelity::new(0.80), 1.0));
        // Total: 0.64 < 0.90
        assert!(!chain.verify_preservation());
    }

    #[test]
    fn relay_chain_ignores_inactive_hops() {
        let mut chain = RelayChain::new(0.50);
        chain.add_hop(RelayHop::new("active", Fidelity::new(0.90), 1.0));
        chain.add_hop(RelayHop::inactive("blocked", 5.0));
        assert_eq!(chain.active_hop_count(), 1);
        assert!((chain.total_fidelity().value() - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn relay_chain_weakest_hop() {
        let mut chain = RelayChain::new(0.50);
        chain.add_hop(RelayHop::new("strong", Fidelity::new(0.98), 1.0));
        chain.add_hop(RelayHop::new("weak", Fidelity::new(0.70), 1.0));
        chain.add_hop(RelayHop::new("medium", Fidelity::new(0.85), 1.0));
        let weakest = chain.weakest_hop();
        assert!(weakest.is_some());
        assert_eq!(weakest.map(|h| h.stage.as_str()), Some("weak"));
    }

    #[test]
    fn relay_chain_signal_loss() {
        let mut chain = RelayChain::new(0.50);
        chain.add_hop(RelayHop::new("hop", Fidelity::new(0.80), 1.0));
        assert!((chain.signal_loss() - 0.20).abs() < f64::EPSILON);
    }

    #[test]
    fn relay_chain_record_convenience() {
        let mut chain = RelayChain::safety_critical();
        chain.record("ingest", 0.98, 0.0, true);
        chain.record("detect", 0.95, 2.0, true);
        chain.record("evaluate", 0.92, 1.0, true);
        chain.record("alert", 0.97, 3.841, true);
        assert_eq!(chain.active_hop_count(), 4);
        // 0.98 * 0.95 * 0.92 * 0.97 ≈ 0.8306
        assert!(chain.total_fidelity().value() > 0.82);
        assert!(chain.total_fidelity().value() < 0.84);
    }

    #[test]
    fn relay_verification_all_pass() {
        let mut chain = RelayChain::new(0.50);
        chain.add_hop(RelayHop::new("hop", Fidelity::new(0.90), 1.0));
        let v = chain.verify();
        assert!(v.is_valid());
        assert_eq!(v.axioms_passing(), 5);
    }

    #[test]
    fn relay_verification_fails_on_empty_chain() {
        let chain = RelayChain::new(0.50);
        let v = chain.verify();
        assert!(!v.a1_directionality);
        assert!(!v.a2_mediation);
        assert!(!v.is_valid());
    }

    #[test]
    fn relay_verification_fails_preservation() {
        let mut chain = RelayChain::new(0.99);
        chain.add_hop(RelayHop::new("lossy", Fidelity::new(0.50), 1.0));
        let v = chain.verify();
        assert!(!v.a3_preservation); // 0.50 < 0.99
        assert!(v.a1_directionality);
        assert!(v.a2_mediation);
        assert_eq!(v.axioms_passing(), 4);
    }

    #[test]
    fn relay_chain_display() {
        let mut chain = RelayChain::new(0.80);
        chain.add_hop(RelayHop::new("a", Fidelity::new(0.95), 1.0));
        chain.add_hop(RelayHop::new("b", Fidelity::new(0.90), 1.0));
        let display = format!("{chain}");
        assert!(display.contains("2 hops"));
        assert!(display.contains("RelayChain"));
    }

    #[test]
    fn relay_verification_display() {
        let mut chain = RelayChain::new(0.50);
        chain.add_hop(RelayHop::new("hop", Fidelity::new(0.90), 1.0));
        let v = chain.verify();
        let display = format!("{v}");
        assert!(display.contains("VALID"));
        assert!(display.contains("5/5"));
    }

    #[test]
    fn fidelity_serde_roundtrip() {
        let f = Fidelity::new(0.87);
        let json = serde_json::to_string(&f);
        assert!(json.is_ok());
        if let Ok(j) = json {
            let back: Result<Fidelity, _> = serde_json::from_str(&j);
            assert!(back.is_ok());
            if let Ok(b) = back {
                assert!((b.value() - 0.87).abs() < f64::EPSILON);
            }
        }
    }

    #[test]
    fn relay_chain_serde_roundtrip() {
        let mut chain = RelayChain::new(0.80);
        chain.add_hop(RelayHop::new("stage", Fidelity::new(0.95), 2.0));
        let json = serde_json::to_string(&chain);
        assert!(json.is_ok());
        if let Ok(j) = json {
            let back: Result<RelayChain, _> = serde_json::from_str(&j);
            assert!(back.is_ok());
            if let Ok(b) = back {
                assert_eq!(b.hop_count(), 1);
                assert!((b.total_fidelity().value() - 0.95).abs() < f64::EPSILON);
            }
        }
    }

    #[test]
    fn pv_signal_pipeline_scenario() {
        // Real-world scenario: PV signal detection pipeline
        // FAERS → detect → evaluate → alert
        let mut pipeline = RelayChain::safety_critical(); // F_min = 0.80

        // Stage 1: FAERS data ingestion (high fidelity — structured data)
        pipeline.record("faers_ingest", 0.98, 0.0, true);

        // Stage 2: Signal detection (PRR/ROR) — some statistical uncertainty
        pipeline.record("signal_detect", 0.93, 2.0, true);

        // Stage 3: Guardian evaluation — risk context addition
        pipeline.record("guardian_eval", 0.95, 1.0, true);

        // Stage 4: Alert generation — threshold gating
        pipeline.record("alert_action", 0.97, 3.841, true);

        let verification = pipeline.verify();
        assert!(
            verification.is_valid(),
            "PV pipeline should pass all axioms"
        );

        // Total fidelity: 0.98 * 0.93 * 0.95 * 0.97 ≈ 0.8396
        assert!(
            pipeline.total_fidelity().value() > 0.83,
            "PV pipeline fidelity should exceed 0.83"
        );
        assert!(
            pipeline.verify_preservation(),
            "PV pipeline should meet safety-critical F_min"
        );

        // Identify weakest link
        let weakest = pipeline.weakest_hop();
        assert_eq!(
            weakest.map(|h| h.stage.as_str()),
            Some("signal_detect"),
            "Signal detection should be weakest hop"
        );
    }
}
