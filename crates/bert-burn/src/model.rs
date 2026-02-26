use burn::module::Module;
use burn::nn::Gelu;
use burn::nn::{
    Dropout, DropoutConfig, Embedding, EmbeddingConfig, LayerNorm, LayerNormConfig, Linear,
    LinearConfig,
    attention::{MhaInput, MultiHeadAttention, MultiHeadAttentionConfig},
};
use burn::tensor::{Bool, Int, Tensor, backend::Backend};

/// Transformer encoder block with multi-head self-attention
#[derive(Module, Debug)]
pub struct TransformerBlock<B: Backend> {
    self_attn: MultiHeadAttention<B>,
    attn_norm: LayerNorm<B>,
    attn_dropout: Dropout,
    ff_linear1: Linear<B>,
    ff_linear2: Linear<B>,
    ff_norm: LayerNorm<B>,
    ff_dropout: Dropout,
    gelu: Gelu,
}

impl<B: Backend> TransformerBlock<B> {
    pub fn new(hidden_dim: usize, num_heads: usize, ff_dim: usize, dropout: f64) -> Self {
        let device = B::Device::default();

        Self {
            self_attn: MultiHeadAttentionConfig::new(hidden_dim, num_heads)
                .with_dropout(dropout)
                .init(&device),
            attn_norm: LayerNormConfig::new(hidden_dim).init(&device),
            attn_dropout: DropoutConfig::new(dropout).init(),
            ff_linear1: LinearConfig::new(hidden_dim, ff_dim).init(&device),
            ff_linear2: LinearConfig::new(ff_dim, hidden_dim).init(&device),
            ff_norm: LayerNormConfig::new(hidden_dim).init(&device),
            ff_dropout: DropoutConfig::new(dropout).init(),
            gelu: Gelu::new(),
        }
    }

    pub fn forward(
        &self,
        x: Tensor<B, 3>,
        mask_pad: Option<Tensor<B, 2, Bool>>,
        training: bool,
    ) -> Tensor<B, 3> {
        // Multi-head self-attention with residual + padding mask
        let attn_input = match mask_pad {
            Some(mask) => MhaInput::self_attn(x.clone()).mask_pad(mask),
            None => MhaInput::self_attn(x.clone()),
        };
        let attn_out = self.self_attn.forward(attn_input).context;
        let attn_out = if training {
            self.attn_dropout.forward(attn_out)
        } else {
            attn_out
        };
        let x = self.attn_norm.forward(x + attn_out);

        // Feed-forward with GELU activation and residual
        let ff_out = self.ff_linear1.forward(x.clone());
        let ff_out = self.gelu.forward(ff_out);
        let ff_out = self.ff_linear2.forward(ff_out);
        let ff_out = if training {
            self.ff_dropout.forward(ff_out)
        } else {
            ff_out
        };
        self.ff_norm.forward(x + ff_out)
    }
}

/// BERT model — transformer-based language model for DNA sequences
#[derive(Module, Debug)]
pub struct BertModel<B: Backend> {
    token_embedding: Embedding<B>,
    position_embedding: Embedding<B>,
    transformer_blocks: Vec<TransformerBlock<B>>,
    embedding_norm: LayerNorm<B>,
    embedding_dropout: Dropout,
    mlm_head: Linear<B>,
    mlm_norm: LayerNorm<B>,
    #[allow(
        dead_code,
        reason = "carried for checkpoint introspection and future config extraction"
    )]
    hidden_dim: usize,
    #[allow(
        dead_code,
        reason = "carried for checkpoint introspection and future config extraction"
    )]
    vocab_size: usize,
}

impl<B: Backend> BertModel<B> {
    pub fn new(
        vocab_size: usize,
        seq_length: usize,
        hidden_dim: usize,
        num_layers: usize,
        num_heads: usize,
        ff_dim: usize,
    ) -> Self {
        let device = B::Device::default();
        let dropout = 0.1;

        let mut transformer_blocks = Vec::new();
        for _ in 0..num_layers {
            transformer_blocks.push(TransformerBlock::new(
                hidden_dim, num_heads, ff_dim, dropout,
            ));
        }

        Self {
            token_embedding: EmbeddingConfig::new(vocab_size, hidden_dim).init(&device),
            position_embedding: EmbeddingConfig::new(seq_length, hidden_dim).init(&device),
            transformer_blocks,
            embedding_norm: LayerNormConfig::new(hidden_dim).init(&device),
            embedding_dropout: DropoutConfig::new(dropout).init(),
            mlm_head: LinearConfig::new(hidden_dim, vocab_size).init(&device),
            mlm_norm: LayerNormConfig::new(hidden_dim).init(&device),
            hidden_dim,
            vocab_size,
        }
    }

    pub fn forward(
        &self,
        input_ids: Tensor<B, 2, Int>,
        mask_pad: Option<Tensor<B, 2, Bool>>,
        training: bool,
    ) -> Tensor<B, 3> {
        let [batch_size, seq_len] = input_ids.dims();
        let device = input_ids.device();

        // Token embeddings
        let token_embeds = self.token_embedding.forward(input_ids);

        // Position IDs: [0, 1, 2, ..., seq_len-1] for each batch element
        let pos_data: Vec<i64> = (0..seq_len as i64).collect();
        let pos_row = Tensor::<B, 1, Int>::from_data(
            burn::tensor::TensorData::from(pos_data.as_slice()),
            &device,
        );
        let position_ids = pos_row.unsqueeze::<2>().expand([batch_size, seq_len]);
        let position_embeds = self.position_embedding.forward(position_ids);

        // Combine embeddings
        let embeddings = token_embeds + position_embeds;
        let embeddings = self.embedding_norm.forward(embeddings);
        let embeddings = if training {
            self.embedding_dropout.forward(embeddings)
        } else {
            embeddings
        };

        // Transformer blocks (pass padding mask to each block's attention)
        let mut hidden = embeddings;
        for block in &self.transformer_blocks {
            hidden = block.forward(hidden, mask_pad.clone(), training);
        }

        // MLM prediction head
        let mlm_logits = self.mlm_norm.forward(hidden);
        self.mlm_head.forward(mlm_logits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::ndarray::NdArray;
    use burn::tensor::TensorData;

    type TestBackend = NdArray;

    #[test]
    fn test_model_forward_shape() {
        let model = BertModel::<TestBackend>::new(9, 32, 64, 2, 4, 256);
        let device = <TestBackend as Backend>::Device::default();
        let data: Vec<i64> = vec![
            2, 5, 6, 7, 8, 5, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        let input =
            Tensor::<TestBackend, 1, Int>::from_data(TensorData::from(data.as_slice()), &device)
                .reshape([1, 32]);

        let logits = model.forward(input, None, false);
        let [batch, seq, vocab] = logits.dims();
        assert_eq!(batch, 1);
        assert_eq!(seq, 32);
        assert_eq!(vocab, 9);
    }

    #[test]
    fn test_model_batch_forward() {
        let model = BertModel::<TestBackend>::new(9, 16, 32, 1, 2, 128);
        let device = <TestBackend as Backend>::Device::default();
        let data: Vec<i64> = vec![
            2, 5, 6, 7, 8, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 8, 7, 6, 5, 3, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        let input =
            Tensor::<TestBackend, 1, Int>::from_data(TensorData::from(data.as_slice()), &device)
                .reshape([2, 16]);

        let logits = model.forward(input, None, false);
        let [batch, seq, vocab] = logits.dims();
        assert_eq!(batch, 2);
        assert_eq!(seq, 16);
        assert_eq!(vocab, 9);
    }

    #[test]
    fn test_transformer_block_preserves_shape() {
        let block = TransformerBlock::<TestBackend>::new(64, 4, 256, 0.1);
        let device = <TestBackend as Backend>::Device::default();
        let x = Tensor::<TestBackend, 3>::zeros([2, 16, 64], &device);
        let out = block.forward(x, None, false);
        assert_eq!(out.dims(), [2, 16, 64]);
    }

    /// Verify forward pass with training=true activates dropout without changing output shape.
    /// This exercises the dropout branches that are dead in inference-only tests.
    #[test]
    fn test_model_training_forward_shape() {
        let model = BertModel::<TestBackend>::new(9, 32, 64, 2, 4, 256);
        let device = <TestBackend as Backend>::Device::default();
        let data: Vec<i64> = vec![
            2, 5, 6, 7, 8, 5, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        let input =
            Tensor::<TestBackend, 1, Int>::from_data(TensorData::from(data.as_slice()), &device)
                .reshape([1, 32]);

        // training=true activates embedding dropout and all layer dropouts
        let logits = model.forward(input, None, true);
        let [batch, seq, vocab] = logits.dims();
        assert_eq!(batch, 1);
        assert_eq!(seq, 32);
        assert_eq!(vocab, 9);
    }

    /// Verify output values are finite (not NaN/Inf) for a well-formed input.
    /// Shape-only tests miss degenerate numerical states.
    #[test]
    fn test_model_output_is_finite() {
        let model = BertModel::<TestBackend>::new(9, 16, 32, 1, 2, 128);
        let device = <TestBackend as Backend>::Device::default();
        let data: Vec<i64> = vec![2, 5, 6, 7, 8, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let input =
            Tensor::<TestBackend, 1, Int>::from_data(TensorData::from(data.as_slice()), &device)
                .reshape([1, 16]);

        let logits = model.forward(input, None, false);
        let values = logits.into_data().to_vec::<f32>().unwrap_or_default();
        assert!(!values.is_empty(), "logits should not be empty");
        assert!(
            values.iter().all(|v| v.is_finite()),
            "all logit values must be finite (no NaN or Inf)"
        );
    }

    /// Verify forward pass with padding mask correctly shapes output.
    /// Exercises the attention masking path that prevents padding tokens
    /// from attending to real tokens.
    #[test]
    fn test_model_forward_with_padding_mask() {
        let model = BertModel::<TestBackend>::new(9, 16, 32, 1, 2, 128);
        let device = <TestBackend as Backend>::Device::default();
        // [CLS] A T G C [SEP] [PAD]*10
        let data: Vec<i64> = vec![2, 5, 6, 7, 8, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let input =
            Tensor::<TestBackend, 1, Int>::from_data(TensorData::from(data.as_slice()), &device)
                .reshape([1, 16]);

        // Build padding mask: 1.0 for real tokens, 0.0 for PAD
        let mask_data: Vec<f32> = vec![
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];
        let mask_float =
            Tensor::<TestBackend, 1>::from_data(TensorData::from(mask_data.as_slice()), &device)
                .reshape([1, 16]);
        let pad_mask = mask_float.equal_elem(0.0); // true = padding

        let logits = model.forward(input, Some(pad_mask), false);
        let [batch, seq, vocab] = logits.dims();
        assert_eq!(batch, 1);
        assert_eq!(seq, 16);
        assert_eq!(vocab, 9);

        // Output should still be finite with masking active
        let values = logits.into_data().to_vec::<f32>().unwrap_or_default();
        assert!(
            values.iter().all(|v| v.is_finite()),
            "logits must be finite with padding mask applied"
        );
    }
}
