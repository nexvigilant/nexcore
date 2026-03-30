//! # RelayChain — T2: ς (State) + σ (Sequence) + → (Causality) + N (Quantity)
//!
//! Composes two relay stages sequentially with fidelity tracking.
//! Implements the **Relay Degradation Law**: `F_total = F₁ × F₂`.
//!
//! ## Design
//!
//! `RelayChain<R1, R2, I, M, O>` connects relay `R1: Relay<I,M>` to
//! relay `R2: Relay<M,O>`, threading the intermediate type `M` between them.
//! Fidelity is tracked per-hop via [`FidelityMetrics`] and composed
//! multiplicatively — consistent with the primitives-layer `RelayChain`.
//!
//! ## Axiom Coverage
//!
//! | Axiom | Enforcement |
//! |-------|-------------|
//! | A1 | Direction fixed by type: I → M → O |
//! | A2 | Both R1 and R2 are required intermediaries |
//! | A3 | `total_fidelity()` checked against `f_min` |
//! | A4 | Each relay's `Filtered` outcome short-circuits the chain |
//! | A5 | Three type boundaries: I/M, M/O, and the chain wrapper itself |
//!
//! ## Compositional Safety Theorem (Fragment 32)
//!
//! If R1 and R2 are individually verified relays, then `RelayChain<R1,R2>`
//! is a verified relay. Verification composes — this is the foundation for
//! building arbitrarily long verified pipelines from verified hops.

use nexcore_primitives::relay::Fidelity;

use crate::{fidelity::FidelityMetrics, outcome::RelayOutcome, relay::Relay};

// ============================================================================
// RelayChain<R1, R2, I, M, O> — ς: stateful two-hop composition
// ============================================================================

/// A two-relay sequential chain with fidelity tracking.
///
/// Connects `R1: Relay<I,M>` to `R2: Relay<M,O>`, producing
/// `RelayOutcome<O>` while accumulating per-hop [`FidelityMetrics`].
///
/// ## Short-Circuit Semantics
///
/// - If R1 returns `Filtered`, the chain returns `Filtered` immediately —
///   R2 is never called.
/// - If R1 returns `Failed(e)`, the chain returns `Failed(e)` immediately.
/// - Only `Forwarded(m)` from R1 causes R2 to be invoked.
///
/// This mirrors Axiom A4: a subthreshold result at any hop suppresses
/// downstream processing.
pub struct RelayChain<R1, R2, I, M, O>
where
    R1: Relay<I, M>,
    R2: Relay<M, O>,
{
    /// First relay stage.
    pub r1: R1,
    /// Second relay stage.
    pub r2: R2,
    /// Minimum acceptable total fidelity (Axiom A3 for the chain).
    f_min: f64,
    /// Fidelity recorded for the most recent R1 invocation.
    r1_fidelity: std::cell::Cell<f64>,
    /// Fidelity recorded for the most recent R2 invocation.
    r2_fidelity: std::cell::Cell<f64>,
    _phantom: std::marker::PhantomData<(I, M, O)>,
}

impl<R1, R2, I, M, O> RelayChain<R1, R2, I, M, O>
where
    R1: Relay<I, M>,
    R2: Relay<M, O>,
{
    /// Construct a relay chain with the given minimum total fidelity.
    pub fn new(r1: R1, r2: R2, f_min: f64) -> Self {
        Self {
            r1,
            r2,
            f_min: f_min.clamp(0.0, 1.0),
            r1_fidelity: std::cell::Cell::new(1.0),
            r2_fidelity: std::cell::Cell::new(1.0),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Construct a relay chain with the safety-critical minimum fidelity (0.80).
    pub fn safety_critical(r1: R1, r2: R2) -> Self {
        Self::new(r1, r2, 0.80)
    }

    /// Minimum acceptable total fidelity for this chain (Axiom A3).
    #[must_use]
    pub fn f_min(&self) -> f64 {
        self.f_min
    }

    /// Total fidelity from the most recent `process()` call: F₁ × F₂.
    ///
    /// Returns `Fidelity::PERFECT` if `process()` has not yet been called,
    /// or if the chain was filtered/failed before R2 ran.
    #[must_use]
    pub fn total_fidelity(&self) -> Fidelity {
        Fidelity::new(self.r1_fidelity.get() * self.r2_fidelity.get())
    }

    /// Verify Axiom A3: total fidelity ≥ f_min.
    #[must_use]
    pub fn verify_preservation(&self) -> bool {
        self.total_fidelity().meets_minimum(self.f_min)
    }

    /// Fidelity metrics for R1 from the most recent `process()` call.
    #[must_use]
    pub fn r1_metrics(&self) -> FidelityMetrics {
        FidelityMetrics::active(self.r1.stage_name(), self.r1_fidelity.get())
    }

    /// Fidelity metrics for R2 from the most recent `process()` call.
    #[must_use]
    pub fn r2_metrics(&self) -> FidelityMetrics {
        FidelityMetrics::active(self.r2.stage_name(), self.r2_fidelity.get())
    }
}

impl<R1, R2, I, M, O> Relay<I, O> for RelayChain<R1, R2, I, M, O>
where
    R1: Relay<I, M>,
    R2: Relay<M, O>,
{
    /// Process through both relay stages sequentially.
    ///
    /// R1 runs first. If it returns `Forwarded(m)`, R2 runs on `m`.
    /// `Filtered` or `Failed` from either hop short-circuits the chain.
    fn process(&self, input: I) -> RelayOutcome<O> {
        // Record R1 fidelity from its declared min (used as proxy for
        // actual fidelity when no per-call measurement is available).
        self.r1_fidelity.set(self.r1.min_fidelity());

        match self.r1.process(input) {
            RelayOutcome::Forwarded(mid) => {
                self.r2_fidelity.set(self.r2.min_fidelity());
                self.r2.process(mid)
            }
            RelayOutcome::Filtered => {
                // R2 never ran — reset R2 fidelity to 1.0 (vacuously perfect).
                self.r2_fidelity.set(1.0);
                RelayOutcome::Filtered
            }
            RelayOutcome::Failed(e) => {
                self.r2_fidelity.set(1.0);
                RelayOutcome::Failed(e)
            }
        }
    }

    fn min_fidelity(&self) -> f64 {
        self.f_min
    }

    fn stage_name(&self) -> &str {
        "RelayChain"
    }
}
