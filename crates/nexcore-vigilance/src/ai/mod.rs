//! # AI Model Clients
//!
//! AI model client abstractions with fallback orchestration.
//!
//! ## Features
//!
//! - **ModelClient trait**: Unified interface for AI model access
//! - **ClaudeClient**: Anthropic Claude API client
//! - **GeminiClient**: Google Gemini API client
//! - **ModelOrchestrator**: Fallback chain for reliability

mod clients;
mod orchestrator;

pub use clients::{ClaudeClient, GeminiClient, GenerationOptions, ModelClient};
pub use orchestrator::ModelOrchestrator;
