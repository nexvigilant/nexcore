//! # nexcore Quiz - Real-time Multiplayer Quiz Platform
//!
//! Rust implementation of ClassQuiz, migrated from Python FastAPI.
//! Provides real-time multiplayer quiz functionality with WebSocket support.
//!
//! ## Constructive Epistemology
//!
//! This crate applies the Construction Pipeline (SEE -> SPEAK -> DECOMPOSE ->
//! COMPOSE -> TRANSLATE -> VALIDATE -> DEPLOY -> IMPROVE) to extract pure
//! algorithms from the ClassQuiz Python codebase.
//!
//! ## UACA Hierarchy
//!
//! - **L0 Quarks**: Score constants, timing thresholds
//! - **L1 Atoms**: Score calculations, answer validation (<20 LOC)
//! - **L2 Molecules**: Game state management, composite validators (<50 LOC)
//! - **L3 Organisms**: API handlers, WebSocket handlers
//!
//! ## Modules
//!
//! - [`domain`] - Core domain types (User, Quiz, Game, etc.)
//! - [`question_types`] - Polymorphic question/answer types
//! - [`error`] - Error types for the quiz platform
//! - [`config`] - Configuration from environment
//!
//! ## Safety Axioms
//!
//! - **Score Bounds**: Quiz scores constrained to [0, max_score] range
//! - **Time Bounds**: Answer times cannot exceed question time limit
//! - **Game Pin Format**: 6-digit numeric pins only
//! - **Player Uniqueness**: No duplicate usernames within a game session

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod domain;
pub mod error;
pub mod question_types;

// Re-export key types at crate root
pub use config::QuizConfig;
pub use domain::{
    game::{AnswerData, GamePlayer, GameSession, PlayGame},
    quiz::{Quiz, QuizQuestion},
    quiztivity::{QuizTivity, QuizTivityPage},
    storage::StorageItem,
    user::{User, UserAuthType},
};
pub use error::{QuizError, QuizResult};
pub use question_types::{
    AbcdAnswer, CheckAnswer, OrderAnswer, QuizAnswer, RangeAnswer, TextAnswer, VotingAnswer,
};
