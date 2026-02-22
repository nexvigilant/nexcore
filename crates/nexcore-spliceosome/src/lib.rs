// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # nexcore-spliceosome — Pre-Translation Structural Expectation Generator
//!
//! ## Biology Analog
//!
//! In biology, the spliceosome processes pre-mRNA by removing introns and
//! depositing Exon Junction Complexes (EJCs) at exon-exon junctions.
//! These EJC markers are later consumed by the Nonsense-Mediated mRNA Decay
//! (NMD) surveillance system to detect translation errors.
//!
//! ## Purpose
//!
//! This crate generates structural expectations for LLM pipeline execution.
//! Given a task specification, it produces EJC markers encoding expected
//! phases, tool categories, grounding requirements, and checkpoint intervals.
//!
//! The spliceosome is **cognitively independent** from the pipeline it monitors.
//! It never sees pipeline output — only task specifications. This orthogonality
//! breaks the circularity of asking an LLM to detect its own hallucinations.
//!
//! ## Layer: Foundation (0-3 internal deps)
//!
//! ## Primitive Grounding: sigma(Sequence) + boundary(d) + mapping(mu)
//!
//! ## Usage
//!
//! ```rust
//! use nexcore_spliceosome::Spliceosome;
//!
//! let spliceosome = Spliceosome::new();
//! let expectation = spliceosome.splice("implement a new REST endpoint").unwrap();
//! println!("Category: {:?}", expectation.task_category);
//! println!("Markers: {}", expectation.markers.len());
//! ```

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

pub mod classifier;
pub mod engine;
pub mod error;
pub mod templates;
pub mod types;

// Re-exports for ergonomic API
pub use classifier::TaskClassifier;
pub use engine::Spliceosome;
pub use error::{Result, SpliceosomeError};
pub use types::{EjcMarker, TaskCategory, TranscriptExpectation};
