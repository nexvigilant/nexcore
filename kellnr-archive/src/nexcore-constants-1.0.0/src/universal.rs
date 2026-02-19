//! T1: Universal Constants — Axiomatic bedrock values.
//!
//! Tier: T1 (universal bedrock)

/// UCAS sigmoid integration: center point (μ)
/// Tier: T1 (universal constant)
pub const SIGMOID_MU: f64 = 5.0;

/// UCAS sigmoid integration: spread (σ)
/// Tier: T1 (universal constant)
pub const SIGMOID_SIGMA: f64 = 2.0;

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
