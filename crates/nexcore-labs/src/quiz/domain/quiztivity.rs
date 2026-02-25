//! QuizTivity domain types.
//!
//! Defines interactive slide presentations with various page types.
//! QuizTivity is a feature for creating interactive content beyond quizzes.

use nexcore_chrono::DateTime;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};

/// An interactive slide presentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizTivity {
    /// Unique identifier.
    pub id: NexId,

    /// Presentation title.
    pub title: String,

    /// Creation timestamp.
    pub created_at: DateTime,

    /// Owner user ID.
    pub user_id: NexId,

    /// Pages in the presentation.
    pub pages: Vec<QuizTivityPage>,
}

impl QuizTivity {
    /// Create a new QuizTivity presentation.
    pub fn new(user_id: NexId, title: String) -> Self {
        Self {
            id: NexId::v4(),
            title,
            created_at: DateTime::now(),
            user_id,
            pages: Vec::new(),
        }
    }

    /// Add a page to the presentation.
    pub fn add_page(&mut self, page: QuizTivityPage) {
        self.pages.push(page);
    }

    /// Get the number of pages.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}

/// A single page in a QuizTivity presentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizTivityPage {
    /// Optional page title.
    pub title: Option<String>,

    /// Page type.
    #[serde(rename = "type")]
    pub page_type: QuizTivityType,

    /// Page content (varies by type).
    pub data: QuizTivityData,
}

/// QuizTivity page types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuizTivityType {
    /// Information slide (markdown or HTML).
    Slide,
    /// Embedded PDF document.
    Pdf,
    /// Memory matching game.
    Memory,
    /// Markdown content.
    Markdown,
    /// ABCD quiz question.
    Abcd,
}

/// Page content union type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QuizTivityData {
    /// PDF page data.
    Pdf(PdfData),
    /// Memory game data.
    Memory(MemoryData),
    /// Markdown content.
    Markdown(MarkdownData),
    /// ABCD question.
    Abcd(AbcdData),
    /// Raw slide content (fallback).
    Slide(String),
}

/// PDF page data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfData {
    /// PDF URL.
    pub url: String,
}

/// Memory game data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryData {
    /// Card pairs for matching.
    pub cards: Vec<Vec<MemoryCard>>,
}

/// A single memory card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCard {
    /// Card ID for matching pairs.
    pub id: String,

    /// Optional image URL.
    pub image: Option<String>,

    /// Optional text content.
    pub text: Option<String>,
}

/// Markdown page data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownData {
    /// Markdown content.
    pub markdown: String,
}

/// ABCD question data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcdData {
    /// Question text.
    pub question: String,

    /// Answer options.
    pub answers: Vec<AbcdPageAnswer>,
}

/// ABCD answer option for QuizTivity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcdPageAnswer {
    /// Answer text.
    pub answer: String,

    /// Whether this is the correct answer.
    pub correct: bool,
}

/// QuizTivity share link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizTivityShare {
    /// Share ID.
    pub id: NexId,

    /// Optional share name/label.
    pub name: Option<String>,

    /// Expiration timestamp.
    pub expire_at: Option<DateTime>,

    /// QuizTivity ID being shared.
    pub quiztivity_id: NexId,

    /// Owner user ID.
    pub user_id: NexId,
}

impl QuizTivityShare {
    /// Create a new share link.
    pub fn new(quiztivity_id: NexId, user_id: NexId, expire_in_minutes: Option<i64>) -> Self {
        let expire_at =
            expire_in_minutes.map(|mins| DateTime::now() + nexcore_chrono::Duration::minutes(mins));

        Self {
            id: NexId::v4(),
            name: None,
            expire_at,
            quiztivity_id,
            user_id,
        }
    }

    /// Check if the share link has expired.
    pub fn is_expired(&self) -> bool {
        self.expire_at.is_some_and(|exp| DateTime::now() > exp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_quiztivity() {
        let user_id = NexId::v4();
        let qt = QuizTivity::new(user_id, "My Presentation".into());

        assert_eq!(qt.title, "My Presentation");
        assert_eq!(qt.page_count(), 0);
        assert_eq!(qt.user_id, user_id);
    }

    #[test]
    fn test_add_page() {
        let mut qt = QuizTivity::new(NexId::v4(), "Test".into());

        let page = QuizTivityPage {
            title: Some("Intro".into()),
            page_type: QuizTivityType::Markdown,
            data: QuizTivityData::Markdown(MarkdownData {
                markdown: "# Hello World".into(),
            }),
        };

        qt.add_page(page);
        assert_eq!(qt.page_count(), 1);
    }

    #[test]
    fn test_share_expiration() {
        let share = QuizTivityShare::new(NexId::v4(), NexId::v4(), Some(60));
        assert!(!share.is_expired());

        // Share with no expiration never expires
        let permanent = QuizTivityShare::new(NexId::v4(), NexId::v4(), None);
        assert!(!permanent.is_expired());
    }
}
