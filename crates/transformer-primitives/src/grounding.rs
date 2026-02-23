//! T1 primitive grounding for the transformer attention pipeline.
//!
//! This crate is function-oriented (operates on polars `LazyFrame`s) rather than
//! type-oriented, so GroundsTo impls attach to a pipeline description marker.
//!
//! | Stage | Primitives | Rationale |
//! |-------|-----------|-----------|
//! | positional_encoding | σ (sequence) + ν (frequency) | Sinusoidal position → frequency encoding |
//! | qkv_projection | μ (mapping) | Embedding → Q,K,V subspace mapping |
//! | attention_scores | κ (comparison) + N (quantity) | Scaled dot-product similarity |
//! | softmax_normalize | ∂ (boundary) + Σ (sum) | Normalisation boundary, partition of unity |
//! | weighted_values | Σ (sum) + μ (mapping) | Weighted aggregation mapping |
//! | residual_connection | π (persistence) | Skip connection preserves original signal |
//! | feed_forward | → (causality) + ∂ (boundary) | ReLU threshold gate, causal transform |
//! | sink_prediction | κ (comparison) | Argmax selection by comparison |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

/// Marker type representing the full 8-stage transformer pipeline.
///
/// Grounds to σ+μ+κ — the pipeline is fundamentally a sequential mapping
/// that produces comparison scores (attention weights).
pub struct TransformerPipeline;

impl GroundsTo for TransformerPipeline {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.7)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_grounds_to_sequence() {
        assert_eq!(
            TransformerPipeline::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }
}
