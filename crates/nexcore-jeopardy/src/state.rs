//! Game state tracking.
//!
//! Encapsulates ρ (state) — the evolving context of a game, including
//! scores, current round, board, and history.

use crate::board::Board;
use crate::error::{JeopardyError, Result};
use crate::types::{Category, CluePosition, ClueValue, Confidence, Round};
use serde::{Deserialize, Serialize};

/// A player/contestant in the game (algorithm development team).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Player identifier.
    name: String,
    /// Current score (can go negative from incorrect answers).
    score: i64,
    /// Number of correct answers.
    correct: u32,
    /// Number of incorrect answers.
    incorrect: u32,
    /// Whether this player currently has board control.
    has_control: bool,
}

impl Player {
    /// Create a new player with zero score.
    pub fn new(name: impl Into<String>) -> Self {
        Player {
            name: name.into(),
            score: 0,
            correct: 0,
            incorrect: 0,
            has_control: false,
        }
    }

    /// Player name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Current score (can go negative).
    pub fn score(&self) -> i64 {
        self.score
    }

    /// Number of correct answers.
    pub fn correct_count(&self) -> u32 {
        self.correct
    }

    /// Number of incorrect answers.
    pub fn incorrect_count(&self) -> u32 {
        self.incorrect
    }

    /// Whether this player currently has board control.
    pub fn has_control(&self) -> bool {
        self.has_control
    }

    /// Accuracy as a fraction in [0.0, 1.0]. Returns 0.0 if no attempts.
    pub fn accuracy(&self) -> f64 {
        let total = self.correct + self.incorrect;
        if total == 0 {
            return 0.0;
        }
        f64::from(self.correct) / f64::from(total)
    }

    /// Set score directly (used by MCP tools for state restoration).
    pub fn set_score(&mut self, score: i64) {
        self.score = score;
    }

    /// Set correct count directly (used by MCP tools for state restoration).
    pub fn set_correct(&mut self, count: u32) {
        self.correct = count;
    }

    /// Set incorrect count directly (used by MCP tools for state restoration).
    pub fn set_incorrect(&mut self, count: u32) {
        self.incorrect = count;
    }

    /// Set board control directly (used by MCP tools for state restoration).
    pub fn set_has_control(&mut self, control: bool) {
        self.has_control = control;
    }
}

/// Record of a single clue attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptRecord {
    /// Board position.
    pub position: CluePosition,
    /// Category.
    pub category: Category,
    /// Value of the clue.
    pub value: ClueValue,
    /// Whether the answer was correct.
    pub correct: bool,
    /// Confidence at time of buzzing.
    pub confidence: Confidence,
    /// Was this a Daily Double?
    pub was_daily_double: bool,
    /// If Daily Double, the wager amount.
    pub wager: Option<u64>,
}

/// Full game state — the composite ρ primitive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Current round (derived from board — use `.round()` accessor).
    round: Round,
    /// The game board.
    pub board: Board,
    /// All players (typically 3 in Jeopardy).
    pub players: Vec<Player>,
    /// Index of the active player (has board control).
    active_player: usize,
    /// History of all attempts in this game.
    pub history: Vec<AttemptRecord>,
    /// Number of clues answered in current round.
    clues_answered_this_round: u32,
}

impl GameState {
    /// Create a new game state for the first round.
    pub fn new(player_names: &[&str], board: Board) -> Self {
        let mut players: Vec<Player> = player_names.iter().map(|n| Player::new(*n)).collect();
        if let Some(first) = players.first_mut() {
            first.has_control = true;
        }

        GameState {
            round: board.round(),
            board,
            players,
            active_player: 0,
            history: Vec::new(),
            clues_answered_this_round: 0,
        }
    }

    /// The current round (always consistent with the board).
    pub fn round(&self) -> Round {
        self.round
    }

    /// The currently active player.
    pub fn active_player(&self) -> Option<&Player> {
        self.players.get(self.active_player)
    }

    /// Mutable reference to the active player.
    pub fn active_player_mut(&mut self) -> Option<&mut Player> {
        self.players.get_mut(self.active_player)
    }

    /// Get the score of the active player.
    pub fn active_score(&self) -> i64 {
        self.active_player().map_or(0, |p| p.score)
    }

    /// Get the highest score among all players.
    pub fn leader_score(&self) -> i64 {
        self.players.iter().map(|p| p.score).max().unwrap_or(0)
    }

    /// Get the second-highest score.
    pub fn second_place_score(&self) -> i64 {
        let mut scores: Vec<i64> = self.players.iter().map(|p| p.score).collect();
        scores.sort_unstable_by(|a, b| b.cmp(a));
        scores.get(1).copied().unwrap_or(0)
    }

    /// Whether the active player is in the lead.
    pub fn is_leading(&self) -> bool {
        let active = self.active_score();
        self.players
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != self.active_player)
            .all(|(_, p)| p.score <= active)
    }

    /// Gap between active player and the leader (negative if trailing).
    pub fn gap_to_leader(&self) -> i64 {
        let active = self.active_score();
        let leader = self
            .players
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != self.active_player)
            .map(|(_, p)| p.score)
            .max()
            .unwrap_or(0);
        active - leader
    }

    /// Record a correct answer for the active player.
    pub fn record_correct(
        &mut self,
        value: u64,
        confidence: Confidence,
        clue: &crate::types::Clue,
        pos: CluePosition,
        wager: Option<u64>,
    ) {
        let points = wager.unwrap_or(value) as i64;
        if let Some(player) = self.players.get_mut(self.active_player) {
            player.score += points;
            player.correct += 1;
        }
        self.history.push(AttemptRecord {
            position: pos,
            category: clue.category,
            value: clue.value,
            correct: true,
            confidence,
            was_daily_double: clue.is_daily_double,
            wager,
        });
        self.clues_answered_this_round += 1;
    }

    /// Record an incorrect answer for the active player.
    pub fn record_incorrect(
        &mut self,
        value: u64,
        confidence: Confidence,
        clue: &crate::types::Clue,
        pos: CluePosition,
        wager: Option<u64>,
    ) {
        let penalty = wager.unwrap_or(value) as i64;
        if let Some(player) = self.players.get_mut(self.active_player) {
            player.score -= penalty;
            player.incorrect += 1;
            player.has_control = false;
        }
        self.history.push(AttemptRecord {
            position: pos,
            category: clue.category,
            value: clue.value,
            correct: false,
            confidence,
            was_daily_double: clue.is_daily_double,
            wager,
        });
        self.clues_answered_this_round += 1;
    }

    /// Transfer board control to a different player.
    pub fn transfer_control(&mut self, player_index: usize) {
        for (i, p) in self.players.iter_mut().enumerate() {
            p.has_control = i == player_index;
        }
        self.active_player = player_index;
    }

    /// Maximum wager allowed for a Daily Double.
    ///
    /// In Jeopardy round: max(score, highest_value_on_board).
    /// In Double Jeopardy: max(score, highest_value_on_board).
    pub fn max_daily_double_wager(&self) -> u64 {
        let score = self.active_score().max(0) as u64;
        let max_board = match self.round {
            Round::Jeopardy => 1000,
            Round::DoubleJeopardy | Round::FinalJeopardy => 2000,
        };
        score.max(max_board)
    }

    /// Advance to the next round with a new board.
    ///
    /// Validates:
    /// - Current board is fully answered.
    /// - Score requirements are met for the next round.
    /// - A next round exists (cannot advance past Final Jeopardy).
    ///
    /// Resets `clues_answered_this_round` and updates the round/board.
    pub fn advance_round(&mut self, new_board: Board) -> Result<Round> {
        if !self.board.is_empty() {
            return Err(JeopardyError::CannotAdvance {
                current_round: format!("{:?}", self.round),
                reason: "board still has unanswered clues".into(),
            });
        }

        let next = self
            .round
            .next()
            .ok_or_else(|| JeopardyError::CannotAdvance {
                current_round: format!("{:?}", self.round),
                reason: "no round after Final Jeopardy".into(),
            })?;

        // Score requirements
        match self.round {
            Round::Jeopardy => {
                if self.active_score() < 0 {
                    return Err(JeopardyError::CannotAdvance {
                        current_round: "Jeopardy".into(),
                        reason: "active player has negative score".into(),
                    });
                }
            }
            Round::DoubleJeopardy => {
                if self.active_score() <= 0 {
                    return Err(JeopardyError::CannotAdvance {
                        current_round: "DoubleJeopardy".into(),
                        reason: "active player must have positive score for Final Jeopardy".into(),
                    });
                }
            }
            Round::FinalJeopardy => {
                // Already handled by next() returning None
            }
        }

        self.round = next;
        self.board = new_board;
        self.clues_answered_this_round = 0;
        Ok(next)
    }

    /// Index of the currently active player.
    pub fn active_player_index(&self) -> usize {
        self.active_player
    }

    /// Number of clues answered in the current round.
    pub fn clues_answered_this_round(&self) -> u32 {
        self.clues_answered_this_round
    }

    /// Set active player index directly (test-only).
    #[cfg(test)]
    pub(crate) fn set_active_player_index(&mut self, index: usize) {
        self.active_player = index;
    }
}
