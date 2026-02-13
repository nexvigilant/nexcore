//! Achievement System - Track player accomplishments
//!
//! Tier: T2-C (composed game mechanics)

use serde::{Deserialize, Serialize};

/// Achievement definition
/// Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub points: u32,
    pub requirement: AchievementReq,
}

/// Achievement requirement types
/// Tier: T2-P (enum over conditions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementReq {
    TotalScore(u64),
    ComboReached(u32),
    DepthReached(f64),
    OresDropped(u32),
    SignalsFound(u32),
    StrongSignals(u32),
    RareOres(u32),
    ConsecutiveMines(u32),
}

/// Achievement progress tracker
/// Tier: T2-C
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AchievementTracker {
    pub unlocked: Vec<String>,
    pub total_score_ever: u64,
    pub max_combo: u32,
    pub max_depth: f64,
    pub total_dropped: u32,
    pub signals_found: u32,
    pub strong_signals: u32,
    pub rare_ores_found: u32,
    pub current_streak: u32,
    pub max_streak: u32,
}

impl AchievementTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_unlocks(&mut self) -> Vec<&'static Achievement> {
        let mut newly_unlocked = Vec::new();

        for achievement in ALL_ACHIEVEMENTS.iter() {
            if self.unlocked.iter().any(|id| id == achievement.id) {
                continue;
            }

            if self.meets_requirement(&achievement.requirement) {
                self.unlocked.push(achievement.id.to_string());
                newly_unlocked.push(achievement);
            }
        }

        newly_unlocked
    }

    fn meets_requirement(&self, req: &AchievementReq) -> bool {
        match req {
            AchievementReq::TotalScore(n) => self.total_score_ever >= *n,
            AchievementReq::ComboReached(n) => self.max_combo >= *n,
            AchievementReq::DepthReached(d) => self.max_depth >= *d,
            AchievementReq::OresDropped(n) => self.total_dropped >= *n,
            AchievementReq::SignalsFound(n) => self.signals_found >= *n,
            AchievementReq::StrongSignals(n) => self.strong_signals >= *n,
            AchievementReq::RareOres(n) => self.rare_ores_found >= *n,
            AchievementReq::ConsecutiveMines(n) => self.max_streak >= *n,
        }
    }

    pub fn record_score(&mut self, score: u64) {
        self.total_score_ever = self.total_score_ever.saturating_add(score);
    }

    pub fn record_combo(&mut self, combo: u32) {
        self.max_combo = self.max_combo.max(combo);
    }

    pub fn record_depth(&mut self, depth: f64) {
        self.max_depth = self.max_depth.max(depth);
    }

    pub fn record_drop(&mut self) {
        self.total_dropped += 1;
    }

    pub fn record_signal(&mut self, is_strong: bool) {
        self.signals_found += 1;
        if is_strong {
            self.strong_signals += 1;
        }
    }

    pub fn record_rare_ore(&mut self) {
        self.rare_ores_found += 1;
    }

    pub fn record_mine(&mut self) {
        self.current_streak += 1;
        self.max_streak = self.max_streak.max(self.current_streak);
    }

    pub fn break_streak(&mut self) {
        self.current_streak = 0;
    }

    pub fn unlocked_count(&self) -> usize {
        self.unlocked.len()
    }

    pub fn total_points(&self) -> u32 {
        ALL_ACHIEVEMENTS
            .iter()
            .filter(|a| self.unlocked.iter().any(|id| id == a.id))
            .map(|a| a.points)
            .sum()
    }
}

/// All available achievements
pub static ALL_ACHIEVEMENTS: &[Achievement] = &[
    // Score achievements
    Achievement {
        id: "first_hundred",
        name: "Century",
        description: "Score 100 points",
        icon: "💯",
        points: 10,
        requirement: AchievementReq::TotalScore(100),
    },
    Achievement {
        id: "thousand_club",
        name: "Thousand Club",
        description: "Score 1,000 points",
        icon: "🏆",
        points: 25,
        requirement: AchievementReq::TotalScore(1000),
    },
    Achievement {
        id: "high_roller",
        name: "High Roller",
        description: "Score 10,000 points",
        icon: "💎",
        points: 100,
        requirement: AchievementReq::TotalScore(10000),
    },
    // Combo achievements
    Achievement {
        id: "combo_starter",
        name: "Combo Starter",
        description: "Reach x1.5 combo",
        icon: "🔥",
        points: 10,
        requirement: AchievementReq::ComboReached(5),
    },
    Achievement {
        id: "combo_master",
        name: "Combo Master",
        description: "Reach x2.0 combo",
        icon: "⚡",
        points: 50,
        requirement: AchievementReq::ComboReached(10),
    },
    // Signal achievements
    Achievement {
        id: "signal_seeker",
        name: "Signal Seeker",
        description: "Find your first signal",
        icon: "🔬",
        points: 15,
        requirement: AchievementReq::SignalsFound(1),
    },
    Achievement {
        id: "vigilant_eye",
        name: "Vigilant Eye",
        description: "Find 5 signals",
        icon: "👁️",
        points: 30,
        requirement: AchievementReq::SignalsFound(5),
    },
    Achievement {
        id: "signal_master",
        name: "Signal Master",
        description: "Find 3 strong signals",
        icon: "🔴",
        points: 75,
        requirement: AchievementReq::StrongSignals(3),
    },
    // Mining achievements
    Achievement {
        id: "deep_miner",
        name: "Deep Miner",
        description: "Reach 2.0m depth",
        icon: "⬇️",
        points: 20,
        requirement: AchievementReq::DepthReached(2.0),
    },
    Achievement {
        id: "lucky_strike",
        name: "Lucky Strike",
        description: "Find a rare ore (Gold or Platinum)",
        icon: "🍀",
        points: 15,
        requirement: AchievementReq::RareOres(1),
    },
    Achievement {
        id: "gold_rush",
        name: "Gold Rush",
        description: "Find 5 rare ores",
        icon: "🟡",
        points: 50,
        requirement: AchievementReq::RareOres(5),
    },
    // Drop achievements
    Achievement {
        id: "letting_go",
        name: "Letting Go",
        description: "Drop your first ore",
        icon: "🗑️",
        points: 5,
        requirement: AchievementReq::OresDropped(1),
    },
    Achievement {
        id: "minimalist",
        name: "Minimalist",
        description: "Drop 10 ores",
        icon: "📦",
        points: 25,
        requirement: AchievementReq::OresDropped(10),
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracker_starts_empty() {
        let tracker = AchievementTracker::new();
        assert_eq!(tracker.unlocked_count(), 0);
        assert_eq!(tracker.total_points(), 0);
    }

    #[test]
    fn unlocks_first_hundred() {
        let mut tracker = AchievementTracker::new();
        tracker.record_score(100);
        let unlocked = tracker.check_unlocks();
        assert!(unlocked.iter().any(|a| a.id == "first_hundred"));
    }

    #[test]
    fn tracks_max_combo() {
        let mut tracker = AchievementTracker::new();
        tracker.record_combo(3);
        tracker.record_combo(7);
        tracker.record_combo(5);
        assert_eq!(tracker.max_combo, 7);
    }

    #[test]
    fn signal_tracking_works() {
        let mut tracker = AchievementTracker::new();
        tracker.record_signal(false);
        tracker.record_signal(true);
        tracker.record_signal(true);
        assert_eq!(tracker.signals_found, 3);
        assert_eq!(tracker.strong_signals, 2);
    }

    #[test]
    fn all_achievements_have_unique_ids() {
        let ids: Vec<_> = ALL_ACHIEVEMENTS.iter().map(|a| a.id).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len());
    }
}
