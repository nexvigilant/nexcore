//! Cortex Parameters (Local LLM Inference)
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Model downloading, generation, embeddings, and fine-tuning status.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for downloading a model from HuggingFace Hub.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CortexDownloadParams {
    /// HuggingFace repo ID
    pub repo_id: String,
    /// Filename within the repo
    pub filename: String,
}

/// Parameters for text generation with a local model.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CortexGenerateParams {
    /// The prompt to generate from
    pub prompt: String,
    /// HuggingFace repo ID of the model to use
    pub repo_id: String,
    /// Maximum tokens to generate
    #[serde(default = "default_cortex_max_tokens")]
    pub max_tokens: usize,
    /// Sampling temperature 0.0-1.0
    #[serde(default = "default_cortex_temperature")]
    pub temperature: f64,
}

fn default_cortex_max_tokens() -> usize {
    512
}

fn default_cortex_temperature() -> f64 {
    0.7
}

/// Parameters for listing cached models.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CortexListModelsParams {
    /// Optional filter by repo ID substring
    #[serde(default)]
    pub filter: Option<String>,
}

/// Parameters for getting model info.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CortexModelInfoParams {
    /// HuggingFace repo ID
    pub repo_id: String,
    /// Filename within the repo
    pub filename: String,
}

/// Parameters for generating text embeddings.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CortexEmbedParams {
    /// Text to embed
    pub text: String,
    /// HuggingFace repo ID of the model to use
    pub repo_id: String,
}

/// Parameters for checking fine-tune job status.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CortexFineTuneStatusParams {
    /// Job ID to check
    pub job_id: String,
}
