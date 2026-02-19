//! Community domain types: Flarum integration and engagement tracking.

use chrono::{DateTime, Utc};
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::enums::{EngagementLevel, FlarumSyncStatus};

/// Flarum user linkage and SSO mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlarumUser {
    /// Unique identifier.
    pub id: NexId,

    /// Associated user ID from our system.
    pub user_id: NexId,

    /// Flarum's internal user ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flarum_user_id: Option<i32>,

    /// Flarum username.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flarum_username: Option<String>,

    /// Flarum display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flarum_display_name: Option<String>,

    /// Current SSO token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sso_token: Option<String>,

    /// SSO token expiration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sso_token_expires_at: Option<DateTime<Utc>>,

    /// Last SSO login time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sso_login: Option<DateTime<Utc>>,

    /// Sync status.
    #[serde(default)]
    pub sync_status: FlarumSyncStatus,

    /// Last sync time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync_at: Option<DateTime<Utc>>,

    /// Sync error message (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_error: Option<String>,

    /// When account was created in Flarum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flarum_created_at: Option<DateTime<Utc>>,

    /// Full Flarum user object for reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flarum_data: Option<Value>,

    /// Creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl FlarumUser {
    /// Create a new Flarum user linkage.
    #[must_use]
    pub fn new(user_id: NexId) -> Self {
        Self {
            id: NexId::v4(),
            user_id,
            flarum_user_id: None,
            flarum_username: None,
            flarum_display_name: None,
            sso_token: None,
            sso_token_expires_at: None,
            last_sso_login: None,
            sync_status: FlarumSyncStatus::Pending,
            last_sync_at: None,
            sync_error: None,
            flarum_created_at: None,
            flarum_data: None,
            created_at: Utc::now(),
            updated_at: None,
        }
    }

    /// Check if SSO token is expired.
    #[must_use]
    pub fn is_token_expired(&self) -> bool {
        match self.sso_token_expires_at {
            Some(expires_at) => Utc::now() > expires_at,
            None => true, // No token = expired
        }
    }

    /// Check if user is synced with Flarum.
    #[must_use]
    pub fn is_synced(&self) -> bool {
        self.sync_status == FlarumSyncStatus::Synced && self.flarum_user_id.is_some()
    }

    /// Mark sync as successful.
    pub fn mark_synced(&mut self) {
        self.sync_status = FlarumSyncStatus::Synced;
        self.last_sync_at = Some(Utc::now());
        self.sync_error = None;
        self.updated_at = Some(Utc::now());
    }

    /// Mark sync as failed.
    pub fn mark_sync_failed(&mut self, error: impl Into<String>) {
        self.sync_status = FlarumSyncStatus::Failed;
        self.sync_error = Some(error.into());
        self.updated_at = Some(Utc::now());
    }
}

/// Track user engagement in Flarum community.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommunityEngagement {
    /// Unique identifier.
    pub id: NexId,

    /// Associated Flarum user ID (our UUID, not Flarum's int ID).
    pub flarum_user_id: NexId,

    /// Total posts made.
    #[serde(default)]
    pub total_posts: i32,

    /// Total discussions/threads started.
    #[serde(default)]
    pub total_discussions: i32,

    /// Total replies to discussions.
    #[serde(default)]
    pub total_replies: i32,

    /// Posts marked as best/helpful answer.
    #[serde(default)]
    pub best_answers: i32,

    /// Likes received.
    #[serde(default)]
    pub likes_received: i32,

    /// Likes given to others.
    #[serde(default)]
    pub likes_given: i32,

    /// Times mentioned by others.
    #[serde(default)]
    pub mentions_received: i32,

    /// Last post timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_post_at: Option<DateTime<Utc>>,

    /// Last discussion timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_discussion_at: Option<DateTime<Utc>>,

    /// Total days with activity.
    #[serde(default)]
    pub days_active: i32,

    /// Current consecutive days active.
    #[serde(default)]
    pub current_streak: i32,

    /// Longest streak ever.
    #[serde(default)]
    pub longest_streak: i32,

    /// Custom reputation score.
    #[serde(default)]
    pub reputation_score: i32,

    /// Whether user is a moderator.
    #[serde(default)]
    pub is_moderator: bool,

    /// Whether user is tagged as expert.
    #[serde(default)]
    pub is_expert: bool,

    /// Posts in Medical Affairs category.
    #[serde(default)]
    pub medical_affairs_posts: i32,

    /// Posts in Regulatory Affairs category.
    #[serde(default)]
    pub regulatory_affairs_posts: i32,

    /// Posts in HEOR category.
    #[serde(default)]
    pub heor_posts: i32,

    /// Posts in Clinical Development category.
    #[serde(default)]
    pub clinical_dev_posts: i32,

    /// Calculated engagement level.
    #[serde(default)]
    pub engagement_level: EngagementLevel,

    /// Creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,

    /// When metrics were last calculated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_calculated_at: Option<DateTime<Utc>>,
}

impl CommunityEngagement {
    /// Create a new engagement record.
    #[must_use]
    pub fn new(flarum_user_id: NexId) -> Self {
        Self {
            id: NexId::v4(),
            flarum_user_id,
            created_at: Utc::now(),
            ..Default::default()
        }
    }

    /// Calculate engagement level based on activity.
    ///
    /// Levels:
    /// - Lurker: < 5 posts
    /// - Casual: 5-19 posts
    /// - Active: 20-49 posts
    /// - Power User: 50-99 posts
    /// - Expert: 100+ posts or marked as expert
    #[must_use]
    pub fn calculate_engagement_level(&self) -> EngagementLevel {
        if self.is_expert || self.total_posts >= 100 {
            EngagementLevel::Expert
        } else if self.total_posts >= 50 {
            EngagementLevel::PowerUser
        } else if self.total_posts >= 20 {
            EngagementLevel::Active
        } else if self.total_posts >= 5 {
            EngagementLevel::Casual
        } else {
            EngagementLevel::Lurker
        }
    }

    /// Update engagement level based on current stats.
    pub fn update_engagement_level(&mut self) {
        self.engagement_level = self.calculate_engagement_level();
        self.updated_at = Some(Utc::now());
    }

    /// Calculate reputation score.
    ///
    /// Formula:
    /// - Posts: 1 point each
    /// - Discussions started: 3 points each
    /// - Best answers: 10 points each
    /// - Likes received: 2 points each
    /// - Streak bonus: current_streak * 5
    #[must_use]
    pub fn calculate_reputation(&self) -> i32 {
        self.total_posts
            + (self.total_discussions * 3)
            + (self.best_answers * 10)
            + (self.likes_received * 2)
            + (self.current_streak * 5)
    }

    /// Update reputation score.
    pub fn update_reputation(&mut self) {
        self.reputation_score = self.calculate_reputation();
        self.last_calculated_at = Some(Utc::now());
        self.updated_at = Some(Utc::now());
    }

    /// Get primary vertical based on post distribution.
    #[must_use]
    pub fn primary_vertical(&self) -> Option<&'static str> {
        let posts = [
            (self.medical_affairs_posts, "Medical Affairs"),
            (self.regulatory_affairs_posts, "Regulatory Affairs"),
            (self.heor_posts, "HEOR"),
            (self.clinical_dev_posts, "Clinical Development"),
        ];

        posts
            .into_iter()
            .filter(|(count, _)| *count > 0)
            .max_by_key(|(count, _)| *count)
            .map(|(_, name)| name)
    }

    /// Record a new post.
    pub fn record_post(&mut self, is_discussion: bool) {
        self.total_posts += 1;
        if is_discussion {
            self.total_discussions += 1;
            self.last_discussion_at = Some(Utc::now());
        } else {
            self.total_replies += 1;
        }
        self.last_post_at = Some(Utc::now());
        self.update_engagement_level();
    }

    /// Update streak (call daily).
    pub fn update_streak(&mut self, was_active_today: bool) {
        if was_active_today {
            self.current_streak += 1;
            self.days_active += 1;
            if self.current_streak > self.longest_streak {
                self.longest_streak = self.current_streak;
            }
        } else {
            self.current_streak = 0;
        }
        self.updated_at = Some(Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flarum_user_new() {
        let user_id = NexId::v4();
        let flarum = FlarumUser::new(user_id);

        assert_eq!(flarum.user_id, user_id);
        assert_eq!(flarum.sync_status, FlarumSyncStatus::Pending);
        assert!(!flarum.is_synced());
    }

    #[test]
    fn test_flarum_sync_status() {
        let user_id = NexId::v4();
        let mut flarum = FlarumUser::new(user_id);

        flarum.flarum_user_id = Some(123);
        flarum.mark_synced();

        assert!(flarum.is_synced());
        assert_eq!(flarum.sync_status, FlarumSyncStatus::Synced);

        flarum.mark_sync_failed("Connection timeout");
        assert!(!flarum.is_synced());
        assert_eq!(flarum.sync_error, Some("Connection timeout".to_string()));
    }

    #[test]
    fn test_engagement_levels() {
        let flarum_user_id = NexId::v4();
        let mut engagement = CommunityEngagement::new(flarum_user_id);

        assert_eq!(
            engagement.calculate_engagement_level(),
            EngagementLevel::Lurker
        );

        engagement.total_posts = 10;
        assert_eq!(
            engagement.calculate_engagement_level(),
            EngagementLevel::Casual
        );

        engagement.total_posts = 50;
        assert_eq!(
            engagement.calculate_engagement_level(),
            EngagementLevel::PowerUser
        );

        engagement.total_posts = 100;
        assert_eq!(
            engagement.calculate_engagement_level(),
            EngagementLevel::Expert
        );

        // Expert flag overrides post count
        engagement.total_posts = 5;
        engagement.is_expert = true;
        assert_eq!(
            engagement.calculate_engagement_level(),
            EngagementLevel::Expert
        );
    }

    #[test]
    fn test_reputation_calculation() {
        let flarum_user_id = NexId::v4();
        let mut engagement = CommunityEngagement::new(flarum_user_id);

        engagement.total_posts = 10; // 10 points
        engagement.total_discussions = 5; // 15 points
        engagement.best_answers = 2; // 20 points
        engagement.likes_received = 10; // 20 points
        engagement.current_streak = 7; // 35 points

        assert_eq!(engagement.calculate_reputation(), 100);
    }

    #[test]
    fn test_primary_vertical() {
        let flarum_user_id = NexId::v4();
        let mut engagement = CommunityEngagement::new(flarum_user_id);

        assert!(engagement.primary_vertical().is_none());

        engagement.medical_affairs_posts = 5;
        engagement.regulatory_affairs_posts = 10;
        engagement.heor_posts = 3;

        assert_eq!(engagement.primary_vertical(), Some("Regulatory Affairs"));
    }

    #[test]
    fn test_streak_tracking() {
        let flarum_user_id = NexId::v4();
        let mut engagement = CommunityEngagement::new(flarum_user_id);

        engagement.update_streak(true);
        assert_eq!(engagement.current_streak, 1);
        assert_eq!(engagement.days_active, 1);

        engagement.update_streak(true);
        engagement.update_streak(true);
        assert_eq!(engagement.current_streak, 3);
        assert_eq!(engagement.longest_streak, 3);

        engagement.update_streak(false); // Break streak
        assert_eq!(engagement.current_streak, 0);
        assert_eq!(engagement.longest_streak, 3); // Preserved
    }
}
