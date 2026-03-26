//! # NexVigilant Core — Primitive Scanner
//!
//! Automated primitive extraction with T1/T2/T3 tier classification.
//!
//! ## Components
//!
//! - **CLI**: `primitive-scanner scan --domain X --sources *.md`
//! - **Library**: `Scanner::new().scan(domain, sources)`
//! - **MCP Integration**: Via nexcore-mcp primitive_scan/primitive_batch_test
//!
//! ## Tier Classification
//!
//! | Tier | Coverage | Confidence |
//! |------|----------|------------|
//! | T1 | Universal (ALL domains) | 1.0 |
//! | T2-P | Cross-domain primitive (2+ domains) | 0.9 |
//! | T2-C | Cross-domain composite | 0.7 |
//! | T3 | Domain-specific | 0.4-0.6 |

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![warn(missing_docs)]

pub mod extraction;
pub mod flywheel_bridge;
pub mod graph;
pub mod scanner;
pub mod test_card;
pub mod types;

pub use extraction::{ExtractionContext, ExtractionResult};
pub use scanner::Scanner;
pub use test_card::PrimitiveTestCard;
pub use types::{Primitive, PrimitiveTier, TermDefinition};
