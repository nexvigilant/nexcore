//! Challenge Mode - Timed signal detection challenges
//!
//! Tier: T2-C (composed game mode)

use serde::{Deserialize, Serialize};

/// Challenge difficulty levels
/// Tier: T2-P
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl Difficulty {
    pub fn time_limit_secs(&self) -> u32 {
        match self {
            Self::Easy => 120,
            Self::Medium => 90,
            Self::Hard => 60,
            Self::Expert => 30,
        }
    }

    pub fn target_signals(&self) -> u32 {
        match self {
            Self::Easy => 3,
            Self::Medium => 5,
            Self::Hard => 7,
            Self::Expert => 10,
        }
    }

    pub fn bonus_multiplier(&self) -> f64 {
        match self {
            Self::Easy => 1.0,
            Self::Medium => 1.5,
            Self::Hard => 2.0,
            Self::Expert => 3.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Easy => "Rookie",
            Self::Medium => "Analyst",
            Self::Hard => "Expert",
            Self::Expert => "Legend",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Easy => "🟢",
            Self::Medium => "🟡",
            Self::Hard => "🟠",
            Self::Expert => "🔴",
        }
    }
}

/// Challenge state
/// Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub difficulty: Difficulty,
    pub time_remaining: f64,
    pub signals_found: u32,
    pub strong_signals: u32,
    pub score: u64,
    pub completed: bool,
    pub success: bool,
}

impl Challenge {
    pub fn new(difficulty: Difficulty) -> Self {
        Self {
            difficulty,
            time_remaining: difficulty.time_limit_secs() as f64,
            signals_found: 0,
            strong_signals: 0,
            score: 0,
            completed: false,
            success: false,
        }
    }

    pub fn tick(&mut self, dt: f64) {
        if self.completed {
            return;
        }

        self.time_remaining = (self.time_remaining - dt).max(0.0);

        if self.time_remaining <= 0.0 {
            self.complete();
        }
    }

    pub fn record_signal(&mut self, points: u64, is_strong: bool) {
        self.signals_found += 1;
        if is_strong {
            self.strong_signals += 1;
        }
        self.score += points;

        if self.signals_found >= self.difficulty.target_signals() {
            self.complete();
        }
    }

    fn complete(&mut self) {
        self.completed = true;
        self.success = self.signals_found >= self.difficulty.target_signals();
    }

    pub fn final_score(&self) -> u64 {
        if !self.success {
            return self.score / 2; // Penalty for failure
        }

        let base = self.score;
        let time_bonus = (self.time_remaining * 10.0) as u64;
        let strong_bonus = self.strong_signals as u64 * 50;
        let difficulty_mult = self.difficulty.bonus_multiplier();

        ((base + time_bonus + strong_bonus) as f64 * difficulty_mult) as u64
    }

    pub fn progress_percent(&self) -> f64 {
        let target = self.difficulty.target_signals() as f64;
        (self.signals_found as f64 / target * 100.0).min(100.0)
    }

    pub fn time_percent(&self) -> f64 {
        let total = self.difficulty.time_limit_secs() as f64;
        (self.time_remaining / total * 100.0).max(0.0)
    }
}

/// Pre-defined challenge drug-event pairs
pub static CHALLENGE_PAIRS: &[(&str, &str, bool)] = &[
    // (drug, event, is_strong_signal)
    ("Warfarin", "Hemorrhage", true),
    ("Aspirin", "GI Bleeding", false),
    ("Metformin", "Lactic Acidosis", true),
    ("Lisinopril", "Cough", false),
    ("Atorvastatin", "Rhabdomyolysis", true),
    ("Amiodarone", "Thyroid Disorder", true),
    ("Carbamazepine", "Stevens-Johnson", true),
    ("Clozapine", "Agranulocytosis", true),
    ("Isotretinoin", "Depression", false),
    ("Thalidomide", "Peripheral Neuropathy", false),
];

pub fn random_challenge_pair() -> (&'static str, &'static str, bool) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let idx = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize
        % CHALLENGE_PAIRS.len();
    CHALLENGE_PAIRS[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulty_ordering() {
        assert!(Difficulty::Easy.time_limit_secs() > Difficulty::Expert.time_limit_secs());
        assert!(Difficulty::Easy.target_signals() < Difficulty::Expert.target_signals());
    }

    #[test]
    fn challenge_completes_on_target() {
        let mut challenge = Challenge::new(Difficulty::Easy);
        for _ in 0..3 {
            challenge.record_signal(100, false);
        }
        assert!(challenge.completed);
        assert!(challenge.success);
    }

    #[test]
    fn challenge_fails_on_timeout() {
        let mut challenge = Challenge::new(Difficulty::Easy);
        challenge.tick(200.0); // Way past time limit
        assert!(challenge.completed);
        assert!(!challenge.success);
    }

    #[test]
    fn final_score_includes_bonuses() {
        let mut challenge = Challenge::new(Difficulty::Medium);
        challenge.signals_found = 5;
        challenge.strong_signals = 2;
        challenge.score = 500;
        challenge.time_remaining = 30.0;
        challenge.completed = true;
        challenge.success = true;

        let final_score = challenge.final_score();
        assert!(final_score > 500); // Should have bonuses
    }

    #[test]
    fn challenge_pairs_not_empty() {
        assert!(!CHALLENGE_PAIRS.is_empty());
        assert!(CHALLENGE_PAIRS.len() >= 5);
    }
}
