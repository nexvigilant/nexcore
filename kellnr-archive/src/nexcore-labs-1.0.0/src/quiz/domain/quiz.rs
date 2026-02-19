//! Quiz domain types.
//!
//! Defines quiz definitions, questions, and their variants.
//! Migrated from Python Ormar Quiz model and Pydantic QuizQuestion.

use chrono::{DateTime, Utc};
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};

use super::super::question_types::QuizAnswer;

/// A quiz containing multiple questions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quiz {
    /// Unique quiz identifier.
    pub id: NexId,

    /// Whether the quiz is publicly visible.
    pub public: bool,

    /// Quiz title.
    pub title: String,

    /// Quiz description (optional).
    pub description: Option<String>,

    /// Creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,

    /// Owner user ID.
    pub user_id: NexId,

    /// List of questions in the quiz.
    pub questions: Vec<QuizQuestion>,

    /// Whether this was imported from Kahoot.
    pub imported_from_kahoot: bool,

    /// Cover image URL/path (optional).
    pub cover_image: Option<String>,

    /// Background color (CSS color, optional).
    pub background_color: Option<String>,

    /// Background image URL/path (optional).
    pub background_image: Option<String>,

    /// Original Kahoot quiz ID (if imported).
    pub kahoot_id: Option<NexId>,

    /// Number of likes.
    pub likes: i32,

    /// Number of dislikes.
    pub dislikes: i32,

    /// Number of times played.
    pub plays: i32,

    /// Number of views.
    pub views: i32,

    /// Moderator rating (optional).
    pub mod_rating: Option<i32>,
}

impl Quiz {
    /// Create a new quiz.
    pub fn new(user_id: NexId, title: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: NexId::v4(),
            public: false,
            title,
            description,
            created_at: now,
            updated_at: now,
            user_id,
            questions: Vec::new(),
            imported_from_kahoot: false,
            cover_image: None,
            background_color: None,
            background_image: None,
            kahoot_id: None,
            likes: 0,
            dislikes: 0,
            plays: 0,
            views: 0,
            mod_rating: None,
        }
    }

    /// Get the number of questions in the quiz.
    pub fn question_count(&self) -> usize {
        self.questions.len()
    }

    /// Add a question to the quiz.
    pub fn add_question(&mut self, question: QuizQuestion) {
        self.questions.push(question);
        self.updated_at = Utc::now();
    }

    /// Calculate total play time in seconds.
    pub fn total_time(&self) -> u32 {
        self.questions.iter().map(|q| q.time_seconds).sum()
    }
}

/// A single question in a quiz.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    /// The question text.
    pub question: String,

    /// Time limit in seconds.
    pub time_seconds: u32,

    /// Question type.
    #[serde(rename = "type")]
    pub question_type: QuestionType,

    /// Answers (format depends on question type).
    pub answers: QuizAnswer,

    /// Optional image URL/path.
    pub image: Option<String>,

    /// Whether to hide results from players.
    pub hide_results: bool,
}

impl QuizQuestion {
    /// Create a new ABCD question.
    pub fn new_abcd(question: String, time_seconds: u32, answers: QuizAnswer) -> Self {
        Self {
            question,
            time_seconds,
            question_type: QuestionType::Abcd,
            answers,
            image: None,
            hide_results: false,
        }
    }

    /// Check if this is a slide (non-interactive content).
    pub fn is_slide(&self) -> bool {
        self.question_type == QuestionType::Slide
    }
}

/// Question type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuestionType {
    /// Multiple choice with single correct answer (A, B, C, D).
    #[default]
    Abcd,
    /// Numeric range answer.
    Range,
    /// Voting/poll (no correct answer).
    Voting,
    /// Information slide (no interaction).
    Slide,
    /// Free text answer.
    Text,
    /// Order items in correct sequence.
    Order,
    /// Multiple choice with multiple correct answers.
    Check,
}

/// Input for creating/updating a quiz.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizInput {
    /// Whether the quiz is public.
    pub public: Option<bool>,

    /// Quiz title.
    pub title: String,

    /// Quiz description.
    pub description: String,

    /// Cover image URL.
    pub cover_image: Option<String>,

    /// Background color.
    pub background_color: Option<String>,

    /// Background image URL.
    pub background_image: Option<String>,

    /// Questions in the quiz.
    pub questions: Vec<QuizQuestion>,
}

/// Public quiz response (for API).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicQuizResponse {
    /// Quiz ID.
    pub id: NexId,

    /// Quiz title.
    pub title: String,

    /// Quiz description.
    pub description: Option<String>,

    /// Owner username.
    pub owner_username: String,

    /// Owner user ID.
    pub owner_id: NexId,

    /// Number of questions.
    pub question_count: usize,

    /// Total play time in seconds.
    pub total_time: u32,

    /// Cover image URL.
    pub cover_image: Option<String>,

    /// Like count.
    pub likes: i32,

    /// Dislike count.
    pub dislikes: i32,

    /// Play count.
    pub plays: i32,

    /// View count.
    pub views: i32,
}

#[cfg(test)]
mod tests {
    use super::super::super::question_types::AbcdAnswer;
    use super::*;

    #[test]
    fn test_new_quiz() {
        let user_id = NexId::v4();
        let quiz = Quiz::new(user_id, "Test Quiz".into(), Some("A test".into()));

        assert_eq!(quiz.title, "Test Quiz");
        assert!(!quiz.public);
        assert_eq!(quiz.question_count(), 0);
        assert_eq!(quiz.user_id, user_id);
    }

    #[test]
    fn test_add_question() {
        let mut quiz = Quiz::new(NexId::v4(), "Test".into(), None);

        let answers = QuizAnswer::Abcd(vec![
            AbcdAnswer {
                answer: "A".into(),
                right: true,
                color: None,
            },
            AbcdAnswer {
                answer: "B".into(),
                right: false,
                color: None,
            },
        ]);

        let question = QuizQuestion::new_abcd("What is 1+1?".into(), 30, answers);
        quiz.add_question(question);

        assert_eq!(quiz.question_count(), 1);
        assert_eq!(quiz.total_time(), 30);
    }

    #[test]
    fn test_question_type_default() {
        assert_eq!(QuestionType::default(), QuestionType::Abcd);
    }
}
