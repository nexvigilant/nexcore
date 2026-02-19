//! Lessons Learned Parameters (Semantic Repositories)
//!
//! Storage, retrieval, and search for developmental lessons.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for adding a lesson.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LessonAddParams {
    /// Lesson title.
    pub title: String,
    /// Lesson content.
    pub content: String,
    /// Context.
    pub context: String,
    /// Optional tags.
    pub tags: Option<Vec<String>>,
    /// Optional source.
    pub source: Option<String>,
}

/// Parameters for retrieving a lesson.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LessonGetParams {
    /// Lesson ID.
    pub id: u64,
}

/// Parameters for searching lessons.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LessonSearchParams {
    /// Query string.
    pub query: String,
}

/// Parameters for filtering lessons by context.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LessonByContextParams {
    /// Context to filter by.
    pub context: String,
}

/// Parameters for filtering lessons by tag.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LessonByTagParams {
    /// Tag to filter by.
    pub tag: String,
}
