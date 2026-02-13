//! Intelligence types — articles, series, content hub

use serde::{Deserialize, Serialize};
use crate::common::ContentStatus;

/// Content type matching editorial categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Podcast,
    Publication,
    Perspective,
    FieldNote,
    Signal,
}

/// Intelligence article/content piece
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub content_type: ContentType,
    pub status: ContentStatus,
    pub published_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading_time: Option<u32>,
    #[serde(default)]
    pub featured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_order: Option<u32>,
}

/// Content series (e.g., "FDA Watch 2025")
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub id: String,
    pub title: String,
    pub description: String,
    pub slug: String,
    pub article_count: u32,
}
