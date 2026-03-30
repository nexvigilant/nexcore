//! # RelayStrategy — T2: Σ (Sum) + ς (State) + κ (Comparison)
//!
//! Classification of relay processing strategies from relay theory Cluster 3.
//! Every relay component should declare its strategy — this drives investment
//! priority, fidelity expectations, and audit classification.
//!
//! ## Strategy Definitions (Fragment 6)
//!
//! | Strategy | Abbrev | Processing Model | Use Case |
//! |----------|--------|-----------------|----------|
//! | Decode-and-Forward | DF | Full decode → transform → re-encode | Safety-critical paths |
//! | Compress-and-Forward | CF | Reduce representation, preserve signal | Bandwidth-constrained paths |
//! | Amplify-and-Forward | AF | Boost signal without full decode | Low-latency paths |
//!
//! ## Safety Mandate (Fragment 7)
//!
//! Safety-critical relay paths MUST use [`DecodeAndForward`](RelayStrategy::DecodeAndForward).
//! This is the only strategy that provides full verification of the relayed content
//! and satisfies Axiom A3 (Preservation) at the highest confidence level.

use serde::{Deserialize, Serialize};

// ============================================================================
// RelayStrategy — Σ: three-branch classification
// ============================================================================

/// The processing strategy employed by a relay component.
///
/// Strategy selection determines fidelity characteristics, latency profile,
/// and investment priority for a relay. Use [`RelayStrategy::is_safety_critical`]
/// to enforce the DF mandate on critical paths.
///
/// ## Betweenness Centrality and Investment Priority (Fragment 29)
///
/// High-BC (betweenness centrality) relay nodes should prefer `DecodeAndForward`
/// regardless of latency cost — a failure at a high-BC node propagates to more
/// downstream consumers than a failure at a low-BC node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelayStrategy {
    /// **Decode-and-Forward (DF)**: Full decode, transform, re-encode.
    ///
    /// The relay fully understands the signal before forwarding it.
    /// Highest fidelity guarantee. Mandatory for safety-critical paths.
    ///
    /// Typical fidelity: 0.95–1.0. Latency: highest.
    DecodeAndForward,

    /// **Compress-and-Forward (CF)**: Reduce representation, preserve essential signal.
    ///
    /// The relay reduces information density while retaining the signal.
    /// Acceptable for non-critical paths where bandwidth is constrained.
    ///
    /// Typical fidelity: 0.80–0.95. Latency: medium.
    CompressAndForward,

    /// **Amplify-and-Forward (AF)**: Boost signal without full decode.
    ///
    /// The relay strengthens the signal without fully interpreting it.
    /// Lowest latency. Noise amplification risk — use only where input
    /// signal quality is already high.
    ///
    /// Typical fidelity: 0.70–0.90 (input-quality dependent). Latency: lowest.
    AmplifyAndForward,
}

impl RelayStrategy {
    /// Returns `true` if this strategy satisfies the safety-critical mandate (Fragment 7).
    ///
    /// Only [`DecodeAndForward`](RelayStrategy::DecodeAndForward) satisfies the mandate.
    #[must_use]
    pub fn is_safety_critical(self) -> bool {
        matches!(self, Self::DecodeAndForward)
    }

    /// Typical minimum fidelity for this strategy.
    ///
    /// These are guidance values from relay theory — actual fidelity depends
    /// on the specific relay implementation.
    #[must_use]
    pub fn typical_min_fidelity(self) -> f64 {
        match self {
            Self::DecodeAndForward => 0.95,
            Self::CompressAndForward => 0.80,
            Self::AmplifyAndForward => 0.70,
        }
    }

    /// Short abbreviation string for this strategy.
    #[must_use]
    pub fn abbreviation(self) -> &'static str {
        match self {
            Self::DecodeAndForward => "DF",
            Self::CompressAndForward => "CF",
            Self::AmplifyAndForward => "AF",
        }
    }

    /// Detect a dead relay: a relay whose strategy implies no real transformation.
    ///
    /// A relay is a candidate dead relay if it uses AF with no amplification
    /// logic — it passes signals through unchanged, making it an identity function.
    /// Identity = dead relay (Fragment 33: Category theory confirmation).
    ///
    /// This is a classification hint, not a definitive diagnosis. Use
    /// [`FidelityMetrics::is_dead_relay`](crate::fidelity::FidelityMetrics::is_dead_relay)
    /// for measurement-based detection.
    #[must_use]
    pub fn may_be_dead_relay(self) -> bool {
        // AF with identity amplification (gain=1) is a dead relay.
        // DF and CF always perform meaningful transformation.
        matches!(self, Self::AmplifyAndForward)
    }
}

impl std::fmt::Display for RelayStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DecodeAndForward => write!(f, "Decode-and-Forward (DF)"),
            Self::CompressAndForward => write!(f, "Compress-and-Forward (CF)"),
            Self::AmplifyAndForward => write!(f, "Amplify-and-Forward (AF)"),
        }
    }
}
