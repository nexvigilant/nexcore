//! Cognition Parameters (Transformer Algorithm as Strict Rust)
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Process prompts, analyze attention, profile cognitive patterns, compute entropy.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for running the full cognitive pipeline (generate + measure).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CognitionProcessParams {
    /// Token IDs representing the input prompt.
    pub prompt: Vec<usize>,
    /// Maximum number of new tokens to generate.
    #[serde(default = "default_max_tokens")]
    pub max_new_tokens: usize,
    /// Vocabulary size for the model.
    #[serde(default = "default_vocab_size")]
    pub vocab_size: usize,
    /// Model dimension (embedding width).
    #[serde(default = "default_model_dim")]
    pub model_dim: usize,
    /// Number of attention heads.
    #[serde(default = "default_num_heads")]
    pub num_heads: usize,
    /// Number of transformer layers.
    #[serde(default = "default_num_layers")]
    pub num_layers: usize,
    /// FFN inner dimension (default: 4 × model_dim).
    #[serde(default)]
    pub ffn_inner_dim: Option<usize>,
    /// Maximum sequence length.
    #[serde(default = "default_max_seq_len")]
    pub max_seq_len: usize,
    /// Sampling temperature (0.0 = greedy, 1.0 = creative).
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    /// Top-k filtering (0 = disabled).
    #[serde(default)]
    pub top_k: usize,
    /// Top-p nucleus filtering (1.0 = disabled).
    #[serde(default = "default_top_p")]
    pub top_p: f64,
    /// Repetition penalty (1.0 = disabled, >1.0 = penalize repeats).
    #[serde(default = "default_rep_penalty")]
    pub repetition_penalty: f64,
    /// Optional stop token ID.
    #[serde(default)]
    pub stop_token: Option<usize>,
    /// Minimum confidence threshold — halts generation when confidence drops below.
    /// None = disabled (default).
    #[serde(default)]
    pub min_confidence: Option<f64>,
    /// Random seed for reproducibility (optional).
    #[serde(default)]
    pub seed: Option<u64>,
}

/// Parameters for analyzing attention patterns on an existing token sequence.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CognitionAnalyzeParams {
    /// Token IDs to analyze.
    pub tokens: Vec<usize>,
    /// Vocabulary size.
    #[serde(default = "default_vocab_size")]
    pub vocab_size: usize,
    /// Model dimension.
    #[serde(default = "default_model_dim")]
    pub model_dim: usize,
    /// Number of attention heads.
    #[serde(default = "default_num_heads")]
    pub num_heads: usize,
    /// Number of transformer layers.
    #[serde(default = "default_num_layers")]
    pub num_layers: usize,
    /// FFN inner dimension (default: 4 × model_dim).
    #[serde(default)]
    pub ffn_inner_dim: Option<usize>,
    /// Maximum sequence length.
    #[serde(default = "default_max_seq_len")]
    pub max_seq_len: usize,
    /// Use causal (autoregressive) attention. Default: true.
    #[serde(default = "default_causal")]
    pub causal: bool,
    /// Random seed for reproducibility (optional).
    #[serde(default)]
    pub seed: Option<u64>,
}

/// Parameters for computing Shannon entropy of a probability distribution.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CognitionEntropyParams {
    /// Probability distribution (must sum to ~1.0).
    pub probabilities: Vec<f64>,
}

/// Parameters for computing perplexity of a token sequence given logits.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CognitionPerplexityParams {
    /// Token IDs of the sequence.
    pub token_ids: Vec<usize>,
    /// Logit vectors for each step (each inner vec has vocab_size elements).
    pub logits_per_step: Vec<Vec<f64>>,
}

/// Parameters for a single forward pass (returns rich data, not just shapes).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CognitionForwardParams {
    /// Token IDs for the input sequence.
    pub tokens: Vec<usize>,
    /// Vocabulary size.
    #[serde(default = "default_vocab_size")]
    pub vocab_size: usize,
    /// Model dimension.
    #[serde(default = "default_model_dim")]
    pub model_dim: usize,
    /// Number of attention heads.
    #[serde(default = "default_num_heads")]
    pub num_heads: usize,
    /// Number of transformer layers.
    #[serde(default = "default_num_layers")]
    pub num_layers: usize,
    /// FFN inner dimension (default: 4 × model_dim).
    #[serde(default)]
    pub ffn_inner_dim: Option<usize>,
    /// Maximum sequence length.
    #[serde(default = "default_max_seq_len")]
    pub max_seq_len: usize,
    /// Use causal (autoregressive) attention. Default: true.
    #[serde(default = "default_causal")]
    pub causal: bool,
    /// Random seed for reproducibility (optional).
    #[serde(default)]
    pub seed: Option<u64>,
}

/// Parameters for embedding token IDs into vectors.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CognitionEmbedParams {
    /// Token IDs to embed.
    pub tokens: Vec<usize>,
    /// Vocabulary size.
    #[serde(default = "default_vocab_size")]
    pub vocab_size: usize,
    /// Model dimension (embedding width).
    #[serde(default = "default_model_dim")]
    pub model_dim: usize,
    /// Random seed for reproducibility (optional).
    #[serde(default)]
    pub seed: Option<u64>,
}

/// Parameters for sampling a token from logits.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CognitionSampleParams {
    /// Logit vector (raw scores, one per vocab token).
    pub logits: Vec<f64>,
    /// Sampling temperature (0.0 = greedy).
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    /// Top-k filtering (0 = disabled).
    #[serde(default)]
    pub top_k: usize,
    /// Top-p nucleus filtering (1.0 = disabled).
    #[serde(default = "default_top_p")]
    pub top_p: f64,
    /// Repetition penalty (1.0 = disabled).
    #[serde(default = "default_rep_penalty")]
    pub repetition_penalty: f64,
    /// Context tokens for repetition penalty.
    #[serde(default)]
    pub context: Vec<usize>,
    /// Random seed (optional).
    #[serde(default)]
    pub seed: Option<u64>,
}

/// Parameters for computing generation confidence from logits.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CognitionConfidenceParams {
    /// Logit vector (raw scores).
    pub logits: Vec<f64>,
}

fn default_max_tokens() -> usize {
    10
}

fn default_vocab_size() -> usize {
    256
}

fn default_model_dim() -> usize {
    64
}

fn default_num_heads() -> usize {
    4
}

fn default_num_layers() -> usize {
    2
}

fn default_max_seq_len() -> usize {
    128
}

fn default_temperature() -> f64 {
    0.7
}

fn default_top_p() -> f64 {
    1.0
}

fn default_rep_penalty() -> f64 {
    1.0
}

fn default_causal() -> bool {
    true
}
