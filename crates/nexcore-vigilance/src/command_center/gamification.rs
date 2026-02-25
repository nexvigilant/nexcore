//! Gamification domain types: badges, achievements, and progression.

use nexcore_chrono::DateTime;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::enums::{AchievementRarity, BadgeType};

/// User badge earned through platform activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBadge {
    /// Unique identifier.
    pub id: NexId,

    /// Associated user profile ID.
    pub user_profile_id: NexId,

    /// Badge type.
    pub badge_type: BadgeType,

    /// Display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge_name: Option<String>,

    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge_description: Option<String>,

    /// Icon URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge_icon_url: Option<String>,

    /// When the badge was earned.
    pub earned_at: DateTime,

    /// What triggered earning this badge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earned_for: Option<String>,

    /// Related entity type (post, module, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_entity_type: Option<String>,

    /// Related entity ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_entity_id: Option<String>,

    /// Whether to show prominently on profile.
    #[serde(default)]
    pub is_featured: bool,

    /// Display order in badge list.
    #[serde(default)]
    pub display_order: i32,

    /// Badge tier (1=Bronze, 2=Silver, 3=Gold, etc.).
    #[serde(default = "default_tier")]
    pub tier: i32,

    /// Current progress toward next tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_current: Option<i32>,

    /// Target for next tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_target: Option<i32>,

    /// Badge-specific custom data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge_data: Option<Value>,

    /// Creation timestamp.
    pub created_at: DateTime,
}

fn default_tier() -> i32 {
    1
}

impl UserBadge {
    /// Create a new user badge.
    #[must_use]
    pub fn new(user_profile_id: NexId, badge_type: BadgeType) -> Self {
        let now = DateTime::now();
        Self {
            id: NexId::v4(),
            user_profile_id,
            badge_type,
            badge_name: None,
            badge_description: None,
            badge_icon_url: None,
            earned_at: now,
            earned_for: None,
            related_entity_type: None,
            related_entity_id: None,
            is_featured: false,
            display_order: 0,
            tier: 1,
            progress_current: None,
            progress_target: None,
            badge_data: None,
            created_at: now,
        }
    }

    /// Get tier name.
    #[must_use]
    pub fn tier_name(&self) -> &'static str {
        match self.tier {
            1 => "Bronze",
            2 => "Silver",
            3 => "Gold",
            4 => "Platinum",
            5 => "Diamond",
            _ => "Unknown",
        }
    }

    /// Calculate progress percentage to next tier.
    #[must_use]
    pub fn progress_percentage(&self) -> Option<f64> {
        match (self.progress_current, self.progress_target) {
            (Some(current), Some(target)) if target > 0 => {
                Some(f64::from(current) / f64::from(target) * 100.0)
            }
            _ => None,
        }
    }

    /// Check if badge can be upgraded to next tier.
    #[must_use]
    pub fn can_upgrade(&self) -> bool {
        match (self.progress_current, self.progress_target) {
            (Some(current), Some(target)) => current >= target,
            _ => false,
        }
    }
}

/// Achievement definition and criteria.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    /// Unique identifier.
    pub id: NexId,

    /// Unique achievement key.
    pub achievement_key: String,

    /// Display name.
    pub name: String,

    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Category (learning, community, career, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Associated badge type (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge_type: Option<BadgeType>,

    /// Structured criteria for earning.
    pub criteria: Value,

    /// Points awarded.
    #[serde(default)]
    pub points: i32,

    /// Whether this achievement has multiple tiers.
    #[serde(default)]
    pub has_tiers: bool,

    /// Criteria for each tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier_criteria: Option<Value>,

    /// Whether achievement is hidden until earned.
    #[serde(default)]
    pub is_hidden: bool,

    /// Whether achievement is active.
    #[serde(default = "default_true")]
    pub is_active: bool,

    /// Rarity level.
    #[serde(default)]
    pub rarity: AchievementRarity,

    /// How many users have earned this.
    #[serde(default)]
    pub total_earned: i32,

    /// Icon URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,

    /// Locked icon URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_locked_url: Option<String>,

    /// Creation timestamp.
    pub created_at: DateTime,

    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
}

fn default_true() -> bool {
    true
}

impl Achievement {
    /// Create a new achievement.
    #[must_use]
    pub fn new(
        achievement_key: impl Into<String>,
        name: impl Into<String>,
        criteria: Value,
    ) -> Self {
        Self {
            id: NexId::v4(),
            achievement_key: achievement_key.into(),
            name: name.into(),
            description: None,
            category: None,
            badge_type: None,
            criteria,
            points: 0,
            has_tiers: false,
            tier_criteria: None,
            is_hidden: false,
            is_active: true,
            rarity: AchievementRarity::Common,
            total_earned: 0,
            icon_url: None,
            icon_locked_url: None,
            created_at: DateTime::now(),
            updated_at: None,
        }
    }

    /// Calculate rarity percentage.
    #[must_use]
    pub fn rarity_percentage(&self, total_users: i32) -> Option<f64> {
        if total_users == 0 {
            return None;
        }
        Some(f64::from(self.total_earned) / f64::from(total_users) * 100.0)
    }

    /// Update rarity based on percentage of users who earned it.
    pub fn update_rarity(&mut self, total_users: i32) {
        if let Some(pct) = self.rarity_percentage(total_users) {
            self.rarity = if pct > 50.0 {
                AchievementRarity::Common
            } else if pct > 20.0 {
                AchievementRarity::Uncommon
            } else if pct > 5.0 {
                AchievementRarity::Rare
            } else {
                AchievementRarity::Legendary
            };
            self.updated_at = Some(DateTime::now());
        }
    }

    /// Increment earned count.
    pub fn record_earned(&mut self) {
        self.total_earned += 1;
        self.updated_at = Some(DateTime::now());
    }
}

/// Points and level calculation for gamification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelInfo {
    /// Current level.
    pub level: i32,
    /// Total points.
    pub total_points: i32,
    /// Points needed for next level.
    pub points_to_next_level: i32,
    /// Points at start of current level.
    pub points_at_level_start: i32,
}

impl LevelInfo {
    /// Calculate level from total points.
    ///
    /// Uses a simple quadratic formula: level = floor(sqrt(points / 100)) + 1
    /// This means:
    /// - Level 1: 0-99 points
    /// - Level 2: 100-399 points
    /// - Level 3: 400-899 points
    /// - Level 10: 8100-9999 points
    #[must_use]
    pub fn from_points(total_points: i32) -> Self {
        // Quadratic leveling: level n requires 100 * (n-1)^2 points
        // So level = floor(sqrt(points / 100)) + 1
        let level = ((total_points as f64 / 100.0).sqrt().floor() as i32) + 1;
        let points_at_level_start = 100 * (level - 1) * (level - 1);
        let points_for_next_level = 100 * level * level;

        Self {
            level,
            total_points,
            points_to_next_level: (points_for_next_level - total_points).max(0),
            points_at_level_start,
        }
    }

    /// Get progress percentage within current level.
    #[must_use]
    pub fn level_progress(&self) -> f64 {
        let points_in_level = self.total_points - self.points_at_level_start;
        let level_range = 100 * self.level * self.level - self.points_at_level_start;

        if level_range == 0 {
            100.0
        } else {
            (f64::from(points_in_level) / f64::from(level_range) * 100.0).min(100.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_badge_new() {
        let profile_id = NexId::v4();
        let badge = UserBadge::new(profile_id, BadgeType::NewMember);

        assert_eq!(badge.user_profile_id, profile_id);
        assert_eq!(badge.badge_type, BadgeType::NewMember);
        assert_eq!(badge.tier, 1);
    }

    #[test]
    fn test_badge_tier_names() {
        let profile_id = NexId::v4();
        let mut badge = UserBadge::new(profile_id, BadgeType::ActiveContributor);

        assert_eq!(badge.tier_name(), "Bronze");

        badge.tier = 2;
        assert_eq!(badge.tier_name(), "Silver");

        badge.tier = 3;
        assert_eq!(badge.tier_name(), "Gold");
    }

    #[test]
    fn test_badge_progress() {
        let profile_id = NexId::v4();
        let mut badge = UserBadge::new(profile_id, BadgeType::ActiveContributor);

        badge.progress_current = Some(7);
        badge.progress_target = Some(10);

        let pct = badge.progress_percentage();
        assert!(pct.is_some());
        assert!((pct.unwrap() - 70.0).abs() < f64::EPSILON);
        assert!(!badge.can_upgrade());

        badge.progress_current = Some(10);
        assert!(badge.can_upgrade());
    }

    #[test]
    fn test_achievement_new() {
        let achievement = Achievement::new(
            "first_post",
            "First Post",
            serde_json::json!({ "posts_count": 1 }),
        );

        assert_eq!(achievement.achievement_key, "first_post");
        assert_eq!(achievement.name, "First Post");
        assert!(achievement.is_active);
    }

    #[test]
    fn test_achievement_rarity() {
        let mut achievement = Achievement::new("test", "Test Achievement", serde_json::json!({}));

        // 6% earned = Rare (5-20% range)
        achievement.total_earned = 6;
        achievement.update_rarity(100);
        assert_eq!(achievement.rarity, AchievementRarity::Rare);

        // 60% earned = Common (>50%)
        achievement.total_earned = 60;
        achievement.update_rarity(100);
        assert_eq!(achievement.rarity, AchievementRarity::Common);
    }

    #[test]
    fn test_level_calculation() {
        // Level 1: 0-99 points (100 * 0^2 to 100 * 1^2 - 1)
        let info = LevelInfo::from_points(50);
        assert_eq!(info.level, 1);

        // Level 2: 100-399 points (100 * 1^2 to 100 * 2^2 - 1)
        let info = LevelInfo::from_points(100);
        assert_eq!(info.level, 2);

        let info = LevelInfo::from_points(399);
        assert_eq!(info.level, 2);

        // Level 3: 400-899 points
        let info = LevelInfo::from_points(400);
        assert_eq!(info.level, 3);

        // Level 11: 10000+ points (100 * 10^2)
        let info = LevelInfo::from_points(10000);
        assert_eq!(info.level, 11);
    }

    #[test]
    fn test_level_progress() {
        // 250 points is in level 2 (100-399 range)
        let info = LevelInfo::from_points(250);
        assert_eq!(info.level, 2);

        let progress = info.level_progress();
        // 250 is 150 points into level 2, which spans 300 points (100-400)
        // 150 / 300 = 50%
        assert!(
            progress > 40.0 && progress < 60.0,
            "Progress was {}",
            progress
        );
    }
}
