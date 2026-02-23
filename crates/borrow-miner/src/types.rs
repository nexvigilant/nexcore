//! Game types - OreType, Particle, FloatingScore
//!
//! ## Primitive Tiers (per Codex)
//! - T2-P: Score, Combo, Depth, Points, Lifetime, ParticleId
//! - T2-C: Particle, FloatingScore (composed T2-P)
//! - T3: OreType (domain-specific)

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign};

// ═══════════════════════════════════════════════════════════════════════════
// T2-P PRIMITIVE WRAPPERS
// ═══════════════════════════════════════════════════════════════════════════

/// Score value - accumulated points
/// Tier: T2-P (newtype over u64)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Score(pub u64);

impl Score {
    pub const ZERO: Self = Self(0);

    pub fn saturating_add(self, points: Points) -> Self {
        Self(self.0.saturating_add(points.0))
    }
}

impl Add<Points> for Score {
    type Output = Self;
    fn add(self, rhs: Points) -> Self::Output {
        self.saturating_add(rhs)
    }
}

impl AddAssign<Points> for Score {
    fn add_assign(&mut self, rhs: Points) {
        *self = self.saturating_add(rhs);
    }
}

/// Points earned from an action
/// Tier: T2-P (newtype over u64)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Points(pub u64);

impl Points {
    pub const ZERO: Self = Self(0);

    pub fn from_ore(ore: &OreType, combo: Combo, depth: Depth) -> Self {
        let combo_mult = 1.0 + (combo.0 as f64 * 0.1);
        let depth_mult = depth.0;
        Self((ore.base_value() as f64 * combo_mult * depth_mult) as u64)
    }
}

/// Combo multiplier counter
/// Tier: T2-P (newtype over u32)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Combo(pub u32);

impl Combo {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(crate::MAX_COMBO);

    pub fn increment(self) -> Self {
        Self(self.0.saturating_add(1).min(Self::MAX.0))
    }

    pub fn decrement(self) -> Self {
        Self(self.0.saturating_sub(1))
    }

    pub fn reset(self) -> Self {
        Self::ZERO
    }
}

/// Mining depth multiplier
/// Tier: T2-P (newtype over f64)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Depth(pub f64);

impl Default for Depth {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Depth {
    pub fn increase(self, delta: f64) -> Self {
        Self(self.0 + delta)
    }
}

/// Lifetime for particles/effects (0.0 to 1.0)
/// Tier: T2-P (newtype over f64)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Lifetime(pub f64);

impl Default for Lifetime {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Lifetime {
    pub fn decay(self, rate: f64) -> Self {
        Self((self.0 - rate).max(0.0))
    }

    pub fn is_expired(&self) -> bool {
        self.0 <= 0.0
    }
}

impl fmt::Display for Lifetime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique particle identifier
/// Tier: T2-P (newtype over usize)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParticleId(pub usize);

impl ParticleId {
    pub fn next_batch(self, batch_size: usize) -> Self {
        Self(self.0.wrapping_add(batch_size))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T3 DOMAIN TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Ore types with weighted rarity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OreType {
    Iron,
    Copper,
    Silver,
    Gold,
    Platinum,
}

impl OreType {
    pub fn base_value(&self) -> u64 {
        match self {
            Self::Iron => 10,
            Self::Copper => 25,
            Self::Silver => 50,
            Self::Gold => 100,
            Self::Platinum => 250,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Iron => "Iron",
            Self::Copper => "Copper",
            Self::Silver => "Silver",
            Self::Gold => "Gold",
            Self::Platinum => "Platinum",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::Iron => "#94a3b8",
            Self::Copper => "#f97316",
            Self::Silver => "#e2e8f0",
            Self::Gold => "#fbbf24",
            Self::Platinum => "#22d3ee",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Iron => "⚫",
            Self::Copper => "🟠",
            Self::Silver => "⚪",
            Self::Gold => "🟡",
            Self::Platinum => "💎",
        }
    }

    /// Roll random ore: Iron 40%, Copper 30%, Silver 18%, Gold 10%, Platinum 2%
    pub fn roll() -> Self {
        let roll = random() * 100.0;
        if roll < 2.0 {
            Self::Platinum
        } else if roll < 12.0 {
            Self::Gold
        } else if roll < 30.0 {
            Self::Silver
        } else if roll < 60.0 {
            Self::Copper
        } else {
            Self::Iron
        }
    }

    pub fn is_rare(&self) -> bool {
        matches!(self, Self::Gold | Self::Platinum)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T2-C COMPOSITE TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Particle effect
/// Tier: T2-C (composed of T2-P primitives)
#[derive(Debug, Clone)]
pub struct Particle {
    pub id: ParticleId,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub color: String,
    pub life: Lifetime,
    pub size: f64,
}

/// Floating score indicator
/// Tier: T2-C (composed of T2-P primitives)
#[derive(Debug, Clone)]
pub struct FloatingScore {
    pub id: ParticleId,
    pub x: f64,
    pub y: f64,
    pub value: Points,
    pub life: Lifetime,
}

/// Pseudo-random number generator using system time
pub fn random() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos as f64 % 10000.0) / 10000.0
}
