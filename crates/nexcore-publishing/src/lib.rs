//! # nexcore-publishing
//!
//! Book publishing pipeline for NexVigilant. Converts `.docx` manuscripts
//! to publication-ready EPUB 3.0 and Kindle-compliant ebooks.
//!
//! ## Pipeline
//!
//! Seven stages: **INGEST → METADATA → STRUCTURE → STYLE → COVER → GENERATE → VALIDATE**
//!
//! ```text
//! .docx → DocxDocument → BookMetadata + Chapters → EPUB 3.0 → Validated .epub
//!                                                      ↓
//!                                               KDP Compliance Report
//! ```
//!
//! ## Quick Start
//!
//! ```no_run
//! use nexcore_publishing::pipeline::{PipelineConfig, PublishingTarget};
//!
//! let config = PipelineConfig::new("manuscript.docx", "./output")
//!     .with_target(PublishingTarget::Kindle);
//!
//! let result = nexcore_publishing::pipeline::run(&config)?;
//! println!("{}", result.to_report());
//! # Ok::<(), nexcore_publishing::error::PublishingError>(())
//! ```
//!
//! ## Modules
//!
//! - [`docx`] — DOCX reader (ZIP + XML, no external deps)
//! - [`epub`] — EPUB 3.0 writer with NCX backward compatibility
//! - [`metadata`] — Dublin Core metadata with ISBN validation
//! - [`chapter`] — Chapter extraction and heading detection
//! - [`cover`] — Cover image validation (EPUB + KDP requirements)
//! - [`kindle`] — KDP compliance checking
//! - [`validate`] — EPUB structural validation
//! - [`pipeline`] — Orchestrates all stages end-to-end

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

pub mod chapter;
pub mod cover;
pub mod docx;
pub mod epub;
pub mod error;
pub mod kindle;
pub mod metadata;
pub mod pipeline;
pub mod read;
pub mod validate;
