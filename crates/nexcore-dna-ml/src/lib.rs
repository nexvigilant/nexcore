//! # NexVigilant Core — DNA-ML
//!
//! Combines DNA encoding (`nexcore-dna`) with ML signal detection
//! (`nexcore-ml-pipeline`) into a unified pipeline.
//!
//! ## Pipeline
//!
//! 1. **Extract** — FAERS contingency data → 12-element PV feature vector
//! 2. **Encode** — Feature vector → quantized bytes → DNA strand
//! 3. **Augment** — DNA similarity metrics (hamming, GC content, LCS) → 5 extra features
//! 4. **Train** — Random forest on augmented 17-dim feature space
//! 5. **Predict** — Score new drug-event pairs with DNA-aware model
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Mapping (μ) | features → DNA → similarity → prediction |
//! | T1: Sequence (σ) | DNA strand ordering, pipeline stages |
//! | T1: Comparison (κ) | Hamming distance, GC divergence, LCS ratio |
//! | T1: Recursion (ρ) | Forest ensemble, tree traversal |
//! | T1: Boundary (∂) | Quantization bins, signal thresholds |

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod encode;
pub mod pipeline;
pub mod similarity;

/// Convenience prelude.
pub mod prelude {
    pub use crate::encode::{
        compute_bounds, decode_strand, dequantize, encode_features, quantize_features,
    };
    pub use crate::pipeline::{DnaMlConfig, DnaMlResult, augment_with_dna, run};
    pub use crate::similarity::{DnaSimilarity, compute_similarity, mean_similarity_features};
}
