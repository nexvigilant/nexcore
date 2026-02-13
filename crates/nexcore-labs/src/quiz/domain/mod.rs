//! Domain models for the quiz platform.
//!
//! This module contains all core types representing the quiz platform's
//! domain model, migrated from Python Ormar/Pydantic models.
//!
//! ## Organization
//!
//! - [`user`] - User accounts, authentication types, sessions
//! - [`quiz`] - Quiz definitions, questions
//! - [`game`] - Live game sessions, players, answers
//! - [`quiztivity`] - Interactive slide presentations
//! - [`storage`] - File storage items

pub mod game;
pub mod quiz;
pub mod quiztivity;
pub mod storage;
pub mod user;
