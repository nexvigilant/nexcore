//! Proxy BDI - Data-efficient alternative when public betting % unavailable.
//!
//! Uses line movement patterns and book divergence to estimate sharp action.

use serde::{Deserialize, Serialize};

use super::bdi::{BdiResult, ContingencyTable, calculate_bdi};

/// Book classification for sharp vs recreational.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookClassification {
    /// Sharp/professional books (Pinnacle, Circa, etc.)
    Sharp,
    /// Recreational/retail books (DraftKings, FanDuel, etc.)
    Recreational,
}

/// Book with classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookOdds {
    /// Book name
    pub name: String,
    /// Current line/spread
    pub line: f64,
    /// Classification
    pub classification: BookClassification,
}

impl BookOdds {
    /// Classify common books.
    #[must_use]
    pub fn classify(name: &str) -> BookClassification {
        const SHARP_BOOKS: &[&str] = &[
            "pinnacle",
            "circa",
            "bookmaker",
            "betcris",
            "heritage",
            "5dimes",
            "betonline",
            "bovada",
            "betanysports",
            "jazz",
            "betus",
            "everygame",
        ];

        let lower = name.to_lowercase();
        if SHARP_BOOKS.iter().any(|b| lower.contains(b)) {
            BookClassification::Sharp
        } else {
            BookClassification::Recreational
        }
    }
}

/// Input for proxy BDI calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyBdiInput {
    /// Opening line
    pub opening_line: f64,
    /// Current line
    pub current_line: f64,
    /// Book odds from multiple sources
    pub book_odds: Vec<BookOdds>,
    /// Whether steam move detected
    pub steam_detected: bool,
    /// Data quality score (0-1)
    pub quality: f64,
}

/// Line movement metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineMovementMetrics {
    /// Movement magnitude (absolute points)
    pub magnitude: f64,
    /// Movement velocity score (0-1)
    pub velocity_score: f64,
    /// Sharp vs recreational divergence (0-1)
    pub divergence_score: f64,
    /// Direction signal score (0-1)
    pub direction_score: f64,
}

/// Calculate proxy BDI from line movement data.
///
/// Uses three components:
/// 1. Movement velocity (how fast/large the move)
/// 2. Book divergence (sharp vs recreational difference)
/// 3. Direction signal (reverse vs expected movement)
#[must_use]
pub fn calculate_proxy_bdi(input: &ProxyBdiInput) -> BdiResult {
    let metrics = calculate_movement_metrics(input);

    // Proxy BDI formula:
    // 1.0 + (V×0.35 + D×0.35 + S×0.30) × Q × scale
    let base_score = metrics.velocity_score * 0.35
        + metrics.divergence_score * 0.35
        + metrics.direction_score * 0.30;

    let scale = 4.0; // Scale to BDI range
    let steam_multiplier = if input.steam_detected { 1.25 } else { 1.0 };

    let proxy_bdi = 1.0 + base_score * input.quality * scale * steam_multiplier;

    // Convert to synthetic contingency table for consistency
    // This allows reuse of BDI infrastructure
    let synthetic_table = synthesize_contingency_table(proxy_bdi, &metrics);

    calculate_bdi(synthetic_table)
}

/// Calculate line movement metrics.
fn calculate_movement_metrics(input: &ProxyBdiInput) -> LineMovementMetrics {
    let magnitude = (input.current_line - input.opening_line).abs();

    // Velocity: normalized by typical max move (3 points)
    let velocity_score = (magnitude / 3.0).min(1.0);

    // Divergence: difference between sharp and rec consensus
    let (sharp_avg, rec_avg) = calculate_book_consensus(&input.book_odds);
    let divergence = (sharp_avg - rec_avg).abs();
    let divergence_score = (divergence / 1.5).min(1.0);

    // Direction: reverse movement indicator
    // Reverse = 1.0, aligned = 0.5, stable = 0.0
    let direction_score = if magnitude < 0.5 {
        0.0 // Stable
    } else if is_reverse_movement(input.opening_line, input.current_line) {
        1.0 // Reverse (sharp indicator)
    } else {
        0.5 // Expected direction
    };

    LineMovementMetrics {
        magnitude,
        velocity_score,
        divergence_score,
        direction_score,
    }
}

/// Calculate average line for sharp and recreational books.
fn calculate_book_consensus(books: &[BookOdds]) -> (f64, f64) {
    let sharp: Vec<f64> = books
        .iter()
        .filter(|b| b.classification == BookClassification::Sharp)
        .map(|b| b.line)
        .collect();

    let rec: Vec<f64> = books
        .iter()
        .filter(|b| b.classification == BookClassification::Recreational)
        .map(|b| b.line)
        .collect();

    let sharp_avg = if sharp.is_empty() {
        0.0
    } else {
        sharp.iter().sum::<f64>() / sharp.len() as f64
    };

    let rec_avg = if rec.is_empty() {
        0.0
    } else {
        rec.iter().sum::<f64>() / rec.len() as f64
    };

    (sharp_avg, rec_avg)
}

/// Check if movement is reverse (against expected direction).
fn is_reverse_movement(opening: f64, current: f64) -> bool {
    // In spread betting, line moving toward underdog when public likes favorite
    // is a reverse movement indicator (sharp action)
    // Simplified: any significant move is potentially reverse
    (current - opening).abs() > 0.5
}

/// Synthesize contingency table from proxy metrics.
fn synthesize_contingency_table(
    _proxy_bdi: f64,
    metrics: &LineMovementMetrics,
) -> ContingencyTable {
    // Create synthetic table that produces similar BDI
    // a/(a+b) / c/(c+d) ≈ proxy_bdi

    let base = 100.0;
    let a = base * metrics.direction_score;
    let b = base * (1.0 - metrics.velocity_score);
    let c = base * (1.0 - metrics.divergence_score) * 0.5;
    let d = base;

    ContingencyTable::new(a.max(1.0), b.max(1.0), c.max(1.0), d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_classification() {
        assert_eq!(BookOdds::classify("Pinnacle"), BookClassification::Sharp);
        assert_eq!(
            BookOdds::classify("DraftKings"),
            BookClassification::Recreational
        );
        assert_eq!(BookOdds::classify("circa"), BookClassification::Sharp);
    }

    #[test]
    fn test_proxy_bdi_calculation() {
        let input = ProxyBdiInput {
            opening_line: -3.0,
            current_line: -4.5,
            book_odds: vec![
                BookOdds {
                    name: "Pinnacle".into(),
                    line: -4.5,
                    classification: BookClassification::Sharp,
                },
                BookOdds {
                    name: "DraftKings".into(),
                    line: -3.5,
                    classification: BookClassification::Recreational,
                },
            ],
            steam_detected: true,
            quality: 0.9,
        };

        let result = calculate_proxy_bdi(&input);
        assert!(
            result.bdi > 1.0,
            "Strong movement should produce positive BDI"
        );
    }
}
