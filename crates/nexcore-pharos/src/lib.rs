//! # PHAROS — Pharmacovigilance Autonomous Reconnaissance and Observation System
//!
//! Continuous autonomous signal surveillance pipeline that chains:
//! FAERS ETL → Signal Detection → Threshold Filtering → Guardian Injection →
//! Cytokine Emission → Qdrant Embedding
//!
//! ## Primitive Composition
//! σ(Sequence) + ν(Frequency) + →(Causality) + ∂(Boundary) + π(Persistence)
//!
//! ## Architecture
//! ```text
//! ┌─────────────┐    ┌──────────────┐    ┌────────────┐
//! │  FAERS ETL   │───>│ Signal Detect │───>│  Threshold  │
//! │  (ingest)    │    │  (PRR/ROR/   │    │  Filter     │
//! │              │    │   IC/EBGM)   │    │  (∂ gate)   │
//! └─────────────┘    └──────────────┘    └─────┬──────┘
//!                                              │
//!                    ┌──────────────┐    ┌─────▼──────┐
//!                    │   Cytokine    │<───│  Guardian   │
//!                    │   Emission    │    │  Injection  │
//!                    │  (IFN/CSF)   │    │  (signal)   │
//!                    └──────┬───────┘    └────────────┘
//!                           │
//!                    ┌──────▼───────┐
//!                    │   Persist    │
//!                    │  (Parquet +  │
//!                    │   report)    │
//!                    └─────────────┘
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod config;
pub mod guardian;
pub mod pipeline;
pub mod report;
pub mod thresholds;

pub use config::PharosConfig;
pub use guardian::{to_threat_signal, to_threat_signals};
pub use pipeline::{ActionableSignal, PharosOutput, PharosPipeline};
pub use report::SurveillanceReport;
pub use thresholds::SignalThresholds;
