//! Scoring module — Ferro Forge quality computation and round tracking.
//!
//! Merged from ferro-forge-engine. T1 Primitives: N(Quantity) + ∂(Boundary) + κ(Comparison) + ς(State)
//!
//! ForgeThresholds delegates to ForgeStrategy for evolved parameter values.

use crate::ForgeStrategy;
use serde::{Deserialize, Serialize};

/// Thresholds derived from ForgeStrategy's evolved parameters.
///
/// This is a focused view of the 4 game-relevant thresholds from the full
/// 11-parameter ForgeStrategy. Use ForgeStrategy directly for the complete set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeThresholds {
    /// Don't over-polish mid-task. ~31% quality is shippable if it works.
    pub quality_floor: f64,
    /// Almost never abandon. Only bail at <2.7% confidence.
    pub abandon_threshold: f64,
    /// Slow down at API/module boundaries. Errors cascade further there.
    pub boundary_caution: f64,
    /// Don't exhaustively explore one approach — promote at 89% readiness.
    pub tier_promotion_eagerness: f64,
    /// Clear cheap blockers first to unblock progress faster.
    pub fix_easiest_first: bool,
}

impl Default for ForgeThresholds {
    fn default() -> Self {
        Self::from(&ForgeStrategy::default())
    }
}

impl From<&ForgeStrategy> for ForgeThresholds {
    fn from(strategy: &ForgeStrategy) -> Self {
        Self {
            quality_floor: strategy.quality_floor,
            abandon_threshold: strategy.abandon_threshold,
            boundary_caution: strategy.boundary_caution,
            tier_promotion_eagerness: strategy.tier_promotion_eagerness,
            fix_easiest_first: strategy.fix_easiest_first,
        }
    }
}

/// Quality score computed from game state.
///
/// Q = 0.40×primitives + 0.25×combat + 0.20×efficiency + 0.15×survival
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FerroScore {
    /// Primitives collected (0-16 T1 symbols)
    pub primitive_score: f64,
    /// Combat effectiveness (enemies killed / seen)
    pub combat_score: f64,
    /// Turn efficiency (ideal / actual)
    pub efficiency_score: f64,
    /// Survival (current HP / max HP)
    pub survival_score: f64,
    /// Total weighted score
    pub total: f64,
    /// Letter grade
    pub grade: char,
}

impl FerroScore {
    const W_PRIM: f64 = 0.40;
    const W_COMBAT: f64 = 0.25;
    const W_EFFICIENCY: f64 = 0.20;
    const W_SURVIVAL: f64 = 0.15;

    /// Compute quality score from game state.
    pub fn compute(
        primitives_collected: u8,
        enemies_killed: u32,
        enemies_seen: u32,
        actual_turns: u32,
        ideal_turns: u32,
        current_hp: i32,
        max_hp: i32,
    ) -> Self {
        let primitive_score = f64::from(primitives_collected) / 16.0;

        let combat_score = if enemies_seen > 0 {
            f64::from(enemies_killed) / f64::from(enemies_seen)
        } else {
            1.0
        };

        let efficiency_score = if actual_turns > 0 {
            (f64::from(ideal_turns) / f64::from(actual_turns)).min(1.0)
        } else {
            1.0
        };

        let survival_score = if max_hp > 0 {
            (f64::from(current_hp) / f64::from(max_hp)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let total = Self::W_PRIM * primitive_score
            + Self::W_COMBAT * combat_score
            + Self::W_EFFICIENCY * efficiency_score
            + Self::W_SURVIVAL * survival_score;

        let grade = match total {
            t if t >= 0.90 => 'S',
            t if t >= 0.80 => 'A',
            t if t >= 0.65 => 'B',
            t if t >= 0.50 => 'C',
            t if t >= 0.313 => 'D',
            _ => 'F',
        };

        Self {
            primitive_score,
            combat_score,
            efficiency_score,
            survival_score,
            total,
            grade,
        }
    }

    /// Is this score above the quality floor?
    pub fn above_floor(&self, thresholds: &ForgeThresholds) -> bool {
        self.total >= thresholds.quality_floor
    }

    /// Should we promote to the next tier?
    pub fn should_promote(&self, thresholds: &ForgeThresholds) -> bool {
        self.total >= thresholds.tier_promotion_eagerness
    }

    /// Is the score at a boundary? (requires extra caution)
    pub fn at_boundary(&self, thresholds: &ForgeThresholds) -> bool {
        (self.total - thresholds.quality_floor).abs() < 0.1
    }
}

/// A single round of the Ferro Forge game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRound {
    pub round_number: u32,
    pub action: String,
    pub score_before: f64,
    pub score_after: f64,
    pub delta: f64,
    pub decision: RoundDecision,
}

/// Decision made at the end of a round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoundDecision {
    Continue,
    Promote,
    Ship,
    Abandon,
}

impl std::fmt::Display for RoundDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Continue => write!(f, "CONTINUE"),
            Self::Promote => write!(f, "PROMOTE"),
            Self::Ship => write!(f, "SHIP"),
            Self::Abandon => write!(f, "ABANDON"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_thresholds() {
        let t = ForgeThresholds::default();
        assert!((t.quality_floor - 0.313).abs() < 1e-10);
        assert!((t.abandon_threshold - 0.027).abs() < 1e-10);
        assert!(t.fix_easiest_first);
    }

    #[test]
    fn test_thresholds_from_strategy() {
        let s = ForgeStrategy::default();
        let t = ForgeThresholds::from(&s);
        assert!((t.quality_floor - s.quality_floor).abs() < f64::EPSILON);
        assert!((t.abandon_threshold - s.abandon_threshold).abs() < f64::EPSILON);
        assert!((t.boundary_caution - s.boundary_caution).abs() < f64::EPSILON);
    }

    #[test]
    fn test_perfect_score() {
        let score = FerroScore::compute(16, 10, 10, 40, 40, 100, 100);
        assert!((score.total - 1.0).abs() < 1e-10);
        assert_eq!(score.grade, 'S');
    }

    #[test]
    fn test_zero_score() {
        let score = FerroScore::compute(0, 0, 10, 200, 40, 0, 100);
        assert!(score.total < 0.313);
        assert_eq!(score.grade, 'F');
    }

    #[test]
    fn test_quality_floor() {
        let thresholds = ForgeThresholds::default();
        let above = FerroScore::compute(8, 5, 10, 50, 40, 70, 100);
        assert!(above.above_floor(&thresholds));

        let below = FerroScore::compute(1, 0, 10, 200, 40, 10, 100);
        assert!(!below.above_floor(&thresholds));
    }

    #[test]
    fn test_weighted_sum() {
        let score = FerroScore::compute(8, 5, 10, 80, 40, 50, 100);
        let expected = 0.40 * 0.5 + 0.25 * 0.5 + 0.20 * 0.5 + 0.15 * 0.5;
        assert!((score.total - expected).abs() < 1e-10);
    }
}
