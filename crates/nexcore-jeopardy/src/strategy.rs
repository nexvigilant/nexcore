//! Strategy algorithms: Holzhauer board traversal, wagering, buzz decisions.
//!
//! This is the κ (compare) + ∂ (branch) composite: evaluating risk/reward
//! and making optimal decisions.

use crate::board::Board;
use crate::error::{JeopardyError, Result};
use crate::state::GameState;
use crate::types::{Category, CluePosition, Confidence, Round};
use serde::{Deserialize, Serialize};

/// The result of a buzz decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuzzDecision {
    /// Buzz in and attempt the answer.
    Buzz,
    /// Do not buzz; confidence is below threshold.
    Pass,
}

/// Investment decision for a wager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wager {
    /// Amount to wager (in score units).
    pub amount: u64,
    /// Confidence level backing this wager.
    pub confidence: Confidence,
    /// Category the wager targets.
    pub category: Category,
}

/// Scoring metadata for a clue position during board selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionScore {
    /// The board position.
    pub position: CluePosition,
    /// Composite score (higher = select first).
    pub score: f64,
    /// Reason components for transparency.
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Holzhauer Strategy: Optimal Board Traversal
// ---------------------------------------------------------------------------

/// Compute the optimal selection order for remaining clues (Holzhauer strategy).
///
/// Holzhauer's approach:
/// 1. Go bottom-up (highest value first) to maximize score per clue.
/// 2. Hunt for Daily Doubles before opponents find them.
/// 3. Prefer T2-P categories (compound 29% faster).
///
/// Returns positions sorted from best to worst selection.
pub fn optimal_selection_order(board: &Board) -> Vec<CluePosition> {
    let available = board.available_positions();
    if available.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<(CluePosition, f64)> = available
        .into_iter()
        .filter_map(|pos| {
            let cell = board.get(pos)?;
            let clue = cell.clue()?;

            // Base score: higher value = higher priority (Holzhauer bottom-up)
            let value_score = clue.value.0 as f64;

            // Daily Double hunting bonus: DDs are worth more because of wager potential
            let dd_bonus = if clue.is_daily_double { 5000.0 } else { 0.0 };

            // Category compound multiplier: T2-P categories compound faster
            let compound_bonus = clue.value.0 as f64 * (clue.category.compound_multiplier() - 1.0);

            let total = value_score + dd_bonus + compound_bonus;
            Some((pos, total))
        })
        .collect();

    // Sort descending by score
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    scored.into_iter().map(|(pos, _)| pos).collect()
}

/// Score each position for selection priority.
pub fn score_selections(board: &Board) -> Vec<SelectionScore> {
    let available = board.available_positions();

    available
        .into_iter()
        .filter_map(|pos| {
            let cell = board.get(pos)?;
            let clue = cell.clue()?;

            let value_score = clue.value.0 as f64;
            let dd_bonus = if clue.is_daily_double { 5000.0 } else { 0.0 };
            let compound_bonus = clue.value.0 as f64 * (clue.category.compound_multiplier() - 1.0);
            let total = value_score + dd_bonus + compound_bonus;

            let reason = format!(
                "value={}, dd_bonus={}, compound_bonus={:.0}, category={}",
                clue.value.0,
                dd_bonus,
                compound_bonus,
                clue.category.name()
            );

            Some(SelectionScore {
                position: pos,
                score: total,
                reason,
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Wagering Optimizer
// ---------------------------------------------------------------------------

/// Compute the optimal wager for a Daily Double.
///
/// Strategy depends on game position:
/// - **Leading**: Wager enough to maintain a "crush" lead (2x 2nd place + 1)
///   or wager `confidence * max_wager` if that's smaller.
/// - **Trailing**: Wager aggressively to close the gap.
/// - **Close game**: Wager proportional to confidence.
pub fn optimal_daily_double_wager(state: &GameState, confidence: Confidence) -> Result<Wager> {
    let score = state.active_score();
    let max_wager = state.max_daily_double_wager();
    let conf = confidence.value();

    let active_category = state
        .board
        .remaining_daily_doubles()
        .first()
        .and_then(|pos| state.board.get(*pos))
        .and_then(|cell| cell.clue())
        .map(|c| c.category)
        .unwrap_or(Category::SignalDetection);

    let amount = if state.is_leading() {
        // Leading: wager the minimum needed to maintain a lock
        // "Lock" = score after correct > 2 * 2nd place
        let second = state.second_place_score();
        let lock_target = (second * 2 + 1) - score;
        if lock_target <= 0 {
            // Already have a lock; wager based on confidence
            (conf * max_wager as f64) as u64
        } else {
            // Need to reach lock; wager at least lock_target if confident
            let confidence_wager = (conf * max_wager as f64) as u64;
            if conf >= 0.7 {
                confidence_wager.max(lock_target as u64)
            } else {
                confidence_wager.min(lock_target.unsigned_abs())
            }
        }
    } else {
        // Trailing: aggressive wager to close gap
        let gap = state.gap_to_leader().unsigned_abs();
        let minimum_useful = gap + 1;
        let confidence_wager = (conf * max_wager as f64) as u64;

        if conf >= 0.5 {
            // Confident enough to go big
            confidence_wager.max(minimum_useful).min(max_wager)
        } else {
            // Low confidence: still wager something to stay competitive
            (confidence_wager).min(max_wager)
        }
    };

    let clamped = amount.min(max_wager);

    Ok(Wager {
        amount: clamped,
        confidence,
        category: active_category,
    })
}

/// Compute the optimal Final Jeopardy wager.
///
/// Classic defensive strategy:
/// - **If leading**: wager = (2nd_place * 2) - score + 1
///   This guarantees a win even if 2nd place bets everything and is correct.
/// - **If trailing**: wager everything (all-in on proven algorithm).
pub fn optimal_final_wager(state: &GameState, confidence: Confidence) -> Result<Wager> {
    let score = state.active_score();
    if score <= 0 {
        return Ok(Wager {
            amount: 0,
            confidence,
            category: Category::SignalDetection,
        });
    }

    let score_u = score as u64;
    let second = state.second_place_score();
    let conf = confidence.value();

    let amount = if state.is_leading() {
        // Defensive: wager just enough to stay ahead if 2nd gets theirs right
        let defensive = (second * 2) - score + 1;
        if defensive <= 0 {
            // We have a runaway — wager 0 or a small amount
            if conf >= 0.9 {
                // Very confident: might as well pad the lead
                score_u / 4
            } else {
                0
            }
        } else {
            // Need to wager at least this much to cover 2nd place going all-in
            let d = defensive as u64;
            d.min(score_u)
        }
    } else {
        // Trailing: must bet big
        let gap = state.gap_to_leader().unsigned_abs();
        let minimum_needed = gap + 1;
        minimum_needed.min(score_u)
    };

    Ok(Wager {
        amount,
        confidence,
        category: Category::SignalDetection,
    })
}

// ---------------------------------------------------------------------------
// Buzz Decision (Signal Threshold Gate)
// ---------------------------------------------------------------------------

/// Decide whether to buzz in on a clue.
///
/// The decision maps to signal detection: don't raise a signal unless
/// confidence exceeds a dynamic threshold based on:
/// - Clue value (higher value = higher threshold, more to lose)
/// - Current score (negative score = more conservative)
/// - Difficulty (harder clues need higher confidence)
/// - Lockout penalty awareness (wrong answer = lose control + lose value)
///
/// The threshold formula:
/// `threshold = base + difficulty_penalty + value_penalty - urgency_bonus`
pub fn should_buzz(
    clue: &crate::types::Clue,
    confidence: Confidence,
    state: &GameState,
) -> BuzzDecision {
    let conf = confidence.value();

    // Base threshold: 0.5 (better than coin flip)
    let base = 0.50;

    // Difficulty penalty: harder clues need more confidence
    let difficulty_penalty = clue.difficulty() * 0.15;

    // Value penalty: higher-value clues are riskier (more to lose)
    let max_value = match state.round() {
        Round::Jeopardy => 1000.0,
        Round::DoubleJeopardy => 2000.0,
        Round::FinalJeopardy => 2000.0,
    };
    let value_ratio = clue.value.0 as f64 / max_value;
    let value_penalty = value_ratio * 0.10;

    // Urgency bonus: trailing players should buzz more aggressively
    let urgency = if state.gap_to_leader() < 0 {
        // Trailing: lower the threshold proportionally
        let gap_ratio = (state.gap_to_leader().unsigned_abs() as f64) / max_value;
        (gap_ratio * 0.15).min(0.15)
    } else {
        0.0
    };

    // Score penalty: negative score = be more conservative
    let score_penalty = if state.active_score() < 0 { 0.05 } else { 0.0 };

    let threshold = base + difficulty_penalty + value_penalty + score_penalty - urgency;
    // Clamp threshold to [0.3, 0.95] — never impossible, never automatic
    let threshold = threshold.clamp(0.3, 0.95);

    if conf >= threshold {
        BuzzDecision::Buzz
    } else {
        BuzzDecision::Pass
    }
}

// ---------------------------------------------------------------------------
// Board Control Value
// ---------------------------------------------------------------------------

/// Estimate the value of having board control.
///
/// Board control means you choose the next clue. The value equals the
/// sum of expected gains from making optimal selections, weighted by
/// the player's accuracy and the category compound multipliers.
pub fn board_control_value(board: &Board, state: &GameState) -> f64 {
    let optimal_order = optimal_selection_order(board);
    if optimal_order.is_empty() {
        return 0.0;
    }

    let accuracy = state
        .active_player()
        .map(|p| p.accuracy())
        .filter(|&a| a > 0.0)
        .unwrap_or(0.5);

    // Sum expected value of each remaining clue in optimal order,
    // with a decay factor (you won't necessarily keep control)
    let mut total = 0.0;
    let control_retention = accuracy; // Probability of keeping control each turn

    let mut retention_prob = 1.0;
    for pos in &optimal_order {
        if let Some(cell) = board.get(*pos) {
            if let Some(clue) = cell.clue() {
                let expected = clue.value.0 as f64 * accuracy * clue.category.compound_multiplier();
                total += expected * retention_prob;
                retention_prob *= control_retention;
            }
        }
    }

    total
}

// ---------------------------------------------------------------------------
// Round Transition
// ---------------------------------------------------------------------------

/// Determine if the current state allows advancement to the next round.
///
/// Rules:
/// - Board must be fully answered (or time expired, but we model completion).
/// - Jeopardy -> Double Jeopardy: requires non-negative score.
/// - Double Jeopardy -> Final Jeopardy: requires positive score.
pub fn can_advance(state: &GameState) -> bool {
    if !state.board.is_empty() {
        return false;
    }

    match state.round() {
        Round::Jeopardy => {
            // Must have non-negative score to advance
            state.active_score() >= 0
        }
        Round::DoubleJeopardy => {
            // Must have positive score for Final Jeopardy
            state.active_score() > 0
        }
        Round::FinalJeopardy => {
            // Game over after Final Jeopardy
            false
        }
    }
}
