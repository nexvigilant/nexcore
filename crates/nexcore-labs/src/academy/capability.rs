//! Capability Priority Scoring
//!
//! Migrated from Python `spis/spis/models/capability.py`.
//!
//! ## UACA Hierarchy
//!
//! - **L0 Quarks**: Score bounds (0-100)
//! - **L1 Atoms**: Priority calculation (<20 LOC)
//! - **L2 Molecules**: Batch ranking (<50 LOC)
//!
//! ## Priority Formula
//!
//! ```text
//! Priority = (Impact × Automation Potential) / Complexity
//! ```
//!
//! This formula prioritizes capabilities that are:
//! - High impact on the organization
//! - Highly automatable
//! - Low complexity to implement
//!
//! ## Safety Axiom
//!
//! All scores are bounded [0, 100]. Division by zero returns 0.0.

use serde::{Deserialize, Serialize};

/// Bounded capability score ensuring value is in [0, 100].
///
/// # L0 Quark - Score type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct CapabilityScore(u8);

impl CapabilityScore {
    /// Create a new bounded score.
    ///
    /// # Errors
    /// Returns error if value > 100.
    pub fn new(value: u8) -> Result<Self, CapabilityScoreError> {
        if value > 100 {
            return Err(CapabilityScoreError::OutOfBounds { value });
        }
        Ok(Self(value))
    }

    /// Get the inner value.
    #[must_use]
    pub fn value(&self) -> u8 {
        self.0
    }

    /// Zero score.
    pub const ZERO: Self = Self(0);

    /// Maximum score.
    pub const MAX: Self = Self(100);
}

impl Default for CapabilityScore {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<'de> Deserialize<'de> for CapabilityScore {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        CapabilityScore::new(value).map_err(serde::de::Error::custom)
    }
}

/// Error type for capability score validation.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CapabilityScoreError {
    /// Score value > 100.
    #[error("Capability score {value} exceeds maximum of 100")]
    OutOfBounds {
        /// The invalid value
        value: u8,
    },
}

/// Priority score result (unbounded, can exceed 100).
///
/// # L0 Quark - Result type
///
/// The priority formula can produce values > 100 when impact and
/// automation are high but complexity is low.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PriorityScore(f64);

impl PriorityScore {
    /// Create a new priority score.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    /// Get the inner value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Zero priority.
    pub const ZERO: Self = Self(0.0);
}

impl Default for PriorityScore {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Input scores for priority calculation.
///
/// # L1 Atom - Score container
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CapabilityScores {
    /// Impact score [0, 100]
    pub impact: CapabilityScore,
    /// Automation potential [0, 100]
    pub automation_potential: CapabilityScore,
    /// Complexity score [0, 100]
    pub complexity: CapabilityScore,
}

impl CapabilityScores {
    /// Create new scores with validation.
    ///
    /// # Errors
    /// Returns error if any score > 100.
    pub fn new(
        impact: u8,
        automation_potential: u8,
        complexity: u8,
    ) -> Result<Self, CapabilityScoreError> {
        Ok(Self {
            impact: CapabilityScore::new(impact)?,
            automation_potential: CapabilityScore::new(automation_potential)?,
            complexity: CapabilityScore::new(complexity)?,
        })
    }
}

/// Calculate priority score for a capability.
///
/// # L1 Atom - Priority calculation (<20 LOC)
///
/// Formula: `Priority = (Impact × Automation) / Complexity`
///
/// # Arguments
/// * `scores` - Impact, automation potential, and complexity scores
///
/// # Returns
/// Priority score. Returns 0.0 if complexity is 0 (avoids division by zero).
#[must_use]
pub fn calculate_priority(scores: &CapabilityScores) -> PriorityScore {
    let impact = f64::from(scores.impact.value());
    let automation = f64::from(scores.automation_potential.value());
    let complexity = f64::from(scores.complexity.value());

    if complexity == 0.0 {
        return PriorityScore::ZERO;
    }

    PriorityScore::new((impact * automation) / complexity)
}

/// Capability with scores and calculated priority.
///
/// # L1 Atom - Named capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedCapability {
    /// Capability name
    pub name: String,
    /// Input scores
    pub scores: CapabilityScores,
    /// Calculated priority
    pub priority: PriorityScore,
}

/// Rank capabilities by priority score.
///
/// # L2 Molecule - Batch ranking (<50 LOC)
///
/// Calculates priority for each capability and returns them sorted
/// in descending order (highest priority first).
///
/// # Arguments
/// * `capabilities` - List of (name, scores) tuples
///
/// # Returns
/// Vector of ranked capabilities, sorted by priority descending.
#[must_use]
pub fn rank_capabilities(capabilities: &[(String, CapabilityScores)]) -> Vec<RankedCapability> {
    let mut ranked: Vec<RankedCapability> = capabilities
        .iter()
        .map(|(name, scores)| RankedCapability {
            // CLONE: Moving ownership from slice to owned Vec; unavoidable for return value
            name: name.clone(),
            scores: *scores,
            priority: calculate_priority(scores),
        })
        .collect();

    // Sort by priority descending - use total_cmp for safe f64 comparison
    ranked.sort_by(|a, b| b.priority.value().total_cmp(&a.priority.value()));

    ranked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_score_valid() {
        assert!(CapabilityScore::new(0).is_ok());
        assert!(CapabilityScore::new(50).is_ok());
        assert!(CapabilityScore::new(100).is_ok());
    }

    #[test]
    fn test_capability_score_invalid() {
        assert!(CapabilityScore::new(101).is_err());
        assert!(CapabilityScore::new(255).is_err());
    }

    #[test]
    fn test_calculate_priority_basic() {
        let scores = match CapabilityScores::new(80, 90, 50) {
            Ok(s) => s,
            Err(_) => return,
        };
        let priority = calculate_priority(&scores);
        // (80 * 90) / 50 = 144.0
        assert!((priority.value() - 144.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_priority_zero_complexity() {
        let scores = match CapabilityScores::new(80, 90, 0) {
            Ok(s) => s,
            Err(_) => return,
        };
        let priority = calculate_priority(&scores);
        assert!((priority.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_priority_high_priority() {
        // High impact, high automation, low complexity = very high priority
        let scores = match CapabilityScores::new(100, 100, 10) {
            Ok(s) => s,
            Err(_) => return,
        };
        let priority = calculate_priority(&scores);
        // (100 * 100) / 10 = 1000.0
        assert!((priority.value() - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_rank_capabilities() {
        let low = match CapabilityScores::new(30, 30, 90) {
            Ok(s) => s,
            Err(_) => return,
        };
        let high = match CapabilityScores::new(90, 90, 20) {
            Ok(s) => s,
            Err(_) => return,
        };
        let medium = match CapabilityScores::new(60, 60, 50) {
            Ok(s) => s,
            Err(_) => return,
        };

        let capabilities = vec![
            ("Low Priority".to_string(), low),
            ("High Priority".to_string(), high),
            ("Medium Priority".to_string(), medium),
        ];

        let ranked = rank_capabilities(&capabilities);

        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].name, "High Priority");
        assert_eq!(ranked[1].name, "Medium Priority");
        assert_eq!(ranked[2].name, "Low Priority");
    }

    #[test]
    fn test_deserialization_rejects_invalid() {
        let invalid_json = "150";
        let result: Result<CapabilityScore, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    // === Edge Case Tests ===

    #[test]
    fn test_capability_score_boundary() {
        assert!(CapabilityScore::new(0).is_ok());
        assert!(CapabilityScore::new(100).is_ok());
        assert!(CapabilityScore::new(101).is_err());
    }

    #[test]
    fn test_rank_empty_capabilities() {
        let capabilities: Vec<(String, CapabilityScores)> = vec![];
        let ranked = rank_capabilities(&capabilities);
        assert!(ranked.is_empty());
    }

    #[test]
    fn test_rank_single_capability() {
        let scores = match CapabilityScores::new(50, 50, 50) {
            Ok(s) => s,
            Err(_) => return,
        };
        let capabilities = vec![("Only One".to_string(), scores)];
        let ranked = rank_capabilities(&capabilities);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].name, "Only One");
    }

    #[test]
    fn test_priority_all_zeros() {
        let scores = match CapabilityScores::new(0, 0, 1) {
            Ok(s) => s,
            Err(_) => return,
        };
        let priority = calculate_priority(&scores);
        // 0 * 0 / 1 = 0
        assert!((priority.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_priority_max_values() {
        let scores = match CapabilityScores::new(100, 100, 1) {
            Ok(s) => s,
            Err(_) => return,
        };
        let priority = calculate_priority(&scores);
        // 100 * 100 / 1 = 10000
        assert!((priority.value() - 10000.0).abs() < 0.01);
    }
}
