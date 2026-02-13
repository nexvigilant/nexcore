//! ADV² Stats Notifier - SessionStart Hook
//!
//! Displays advisor hit/miss statistics at session start.
//! Shows which skill recommendations are being followed.
//!
//! Event: SessionStart

use chrono::{DateTime, Utc};
use nexcore_hooks::{exit_success_auto_with, read_input};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Advisor hit/miss statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AdvisorStats {
    hits: HashMap<String, u32>,
    misses: HashMap<String, u32>,
    total_recommendations: u32,
    updated_at: DateTime<Utc>,
}

impl AdvisorStats {
    fn load() -> Option<Self> {
        let path = stats_path();
        let content = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Compute hit rate for a skill
    fn hit_rate(&self, skill: &str) -> Option<f64> {
        let h = *self.hits.get(skill).unwrap_or(&0);
        let m = *self.misses.get(skill).unwrap_or(&0);
        let total = h + m;
        if total == 0 {
            None
        } else {
            Some(f64::from(h) / f64::from(total))
        }
    }

    /// Get all skills with data, sorted by hit rate descending
    fn ranked_skills(&self) -> Vec<(String, f64, u32, u32)> {
        let mut all_skills: HashMap<&str, (u32, u32)> = HashMap::new();

        for (skill, &h) in &self.hits {
            all_skills.entry(skill.as_str()).or_insert((0, 0)).0 = h;
        }
        for (skill, &m) in &self.misses {
            all_skills.entry(skill.as_str()).or_insert((0, 0)).1 = m;
        }

        let mut ranked: Vec<_> = all_skills
            .into_iter()
            .filter(|(_, (h, m))| h + m > 0)
            .map(|(skill, (h, m))| {
                let rate = f64::from(h) / f64::from(h + m);
                (skill.to_string(), rate, h, m)
            })
            .collect();

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked
    }

    /// Overall hit rate
    fn overall_hit_rate(&self) -> Option<f64> {
        let total_hits: u32 = self.hits.values().sum();
        let total_misses: u32 = self.misses.values().sum();
        let total = total_hits + total_misses;
        if total == 0 {
            None
        } else {
            Some(f64::from(total_hits) / f64::from(total))
        }
    }
}

fn stats_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("brain")
        .join("skill_bonds")
        .join("advisor_stats.json")
}

fn format_bar(rate: f64, width: usize) -> String {
    let filled = (rate * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

fn main() {
    let _input = read_input();

    let stats = match AdvisorStats::load() {
        Some(s) => s,
        None => {
            exit_success_auto_with("ADV²: No stats yet (invoke /advisor to start tracking)");
        }
    };

    let ranked = stats.ranked_skills();
    if ranked.is_empty() {
        exit_success_auto_with("ADV²: No recommendation data yet");
    }

    let overall = stats.overall_hit_rate().unwrap_or(0.0);
    let total_obs: u32 = ranked.iter().map(|(_, _, h, m)| h + m).sum();

    let mut msg = format!(
        "📊 ADV² Advisor Stats | Overall: {:.0}% {} | {} obs\n",
        overall * 100.0,
        format_bar(overall, 10),
        total_obs
    );

    // Show top 5 skills
    for (skill, rate, hits, misses) in ranked.iter().take(5) {
        let bar = format_bar(*rate, 8);
        msg.push_str(&format!(
            "  {} {:.0}% {bar} ({hits}✓/{misses}✗)\n",
            skill,
            rate * 100.0
        ));
    }

    // Highlight low performers (<30%)
    let low_performers: Vec<_> = ranked.iter().filter(|(_, r, _, _)| *r < 0.3).collect();
    if !low_performers.is_empty() {
        msg.push_str("  ⚠️ Low hit rate: ");
        msg.push_str(
            &low_performers
                .iter()
                .map(|(s, _, _, _)| s.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        );
    }

    exit_success_auto_with(&msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_rate_calculation() {
        let mut stats = AdvisorStats::default();
        stats.hits.insert("forge".to_string(), 3);
        stats.misses.insert("forge".to_string(), 1);

        let rate = stats.hit_rate("forge").unwrap();
        assert!((rate - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_overall_hit_rate() {
        let mut stats = AdvisorStats::default();
        stats.hits.insert("a".to_string(), 2);
        stats.hits.insert("b".to_string(), 3);
        stats.misses.insert("a".to_string(), 2);
        stats.misses.insert("b".to_string(), 3);

        let rate = stats.overall_hit_rate().unwrap();
        assert!((rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_format_bar() {
        assert_eq!(format_bar(0.5, 10), "[█████░░░░░]");
        assert_eq!(format_bar(1.0, 10), "[██████████]");
        assert_eq!(format_bar(0.0, 10), "[░░░░░░░░░░]");
    }

    #[test]
    fn test_ranked_skills_sorted() {
        let mut stats = AdvisorStats::default();
        stats.hits.insert("high".to_string(), 9);
        stats.misses.insert("high".to_string(), 1);
        stats.hits.insert("low".to_string(), 1);
        stats.misses.insert("low".to_string(), 9);

        let ranked = stats.ranked_skills();
        assert_eq!(ranked[0].0, "high");
        assert_eq!(ranked[1].0, "low");
    }
}
