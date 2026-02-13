// Copyright 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # NexVigilant Core -- Perplexity AI Search API
//!
//! Async Rust client for the Perplexity AI Sonar API, providing
//! search-grounded AI responses with citations for three research use cases:
//!
//! 1. **General Research** -- Open-ended web search
//! 2. **Competitive Intelligence** -- Domain-filtered competitor analysis
//! 3. **Regulatory Intelligence** -- FDA/EMA/ICH/WHO filtered search
//!
//! ## Primitive Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|--------------|--------|
//! | Query to Response | Mapping | mu |
//! | Message Sequence | Sequence | sigma |
//! | API Key / Rate Limit | Boundary | boundary |
//! | Citations | Causality | causality |
//! | Model / Recency Selection | Comparison | kappa |
//!
//! Dominant primitive: **mu (Mapping)** -- the core operation is mapping
//! a query to a search-grounded response with citations.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use nexcore_perplexity::prelude::*;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = PerplexityClient::from_env()?;
//!
//! // Simple query
//! let response = client.query("What are the latest FDA drug approvals in 2026?").await?;
//! println!("{}", response.formatted());
//!
//! // Regulatory research
//! let result = nexcore_perplexity::research::research_regulatory(
//!     &client,
//!     "ICH E2E pharmacovigilance planning",
//!     None,
//!     None,
//! ).await?;
//! println!("{}", result.formatted());
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

pub mod client;
pub mod error;
pub mod grounding;
pub mod research;
pub mod types;

// Re-exports
pub use client::PerplexityClient;
pub use error::{PerplexityError, PerplexityResult};
pub use types::{
    ChatRequest, ChatResponse, Citation, Model, ResearchResult, SearchRecency, SearchResult,
};

/// Prelude for common imports.
pub mod prelude {
    //! Common imports for Perplexity AI API usage.
    pub use crate::client::PerplexityClient;
    pub use crate::error::{PerplexityError, PerplexityResult};
    pub use crate::types::{ChatRequest, ChatResponse, Model, ResearchResult, SearchRecency};
}
