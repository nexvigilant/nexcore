//! Game session domain types.
//!
//! Defines live game sessions, players, and answer tracking.
//! These types are stored in Redis during active games.

use chrono::{DateTime, Utc};
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};

use super::quiz::QuizQuestion;

/// An active game session (stored in Redis).
///
/// Key format: `game:{game_pin}`
/// TTL: 7200 seconds (2 hours)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayGame {
    /// Original quiz ID.
    pub quiz_id: NexId,

    /// Quiz description.
    pub description: String,

    /// Host user ID.
    pub user_id: NexId,

    /// Quiz title.
    pub title: String,

    /// Questions in this game session.
    pub questions: Vec<QuizQuestion>,

    /// Game session ID.
    pub game_id: NexId,

    /// 6-digit game pin for players to join.
    pub game_pin: String,

    /// Whether the game has started.
    pub started: bool,

    /// Whether captcha is required to join.
    pub captcha_enabled: bool,

    /// Cover image URL.
    pub cover_image: Option<String>,

    /// Game mode (e.g., "classic", "team").
    pub game_mode: Option<String>,

    /// Current question index (-1 = not started).
    pub current_question: i32,

    /// Background color.
    pub background_color: Option<String>,

    /// Background image URL.
    pub background_image: Option<String>,

    /// Custom field prompt for players.
    pub custom_field: Option<String>,

    /// Whether the current question is being shown.
    pub question_show: bool,
}

impl PlayGame {
    /// Create a new game session from a quiz.
    pub fn new(
        quiz_id: NexId,
        user_id: NexId,
        title: String,
        description: String,
        questions: Vec<QuizQuestion>,
    ) -> Self {
        Self {
            quiz_id,
            description,
            user_id,
            title,
            questions,
            game_id: NexId::v4(),
            game_pin: generate_game_pin(),
            started: false,
            captcha_enabled: false,
            cover_image: None,
            game_mode: None,
            current_question: -1,
            background_color: None,
            background_image: None,
            custom_field: None,
            question_show: false,
        }
    }

    /// Get the current question (None if not started or ended).
    pub fn get_current_question(&self) -> Option<&QuizQuestion> {
        if self.current_question < 0 {
            return None;
        }
        self.questions.get(self.current_question as usize)
    }

    /// Check if the game has ended.
    pub fn is_ended(&self) -> bool {
        self.current_question >= self.questions.len() as i32
    }

    /// Advance to the next question.
    pub fn next_question(&mut self) -> bool {
        if self.is_ended() {
            return false;
        }
        self.current_question += 1;
        self.question_show = true;
        !self.is_ended()
    }
}

/// A player in a game session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePlayer {
    /// Player's chosen username.
    pub username: String,

    /// WebSocket session ID (for reconnection).
    pub sid: Option<String>,

    /// Custom field value (if required).
    pub custom_field: Option<String>,
}

impl GamePlayer {
    /// Create a new player.
    pub fn new(username: String, sid: Option<String>) -> Self {
        Self {
            username,
            sid,
            custom_field: None,
        }
    }
}

/// Game session state (answer tracking).
///
/// Key format: `game_session:{game_pin}`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    /// Admin WebSocket session ID.
    pub admin: String,

    /// Game ID.
    pub game_id: String,

    /// Answers for each question.
    pub answers: Vec<Option<Vec<AnswerData>>>,
}

impl GameSession {
    /// Create a new game session.
    pub fn new(admin: String, game_id: String, question_count: usize) -> Self {
        Self {
            admin,
            game_id,
            answers: vec![None; question_count],
        }
    }

    /// Record an answer for a question.
    pub fn record_answer(&mut self, question_index: usize, answer: AnswerData) {
        if question_index < self.answers.len() {
            if self.answers[question_index].is_none() {
                self.answers[question_index] = Some(Vec::new());
            }
            if let Some(ref mut answers) = self.answers[question_index] {
                answers.push(answer);
            }
        }
    }
}

/// A single answer submission from a player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerData {
    /// Player username.
    pub username: String,

    /// Answer value (e.g., "A", "B", "42", or free text).
    pub answer: String,

    /// Whether the answer was correct.
    pub right: bool,

    /// Time taken to answer in milliseconds.
    pub time_taken: f64,

    /// Points scored for this answer.
    pub score: i32,
}

impl AnswerData {
    /// Create a new answer record.
    pub fn new(username: String, answer: String, right: bool, time_taken: f64, score: i32) -> Self {
        Self {
            username,
            answer,
            right,
            time_taken,
            score,
        }
    }
}

/// Completed game results (stored in database).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResults {
    /// Result ID.
    pub id: NexId,

    /// Original quiz ID.
    pub quiz_id: NexId,

    /// Host user ID.
    pub user_id: NexId,

    /// Completion timestamp.
    pub timestamp: DateTime<Utc>,

    /// Number of players.
    pub player_count: i32,

    /// Optional note.
    pub note: Option<String>,

    /// All answers for all questions.
    pub answers: Vec<Vec<AnswerData>>,

    /// Final player scores (username -> score string).
    pub player_scores: Option<std::collections::HashMap<String, String>>,

    /// Custom field data (username -> value).
    pub custom_field_data: Option<std::collections::HashMap<String, String>>,

    /// Quiz title at time of play.
    pub title: String,

    /// Quiz description at time of play.
    pub description: String,

    /// Questions at time of play.
    pub questions: Vec<QuizQuestion>,
}

impl GameResults {
    /// Create results from a completed game.
    pub fn from_game(game: &PlayGame, session: &GameSession) -> Self {
        let answers: Vec<Vec<AnswerData>> = session
            .answers
            .iter()
            .map(|opt| opt.clone().unwrap_or_default())
            .collect();

        Self {
            id: NexId::v4(),
            quiz_id: game.quiz_id,
            user_id: game.user_id,
            timestamp: Utc::now(),
            player_count: 0, // Should be calculated from unique usernames
            note: None,
            answers,
            player_scores: None,
            custom_field_data: None,
            title: game.title.clone(),
            description: game.description.clone(),
            questions: game.questions.clone(),
        }
    }
}

/// Game lobby entry (for listing active games).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInLobby {
    /// 6-digit game pin.
    pub game_pin: String,

    /// Quiz title.
    pub quiz_title: String,

    /// Game ID.
    pub game_id: NexId,
}

// === Helper functions ===

/// Generate a random 6-digit game pin.
fn generate_game_pin() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    format!("{:06}", rng.random_range(0..1_000_000))
}

#[cfg(test)]
mod tests {
    use super::super::super::question_types::{AbcdAnswer, QuizAnswer};
    use super::super::quiz::{QuestionType, QuizQuestion};
    use super::*;

    #[test]
    fn test_game_pin_format() {
        let pin = generate_game_pin();
        assert_eq!(pin.len(), 6);
        assert!(pin.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_play_game_progression() {
        let answers = QuizAnswer::Abcd(vec![AbcdAnswer {
            answer: "A".into(),
            right: true,
            color: None,
        }]);

        let question = QuizQuestion {
            question: "Test?".into(),
            time_seconds: 30,
            question_type: QuestionType::Abcd,
            answers,
            image: None,
            hide_results: false,
        };

        let mut game = PlayGame::new(
            NexId::v4(),
            NexId::v4(),
            "Test".into(),
            "Desc".into(),
            vec![question],
        );

        assert!(!game.started);
        assert_eq!(game.current_question, -1);
        assert!(game.get_current_question().is_none());

        game.started = true;
        assert!(game.next_question());
        assert_eq!(game.current_question, 0);
        assert!(game.get_current_question().is_some());

        assert!(!game.next_question()); // No more questions
        assert!(game.is_ended());
    }

    #[test]
    fn test_answer_recording() {
        let mut session = GameSession::new("admin_sid".into(), "game_123".into(), 3);

        let answer = AnswerData::new("player1".into(), "A".into(), true, 1500.0, 100);
        session.record_answer(0, answer);

        assert!(session.answers[0].is_some());
        assert_eq!(session.answers[0].as_ref().map(|a| a.len()), Some(1));
    }
}
