//! # NexVigilant Core — Knowledge
//!
//! KSB (Knowledge-Skills-Behaviors) module for nexcore.
//!
//! Provides access to 628 PV articles across 15 domains:
//! - D01: Foundations of Pharmacovigilance
//! - D02: Regulatory Framework
//! - D03: Data Collection and Management
//! - D04: Signal Detection and Evaluation
//! - D05: Risk Management
//! - D06: Clinical Trials Safety
//! - D07: Benefit-Risk Assessment
//! - D08: Communication and Reporting
//! - D09: Quality Systems
//! - D10: Technology and Innovation
//! - D11: Specialized Populations
//! - D12: Global Pharmacovigilance
//! - D13: Emerging Topics
//! - D14: Leadership and Management
//! - D15: Integration and Application
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexcore_knowledge::{KsbIndex, KsbDomain, search_articles};
//!
//! // Build index from default path
//! let path = default_ksb_path().ok_or("No path")?;
//! let index = KsbIndex::scan(&path)?;
//!
//! // Get article by ID
//! let article = index.get("KSB-D01-K0001");
//!
//! // Search articles
//! let results = search_articles(&index, "signal detection", None, 10);
//!
//! // Get domain counts
//! let counts = index.domain_counts();
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

mod error;
pub mod grounding;
pub mod ksb;

pub use error::KnowledgeError;
pub use ksb::index::default_ksb_path;
pub use ksb::{KsbArticle, KsbDomain, KsbIndex, ProficiencyLevel, SearchResult, search_articles};

/// Returns the NexCore Knowledge version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
