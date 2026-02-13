//! Question and answer types for the quiz platform.
//!
//! Supports polymorphic question types:
//! - ABCD (multiple choice, single answer)
//! - CHECK (multiple choice, multiple answers)
//! - RANGE (numeric range)
//! - VOTING (poll, no correct answer)
//! - TEXT (free text)
//! - ORDER (sequence ordering)
//! - SLIDE (information only)

use serde::{Deserialize, Serialize};

/// Polymorphic answer type that varies by question type.
///
/// Uses serde's adjacently tagged enum for JSON compatibility.
/// Serializes to: `{"type": "ABCD", "answers": [...]}`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "answers", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuizAnswer {
    /// Multiple choice with single correct answer.
    Abcd(Vec<AbcdAnswer>),

    /// Multiple choice with multiple correct answers.
    Check(Vec<CheckAnswer>),

    /// Numeric range answer.
    Range(RangeAnswer),

    /// Poll/voting (no correct answer).
    Voting(Vec<VotingAnswer>),

    /// Free text answer.
    Text(Vec<TextAnswer>),

    /// Sequence ordering.
    Order(Vec<OrderAnswer>),

    /// Information slide (no answer).
    Slide(String),
}

/// ABCD multiple choice answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcdAnswer {
    /// The answer text.
    pub answer: String,

    /// Whether this is the correct answer.
    pub right: bool,

    /// Optional display color (CSS color string).
    pub color: Option<String>,
}

impl AbcdAnswer {
    /// Create a new ABCD answer.
    pub fn new(answer: String, right: bool) -> Self {
        Self {
            answer,
            right,
            color: None,
        }
    }

    /// Create a correct answer.
    pub fn correct(answer: String) -> Self {
        Self::new(answer, true)
    }

    /// Create an incorrect answer.
    pub fn incorrect(answer: String) -> Self {
        Self::new(answer, false)
    }
}

/// CHECK multiple choice answer (same structure as ABCD).
pub type CheckAnswer = AbcdAnswer;

/// Numeric range answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeAnswer {
    /// Minimum allowed value.
    pub min: i64,

    /// Maximum allowed value.
    pub max: i64,

    /// Minimum correct value (inclusive).
    pub min_correct: i64,

    /// Maximum correct value (inclusive).
    pub max_correct: i64,
}

impl RangeAnswer {
    /// Create a new range answer.
    pub fn new(min: i64, max: i64, min_correct: i64, max_correct: i64) -> Self {
        Self {
            min,
            max,
            min_correct,
            max_correct,
        }
    }

    /// Check if a value is within the correct range.
    pub fn is_correct(&self, value: i64) -> bool {
        value >= self.min_correct && value <= self.max_correct
    }

    /// Check if a value is within the allowed range.
    pub fn is_valid(&self, value: i64) -> bool {
        value >= self.min && value <= self.max
    }
}

/// Voting/poll answer (no correct answer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingAnswer {
    /// The answer text.
    pub answer: String,

    /// Optional image URL.
    pub image: Option<String>,

    /// Optional display color.
    pub color: Option<String>,
}

impl VotingAnswer {
    /// Create a new voting answer.
    pub fn new(answer: String) -> Self {
        Self {
            answer,
            image: None,
            color: None,
        }
    }
}

/// Free text answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAnswer {
    /// The correct answer text.
    pub answer: String,

    /// Whether comparison is case-sensitive.
    pub case_sensitive: bool,
}

impl TextAnswer {
    /// Create a new text answer.
    pub fn new(answer: String, case_sensitive: bool) -> Self {
        Self {
            answer,
            case_sensitive,
        }
    }

    /// Check if a submitted answer matches.
    pub fn matches(&self, submitted: &str) -> bool {
        if self.case_sensitive {
            self.answer == submitted
        } else {
            self.answer.to_lowercase() == submitted.to_lowercase()
        }
    }
}

/// Order/sequence answer (same structure as voting).
pub type OrderAnswer = VotingAnswer;

/// Answer without solution (for client display).
///
/// Used when sending questions to players to hide correct answers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcdAnswerWithoutSolution {
    /// The answer text.
    pub answer: String,

    /// Optional display color.
    pub color: Option<String>,
}

impl From<&AbcdAnswer> for AbcdAnswerWithoutSolution {
    fn from(answer: &AbcdAnswer) -> Self {
        Self {
            answer: answer.answer.clone(),
            color: answer.color.clone(),
        }
    }
}

/// Range answer without solution (for client display).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeAnswerWithoutSolution {
    /// Minimum allowed value.
    pub min: i64,

    /// Maximum allowed value.
    pub max: i64,
}

impl From<&RangeAnswer> for RangeAnswerWithoutSolution {
    fn from(answer: &RangeAnswer) -> Self {
        Self {
            min: answer.min,
            max: answer.max,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abcd_answer() {
        let correct = AbcdAnswer::correct("Paris".into());
        assert!(correct.right);

        let incorrect = AbcdAnswer::incorrect("London".into());
        assert!(!incorrect.right);
    }

    #[test]
    fn test_range_answer() {
        let range = RangeAnswer::new(0, 100, 40, 60);

        assert!(range.is_valid(50));
        assert!(range.is_valid(0));
        assert!(!range.is_valid(101));

        assert!(range.is_correct(50));
        assert!(range.is_correct(40));
        assert!(range.is_correct(60));
        assert!(!range.is_correct(39));
        assert!(!range.is_correct(61));
    }

    #[test]
    fn test_text_answer_case_sensitive() {
        let sensitive = TextAnswer::new("Paris".into(), true);
        assert!(sensitive.matches("Paris"));
        assert!(!sensitive.matches("paris"));
        assert!(!sensitive.matches("PARIS"));
    }

    #[test]
    fn test_text_answer_case_insensitive() {
        let insensitive = TextAnswer::new("Paris".into(), false);
        assert!(insensitive.matches("Paris"));
        assert!(insensitive.matches("paris"));
        assert!(insensitive.matches("PARIS"));
    }

    #[test]
    fn test_quiz_answer_serialization() {
        let answer = QuizAnswer::Abcd(vec![
            AbcdAnswer::correct("A".into()),
            AbcdAnswer::incorrect("B".into()),
        ]);

        let json = serde_json::to_string(&answer);
        assert!(json.is_ok());
    }
}
