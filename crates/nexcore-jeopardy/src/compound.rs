//! Compound growth extension: cross-game state and primitive accumulation.
//!
//! Each solved algorithm yields primitives that reduce the cost of future
//! algorithms. This models the "season strategy" — the gap in vanilla
//! Jeopardy optimization.
//!
//! Formula: V(t) = B(t) * eta(t) * r(t)
//! - B(t): basis (accumulated primitive count)
//! - eta(t): efficiency (accuracy over time)
//! - r(t): reuse factor (how often prior work applies)

use crate::state::AttemptRecord;
use crate::types::{Category, Confidence};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Result of a single game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResult {
    /// Final score achieved.
    pub final_score: i64,
    /// Categories with correct answers (primitives extracted).
    pub categories_correct: Vec<Category>,
    /// Overall accuracy this game.
    pub accuracy: f64,
    /// Number of correct answers.
    pub correct_count: u32,
    /// Attempt records for detailed analysis.
    pub attempts: Vec<AttemptRecord>,
}

/// Cross-game compound growth metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundMetrics {
    /// Total games played.
    pub games_played: u32,
    /// Accumulated primitive count B(t).
    pub basis: f64,
    /// Current efficiency eta(t) — clamped to [0.0, 1.0].
    pub efficiency: Confidence,
    /// Current reuse factor r(t) — clamped to [0.0, 1.0].
    pub reuse: Confidence,
    /// Current velocity V(t) = B * eta * r.
    pub velocity: f64,
    /// Per-category primitive counts (deterministic ordering).
    pub category_primitives: BTreeMap<Category, u32>,
    /// Triangular transfer cost: C(n) = 1.0 - T(n-1) * delta
    pub transfer_cost: f64,
}

/// Decay quantum for the triangular transfer law.
const DELTA: f64 = 0.1;

/// Compute the n-th triangular number: T(n) = n * (n + 1) / 2.
fn triangular(n: u32) -> u32 {
    n * (n + 1) / 2
}

/// Compute the transfer cost for the n-th domain transfer.
///
/// C(n) = 1.0 - T(n-1) * delta, clamped to [0.0, 1.0].
/// Produces: {1.0, 0.9, 0.7, 0.4, 0.0, ...}
fn transfer_cost(n: u32) -> f64 {
    if n == 0 {
        return 1.0;
    }
    let t = triangular(n - 1);
    (1.0 - t as f64 * DELTA).max(0.0)
}

/// Compute compound velocity from game history.
///
/// - B(t): Number of unique (category, correct) primitives accumulated.
/// - eta(t): Running accuracy over all games.
/// - r(t): Fraction of categories where prior correct answers exist.
pub fn compound_velocity(history: &[GameResult]) -> CompoundMetrics {
    if history.is_empty() {
        let empty_cats: BTreeMap<Category, u32> = Category::all().iter().map(|c| (*c, 0)).collect();
        return CompoundMetrics {
            games_played: 0,
            basis: 0.0,
            efficiency: Confidence::new(0.0),
            reuse: Confidence::new(0.0),
            velocity: 0.0,
            category_primitives: empty_cats,
            transfer_cost: 1.0,
        };
    }

    let games_played = history.len() as u32;

    // Count primitives per category
    let mut cat_counts: BTreeMap<Category, u32> =
        Category::all().iter().map(|c| (*c, 0u32)).collect();

    let mut total_correct = 0u32;
    let mut total_attempts = 0u32;

    for game in history {
        total_correct += game.correct_count;
        total_attempts += game.attempts.len() as u32;

        for cat in &game.categories_correct {
            if let Some(count) = cat_counts.get_mut(cat) {
                *count += 1;
            }
        }
    }

    // B(t): total unique primitives (with diminishing returns per category)
    let basis: f64 = cat_counts
        .iter()
        .map(|(cat, count)| {
            let raw = *count as f64;
            // Apply compound multiplier (T2-P categories grow faster)
            let multiplied = raw * cat.compound_multiplier();
            // Diminishing returns: sqrt to prevent unbounded growth
            multiplied.sqrt()
        })
        .sum();

    // eta(t): accuracy
    let efficiency_raw = if total_attempts > 0 {
        f64::from(total_correct) / f64::from(total_attempts)
    } else {
        0.0
    };
    let efficiency = Confidence::new(efficiency_raw);

    // r(t): reuse — fraction of categories with at least 1 primitive
    let categories_with_prims = cat_counts.values().filter(|c| **c > 0).count();
    let reuse_raw = categories_with_prims as f64 / cat_counts.len() as f64;
    let reuse = Confidence::new(reuse_raw);

    // V(t) = B * eta * r
    let velocity = basis * efficiency.value() * reuse.value();

    // Transfer cost for the number of domains covered
    let tc = transfer_cost(categories_with_prims as u32);

    CompoundMetrics {
        games_played,
        basis,
        efficiency,
        reuse,
        velocity,
        category_primitives: cat_counts,
        transfer_cost: tc,
    }
}
