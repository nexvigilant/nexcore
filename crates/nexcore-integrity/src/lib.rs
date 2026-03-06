//! # NexVigilant Core — integrity
//!
//! KSB assessment integrity via AI text detection.
//!
//! Detects AI-generated text through 5 statistical features aggregated via
//! chemistry primitives (Beer-Lambert + Hill + Arrhenius), with Bloom-level
//! threshold adaptation for PV education assessment.
//!
//! ## Pipeline
//!
//! 1. **Tokenize** (σ): text → ordered token stream with TTR
//! 2. **Zipf** (κ): log-log regression deviation from Zipf's law
//! 3. **Entropy** (Σ): sliding window Shannon entropy variance
//! 4. **Burstiness** (ν): inter-arrival clustering coefficient
//! 5. **Perplexity** (ν): per-sentence entropy variance
//! 6. **Aggregation** (Σ + ρ): Beer-Lambert + Hill → composite
//! 7. **Classify** (∂ + →): Arrhenius gate → verdict
//!
//! ## Bloom-Level Adaptation
//!
//! Higher Bloom levels (analysis, synthesis, evaluation) require more
//! original thought — lower detection thresholds flag AI assistance
//! at higher cognitive levels.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod aggregation;
pub mod assessment;
pub mod bloom;
pub mod burstiness;
pub mod chemistry;
pub mod classify;
pub mod entropy;
pub mod error;
#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;
pub mod grounding;
pub mod perplexity;
pub mod profile;
pub mod tokenize;
pub mod zipf;

// Public API re-exports
pub use assessment::{AssessmentContext, IntegrityAssessment, assess_ksb_response, assess_text};
pub use bloom::BloomThresholds;
pub use classify::{Classification, Verdict};
pub use error::IntegrityError;
pub use profile::CalibrationProfile;
