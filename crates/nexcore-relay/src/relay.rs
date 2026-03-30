//! # Relay<I,O> — T2: μ (Mapping) + → (Causality) + ∂ (Boundary)
//!
//! Core trait for all relay components in the nexcore system.
//! A relay receives input of type `I`, applies a transformation, and
//! produces a [`RelayOutcome<O>`] — forwarded, filtered, or failed.
//!
//! ## Relay Axioms (A1–A5)
//!
//! | Axiom | Statement | Enforcement Point |
//! |-------|-----------|-------------------|
//! | A1 | Signal flows source → destination | Trait method signature (unidirectional) |
//! | A2 | Intermediary required for transit | `impl Relay<I,O>` IS the intermediary |
//! | A3 | Fidelity ≥ F_min after relay | `RelayOutcome::Forwarded` carries preserved output |
//! | A4 | Relay activates only when input ≥ threshold | `RelayOutcome::Filtered` for sub-threshold inputs |
//! | A5 | Relay bridges a boundary between regions | Type parameters I ≠ O encode the boundary |
//!
//! ## Three Relay Strategies (Cluster 3)
//!
//! Implementors choose one strategy via [`RelayStrategy`](crate::strategy::RelayStrategy):
//! - **DF** (Decode-and-Forward): full decode then re-encode — safety-critical paths
//! - **CF** (Compress-and-Forward): reduce representation while preserving signal
//! - **AF** (Amplify-and-Forward): boost signal strength without full decode

use crate::outcome::RelayOutcome;

// ============================================================================
// Relay<I,O> trait — μ: I → RelayOutcome<O>
// ============================================================================

/// Core trait for relay components: receive `I`, produce [`RelayOutcome<O>`].
///
/// Implementing this trait declares that a type is a relay intermediary
/// satisfying Axioms A1–A5. The type parameters encode the boundary crossing
/// (A5): `I` is the input domain, `O` is the output domain.
///
/// ## Implementing Relay
///
/// ```rust
/// use nexcore_relay::relay::Relay;
/// use nexcore_relay::outcome::RelayOutcome;
/// use nexcore_error::NexError;
///
/// struct PassThrough;
///
/// impl Relay<String, String> for PassThrough {
///     fn process(&self, input: String) -> RelayOutcome<String> {
///         if input.is_empty() {
///             RelayOutcome::Filtered
///         } else {
///             RelayOutcome::Forwarded(input)
///         }
///     }
/// }
/// ```
///
/// ## Safety-Critical Relays
///
/// For safety-critical paths (e.g., PV signal propagation), use
/// `DecodeAndForward` strategy and set `min_fidelity()` to `0.80` or higher.
pub trait Relay<I, O> {
    /// Process the input and produce a relay outcome.
    ///
    /// - Return `Forwarded(output)` when the relay activates and succeeds
    /// - Return `Filtered` when input is below threshold (healthy suppression)
    /// - Return `Failed(err)` when the relay activates but encounters an error
    fn process(&self, input: I) -> RelayOutcome<O>;

    /// Minimum acceptable fidelity for this relay (Axiom A3 threshold).
    ///
    /// Default: `0.80` — safety-critical minimum per relay theory.
    /// Override to relax (non-critical paths) or tighten (critical paths).
    fn min_fidelity(&self) -> f64 {
        0.80
    }

    /// Human-readable name for this relay stage.
    ///
    /// Used in fidelity metrics and diagnostic output.
    fn stage_name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
