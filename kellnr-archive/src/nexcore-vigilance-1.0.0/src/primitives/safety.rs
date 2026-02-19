//! # Safety T2-P and T2-C Primitives
//!
//! Types modeling boundaries, harm, violations, and monitoring state.
//!
//! | Type | Tier | LP Composition | Dominant |
//! |------|------|----------------|----------|
//! | [`Boundary`] | T2-P | κ (comparison/region) | κ Comparison |
//! | [`Harm`] | T2-P | ∂+∅ (boundary + void) | ∂ Boundary |
//! | [`Violation`] | T2-C | ∂+κ+∅ | ∂ Boundary |
//! | [`Monitoring`] | T2-C | σ+ρ (sequence + recursion) | σ Sequence |

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// κ (Comparison/Region) → Boundary
// ============================================================================

/// Numeric range defining a safe operating region.
///
/// Grounds κ (Comparison): the act of comparing a value against
/// defined bounds — the fundamental safety gate.
///
/// # Domain Mappings
/// - PV: therapeutic window (min effective dose ↔ max tolerated dose)
/// - Signal: PRR threshold range (normal ↔ signal)
/// - Guardian: operational parameter limits
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyBoundary<T: PartialOrd> {
    /// Lower bound (inclusive).
    lower: T,
    /// Upper bound (inclusive).
    upper: T,
}

impl<T: PartialOrd> SafetyBoundary<T> {
    /// Creates a new boundary. Swaps if lower > upper.
    #[must_use]
    pub fn new(lower: T, upper: T) -> Self {
        if lower <= upper {
            Self { lower, upper }
        } else {
            Self {
                lower: upper,
                upper: lower,
            }
        }
    }

    /// Returns true if the value falls within [lower, upper].
    #[must_use]
    pub fn contains(&self, value: &T) -> bool {
        value >= &self.lower && value <= &self.upper
    }

    /// Returns true if the value falls outside the boundary.
    #[must_use]
    pub fn is_violated(&self, value: &T) -> bool {
        !self.contains(value)
    }

    /// Returns a reference to the lower bound.
    #[must_use]
    pub fn lower(&self) -> &T {
        &self.lower
    }

    /// Returns a reference to the upper bound.
    #[must_use]
    pub fn upper(&self) -> &T {
        &self.upper
    }
}

impl SafetyBoundary<f64> {
    /// Returns the width of the boundary range.
    #[must_use]
    pub fn width(&self) -> f64 {
        self.upper - self.lower
    }
}

impl fmt::Display for SafetyBoundary<f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "∂:[{:.2},{:.2}]", self.lower, self.upper)
    }
}

impl fmt::Display for SafetyBoundary<i64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "∂:[{},{}]", self.lower, self.upper)
    }
}

// ============================================================================
// ∂+∅ (Boundary + Void) → Harm
// ============================================================================

/// Realized or potential harm state.
///
/// Composes ∂ (Boundary) + ∅ (Void): harm is a boundary breach that
/// may or may not have manifested. Absence of harm (None variant) is
/// informative per ∅.
///
/// # Domain Mappings
/// - PV: adverse event severity (none, suspected, confirmed)
/// - Signal: false positive vs true signal
/// - ToV: 8 harm types (A-H) at varying severity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Harm {
    /// No harm detected.
    None,
    /// Harm suspected but not confirmed.
    Potential {
        /// Description of the potential harm.
        description: String,
    },
    /// Harm has occurred with measured severity.
    Realized {
        /// Description of the realized harm.
        description: String,
        /// Severity score (0.0 = minimal, 1.0 = fatal).
        severity: f64,
    },
}

impl Harm {
    /// Returns true if any harm is present (potential or realized).
    #[must_use]
    pub fn is_present(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns severity: 0.0 for None, 0.5 for Potential, actual for Realized.
    #[must_use]
    pub fn severity(&self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Potential { .. } => 0.5,
            Self::Realized { severity, .. } => severity.clamp(0.0, 1.0),
        }
    }

    /// Returns the description if harm is present.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        match self {
            Self::None => None,
            Self::Potential { description } | Self::Realized { description, .. } => {
                Some(description)
            }
        }
    }
}

impl fmt::Display for Harm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "∂∅:none"),
            Self::Potential { .. } => write!(f, "∂∅:potential"),
            Self::Realized { severity, .. } => write!(f, "∂∅:realized({:.2})", severity),
        }
    }
}

// ============================================================================
// ∂+κ+∅ (Boundary + Comparison + Void) → Violation
// ============================================================================

/// A boundary that has been breached by an actual value.
///
/// Composes ∂ (Boundary) + κ (Comparison) + ∅ (Void): the concrete
/// act of a measurement exceeding defined limits, possibly indicating
/// harm (∅ = gap between expected and actual).
///
/// # Domain Mappings
/// - PV: dose exceeding maximum recommended amount
/// - Signal: PRR exceeding threshold (signal detected)
/// - Guardian: sensor reading outside operational bounds
///
/// Tier: T2-C (∂ + κ + ∅ — boundary with comparison and void)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundaryBreach {
    /// The boundary that was violated.
    boundary: SafetyBoundary<f64>,
    /// The actual value that caused the violation.
    actual: f64,
    /// Description of the violation context.
    description: String,
}

/// Backward-compatible alias.
#[deprecated(note = "use BoundaryBreach — F2 equivocation fix")]
pub type Violation = BoundaryBreach;

impl BoundaryBreach {
    /// Creates a new violation record.
    #[must_use]
    pub fn new(boundary: SafetyBoundary<f64>, actual: f64, description: impl Into<String>) -> Self {
        Self {
            boundary,
            actual,
            description: description.into(),
        }
    }

    /// Returns true if the actual value violates the boundary.
    #[must_use]
    pub fn is_violated(&self) -> bool {
        self.boundary.is_violated(&self.actual)
    }

    /// Returns the distance from the nearest boundary edge.
    /// Negative = within bounds, positive = outside bounds.
    #[must_use]
    pub fn margin(&self) -> f64 {
        if self.actual < *self.boundary.lower() {
            *self.boundary.lower() - self.actual
        } else if self.actual > *self.boundary.upper() {
            self.actual - *self.boundary.upper()
        } else {
            // Within bounds: negative margin (distance to nearest edge)
            let to_lower = self.actual - *self.boundary.lower();
            let to_upper = *self.boundary.upper() - self.actual;
            -to_lower.min(to_upper)
        }
    }

    /// Returns the violation description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl fmt::Display for BoundaryBreach {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.is_violated() {
            "VIOLATED"
        } else {
            "within"
        };
        write!(
            f,
            "∂κ∅:{:.2} vs {} [{}]",
            self.actual, self.boundary, status
        )
    }
}

// ============================================================================
// σ+ρ (Sequence + Recursion) → Monitoring
// ============================================================================

/// Recurring check state with violation tracking.
///
/// Composes σ (Sequence) + ρ (Recursion): a sequence of repeated
/// checks that recursively builds context from consecutive outcomes.
///
/// # Domain Mappings
/// - PV: periodic safety update report (PSUR) cycle
/// - Signal: ongoing signal monitoring with escalation
/// - Guardian: homeostasis iteration counter
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Monitoring {
    /// Total checks performed.
    check_count: u64,
    /// Whether the last check was clean (no violation).
    last_clean: bool,
    /// Number of consecutive violations.
    consecutive_violations: u32,
}

impl Monitoring {
    /// Creates a new monitoring state (clean, no checks).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            check_count: 0,
            last_clean: true,
            consecutive_violations: 0,
        }
    }

    /// Records a check result. `clean = true` means no violation found.
    #[must_use]
    pub const fn record_check(self, clean: bool) -> Self {
        if clean {
            Self {
                check_count: self.check_count + 1,
                last_clean: true,
                consecutive_violations: 0,
            }
        } else {
            Self {
                check_count: self.check_count + 1,
                last_clean: false,
                consecutive_violations: self.consecutive_violations + 1,
            }
        }
    }

    /// Returns true if consecutive violations >= 3 (alarm threshold).
    #[must_use]
    pub const fn is_alarming(&self) -> bool {
        self.consecutive_violations >= 3
    }

    /// Resets violation tracking while preserving check count.
    #[must_use]
    pub const fn reset(self) -> Self {
        Self {
            check_count: self.check_count,
            last_clean: true,
            consecutive_violations: 0,
        }
    }

    /// Returns the total number of checks performed.
    #[must_use]
    pub const fn check_count(&self) -> u64 {
        self.check_count
    }

    /// Returns the number of consecutive violations.
    #[must_use]
    pub const fn consecutive_violations(&self) -> u32 {
        self.consecutive_violations
    }
}

impl Default for Monitoring {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Monitoring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let alarm = if self.is_alarming() { " ALARM" } else { "" };
        write!(
            f,
            "σρ:({} checks, {} consecutive violations{})",
            self.check_count, self.consecutive_violations, alarm
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- SafetyBoundary (κ) --

    #[test]
    fn test_boundary_contains() {
        let b = SafetyBoundary::new(1.0, 10.0);
        assert!(b.contains(&5.0));
        assert!(b.contains(&1.0)); // inclusive
        assert!(b.contains(&10.0)); // inclusive
        assert!(!b.contains(&0.5));
        assert!(!b.contains(&10.5));
    }

    #[test]
    fn test_boundary_swaps_inverted() {
        let b: SafetyBoundary<f64> = SafetyBoundary::new(10.0, 1.0);
        assert!((*b.lower() - 1.0).abs() < f64::EPSILON);
        assert!((*b.upper() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_boundary_width() {
        let b = SafetyBoundary::new(2.0, 8.0);
        assert!((b.width() - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_boundary_display() {
        let b = SafetyBoundary::new(1.5, 3.5);
        assert_eq!(format!("{b}"), "∂:[1.50,3.50]");
    }

    // -- Harm (∂+∅) --

    #[test]
    fn test_harm_none() {
        let h = Harm::None;
        assert!(!h.is_present());
        assert!((h.severity() - 0.0).abs() < f64::EPSILON);
        assert!(h.description().is_none());
    }

    #[test]
    fn test_harm_potential() {
        let h = Harm::Potential {
            description: "suspected ADR".to_string(),
        };
        assert!(h.is_present());
        assert!((h.severity() - 0.5).abs() < f64::EPSILON);
        assert_eq!(h.description(), Some("suspected ADR"));
    }

    #[test]
    fn test_harm_realized() {
        let h = Harm::Realized {
            description: "hepatotoxicity".to_string(),
            severity: 0.85,
        };
        assert!(h.is_present());
        assert!((h.severity() - 0.85).abs() < f64::EPSILON);
        assert_eq!(h.description(), Some("hepatotoxicity"));
    }

    #[test]
    fn test_harm_display() {
        assert_eq!(format!("{}", Harm::None), "∂∅:none");
        assert_eq!(
            format!(
                "{}",
                Harm::Potential {
                    description: "x".to_string()
                }
            ),
            "∂∅:potential"
        );
        assert_eq!(
            format!(
                "{}",
                Harm::Realized {
                    description: "x".to_string(),
                    severity: 0.75
                }
            ),
            "∂∅:realized(0.75)"
        );
    }

    // -- BoundaryBreach (∂+κ+∅) --

    #[test]
    fn test_violation_detected() {
        let b = SafetyBoundary::new(2.0, 5.0);
        let v = BoundaryBreach::new(b, 7.0, "PRR exceeds threshold");
        assert!(v.is_violated());
        assert!((v.margin() - 2.0).abs() < f64::EPSILON);
        assert_eq!(v.description(), "PRR exceeds threshold");
    }

    #[test]
    fn test_violation_within_bounds() {
        let b = SafetyBoundary::new(0.0, 10.0);
        let v = BoundaryBreach::new(b, 3.0, "within range");
        assert!(!v.is_violated());
        assert!(v.margin() < 0.0); // negative = within bounds
    }

    #[test]
    fn test_violation_display() {
        let b = SafetyBoundary::new(1.0, 5.0);
        let v = BoundaryBreach::new(b, 6.5, "over");
        let display = format!("{v}");
        assert!(display.contains("VIOLATED"));
        assert!(display.contains("6.50"));
    }

    // -- Monitoring (σ+ρ) --

    #[test]
    fn test_monitoring_new() {
        let m = Monitoring::new();
        assert_eq!(m.check_count(), 0);
        assert_eq!(m.consecutive_violations(), 0);
        assert!(!m.is_alarming());
    }

    #[test]
    fn test_monitoring_clean_checks() {
        let m = Monitoring::new()
            .record_check(true)
            .record_check(true)
            .record_check(true);
        assert_eq!(m.check_count(), 3);
        assert_eq!(m.consecutive_violations(), 0);
        assert!(!m.is_alarming());
    }

    #[test]
    fn test_monitoring_alarm_threshold() {
        let m = Monitoring::new()
            .record_check(false)
            .record_check(false)
            .record_check(false);
        assert_eq!(m.consecutive_violations(), 3);
        assert!(m.is_alarming());
    }

    #[test]
    fn test_monitoring_violation_reset_on_clean() {
        let m = Monitoring::new()
            .record_check(false)
            .record_check(false)
            .record_check(true);
        assert_eq!(m.consecutive_violations(), 0);
        assert!(!m.is_alarming());
        assert_eq!(m.check_count(), 3);
    }

    #[test]
    fn test_monitoring_manual_reset() {
        let m = Monitoring::new()
            .record_check(false)
            .record_check(false)
            .record_check(false)
            .reset();
        assert!(!m.is_alarming());
        assert_eq!(m.check_count(), 3); // preserves count
        assert_eq!(m.consecutive_violations(), 0);
    }

    #[test]
    fn test_monitoring_display() {
        let m = Monitoring::new()
            .record_check(false)
            .record_check(false)
            .record_check(false);
        let display = format!("{m}");
        assert!(display.contains("ALARM"));
        assert!(display.contains("3 consecutive"));
    }
}
