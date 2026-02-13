//! # State Management Module
//!
//! Checkpoint persistence and state server for distributed execution.

pub mod manager;

pub use manager::{Checkpoint, StateManager};
