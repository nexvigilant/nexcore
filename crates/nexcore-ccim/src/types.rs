//! Core CCIM types.
//!
//! Grounding: N(Quantity) for all CU values, ∂(Boundary) for ratio clamping.

use serde::{Deserialize, Serialize};

use crate::error::CcimError;

/// Compounding ratio (rho): sovereign tool invocations / total analysis tasks.
///
/// Bounded to \[0.0, 1.0\]. Represents the reinvestment rate in the CCIM model.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CompoundingRatio(f64);

impl CompoundingRatio {
    /// Cash under mattress — zero reinvestment.
    pub const MATTRESS: Self = Self(0.0);
    /// Savings account — minimal reinvestment.
    pub const SAVINGS: Self = Self(0.05);
    /// Bond portfolio — moderate reinvestment.
    pub const BOND: Self = Self(0.15);
    /// Index fund — balanced reinvestment.
    pub const INDEX: Self = Self(0.30);
    /// Growth portfolio — aggressive reinvestment.
    pub const GROWTH: Self = Self(0.50);
    /// Aggressive growth — near-maximum reinvestment.
    pub const AGGRESSIVE: Self = Self(0.75);

    /// Create a new compounding ratio, clamped to \[0.0, 1.0\].
    ///
    /// Returns `Err` if the value is NaN or infinite.
    pub fn new(value: f64) -> Result<Self, CcimError> {
        if value.is_nan() || value.is_infinite() {
            return Err(CcimError::InvalidRho {
                value,
                reason: "NaN or infinite".to_string(),
            });
        }
        Ok(Self(value.clamp(0.0, 1.0)))
    }

    /// Get the inner value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }

    /// Check if zero (no reinvestment).
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.0 == 0.0
    }
}

/// Depreciation category with fixed per-directive rate.
///
/// Rates from CCIM model specification (ToV §4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DepreciationCategory {
    /// Code without recent maintenance: 2% per directive.
    UnmaintainedCode,
    /// Tests that fail: 10% per directive.
    FailingTests,
    /// Security vulnerabilities: 25% per directive.
    SecurityCves,
    /// MCP tools not visible on all 3 surfaces: 5% per directive.
    InvisibleTools,
    /// Documentation that hasn't been updated: 1% per directive.
    StaleDocs,
}

impl DepreciationCategory {
    /// Per-directive depreciation rate (delta).
    #[must_use]
    pub fn rate(self) -> f64 {
        match self {
            Self::UnmaintainedCode => 0.02,
            Self::FailingTests => 0.10,
            Self::SecurityCves => 0.25,
            Self::InvisibleTools => 0.05,
            Self::StaleDocs => 0.01,
        }
    }

    /// All depreciation categories.
    #[must_use]
    pub fn all() -> [Self; 5] {
        [
            Self::UnmaintainedCode,
            Self::FailingTests,
            Self::SecurityCves,
            Self::InvisibleTools,
            Self::StaleDocs,
        ]
    }

    /// Human-readable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::UnmaintainedCode => "Unmaintained Code",
            Self::FailingTests => "Failing Tests",
            Self::SecurityCves => "Security CVEs",
            Self::InvisibleTools => "Invisible Tools",
            Self::StaleDocs => "Stale Documentation",
        }
    }
}

/// A single depreciating asset in the capability portfolio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepreciationEntry {
    /// Description of what is depreciating.
    pub description: String,
    /// Category determines the rate.
    pub category: DepreciationCategory,
    /// Capability units at risk.
    pub capability_at_risk: f64,
    /// Periods (directives) since last maintenance.
    pub periods_unmaintained: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compounding_ratio_clamps_to_range() {
        let high = CompoundingRatio::new(1.5).expect("should clamp");
        assert!((high.value() - 1.0).abs() < f64::EPSILON);

        let low = CompoundingRatio::new(-0.1).expect("should clamp");
        assert!((low.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compounding_ratio_rejects_nan() {
        let result = CompoundingRatio::new(f64::NAN);
        assert!(result.is_err());
    }

    #[test]
    fn test_compounding_ratio_named_constants() {
        assert!((CompoundingRatio::MATTRESS.value() - 0.0).abs() < f64::EPSILON);
        assert!((CompoundingRatio::SAVINGS.value() - 0.05).abs() < f64::EPSILON);
        assert!((CompoundingRatio::BOND.value() - 0.15).abs() < f64::EPSILON);
        assert!((CompoundingRatio::INDEX.value() - 0.30).abs() < f64::EPSILON);
        assert!((CompoundingRatio::GROWTH.value() - 0.50).abs() < f64::EPSILON);
        assert!((CompoundingRatio::AGGRESSIVE.value() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_depreciation_category_rates() {
        assert!((DepreciationCategory::UnmaintainedCode.rate() - 0.02).abs() < f64::EPSILON);
        assert!((DepreciationCategory::FailingTests.rate() - 0.10).abs() < f64::EPSILON);
        assert!((DepreciationCategory::SecurityCves.rate() - 0.25).abs() < f64::EPSILON);
        assert!((DepreciationCategory::InvisibleTools.rate() - 0.05).abs() < f64::EPSILON);
        assert!((DepreciationCategory::StaleDocs.rate() - 0.01).abs() < f64::EPSILON);
    }

    #[test]
    fn test_depreciation_category_all() {
        assert_eq!(DepreciationCategory::all().len(), 5);
    }
}
