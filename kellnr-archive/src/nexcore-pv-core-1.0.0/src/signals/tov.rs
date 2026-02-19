//! # Theory of Vigilance (ToV) Implementation
//!
//! Universal signal detection theory based on the formula S = U × R × T.
//!
//! ## Migration Notice
//!
//! The types in this module are deprecated. Use the Codex-compliant types
//! from [`signal_equation`](super::signal_equation) instead:
//!
//! | Old | New (Codex-compliant) |
//! |-----|----------------------|
//! | `TovResult` | `SignalEquationResult` |
//! | `f64` fields | `UnrepeatablePattern`, `RecognitionPresence`, `TemporalWindow`, `SignalStrength` |
//!
//! The new types provide:
//! - Tier classification (T2-P/T2-C/T3)
//! - Type safety (no mixing of different measures)
//! - Validation (NaN, infinite, range checks)
//! - Codex compliance (WRAP, CLASSIFY, GROUND)

use serde::{Deserialize, Serialize};

// Re-export new Codex-compliant types
pub use super::signal_equation::{
    RecognitionPresence, SignalComponentError, SignalEquationResult, SignalStrength,
    TemporalWindow, UnrepeatablePattern,
};

/// Result of a ToV signal evaluation.
///
/// **DEPRECATED**: Use [`SignalEquationResult`] instead for Codex compliance.
#[deprecated(
    since = "0.1.0",
    note = "Use SignalEquationResult from signal_equation module for Codex-compliant types"
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TovResult {
    /// Total signal strength (S)
    pub signal_strength: f64,
    /// Unrepeatable pattern measure (U)
    pub unrepeatability: f64,
    /// Recognition presence (R)
    pub recognition: f64,
    /// Temporal window (T)
    pub temporal_factor: f64,
}

// ═══════════════════════════════════════════════════════════════════════════
// L1 ATOMS (Pure Calculations, < 20 LOC)
// DEPRECATED: Use signal_equation module for Codex-compliant types
// ═══════════════════════════════════════════════════════════════════════════

/// Atom: Calculate unrepeatable pattern measure (U)
///
/// **DEPRECATED**: Use `UnrepeatablePattern::from_ln1p_ratio()` instead.
#[deprecated(since = "0.1.0", note = "Use UnrepeatablePattern::from_ln1p_ratio()")]
#[must_use]
pub fn calculate_u(observed: u64, expected: f64) -> f64 {
    if expected <= 0.0 {
        return 0.0;
    }
    (observed as f64 / expected).ln_1p()
}

/// Atom: Calculate recognition presence (R)
///
/// **DEPRECATED**: Use `RecognitionPresence::from_dme_status()` instead.
#[deprecated(since = "0.1.0", note = "Use RecognitionPresence::from_dme_status()")]
#[must_use]
pub fn calculate_r(is_dme: bool, reported_previously: bool) -> f64 {
    let base = if is_dme { 2.0 } else { 1.0 };
    if reported_previously {
        base * 0.5
    } else {
        base
    }
}

/// Atom: Calculate temporal window factor (T)
///
/// **DEPRECATED**: Use `TemporalWindow::from_elapsed()` instead.
#[deprecated(since = "0.1.0", note = "Use TemporalWindow::from_elapsed()")]
#[must_use]
pub fn calculate_t(days_since_first: u32, total_days: u32) -> f64 {
    if total_days == 0 {
        return 1.0;
    }
    1.0 - (f64::from(days_since_first) / f64::from(total_days)).min(1.0)
}

// ═══════════════════════════════════════════════════════════════════════════
// L2 MOLECULES (Composed Logic, < 50 LOC)
// DEPRECATED: Use SignalEquationResult::evaluate() instead
// ═══════════════════════════════════════════════════════════════════════════

/// Molecule: Evaluate core ToV signal equation S = U × R × T
///
/// **DEPRECATED**: Use `SignalEquationResult::evaluate()` instead for
/// Codex-compliant typed results.
///
/// # Migration
///
/// ```rust,ignore
/// // Old (deprecated):
/// let result = evaluate_tov_signal(obs, exp, dme, prev, days, total);
/// let s = result.signal_strength; // f64
///
/// // New (Codex-compliant):
/// let result = SignalEquationResult::evaluate(obs, exp, dme, prev, days, total);
/// let s = result.signal_strength; // SignalStrength (typed)
/// ```
#[deprecated(since = "0.1.0", note = "Use SignalEquationResult::evaluate()")]
#[allow(deprecated)]
#[must_use]
pub fn evaluate_tov_signal(
    observed: u64,
    expected: f64,
    is_dme: bool,
    reported_previously: bool,
    days_since_first: u32,
    total_days: u32,
) -> TovResult {
    #[allow(deprecated)]
    let u = calculate_u(observed, expected);
    #[allow(deprecated)]
    let r = calculate_r(is_dme, reported_previously);
    #[allow(deprecated)]
    let t = calculate_t(days_since_first, total_days);

    #[allow(deprecated)]
    TovResult {
        signal_strength: u * r * t,
        unrepeatability: u,
        recognition: r,
        temporal_factor: t,
    }
}
