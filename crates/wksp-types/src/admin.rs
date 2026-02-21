//! Admin types — dashboard stats, content pipeline, leads

use serde::{Deserialize, Serialize};
use crate::common::Timestamp;

/// Admin dashboard summary statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdminStats {
    pub total_users: u32,
    pub active_users_30d: u32,
    pub total_enrollments: u32,
    pub total_posts: u32,
    pub total_leads: u32,
    pub pending_moderation: u32,
}

/// Content pipeline item (course, article, etc. in review)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentPipelineItem {
    pub id: String,
    pub title: String,
    pub content_type: ContentType,
    pub status: ContentStatus,
    pub author_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<Timestamp>,
}

/// Content type in the pipeline
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Course,
    Article,
    Template,
    Resource,
}

/// Content pipeline status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentStatus {
    Draft,
    InReview,
    Approved,
    Published,
    Archived,
}

/// Website lead from contact/demo/trial forms
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadEntry {
    pub id: String,
    pub name: String,
    pub email: String,
    pub lead_type: LeadType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    pub status: LeadStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<Timestamp>,
}

/// Lead source type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LeadType {
    Contact,
    DemoRequest,
    TrialSignup,
    Enterprise,
}

/// Lead follow-up status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LeadStatus {
    New,
    Contacted,
    Qualified,
    Converted,
    Closed,
}
