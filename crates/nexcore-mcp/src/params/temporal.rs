//! Temporal Analysis Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Time-to-onset, dechallenge/rechallenge, and temporal plausibility parameters.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

// =============================================================================
// Time-to-Onset Parameters
// =============================================================================

/// Parameters for time-to-onset calculation.
///
/// Calculates days between exposure and event onset,
/// classifies into 6 TTO categories (Immediate through Chronic),
/// and returns a plausibility score (0.0-1.0).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TtoParams {
    /// First drug exposure date in YYYYMMDD format (e.g., "20240101")
    pub exposure_date: String,
    /// Adverse event onset date in YYYYMMDD format (e.g., "20240115")
    pub event_date: String,
}

// =============================================================================
// Challenge Assessment Parameters
// =============================================================================

/// Dechallenge response classification
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde", rename_all = "lowercase")]
pub enum DechallengeParam {
    /// Event resolved after stopping drug
    Positive,
    /// Event did not resolve after stopping drug
    Negative,
    /// Event partially improved
    Partial,
    /// Dechallenge not performed or not applicable
    NotApplicable,
    /// Information not available
    #[default]
    Unknown,
}

/// Rechallenge response classification
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde", rename_all = "lowercase")]
pub enum RechallengeParam {
    /// Event recurred after restarting drug
    Positive,
    /// Event did not recur after restarting drug
    Negative,
    /// Rechallenge not performed (often contraindicated)
    NotPerformed,
    /// Information not available
    #[default]
    Unknown,
}

/// Parameters for dechallenge/rechallenge assessment.
///
/// Evaluates drug withdrawal (dechallenge) and re-introduction (rechallenge)
/// responses with optional timing for confidence bonuses.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChallengeParams {
    /// Dechallenge response (drug withdrawal outcome)
    #[serde(default)]
    pub dechallenge: DechallengeParam,
    /// Rechallenge response (drug re-introduction outcome)
    #[serde(default)]
    pub rechallenge: RechallengeParam,
    /// Days to improvement after dechallenge (optional, <7 days adds confidence bonus)
    pub dechallenge_days: Option<f64>,
    /// Days to recurrence after rechallenge (optional, <3 days adds confidence bonus)
    pub rechallenge_days: Option<f64>,
}

// =============================================================================
// Temporal Plausibility Parameters
// =============================================================================

/// Parameters for unified temporal plausibility assessment.
///
/// Combines time-to-onset and challenge assessment into an overall
/// temporal plausibility score (0.0-1.0).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TemporalPlausibilityParams {
    /// Exposure date in YYYYMMDD format (optional if days_to_onset provided)
    pub exposure_date: Option<String>,
    /// Event onset date in YYYYMMDD format (optional if days_to_onset provided)
    pub event_date: Option<String>,
    /// Direct days-to-onset value (alternative to date pair)
    pub days_to_onset: Option<f64>,
    /// Dechallenge response (default: unknown)
    #[serde(default)]
    pub dechallenge: DechallengeParam,
    /// Rechallenge response (default: unknown)
    #[serde(default)]
    pub rechallenge: RechallengeParam,
    /// Minimum expected onset days (mechanism-based, optional)
    pub expected_min_days: Option<f64>,
    /// Maximum expected onset days (mechanism-based, optional)
    pub expected_max_days: Option<f64>,
}
