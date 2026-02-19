//! T2-P newtypes and core enums for the CCP pharmacokinetic engine.
//!
//! # T1 Grounding
//! - ∝ (proportionality): All newtypes wrap f64 measures
//! - κ (comparison): Phase/Strategy/Interaction enums enable dispatch
//! - ∂ (boundary): Constructors enforce domain constraints

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::CcpError;

// ---------------------------------------------------------------------------
// T2-P Newtypes
// ---------------------------------------------------------------------------

/// Current support intensity in the system.
///
/// Tier: T2-P (f64 ∈ [0, ∞) — unclamped to allow superposition)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PlasmaLevel(pub f64);

impl PlasmaLevel {
    /// Zero plasma level (no active support).
    pub const ZERO: Self = Self(0.0);

    /// Clamp to non-negative.
    #[must_use]
    pub fn clamped(self) -> Self {
        Self(self.0.max(0.0))
    }

    /// Inner value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for PlasmaLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

/// Transfer efficiency of an intervention.
///
/// Tier: T2-P (f64 ∈ (0, 1])
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct BioAvailability(f64);

impl BioAvailability {
    /// Create a new bioavailability value.
    ///
    /// # Errors
    /// Returns `CcpError::InvalidBioavailability` if value is not in (0, 1].
    pub fn new(value: f64) -> Result<Self, CcpError> {
        if value > 0.0 && value <= 1.0 {
            Ok(Self(value))
        } else {
            Err(CcpError::InvalidBioavailability { value })
        }
    }

    /// Inner value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for BioAvailability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

/// Decay half-life in hours.
///
/// Tier: T2-P (f64 > 0)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct HalfLife(f64);

impl HalfLife {
    /// Create a new half-life value.
    ///
    /// # Errors
    /// Returns `CcpError::InvalidHalfLife` if value is not positive.
    pub fn new(value: f64) -> Result<Self, CcpError> {
        if value > 0.0 {
            Ok(Self(value))
        } else {
            Err(CcpError::InvalidHalfLife { value })
        }
    }

    /// Inner value in hours.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for HalfLife {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}h", self.0)
    }
}

/// Intervention intensity.
///
/// Tier: T2-P (f64 ∈ [0, 1])
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Dose(f64);

impl Dose {
    /// Zero dose (no intervention).
    pub const ZERO: Self = Self(0.0);

    /// Maximum dose (full intervention).
    pub const MAX: Self = Self(1.0);

    /// Create a new dose value.
    ///
    /// # Errors
    /// Returns `CcpError::InvalidDose` if value is not in [0, 1].
    pub fn new(value: f64) -> Result<Self, CcpError> {
        if (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(CcpError::InvalidDose { value })
        }
    }

    /// Inner value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for Dose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

/// Therapeutic window defining safe support levels.
///
/// Tier: T2-P (boundary pair)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TherapeuticWindow {
    /// Minimum effective level.
    pub lower: f64,
    /// Maximum safe level.
    pub upper: f64,
}

impl TherapeuticWindow {
    /// Create a new therapeutic window.
    ///
    /// # Errors
    /// Returns `CcpError::InvalidWindow` if lower >= upper or bounds invalid.
    pub fn new(lower: f64, upper: f64) -> Result<Self, CcpError> {
        if lower < upper && lower >= 0.0 && upper <= 1.0 {
            Ok(Self { lower, upper })
        } else {
            Err(CcpError::InvalidWindow { lower, upper })
        }
    }

    /// Default therapeutic window: [0.3, 0.8].
    #[must_use]
    pub fn default_window() -> Self {
        Self {
            lower: 0.3,
            upper: 0.8,
        }
    }

    /// Check if a plasma level is within the window.
    #[must_use]
    pub fn contains(&self, level: PlasmaLevel) -> bool {
        level.0 >= self.lower && level.0 <= self.upper
    }

    /// Width of the window.
    #[must_use]
    pub fn width(&self) -> f64 {
        self.upper - self.lower
    }
}

impl fmt::Display for TherapeuticWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:.2}, {:.2}]", self.lower, self.upper)
    }
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Care process phases (5-phase FSM).
///
/// Tier: T2-P (enum over σ sequence positions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Phase {
    /// Gather information and context.
    Collect,
    /// Evaluate needs and capabilities.
    Assess,
    /// Design intervention strategy.
    Plan,
    /// Execute the support plan.
    Implement,
    /// Monitor outcomes and reinforce.
    FollowUp,
}

impl Phase {
    /// Ordinal position (0-indexed).
    #[must_use]
    pub fn ordinal(self) -> usize {
        match self {
            Self::Collect => 0,
            Self::Assess => 1,
            Self::Plan => 2,
            Self::Implement => 3,
            Self::FollowUp => 4,
        }
    }

    /// All phases in order.
    #[must_use]
    pub fn all() -> &'static [Phase] {
        &[
            Self::Collect,
            Self::Assess,
            Self::Plan,
            Self::Implement,
            Self::FollowUp,
        ]
    }

    /// Next phase in sequence, if any.
    #[must_use]
    pub fn next(self) -> Option<Phase> {
        match self {
            Self::Collect => Some(Self::Assess),
            Self::Assess => Some(Self::Plan),
            Self::Plan => Some(Self::Implement),
            Self::Implement => Some(Self::FollowUp),
            Self::FollowUp => None,
        }
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Collect => write!(f, "Collect"),
            Self::Assess => write!(f, "Assess"),
            Self::Plan => write!(f, "Plan"),
            Self::Implement => write!(f, "Implement"),
            Self::FollowUp => write!(f, "FollowUp"),
        }
    }
}

/// Dosing strategy for interventions.
///
/// Tier: T2-P (enum classifying κ comparison outcomes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DosingStrategy {
    /// Below therapeutic range — needs increase.
    Subtherapeutic,
    /// Within therapeutic range — maintain.
    Therapeutic,
    /// High initial dose to reach target quickly.
    Loading,
    /// Steady-state dose to sustain level.
    Maintenance,
}

impl fmt::Display for DosingStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Subtherapeutic => write!(f, "Subtherapeutic"),
            Self::Therapeutic => write!(f, "Therapeutic"),
            Self::Loading => write!(f, "Loading"),
            Self::Maintenance => write!(f, "Maintenance"),
        }
    }
}

/// Type of interaction between two interventions.
///
/// Tier: T2-P (enum classifying ∝ proportionality modifiers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InteractionType {
    /// Combined effect > sum of parts (1.5x).
    Synergistic,
    /// Combined effect < sum of parts (0.7x).
    Antagonistic,
    /// Combined effect = sum of parts (1.0x).
    Additive,
    /// One intervention amplifies the other (2.0x).
    Potentiating,
}

impl InteractionType {
    /// Multiplier applied to combined plasma level.
    #[must_use]
    pub fn multiplier(self) -> f64 {
        match self {
            Self::Synergistic => 1.5,
            Self::Antagonistic => 0.7,
            Self::Additive => 1.0,
            Self::Potentiating => 2.0,
        }
    }
}

impl fmt::Display for InteractionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Synergistic => write!(f, "Synergistic"),
            Self::Antagonistic => write!(f, "Antagonistic"),
            Self::Additive => write!(f, "Additive"),
            Self::Potentiating => write!(f, "Potentiating"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plasma_level_clamp() {
        let pl = PlasmaLevel(-0.5).clamped();
        assert!((pl.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bioavailability_bounds() {
        assert!(BioAvailability::new(0.0).is_err());
        assert!(BioAvailability::new(1.1).is_err());
        assert!(BioAvailability::new(0.5).is_ok());
        assert!(BioAvailability::new(1.0).is_ok());
    }

    #[test]
    fn half_life_bounds() {
        assert!(HalfLife::new(0.0).is_err());
        assert!(HalfLife::new(-1.0).is_err());
        assert!(HalfLife::new(24.0).is_ok());
    }

    #[test]
    fn dose_bounds() {
        assert!(Dose::new(-0.1).is_err());
        assert!(Dose::new(1.1).is_err());
        assert!(Dose::new(0.0).is_ok());
        assert!(Dose::new(1.0).is_ok());
        assert!(Dose::new(0.5).is_ok());
    }

    #[test]
    fn therapeutic_window_validation() {
        assert!(TherapeuticWindow::new(0.8, 0.2).is_err());
        assert!(TherapeuticWindow::new(-0.1, 0.5).is_err());
        assert!(TherapeuticWindow::new(0.2, 1.1).is_err());
        assert!(TherapeuticWindow::new(0.3, 0.8).is_ok());
    }

    #[test]
    fn window_contains() {
        let w = TherapeuticWindow::default_window();
        assert!(w.contains(PlasmaLevel(0.5)));
        assert!(!w.contains(PlasmaLevel(0.1)));
        assert!(!w.contains(PlasmaLevel(0.9)));
    }

    #[test]
    fn phase_ordering() {
        assert_eq!(Phase::Collect.ordinal(), 0);
        assert_eq!(Phase::FollowUp.ordinal(), 4);
        assert_eq!(Phase::Collect.next(), Some(Phase::Assess));
        assert_eq!(Phase::FollowUp.next(), None);
    }

    #[test]
    fn interaction_multipliers() {
        assert!((InteractionType::Synergistic.multiplier() - 1.5).abs() < f64::EPSILON);
        assert!((InteractionType::Antagonistic.multiplier() - 0.7).abs() < f64::EPSILON);
        assert!((InteractionType::Additive.multiplier() - 1.0).abs() < f64::EPSILON);
        assert!((InteractionType::Potentiating.multiplier() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn display_impls() {
        assert_eq!(PlasmaLevel(0.5).to_string(), "0.5000");
        assert_eq!(Phase::Plan.to_string(), "Plan");
        assert_eq!(DosingStrategy::Loading.to_string(), "Loading");
        assert_eq!(InteractionType::Synergistic.to_string(), "Synergistic");
    }
}
