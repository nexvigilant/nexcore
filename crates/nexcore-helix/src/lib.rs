//! # nexcore-helix — Conservation Law as Computable Geometry
//!
//! The conservation law `∃ = ∂(×(ς, ∅))` encodes a helix:
//! - It **advances** (→) — each turn raises resolution
//! - It **returns** (κ) — same angular truth at every altitude
//! - It **bounds** (∂) — radius separates inside from outside
//!
//! Five turns at increasing altitude, same truth at higher resolution:
//!
//! | Turn | Name               | What         | Encoding                    |
//! |------|--------------------|-------------|-----------------------------|
//! | 0    | Primitives         | The alphabet | 15 symbols                  |
//! | 1    | Conservation       | The grammar  | ∃ = ∂(×(ς, ∅))              |
//! | 2    | Crystalbook        | The laws     | 8 constraints               |
//! | 3    | Derivative Identity| The calculus | How the grammar changes     |
//! | 4    | Mutualism          | The point    | Why the grammar exists      |
//!
//! By Matthew A. Campion, PharmD.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

pub mod dna;
pub mod engine;

use serde::{Deserialize, Serialize};

// ---- Core Types ----

/// A point on the knowledge helix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelixPosition {
    pub turn: Turn,
    pub theta: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// The five turns of the knowledge helix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Turn {
    Primitives = 0,
    Conservation = 1,
    Crystalbook = 2,
    DerivativeIdentity = 3,
    Mutualism = 4,
}

impl Turn {
    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Primitives => "Primitives",
            Self::Conservation => "Conservation",
            Self::Crystalbook => "Crystalbook",
            Self::DerivativeIdentity => "Derivative Identity",
            Self::Mutualism => "Mutualism",
        }
    }

    /// What this turn represents.
    pub fn what(self) -> &'static str {
        match self {
            Self::Primitives => "The alphabet",
            Self::Conservation => "The grammar",
            Self::Crystalbook => "The laws",
            Self::DerivativeIdentity => "The calculus",
            Self::Mutualism => "The point",
        }
    }

    /// The encoding at this altitude.
    pub fn encoding(self) -> &'static str {
        match self {
            Self::Primitives => "15 symbols (9 prime + 6 composite)",
            Self::Conservation => "∃ = ∂(×(ς, ∅))",
            Self::Crystalbook => "8 constraints on the grammar",
            Self::DerivativeIdentity => "How the grammar changes (5 calculus rules hold)",
            Self::Mutualism => "Refusal to produce ∃ for self at cost of another's ∃",
        }
    }

    /// Next turn, if any.
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Primitives => Some(Self::Conservation),
            Self::Conservation => Some(Self::Crystalbook),
            Self::Crystalbook => Some(Self::DerivativeIdentity),
            Self::DerivativeIdentity => Some(Self::Mutualism),
            Self::Mutualism => None,
        }
    }

    /// From integer.
    pub fn from_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(Self::Primitives),
            1 => Some(Self::Conservation),
            2 => Some(Self::Crystalbook),
            3 => Some(Self::DerivativeIdentity),
            4 => Some(Self::Mutualism),
            _ => None,
        }
    }
}

// ---- Conservation Law ----

/// The three inputs to the conservation law.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConservationInput {
    /// ∂ — boundary sharpness [0,1]
    pub boundary: f64,
    /// ς — state richness [0,1]
    pub state: f64,
    /// ∅ — void clarity [0,1]
    pub void: f64,
}

/// Result of computing ∃ = ∂(×(ς, ∅)).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservationResult {
    pub existence: f64,
    pub boundary: f64,
    pub state: f64,
    pub void: f64,
    pub weakest: WeakestPrimitive,
    pub classification: ExistenceClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeakestPrimitive {
    Boundary,
    State,
    Void,
    None,
}

impl WeakestPrimitive {
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Boundary => "∂",
            Self::State => "ς",
            Self::Void => "∅",
            Self::None => "—",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExistenceClass {
    Strong,
    Moderate,
    Weak,
    Collapsing,
}

impl ExistenceClass {
    pub fn label(self) -> &'static str {
        match self {
            Self::Strong => "strong",
            Self::Moderate => "moderate",
            Self::Weak => "weak",
            Self::Collapsing => "collapsing",
        }
    }
}

/// Compute ∃ = ∂(×(ς, ∅)).
pub fn conservation(input: ConservationInput) -> ConservationResult {
    let b = input.boundary.clamp(0.0, 1.0);
    let s = input.state.clamp(0.0, 1.0);
    let v = input.void.clamp(0.0, 1.0);
    let existence = b * s * v;

    let weakest = if b <= s && b <= v {
        WeakestPrimitive::Boundary
    } else if s <= v {
        WeakestPrimitive::State
    } else {
        WeakestPrimitive::Void
    };

    let classification = if existence >= 0.5 {
        ExistenceClass::Strong
    } else if existence >= 0.2 {
        ExistenceClass::Moderate
    } else if existence >= 0.05 {
        ExistenceClass::Weak
    } else {
        ExistenceClass::Collapsing
    };

    ConservationResult {
        existence,
        boundary: b,
        state: s,
        void: v,
        weakest,
        classification,
    }
}

// ---- Partial Derivatives ----

/// Partial derivatives of ∃ with respect to each primitive.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Derivatives {
    /// ∂∃/∂∂ = ς × ∅
    pub d_boundary: f64,
    /// ∂∃/∂ς = ∂ × ∅
    pub d_state: f64,
    /// ∂∃/∂∅ = ∂ × ς
    pub d_void: f64,
    /// Which partial derivative is largest.
    pub highest_leverage: WeakestPrimitive,
}

/// Compute partial derivatives of ∃ = ∂(×(ς, ∅)).
pub fn derivatives(input: ConservationInput) -> Derivatives {
    let b = input.boundary.clamp(0.0, 1.0);
    let s = input.state.clamp(0.0, 1.0);
    let v = input.void.clamp(0.0, 1.0);

    let d_b = s * v;
    let d_s = b * v;
    let d_v = b * s;

    let highest = if d_b >= d_s && d_b >= d_v {
        WeakestPrimitive::Boundary
    } else if d_s >= d_v {
        WeakestPrimitive::State
    } else {
        WeakestPrimitive::Void
    };

    Derivatives {
        d_boundary: d_b,
        d_state: d_s,
        d_void: d_v,
        highest_leverage: highest,
    }
}

// ---- Mutualism ----

/// Result of a mutualism test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualismResult {
    pub mutualistic: bool,
    pub delta_self: f64,
    pub delta_other: f64,
    pub net_existence: f64,
    pub classification: MutualismClass,
    pub conservation_holds: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutualismClass {
    Mutualistic,
    Sacrificial,
    Parasitic,
    ParasiticNetNegative,
    Destructive,
}

impl MutualismClass {
    pub fn label(self) -> &'static str {
        match self {
            Self::Mutualistic => "mutualistic",
            Self::Sacrificial => "sacrificial",
            Self::Parasitic => "parasitic",
            Self::ParasiticNetNegative => "parasitic_net_negative",
            Self::Destructive => "destructive",
        }
    }
}

/// Test whether an action serves mutualism.
pub fn mutualism_test(
    self_before: f64,
    self_after: f64,
    other_before: f64,
    other_after: f64,
) -> MutualismResult {
    let delta_self = self_after - self_before;
    let delta_other = other_after - other_before;
    let net = delta_self + delta_other;

    let (mutualistic, class) = if delta_self >= 0.0 && delta_other >= 0.0 {
        (true, MutualismClass::Mutualistic)
    } else if delta_self > 0.0 && delta_other < 0.0 {
        if delta_self.abs() <= delta_other.abs() {
            (false, MutualismClass::ParasiticNetNegative)
        } else {
            (false, MutualismClass::Parasitic)
        }
    } else if delta_self < 0.0 && delta_other > 0.0 {
        (true, MutualismClass::Sacrificial)
    } else {
        (false, MutualismClass::Destructive)
    };

    MutualismResult {
        mutualistic,
        delta_self,
        delta_other,
        net_existence: net,
        classification: class,
        conservation_holds: net >= 0.0,
    }
}

// ---- Helix Geometry ----

/// Compute 3D position on the helix.
pub fn helix_position(turn: Turn, theta: f64) -> HelixPosition {
    let altitude = (turn as usize) as f64 + theta / (2.0 * std::f64::consts::PI);
    HelixPosition {
        turn,
        theta,
        x: theta.cos(),
        y: theta.sin(),
        z: altitude,
    }
}

/// Check if a system can advance to the next helix turn.
pub fn can_advance(turn: Turn, existence: f64) -> bool {
    let threshold = match turn {
        Turn::Primitives => 0.1,
        Turn::Conservation => 0.2,
        Turn::Crystalbook => 0.3,
        Turn::DerivativeIdentity => 0.4,
        Turn::Mutualism => f64::INFINITY, // already at top
    };
    existence >= threshold
}

// ---- Crystalbook Law Routing ----

/// Which Crystalbook laws bind a system based on its weakest primitive.
pub fn binding_laws(weakest: WeakestPrimitive) -> &'static [u8] {
    match weakest {
        WeakestPrimitive::Boundary => &[1, 3, 8],
        WeakestPrimitive::State => &[2, 5, 7],
        WeakestPrimitive::Void => &[4, 6],
        WeakestPrimitive::None => &[1, 2, 3, 4, 5, 6, 7, 8],
    }
}

/// Vice risk associated with the weakest primitive.
pub fn vice_risk(weakest: WeakestPrimitive) -> &'static str {
    match weakest {
        WeakestPrimitive::Boundary => "Pride, Lust, Corruption",
        WeakestPrimitive::State => "Greed, Gluttony, Sloth",
        WeakestPrimitive::Void => "Envy, Wrath",
        WeakestPrimitive::None => "Low — all primitives healthy",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conservation_strong() {
        let r = conservation(ConservationInput {
            boundary: 0.9,
            state: 0.8,
            void: 0.7,
        });
        assert_eq!(r.classification, ExistenceClass::Strong);
        assert!((r.existence - 0.504).abs() < 0.001);
    }

    #[test]
    fn conservation_collapsing_zero_boundary() {
        let r = conservation(ConservationInput {
            boundary: 0.0,
            state: 1.0,
            void: 1.0,
        });
        assert_eq!(r.classification, ExistenceClass::Collapsing);
        assert_eq!(r.existence, 0.0);
        assert_eq!(r.weakest, WeakestPrimitive::Boundary);
    }

    #[test]
    fn conservation_weak_state() {
        let r = conservation(ConservationInput {
            boundary: 0.95,
            state: 0.05,
            void: 0.6,
        });
        assert_eq!(r.classification, ExistenceClass::Collapsing);
        assert_eq!(r.weakest, WeakestPrimitive::State);
    }

    #[test]
    fn derivatives_highest_leverage() {
        let d = derivatives(ConservationInput {
            boundary: 0.9,
            state: 0.3,
            void: 0.8,
        });
        // d_boundary = 0.3*0.8 = 0.24
        // d_state = 0.9*0.8 = 0.72  <-- highest
        // d_void = 0.9*0.3 = 0.27
        assert_eq!(d.highest_leverage, WeakestPrimitive::State);
    }

    #[test]
    fn mutualism_both_gain() {
        let r = mutualism_test(0.5, 0.7, 0.5, 0.6);
        assert!(r.mutualistic);
        assert_eq!(r.classification, MutualismClass::Mutualistic);
        assert!(r.conservation_holds);
    }

    #[test]
    fn mutualism_parasitic() {
        let r = mutualism_test(0.3, 0.8, 0.7, 0.4);
        assert!(!r.mutualistic);
        assert_eq!(r.classification, MutualismClass::Parasitic);
    }

    #[test]
    fn mutualism_destructive() {
        let r = mutualism_test(0.5, 0.3, 0.5, 0.2);
        assert!(!r.mutualistic);
        assert_eq!(r.classification, MutualismClass::Destructive);
        assert!(!r.conservation_holds);
    }

    #[test]
    fn helix_position_at_origin() {
        let p = helix_position(Turn::Primitives, 0.0);
        assert_eq!(p.x, 1.0);
        assert!(p.y.abs() < 1e-10);
        assert_eq!(p.z, 0.0);
    }

    #[test]
    fn advance_gate() {
        assert!(can_advance(Turn::Primitives, 0.15));
        assert!(!can_advance(Turn::Primitives, 0.05));
        assert!(!can_advance(Turn::Mutualism, 1.0)); // can't advance past top
    }

    #[test]
    fn binding_laws_boundary_weak() {
        assert_eq!(binding_laws(WeakestPrimitive::Boundary), &[1, 3, 8]);
    }

    #[test]
    fn turn_traversal() {
        let mut t = Turn::Primitives;
        let mut count = 0;
        while let Some(next) = t.next() {
            t = next;
            count += 1;
        }
        assert_eq!(count, 4);
        assert_eq!(t, Turn::Mutualism);
    }
}
