//! # Core PV Types
//!
//! Shared types for pharmacovigilance operations.
//!
//! # Codex Compliance
//! - **Unification**: Re-exports core primitives to ensure single source of truth.

pub use crate::signals::core::types::{ContingencyTable, SignalResult};
use serde::{Deserialize, Serialize};

/// Complete signal analysis with all algorithms.
///
/// # Tier: T3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteSignalResult {
    /// PRR result
    pub prr: SignalResult,
    /// ROR result
    pub ror: SignalResult,
    /// IC result
    pub ic: SignalResult,
    /// EBGM result
    pub ebgm: SignalResult,
    /// Chi-square statistic
    pub chi_square: f64,
    /// Number of cases
    pub n: u64,
}
