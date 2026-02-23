//! Community types — posts, circles, messages, members

use crate::common::Timestamp;
use serde::{Deserialize, Serialize};

/// Community post
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub content: String,
    pub post_type: PostType,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circle_id: Option<String>,
    pub like_count: u32,
    pub reply_count: u32,
    pub view_count: u32,
    pub is_pinned: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<Timestamp>,
}

/// Post type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PostType {
    Discussion,
    Question,
    Article,
    Resource,
    Announcement,
}

/// Reply to a post
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reply {
    pub id: String,
    pub post_id: String,
    pub author_id: String,
    pub content: String,
    pub like_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_reply_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<Timestamp>,
}

/// Community circle (topic-based group)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Circle {
    pub id: String,
    pub name: String,
    pub description: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub member_count: u32,
    pub post_count: u32,
    pub is_public: bool,
    pub owner_id: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<Timestamp>,
}

/// Direct message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: String,
    pub is_read: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<Timestamp>,
}

/// Conversation thread
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub participant_ids: Vec<String>,
    pub last_message_preview: String,
    pub unread_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<Timestamp>,
}

/// Notification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    pub is_read: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<Timestamp>,
}

/// Notification type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    NewReply,
    NewLike,
    NewFollower,
    NewMessage,
    CircleInvite,
    SystemAlert,
}

/// Circle membership
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CircleMembership {
    pub circle_id: String,
    pub user_id: String,
    pub role: CircleRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub joined_at: Option<Timestamp>,
}

/// Member role within a circle
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CircleRole {
    Member,
    Moderator,
    Owner,
}
