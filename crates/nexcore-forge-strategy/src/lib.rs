// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Forge Strategy — Evolved Decision Parameters for Code Generation
//!
//! Strategy parameters discovered through evolutionary training (genetic algorithm,
//! 12,000 simulated games, 40 generations). Each parameter was randomly initialized,
//! then refined through tournament selection, crossover, and mutation.
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol |
//! |---------|-----------|--------|
//! | Decision engine | Causality | → |
//! | Condition evaluation | Comparison | κ |
//! | Mutable context | State | ς |
//!
//! ## Origin
//!
//! Parameters evolved via [Primitive Depths: Code Forge](forge.html), a roguelike
//! where an AI collects 16 Lex Primitiva symbols across 5 dungeon floors. The game's
//! decision topology (Flee → Fight → Heal → Hunt → Collect → Descend → Explore)
//! maps structurally onto the Forge protocol (MINE → DECOMPOSE → GENERATE →
//! VALIDATE → REFINE). Evolved values transferred because both share the same
//! Causality → Comparison → State primitive composition.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;
use nexcore_lex_primitiva::tier::Tier;
use std::fmt;

// ============================================================================
// ForgeDecision — What the forge should do next
// ============================================================================

/// Decision priority for the forge's code generation loop.
///
/// Ordered by urgency: abandon is highest priority, explore is lowest.
/// Maps 1:1 from the game AI's decision chain.
///
/// | ForgeDecision | Game Decision | When |
/// |---------------|---------------|------|
/// | Abandon | Flee | Confidence below threshold |
/// | FixBlocker | Fight | Compile error adjacent |
/// | Refactor | Heal | Quality below floor |
/// | LintFix | Hunt | Warning within radius |
/// | Decompose | CollectPrimitive | Primitives available |
/// | Promote | Descend | Ready for next tier |
/// | Explore | Explore | Try alternative paths |
/// | Stuck | Stuck | No progress possible |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ForgeDecision {
    /// Abandon current generation attempt (confidence too low).
    Abandon,
    /// Fix a blocking compiler error immediately.
    FixBlocker,
    /// Refactor: improve code quality before proceeding.
    Refactor,
    /// Fix a nearby clippy warning or lint issue.
    LintFix,
    /// Mine and decompose primitives from the task domain.
    Decompose,
    /// Promote type to the next tier (T1 → T2-P → T2-C → T3).
    Promote,
    /// Explore alternative decompositions or approaches.
    Explore,
    /// No progress possible; needs external intervention.
    Stuck,
}

impl fmt::Display for ForgeDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Abandon => write!(f, "ABANDON"),
            Self::FixBlocker => write!(f, "FIX_BLOCKER"),
            Self::Refactor => write!(f, "REFACTOR"),
            Self::LintFix => write!(f, "LINT_FIX"),
            Self::Decompose => write!(f, "DECOMPOSE"),
            Self::Promote => write!(f, "PROMOTE"),
            Self::Explore => write!(f, "EXPLORE"),
            Self::Stuck => write!(f, "STUCK"),
        }
    }
}

// ============================================================================
// ForgeStrategy — Evolved parameters for forge decision-making
// ============================================================================

/// Strategy parameters discovered by evolutionary training.
///
/// Tier: T2-P (→ Causality + κ Comparison + ς State)
/// Dominant: → Causality (decision engine)
///
/// Each field has a game-domain origin and a forge-domain interpretation.
/// The evolved values transfer because both domains share the same primitive
/// composition: sequential decisions (→) evaluated against thresholds (κ)
/// in mutable context (ς).
#[derive(Debug, Clone)]
pub struct ForgeStrategy {
    /// Minimum code quality score before triggering refactoring.
    ///
    /// Game origin: `heal_threshold` (HP ratio to start healing).
    /// Evolved: 0.313 — surprisingly low. Don't over-polish mid-generation;
    /// accept ~31% quality floor and keep making progress.
    pub quality_floor: f64,

    /// Minimum "distance" to blocking errors before refactoring is safe.
    ///
    /// Game origin: `heal_safe_radius` (Manhattan distance to nearest enemy).
    /// Evolved: 5 — only refactor when no compile errors are within 5
    /// dependency hops.
    pub safe_refactor_distance: u32,

    /// Fix easiest compiler errors first?
    ///
    /// Game origin: `fight_weakest` (target weakest adjacent enemy).
    /// Evolved: true — clear cheap blockers to unblock progress faster.
    pub fix_easiest_first: bool,

    /// Abandon generation when confidence drops below this threshold.
    ///
    /// Game origin: `fight_flee_hp` (flee when HP ratio drops below).
    /// Evolved: 0.027 — almost never abandon. Persistence beats perfection.
    pub abandon_threshold: f64,

    /// How deep to search for primitive decompositions.
    ///
    /// Game origin: `prim_hunger_radius` (BFS search radius for primitives).
    /// Evolved: 13 — search wide. Cast a broad net for applicable primitives.
    pub decomposition_depth: u32,

    /// Fix warnings within this many tiers of current work.
    ///
    /// Game origin: `aggro_radius` (hunt enemies within Manhattan distance).
    /// Evolved: 2 — only fix warnings in immediately adjacent code.
    pub lint_radius: u32,

    /// Lint strictness: <0.5 = pragmatic, >0.5 = pedantic.
    ///
    /// Game origin: `aggro_priority` (how aggressively to hunt enemies).
    /// Evolved: 0.391 — pragmatic won. Don't proactively hunt warnings;
    /// fix them when they're adjacent to what you're already touching.
    pub lint_strictness: f64,

    /// How eagerly to promote types to the next tier.
    ///
    /// Game origin: `stair_eagerness` (how eagerly to descend stairs).
    /// Evolved: 0.890 — promote quickly. Once primitives are mined on a
    /// tier, advance rather than exhaustively exploring the current level.
    pub tier_promotion_eagerness: f64,

    /// Explore alternative decompositions even when current approach works?
    ///
    /// Game origin: `explore_when_safe` (explore unseen map areas).
    /// Evolved: true — yes, speculative exploration finds better solutions.
    pub speculative_generation: bool,

    /// When refactoring, how clean before resuming generation?
    ///
    /// Game origin: `wait_heal_ratio` (heal up to this HP ratio).
    /// Evolved: 0.836 — when you do refactor, be thorough (84% quality)
    /// before resuming. Half-hearted refactoring wastes effort.
    pub refactor_completeness: f64,

    /// Caution at module boundaries (0.0 = fast, 1.0 = careful).
    ///
    /// Game origin: `corridor_caution` (caution near narrow corridors).
    /// Evolved: 0.718 — be careful at API boundaries. Module interfaces
    /// are bottlenecks; errors there cascade further.
    pub boundary_caution: f64,
}

impl Default for ForgeStrategy {
    /// Evolved defaults from primitive-forge training.
    ///
    /// Generation 40, fitness 2595.0, 16/16 primitives, 6/6 victories.
    /// Trained: 2026-02-13, 12,000 games (50 pop × 40 gen × 6 trials).
    fn default() -> Self {
        Self {
            quality_floor: 0.313,
            safe_refactor_distance: 5,
            fix_easiest_first: true,
            abandon_threshold: 0.027,
            decomposition_depth: 13,
            lint_radius: 2,
            lint_strictness: 0.391,
            tier_promotion_eagerness: 0.890,
            speculative_generation: true,
            refactor_completeness: 0.836,
            boundary_caution: 0.718,
        }
    }
}

impl fmt::Display for ForgeStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ForgeStrategy(quality>{:.0}% abandon<{:.0}% lint={} promote@{:.0}%)",
            self.quality_floor * 100.0,
            self.abandon_threshold * 100.0,
            if self.lint_strictness > 0.5 {
                "pedantic"
            } else {
                "pragmatic"
            },
            self.tier_promotion_eagerness * 100.0,
        )
    }
}

// ============================================================================
// Decision Engine
// ============================================================================

/// Snapshot of the current forge state, used as input to `decide()`.
#[derive(Debug, Clone)]
pub struct ForgeState {
    /// Current code quality ratio (0.0 = broken, 1.0 = perfect).
    pub quality_ratio: f64,
    /// Number of blocking compiler errors.
    pub blocker_count: u32,
    /// Distance (in dependency hops) to nearest blocker.
    pub nearest_blocker_dist: u32,
    /// Number of clippy warnings in scope.
    pub warning_count: u32,
    /// Number of unmined primitives available for decomposition.
    pub primitives_available: u32,
    /// Whether the current tier's work is complete.
    pub tier_complete: bool,
    /// Overall generation confidence (0.0-1.0).
    pub confidence: f64,
}

impl ForgeStrategy {
    /// Determine the next forge action based on evolved parameters.
    ///
    /// Decision priority chain (highest to lowest):
    /// 1. Abandon — confidence below threshold
    /// 2. FixBlocker — compiler errors present
    /// 3. Refactor — quality below floor AND safe to do so
    /// 4. LintFix — warnings nearby AND strictness demands it
    /// 5. Decompose — primitives available to mine
    /// 6. Promote — current tier complete, advance eagerly
    /// 7. Explore — speculative alternative search
    /// 8. Stuck — no actionable state
    #[must_use]
    pub fn decide(&self, state: &ForgeState) -> ForgeDecision {
        // 1. Abandon when confidence is critically low
        if state.blocker_count > 0 && state.confidence < self.abandon_threshold {
            return ForgeDecision::Abandon;
        }

        // 2. Fix blockers immediately
        if state.blocker_count > 0 {
            return ForgeDecision::FixBlocker;
        }

        // 3. Refactor when quality is below floor and no nearby blockers
        if state.quality_ratio < self.quality_floor
            && state.nearest_blocker_dist >= self.safe_refactor_distance
            && state.quality_ratio < self.refactor_completeness
        {
            return ForgeDecision::Refactor;
        }

        // 4. Fix lints when strictness demands it and warnings are nearby
        if self.lint_strictness > 0.5 && state.warning_count > 0 {
            return ForgeDecision::LintFix;
        }

        // 5. Decompose when primitives are available
        if state.primitives_available > 0 {
            return ForgeDecision::Decompose;
        }

        // 6. Promote to next tier if current work is complete
        if state.tier_complete {
            return ForgeDecision::Promote;
        }

        // 7. Explore alternatives if strategy allows
        if self.speculative_generation {
            return ForgeDecision::Explore;
        }

        ForgeDecision::Stuck
    }

    /// Check if the strategy considers the quality level acceptable.
    #[must_use]
    pub fn quality_acceptable(&self, quality_ratio: f64) -> bool {
        quality_ratio >= self.quality_floor
    }

    /// Check if refactoring has reached the completeness target.
    #[must_use]
    pub fn refactor_complete(&self, quality_ratio: f64) -> bool {
        quality_ratio >= self.refactor_completeness
    }

    /// Determine whether to fix lints at the current state.
    #[must_use]
    pub fn should_lint(&self, warning_count: u32, distance_to_work: u32) -> bool {
        warning_count > 0
            && distance_to_work <= self.lint_radius
            && self.lint_strictness > 0.5
    }

    /// Get the transfer confidence for a tier, modulated by boundary caution.
    ///
    /// At module boundaries (high caution), confidence is reduced proportionally.
    /// This reflects the evolved finding that corridor (boundary) caution of 0.718
    /// significantly impacts survival.
    #[must_use]
    pub fn tier_confidence(&self, tier: &Tier, at_boundary: bool) -> f64 {
        let base = tier.transfer_multiplier();
        if at_boundary {
            base * (1.0 - self.boundary_caution * 0.3)
        } else {
            base
        }
    }
}

// ============================================================================
// Fitness Function
// ============================================================================

/// Metrics from a forge generation attempt, used for evolutionary fitness.
#[derive(Debug, Clone)]
pub struct ForgeMetrics {
    /// Number of primitives successfully mined and decomposed.
    pub primitives_mined: u32,
    /// Number of tiers completed (0-4).
    pub tiers_completed: u32,
    /// Whether the generated code compiles.
    pub compiles: bool,
    /// Code quality ratio after generation (0.0-1.0).
    pub quality_ratio: f64,
    /// Turn efficiency: primitives per generation step.
    pub turn_efficiency: f64,
    /// Ratio of warnings fixed vs introduced.
    pub lint_efficiency: f64,
    /// Whether the full forge pipeline succeeded.
    pub success: bool,
}

/// Compute fitness for a forge generation attempt.
///
/// Weights evolved from the game's fitness function:
/// - Primitives × 100 (most important: collect all the things)
/// - Tiers × 30 (steady advancement)
/// - Compiles: +200 (survival)
/// - Quality × 50 (HP ratio)
/// - Turn efficiency × 80 (don't waste steps)
/// - Lint efficiency × 60 (combat efficiency)
/// - Success: +500 (victory bonus)
#[must_use]
pub fn compute_fitness(metrics: &ForgeMetrics) -> f64 {
    metrics.primitives_mined as f64 * 100.0
        + metrics.tiers_completed as f64 * 30.0
        + if metrics.compiles { 200.0 } else { 0.0 }
        + metrics.quality_ratio * 50.0
        + metrics.turn_efficiency * 80.0
        + metrics.lint_efficiency * 60.0
        + if metrics.success { 500.0 } else { 0.0 }
}

// ============================================================================
// GroundsTo — Primitive composition for ForgeStrategy
// ============================================================================

impl GroundsTo for ForgeStrategy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Comparison,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for ForgeDecision {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Causality])
            .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

impl GroundsTo for ForgeState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

impl GroundsTo for ForgeMetrics {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Default strategy validation ---

    #[test]
    fn default_strategy_parameters_in_range() {
        let s = ForgeStrategy::default();
        assert!(s.quality_floor > 0.0 && s.quality_floor < 1.0);
        assert!(s.abandon_threshold >= 0.0 && s.abandon_threshold < 1.0);
        assert!(s.lint_strictness >= 0.0 && s.lint_strictness <= 1.0);
        assert!(s.tier_promotion_eagerness > 0.0 && s.tier_promotion_eagerness <= 1.0);
        assert!(s.refactor_completeness > 0.0 && s.refactor_completeness <= 1.0);
        assert!(s.boundary_caution >= 0.0 && s.boundary_caution <= 1.0);
    }

    #[test]
    fn default_strategy_evolved_values() {
        let s = ForgeStrategy::default();
        assert!((s.quality_floor - 0.313).abs() < 0.001);
        assert_eq!(s.safe_refactor_distance, 5);
        assert!(s.fix_easiest_first);
        assert!((s.abandon_threshold - 0.027).abs() < 0.001);
        assert_eq!(s.decomposition_depth, 13);
        assert_eq!(s.lint_radius, 2);
        assert!((s.lint_strictness - 0.391).abs() < 0.001);
        assert!((s.tier_promotion_eagerness - 0.890).abs() < 0.001);
        assert!(s.speculative_generation);
        assert!((s.refactor_completeness - 0.836).abs() < 0.001);
        assert!((s.boundary_caution - 0.718).abs() < 0.001);
    }

    // --- Decision engine ---

    #[test]
    fn decide_fix_blocker_when_errors_present() {
        let s = ForgeStrategy::default();
        let state = ForgeState {
            quality_ratio: 0.5,
            blocker_count: 3,
            nearest_blocker_dist: 1,
            warning_count: 0,
            primitives_available: 5,
            tier_complete: false,
            confidence: 0.8,
        };
        assert_eq!(s.decide(&state), ForgeDecision::FixBlocker);
    }

    #[test]
    fn decide_abandon_when_confidence_critical() {
        let s = ForgeStrategy::default();
        let state = ForgeState {
            quality_ratio: 0.1,
            blocker_count: 5,
            nearest_blocker_dist: 1,
            warning_count: 10,
            primitives_available: 0,
            tier_complete: false,
            confidence: 0.01, // below abandon_threshold of 0.027
        };
        assert_eq!(s.decide(&state), ForgeDecision::Abandon);
    }

    #[test]
    fn decide_decompose_when_primitives_available() {
        let s = ForgeStrategy::default();
        let state = ForgeState {
            quality_ratio: 0.9,
            blocker_count: 0,
            nearest_blocker_dist: 20,
            warning_count: 0,
            primitives_available: 4,
            tier_complete: false,
            confidence: 0.9,
        };
        assert_eq!(s.decide(&state), ForgeDecision::Decompose);
    }

    #[test]
    fn decide_promote_when_tier_complete() {
        let s = ForgeStrategy::default();
        let state = ForgeState {
            quality_ratio: 0.9,
            blocker_count: 0,
            nearest_blocker_dist: 20,
            warning_count: 0,
            primitives_available: 0,
            tier_complete: true,
            confidence: 0.9,
        };
        assert_eq!(s.decide(&state), ForgeDecision::Promote);
    }

    #[test]
    fn decide_explore_as_fallback() {
        let s = ForgeStrategy::default();
        let state = ForgeState {
            quality_ratio: 0.9,
            blocker_count: 0,
            nearest_blocker_dist: 20,
            warning_count: 0,
            primitives_available: 0,
            tier_complete: false,
            confidence: 0.9,
        };
        // With speculative_generation=true (evolved default), falls to Explore
        assert_eq!(s.decide(&state), ForgeDecision::Explore);
    }

    #[test]
    fn decide_refactor_when_quality_low_and_safe() {
        let s = ForgeStrategy::default();
        let state = ForgeState {
            quality_ratio: 0.2, // below quality_floor of 0.313
            blocker_count: 0,
            nearest_blocker_dist: 10, // above safe_refactor_distance of 5
            warning_count: 0,
            primitives_available: 0,
            tier_complete: false,
            confidence: 0.9,
        };
        assert_eq!(s.decide(&state), ForgeDecision::Refactor);
    }

    #[test]
    fn pragmatic_lint_skips_warnings() {
        // Default lint_strictness is 0.391 (< 0.5), so warnings don't trigger LintFix
        let s = ForgeStrategy::default();
        let state = ForgeState {
            quality_ratio: 0.9,
            blocker_count: 0,
            nearest_blocker_dist: 20,
            warning_count: 15,
            primitives_available: 3,
            tier_complete: false,
            confidence: 0.9,
        };
        // Should decompose, not lint
        assert_eq!(s.decide(&state), ForgeDecision::Decompose);
    }

    #[test]
    fn pedantic_lint_fixes_warnings() {
        let s = ForgeStrategy {
            lint_strictness: 0.8, // pedantic
            ..ForgeStrategy::default()
        };
        let state = ForgeState {
            quality_ratio: 0.9,
            blocker_count: 0,
            nearest_blocker_dist: 20,
            warning_count: 5,
            primitives_available: 3,
            tier_complete: false,
            confidence: 0.9,
        };
        assert_eq!(s.decide(&state), ForgeDecision::LintFix);
    }

    // --- Utility methods ---

    #[test]
    fn quality_acceptable_boundary() {
        let s = ForgeStrategy::default();
        assert!(!s.quality_acceptable(0.3));
        assert!(s.quality_acceptable(0.5));
    }

    #[test]
    fn refactor_complete_boundary() {
        let s = ForgeStrategy::default();
        assert!(!s.refactor_complete(0.8));
        assert!(s.refactor_complete(0.9));
    }

    #[test]
    fn tier_confidence_at_boundary() {
        let s = ForgeStrategy::default();
        let base = s.tier_confidence(&Tier::T2Primitive, false);
        let boundary = s.tier_confidence(&Tier::T2Primitive, true);
        assert!(boundary < base);
        // boundary_caution=0.718, so penalty = 0.718 * 0.3 ≈ 0.215
        // T2-P base = 0.9, boundary ≈ 0.9 * (1 - 0.215) ≈ 0.707
        assert!((boundary - 0.707).abs() < 0.01);
    }

    // --- Fitness ---

    #[test]
    fn fitness_perfect_run() {
        let metrics = ForgeMetrics {
            primitives_mined: 16,
            tiers_completed: 4,
            compiles: true,
            quality_ratio: 1.0,
            turn_efficiency: 1.0,
            lint_efficiency: 1.0,
            success: true,
        };
        let f = compute_fitness(&metrics);
        // 16*100 + 4*30 + 200 + 1.0*50 + 1.0*80 + 1.0*60 + 500 = 2610
        assert!((f - 2610.0).abs() < 0.001);
    }

    #[test]
    fn fitness_zero_for_empty() {
        let metrics = ForgeMetrics {
            primitives_mined: 0,
            tiers_completed: 0,
            compiles: false,
            quality_ratio: 0.0,
            turn_efficiency: 0.0,
            lint_efficiency: 0.0,
            success: false,
        };
        assert!((compute_fitness(&metrics) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fitness_compile_bonus() {
        let base = ForgeMetrics {
            primitives_mined: 0,
            tiers_completed: 0,
            compiles: false,
            quality_ratio: 0.0,
            turn_efficiency: 0.0,
            lint_efficiency: 0.0,
            success: false,
        };
        let with_compile = ForgeMetrics {
            compiles: true,
            ..base.clone()
        };
        assert!((compute_fitness(&with_compile) - compute_fitness(&base) - 200.0).abs() < 0.001);
    }

    // --- GroundsTo ---

    #[test]
    fn forge_strategy_grounds_to_causality() {
        let comp = ForgeStrategy::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert_eq!(comp.primitives.len(), 3);
        assert!(!comp.is_pure());
    }

    #[test]
    fn forge_strategy_is_t2p() {
        assert_eq!(ForgeStrategy::tier(), Tier::T2Primitive);
    }

    #[test]
    fn forge_decision_grounds_to_sum() {
        let comp = ForgeDecision::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn forge_state_is_mutable() {
        assert_eq!(ForgeState::state_mode(), Some(StateMode::Accumulated));
    }

    // --- Display ---

    #[test]
    fn strategy_display() {
        let s = ForgeStrategy::default();
        let d = format!("{s}");
        assert!(d.contains("ForgeStrategy"));
        assert!(d.contains("pragmatic"));
    }

    #[test]
    fn decision_display() {
        assert_eq!(format!("{}", ForgeDecision::FixBlocker), "FIX_BLOCKER");
        assert_eq!(format!("{}", ForgeDecision::Decompose), "DECOMPOSE");
    }
}
