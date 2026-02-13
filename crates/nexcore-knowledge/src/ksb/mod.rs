//! KSB (Knowledge-Skills-Behaviors) module.
//!
//! Provides access to 628 PV articles across 15 domains.

mod article;
pub mod index;
mod search;

pub use article::{KsbArticle, KsbDomain, ProficiencyLevel};
pub use index::KsbIndex;
pub use search::{SearchResult, search_articles};
