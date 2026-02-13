//! Primitive types

use serde::{Deserialize, Serialize};

/// Primitive tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveTier {
    /// Universal primitive (σ, μ, ρ, ς, ∅, etc.)
    T1,
    /// Cross-domain primitive (newtypes over T1)
    T2P,
    /// Cross-domain composite (combined T1/T2-P)
    T2C,
    /// Domain-specific
    T3,
}

/// A single extracted primitive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Primitive {
    /// Term name
    pub term: String,
    /// Definition
    pub definition: String,
    /// Tier classification
    pub tier: PrimitiveTier,
    /// T1 grounding (if not T1 itself)
    pub grounding: Option<String>,
    /// Transfer confidence (0.0-1.0)
    pub transfer_confidence: f64,
}
