//! # Patient/Recipient T2-P and T2-C Primitives
//!
//! Types modeling entities that receive actions and their monitoring state.
//!
//! | Type | Tier | LP Composition | Dominant |
//! |------|------|----------------|----------|
//! | [`Recipient`] | T2-P | → (arrow target) | → Causality |
//! | [`Vulnerable`] | T2-C | ∂+∃ (boundary + existence) | ∂ Boundary |
//! | [`Tracked`] | T2-C | Σ+ρ (sum + sequence) | σ Sequence |

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// → (Causality, arrow target) → Recipient
// ============================================================================

/// Entity receiving an action or effect.
///
/// Grounds → (Causality): the arrow's target — who or what receives
/// the causal consequence.
///
/// # Domain Mappings
/// - PV: patient receiving a drug (ICSR subject)
/// - Signal: drug-event pair target entity
/// - Guardian: protected system or user
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Recipient(String);

impl Recipient {
    /// Creates a new recipient with an identifier.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the recipient identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.0
    }

    /// Returns true if the recipient has no identifying information.
    #[must_use]
    pub fn is_anonymous(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Recipient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_anonymous() {
            write!(f, "→:anonymous")
        } else {
            write!(f, "→:{}", self.0)
        }
    }
}

// ============================================================================
// ∂+∃ (Boundary + Existence) → Vulnerable
// ============================================================================

/// Recipient with measured susceptibility to harm.
///
/// Composes ∂ (Boundary) + ∃ (Existence): an existing entity whose
/// boundary defenses are quantifiably weakened.
///
/// # Domain Mappings
/// - PV: pediatric/geriatric/immunocompromised patient
/// - Signal: drug with narrow therapeutic index
/// - Guardian: system with known vulnerability surface
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vulnerable {
    /// The recipient at risk.
    recipient: Recipient,
    /// Susceptibility score (0.0 = resilient, 1.0 = maximally vulnerable).
    susceptibility: f64,
}

impl Vulnerable {
    /// Creates a new vulnerable recipient.
    #[must_use]
    pub fn new(recipient: Recipient, susceptibility: f64) -> Self {
        Self {
            recipient,
            susceptibility: susceptibility.clamp(0.0, 1.0),
        }
    }

    /// Returns true if susceptibility exceeds 0.7 (high-risk threshold).
    #[must_use]
    pub fn is_high_risk(&self) -> bool {
        self.susceptibility > 0.7
    }

    /// Returns the underlying recipient.
    #[must_use]
    pub fn recipient(&self) -> &Recipient {
        &self.recipient
    }

    /// Returns the susceptibility score.
    #[must_use]
    pub fn susceptibility(&self) -> f64 {
        self.susceptibility
    }
}

impl fmt::Display for Vulnerable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "∂∃:{}(susceptibility={:.2})",
            self.recipient.id(),
            self.susceptibility
        )
    }
}

// ============================================================================
// Σ+σ (Sum + Sequence) → Tracked
// ============================================================================

/// Longitudinally monitored recipient with observation history.
///
/// Composes Σ (Sum) + σ (Sequence): an accumulating series of
/// observations over time with active/closed lifecycle.
///
/// # Domain Mappings
/// - PV: patient follow-up in post-marketing surveillance
/// - Signal: drug under enhanced monitoring
/// - CCP: care process recipient through 5 phases
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tracked {
    /// The monitored recipient.
    recipient: Recipient,
    /// Number of observations recorded.
    observation_count: u64,
    /// Whether tracking is still active.
    active: bool,
}

impl Tracked {
    /// Creates a new actively tracked recipient.
    #[must_use]
    pub fn new(recipient: Recipient) -> Self {
        Self {
            recipient,
            observation_count: 0,
            active: true,
        }
    }

    /// Records an observation. Returns `None` if tracking is closed.
    #[must_use]
    pub fn observe(self) -> Option<Self> {
        if !self.active {
            return None;
        }
        Some(Self {
            recipient: self.recipient,
            observation_count: self.observation_count + 1,
            active: self.active,
        })
    }

    /// Closes tracking, preventing further observations.
    #[must_use]
    pub fn close(self) -> Self {
        Self {
            recipient: self.recipient,
            observation_count: self.observation_count,
            active: false,
        }
    }

    /// Returns true if tracking is still active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Returns the number of observations recorded.
    #[must_use]
    pub const fn observation_count(&self) -> u64 {
        self.observation_count
    }

    /// Returns the tracked recipient.
    #[must_use]
    pub fn recipient(&self) -> &Recipient {
        &self.recipient
    }
}

impl fmt::Display for Tracked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.active { "active" } else { "closed" };
        write!(
            f,
            "Σσ:{}({} obs, {})",
            self.recipient.id(),
            self.observation_count,
            status
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- Recipient (→) --

    #[test]
    fn test_recipient_new() {
        let r = Recipient::new("PAT-001");
        assert_eq!(r.id(), "PAT-001");
        assert!(!r.is_anonymous());
    }

    #[test]
    fn test_recipient_anonymous() {
        let r = Recipient::new("");
        assert!(r.is_anonymous());
        assert_eq!(format!("{r}"), "→:anonymous");
    }

    #[test]
    fn test_recipient_display() {
        let r = Recipient::new("subject-42");
        assert_eq!(format!("{r}"), "→:subject-42");
    }

    // -- Vulnerable (∂+∃) --

    #[test]
    fn test_vulnerable_new() {
        let r = Recipient::new("PAT-002");
        let v = Vulnerable::new(r, 0.85);
        assert!(v.is_high_risk());
        assert!((v.susceptibility() - 0.85).abs() < f64::EPSILON);
        assert_eq!(v.recipient().id(), "PAT-002");
    }

    #[test]
    fn test_vulnerable_low_risk() {
        let r = Recipient::new("PAT-003");
        let v = Vulnerable::new(r, 0.3);
        assert!(!v.is_high_risk());
    }

    #[test]
    fn test_vulnerable_clamping() {
        let r = Recipient::new("PAT-004");
        let v = Vulnerable::new(r, 1.5);
        assert!((v.susceptibility() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_vulnerable_display() {
        let r = Recipient::new("PAT-005");
        let v = Vulnerable::new(r, 0.9);
        assert_eq!(format!("{v}"), "∂∃:PAT-005(susceptibility=0.90)");
    }

    // -- Tracked (Σ+σ) --

    #[test]
    fn test_tracked_new() {
        let r = Recipient::new("PAT-006");
        let t = Tracked::new(r);
        assert!(t.is_active());
        assert_eq!(t.observation_count(), 0);
    }

    #[test]
    fn test_tracked_observe() {
        let r = Recipient::new("PAT-007");
        let t = Tracked::new(r);
        let t = t.observe();
        assert!(t.is_some());
        if let Some(t) = t {
            assert_eq!(t.observation_count(), 1);
            let t = t.observe();
            assert!(t.is_some());
            if let Some(t) = t {
                assert_eq!(t.observation_count(), 2);
            }
        }
    }

    #[test]
    fn test_tracked_close_blocks_observe() {
        let r = Recipient::new("PAT-008");
        let t = Tracked::new(r).close();
        assert!(!t.is_active());
        assert!(t.observe().is_none());
    }

    #[test]
    fn test_tracked_display() {
        let r = Recipient::new("PAT-009");
        let t = Tracked::new(r);
        assert_eq!(format!("{t}"), "Σσ:PAT-009(0 obs, active)");
        let t = t.close();
        assert_eq!(format!("{t}"), "Σσ:PAT-009(0 obs, closed)");
    }
}
