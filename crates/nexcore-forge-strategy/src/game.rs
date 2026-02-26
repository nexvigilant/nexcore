//! Game loop module — orchestrates rounds via Arrhenius gating and Hill ship decisions.
//!
//! Merged from ferro-forge-engine. T1 Primitives: σ(Sequence) + ρ(Recursion) + ∃(Existence)
//!     + Σ(Sum) + ×(Product) + μ(Mapping) + ∅(Void) + π(Persistence)

use crate::kinetics::{ArrheniusGate, HillCascade};
use crate::nash::{NashSolver, PayoffMatrix};
use crate::scoring::{FerroScore, ForgeThresholds, GameRound, RoundDecision};
use serde::{Deserialize, Serialize};

/// Game state — tracks the full Ferro Forge session.
///
/// T1: ς(State) + σ(Sequence) + π(Persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Thresholds evolved from 12k simulated games
    pub thresholds: ForgeThresholds,
    /// History of rounds played — σ(Sequence) + π(Persistence)
    pub rounds: Vec<GameRound>,
    /// Accumulated quality signals for Hill cascade
    pub quality_signals: Vec<f64>,
    /// Current game phase — Σ(Sum type)
    pub phase: GamePhase,
    /// Architecture decision from Nash equilibrium — ∅(Void) until decided
    pub architecture: Option<ArchitectureChoice>,
    /// Primitives collected this game
    pub primitives_collected: u8,
    /// Current HP (errors reduce HP)
    pub hp: i32,
    /// Max HP
    pub max_hp: i32,
}

/// Game phases — Σ(Sum type, exclusive disjunction)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Architect,
    Implement,
    QualityGate,
    ShipDecision,
    Complete,
}

/// Architecture choice — ×(Product) of strategy and expected payoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureChoice {
    pub strategy: String,
    pub expected_payoff: f64,
    pub nash_p: f64,
}

/// Outcome of a game tick — μ(Mapping) from state to decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TickOutcome {
    Decided(ArchitectureChoice),
    Scored(FerroScore),
    GateResult {
        rate: f64,
        proceed: bool,
    },
    ShipResult {
        response: f64,
        ship: bool,
    },
    Finished {
        final_score: f64,
        grade: char,
        rounds_played: u32,
    },
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            thresholds: ForgeThresholds::default(),
            rounds: Vec::new(),
            quality_signals: Vec::new(),
            phase: GamePhase::Architect,
            architecture: None,
            primitives_collected: 0,
            hp: 100,
            max_hp: 100,
        }
    }
}

impl GameState {
    /// Create a new game with default thresholds.
    pub fn new() -> Self {
        Self::default()
    }

    /// Current round number — N(Quantity)
    pub fn round_number(&self) -> u32 {
        self.rounds.len() as u32 + 1
    }

    /// Run the architect phase — Nash equilibrium decision.
    pub fn architect(&mut self, matrix: &PayoffMatrix) -> TickOutcome {
        let nash = NashSolver::solve(matrix);
        let preferred = nash.preferred_row();
        let choice = ArchitectureChoice {
            strategy: matrix.row_labels[preferred].clone(),
            expected_payoff: nash.expected_payoff,
            nash_p: nash.row_strategy[preferred],
        };
        self.architecture = Some(choice.clone());
        self.phase = GamePhase::Implement;
        TickOutcome::Decided(choice)
    }

    /// Record implementation progress.
    pub fn record_implementation(&mut self, primitives_found: u8, errors: u32) {
        self.primitives_collected = self.primitives_collected.saturating_add(primitives_found);
        self.hp = (self.hp - (errors as i32 * 5)).max(0);
        self.phase = GamePhase::QualityGate;
    }

    /// Run quality gate — Arrhenius decision gate.
    pub fn quality_gate(
        &mut self,
        tests_passed: u32,
        tests_total: u32,
        actual_turns: u32,
    ) -> (FerroScore, ArrheniusGate) {
        let score = FerroScore::compute(
            self.primitives_collected,
            tests_passed,
            tests_total,
            actual_turns,
            self.round_number() * 40,
            self.hp,
            self.max_hp,
        );

        let barrier = 8.314 * (1.0 - score.total).max(0.01);
        let gate = ArrheniusGate::compute(1.0, barrier, 350.0 + score.total * 50.0);

        let score_before = self.rounds.last().map_or(0.0, |r| r.score_after);
        let decision = if !gate.should_proceed(self.thresholds.abandon_threshold) {
            RoundDecision::Abandon
        } else if score.should_promote(&self.thresholds) {
            RoundDecision::Promote
        } else {
            RoundDecision::Continue
        };

        self.rounds.push(GameRound {
            round_number: self.round_number(),
            action: "quality_gate".to_string(),
            score_before,
            score_after: score.total,
            delta: score.total - score_before,
            decision: decision.clone(),
        });

        self.quality_signals.push(score.total);
        self.phase = GamePhase::ShipDecision;

        (score, gate)
    }

    /// Ship decision via Hill cascade — cooperative signal amplification.
    pub fn ship_decision(&mut self, n_hill: f64, k_half: f64) -> (HillCascade, bool) {
        let cascade = HillCascade::compute(n_hill, k_half, &self.quality_signals);
        let ship = cascade.should_ship(self.thresholds.boundary_caution);

        if ship {
            if let Some(last) = self.rounds.last_mut() {
                last.decision = RoundDecision::Ship;
            }
            self.phase = GamePhase::Complete;
        } else {
            self.phase = GamePhase::Implement;
        }

        (cascade, ship)
    }

    /// Is the game finished?
    pub fn is_complete(&self) -> bool {
        self.phase == GamePhase::Complete
    }

    /// Final summary — ×(Product) of all metrics
    pub fn summary(&self) -> TickOutcome {
        let final_score = self.rounds.last().map_or(0.0, |r| r.score_after);
        let grade = match final_score {
            t if t >= 0.90 => 'S',
            t if t >= 0.80 => 'A',
            t if t >= 0.65 => 'B',
            t if t >= 0.50 => 'C',
            t if t >= 0.313 => 'D',
            _ => 'F',
        };
        TickOutcome::Finished {
            final_score,
            grade,
            rounds_played: self.rounds.len() as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = GameState::new();
        assert_eq!(game.phase, GamePhase::Architect);
        assert_eq!(game.hp, 100);
        assert_eq!(game.primitives_collected, 0);
        assert!(game.rounds.is_empty());
    }

    #[test]
    fn test_architect_phase() {
        let mut game = GameState::new();
        let matrix = PayoffMatrix::new(
            ["Monolith", "Modular"],
            ["Speed", "Quality"],
            [[7.0, 3.0], [4.0, 8.0]],
        );
        let outcome = game.architect(&matrix);
        assert_eq!(game.phase, GamePhase::Implement);
        assert!(game.architecture.is_some());
        assert!(matches!(outcome, TickOutcome::Decided(_)));
    }

    #[test]
    fn test_implementation_records() {
        let mut game = GameState::new();
        game.phase = GamePhase::Implement;
        game.record_implementation(8, 2);
        assert_eq!(game.primitives_collected, 8);
        assert_eq!(game.hp, 90);
        assert_eq!(game.phase, GamePhase::QualityGate);
    }

    #[test]
    fn test_quality_gate_proceed() {
        let mut game = GameState::new();
        game.phase = GamePhase::QualityGate;
        game.primitives_collected = 12;
        let (score, gate) = game.quality_gate(17, 17, 8);
        assert!(score.total > game.thresholds.quality_floor);
        assert!(gate.should_proceed(game.thresholds.abandon_threshold));
        assert_eq!(game.phase, GamePhase::ShipDecision);
    }

    #[test]
    fn test_ship_decision_below_threshold() {
        let mut game = GameState::new();
        game.phase = GamePhase::ShipDecision;
        game.quality_signals.push(0.5);
        let (cascade, ship) = game.ship_decision(2.5, 1.0);
        assert!(!ship);
        assert_eq!(game.phase, GamePhase::Implement);
        assert!(cascade.response < game.thresholds.boundary_caution);
    }

    #[test]
    fn test_ship_decision_above_threshold() {
        let mut game = GameState::new();
        game.phase = GamePhase::ShipDecision;
        game.quality_signals = vec![0.85, 0.90, 0.88];
        game.rounds.push(GameRound {
            round_number: 1,
            action: "quality_gate".to_string(),
            score_before: 0.0,
            score_after: 0.88,
            delta: 0.88,
            decision: RoundDecision::Continue,
        });
        let (cascade, ship) = game.ship_decision(2.5, 1.0);
        assert!(ship);
        assert_eq!(game.phase, GamePhase::Complete);
        assert!(cascade.response >= game.thresholds.boundary_caution);
    }

    #[test]
    fn test_full_game_loop() {
        let mut game = GameState::new();

        let matrix = PayoffMatrix::new(
            ["Monolith", "Modular"],
            ["Speed", "Quality"],
            [[7.0, 3.0], [4.0, 8.0]],
        );
        game.architect(&matrix);
        assert_eq!(game.phase, GamePhase::Implement);

        game.record_implementation(12, 0);
        assert_eq!(game.phase, GamePhase::QualityGate);

        let (score, _gate) = game.quality_gate(17, 17, 6);
        assert!(score.total > 0.5);

        let (_cascade, ship) = game.ship_decision(2.5, 1.0);
        if !ship {
            game.record_implementation(4, 0);
            game.quality_gate(20, 20, 5);
            // After two rounds of high-quality results, total_signal is large enough
            // that the Hill cascade must exceed boundary_caution (0.718).
            let (_cascade2, ship2) = game.ship_decision(2.5, 1.0);
            assert!(
                ship2,
                "Two high-quality rounds should trigger ship decision"
            );
        }
    }
}
