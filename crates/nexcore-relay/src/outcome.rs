//! # RelayOutcome — T2: Σ (Sum) + ∂ (Boundary) + ∝ (Irreversibility)
//!
//! Three-branch enum representing the result of a relay processing step.
//! Grounds to: `∂ + Σ` — a boundary that partitions inputs into three
//! mutually exclusive states: forwarded, filtered, or failed.
//!
//! ## Relay Axiom Alignment
//!
//! | Branch | Axiom | Meaning |
//! |--------|-------|---------|
//! | `Forwarded(T)` | A1/A3 | Signal passed through with preserved content |
//! | `Filtered` | A4 | Input below threshold — relay did not activate |
//! | `Failed(NexError)` | A3 violated | Relay activated but fidelity collapsed |

use nexcore_error::NexError;

// ============================================================================
// RelayOutcome<T> — Σ: three-branch boundary partition
// ============================================================================

/// Result of a single relay processing step.
///
/// Distinguishes three structurally different relay outcomes:
/// - [`Forwarded`](RelayOutcome::Forwarded): The relay processed the input and
///   produced output — Axioms A1 and A3 satisfied.
/// - [`Filtered`](RelayOutcome::Filtered): The input did not meet the relay's
///   activation threshold (Axiom A4) — the relay is silent, not broken.
/// - [`Failed`](RelayOutcome::Failed): The relay activated but encountered an
///   error — Axiom A3 violated, fidelity collapsed.
///
/// # Design Rationale
///
/// `Filtered` and `Failed` are semantically different from `Result::Err`:
/// filtering is intentional (subthreshold signals should not propagate),
/// while failure is unintentional (the relay tried and broke). Both are
/// non-forwarding, but only `Failed` indicates a fault.
#[derive(Debug)]
pub enum RelayOutcome<T> {
    /// The relay processed the input and forwarded a result.
    ///
    /// Axioms A1 (directionality) and A3 (preservation) satisfied.
    Forwarded(T),

    /// The input did not meet the activation threshold (Axiom A4).
    ///
    /// This is a healthy outcome: subthreshold signals are intentionally
    /// suppressed. The relay is functioning correctly.
    Filtered,

    /// The relay activated but encountered an unrecoverable error.
    ///
    /// Axiom A3 (preservation) violated — fidelity collapsed.
    /// The inner [`NexError`] carries the diagnostic context.
    Failed(NexError),
}

impl<T> RelayOutcome<T> {
    /// Returns `true` if the outcome is [`Forwarded`](RelayOutcome::Forwarded).
    #[must_use]
    pub fn is_forwarded(&self) -> bool {
        matches!(self, Self::Forwarded(_))
    }

    /// Returns `true` if the outcome is [`Filtered`](RelayOutcome::Filtered).
    #[must_use]
    pub fn is_filtered(&self) -> bool {
        matches!(self, Self::Filtered)
    }

    /// Returns `true` if the outcome is [`Failed`](RelayOutcome::Failed).
    #[must_use]
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }

    /// Extract the forwarded value, returning `None` for `Filtered` or `Failed`.
    #[must_use]
    pub fn into_forwarded(self) -> Option<T> {
        match self {
            Self::Forwarded(v) => Some(v),
            Self::Filtered | Self::Failed(_) => None,
        }
    }

    /// Map the forwarded value with a function, leaving other variants unchanged.
    #[must_use]
    pub fn map<U, F>(self, f: F) -> RelayOutcome<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Forwarded(v) => RelayOutcome::Forwarded(f(v)),
            Self::Filtered => RelayOutcome::Filtered,
            Self::Failed(e) => RelayOutcome::Failed(e),
        }
    }

    /// Convert to `Result<Option<T>, NexError>`:
    /// - `Forwarded(v)` → `Ok(Some(v))`
    /// - `Filtered` → `Ok(None)`
    /// - `Failed(e)` → `Err(e)`
    pub fn into_result(self) -> Result<Option<T>, NexError> {
        match self {
            Self::Forwarded(v) => Ok(Some(v)),
            Self::Filtered => Ok(None),
            Self::Failed(e) => Err(e),
        }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for RelayOutcome<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forwarded(v) => write!(f, "Forwarded({v})"),
            Self::Filtered => write!(f, "Filtered"),
            Self::Failed(e) => write!(f, "Failed({e})"),
        }
    }
}
