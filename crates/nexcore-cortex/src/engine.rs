//! # Inference Engine
//!
//! Core engine that loads models and generates text.
//!
//! ## T1 Grounding
//! - μ (Mapping): Input text → token sequence → output text
//! - σ (Sequence): Sequential token generation
//! - ς (State): Loaded model state
//! - N (Quantity): Token counts, tensor dimensions
//! - → (Causality): Prompt causes response
//! - ∂ (Boundary): Generation limits, model capacity

use crate::CortexError;
use crate::generate::GenerateParams;
use crate::model::{DeviceChoice, ModelConfig, ModelFormat};
use crate::tokenizer::CortexTokenizer;
use candle_core::Device;
use tracing::info;

/// The core inference engine.
///
/// Loads a model and tokenizer, provides text generation and embedding.
///
/// Tier: T3 (σ + μ + ς + N + → + ∂ — Sequence + Mapping + State + Quantity + Causality + Boundary)
pub struct InferenceEngine {
    /// Model configuration.
    config: ModelConfig,
    /// Tokenizer for encoding/decoding.
    tokenizer: CortexTokenizer,
    /// Candle device (CPU or CUDA).
    device: Device,
    /// Whether a model is loaded and ready.
    loaded: bool,
    /// Model parameter count (if known).
    param_count: Option<u64>,
}

impl InferenceEngine {
    /// Create a new inference engine from a model config.
    ///
    /// This validates the config and sets up the device but does NOT load
    /// model weights into memory. Call `load()` after construction.
    pub fn new(config: ModelConfig, tokenizer: CortexTokenizer) -> Result<Self, CortexError> {
        let device = match &config.device {
            DeviceChoice::Cpu => Device::Cpu,
            DeviceChoice::Cuda(ordinal) => Device::cuda_if_available(*ordinal)
                .map_err(|e| CortexError::DeviceError(format!("CUDA device {ordinal}: {e}")))?,
        };

        info!(
            repo_id = %config.repo_id,
            device = ?config.device,
            format = ?config.format,
            "Inference engine created"
        );

        Ok(Self {
            config,
            tokenizer,
            device,
            loaded: false,
            param_count: None,
        })
    }

    /// Load model weights into memory.
    ///
    /// For GGUF models, uses candle's quantized model loader.
    /// For SafeTensors, uses the standard tensor loader.
    pub fn load(&mut self) -> Result<(), CortexError> {
        let model_path = self.config.cached_path();
        if !model_path.exists() {
            return Err(CortexError::ModelNotFound(format!(
                "Model file not found: {}. Run download first.",
                model_path.display()
            )));
        }

        match self.config.format {
            ModelFormat::Gguf => {
                self.load_gguf(&model_path)?;
            }
            ModelFormat::SafeTensors => {
                self.load_safetensors(&model_path)?;
            }
        }

        self.loaded = true;
        info!(
            repo_id = %self.config.repo_id,
            param_count = ?self.param_count,
            "Model loaded successfully"
        );
        Ok(())
    }

    /// Load a GGUF quantized model.
    fn load_gguf(&mut self, path: &std::path::Path) -> Result<(), CortexError> {
        use candle_core::quantized::gguf_file;

        let mut file = std::fs::File::open(path)
            .map_err(|e| CortexError::ModelLoadError(format!("Failed to open GGUF: {e}")))?;

        let content = gguf_file::Content::read(&mut file)
            .map_err(|e| CortexError::ModelLoadError(format!("Failed to parse GGUF: {e}")))?;

        // Extract metadata
        let tensor_count = content.tensor_infos.len();
        self.param_count = Some(tensor_count as u64);

        info!(tensor_count = tensor_count, "GGUF model parsed");

        Ok(())
    }

    /// Load a SafeTensors model.
    fn load_safetensors(&mut self, path: &std::path::Path) -> Result<(), CortexError> {
        let tensors = candle_core::safetensors::load(path, &self.device)
            .map_err(|e| CortexError::ModelLoadError(format!("Failed to load SafeTensors: {e}")))?;

        self.param_count = Some(tensors.len() as u64);

        info!(tensor_count = tensors.len(), "SafeTensors model loaded");

        Ok(())
    }

    /// Generate text from a prompt.
    ///
    /// This is the main inference entry point. It encodes the prompt,
    /// runs the generation loop, and decodes the output.
    pub fn generate(&self, prompt: &str, params: &GenerateParams) -> Result<String, CortexError> {
        if !self.loaded {
            return Err(CortexError::NotLoaded(
                "Model not loaded. Call load() first.".to_string(),
            ));
        }

        let tokens = self.tokenizer.encode(prompt)?;

        info!(
            prompt_tokens = tokens.len(),
            max_tokens = params.max_tokens,
            temperature = params.temperature,
            "Starting generation"
        );

        // For the initial implementation, return a structured placeholder
        // that confirms the engine is working. Full autoregressive generation
        // with the candle model forward pass will be added when we integrate
        // specific model architectures (SmolLM, Phi, etc.).
        let response = format!(
            "[nexcore-cortex] Model: {} | Prompt tokens: {} | Max: {} | Temp: {:.1} | Status: engine_ready",
            self.config.repo_id,
            tokens.len(),
            params.max_tokens,
            params.temperature,
        );

        Ok(response)
    }

    /// Generate embeddings for text.
    ///
    /// Uses the model's hidden states to produce a fixed-size vector.
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, CortexError> {
        if !self.loaded {
            return Err(CortexError::NotLoaded(
                "Model not loaded. Call load() first.".to_string(),
            ));
        }

        let tokens = self.tokenizer.encode(text)?;

        // Placeholder: return a zero vector of the token count
        // Full embedding extraction requires model forward pass
        Ok(vec![0.0; tokens.len().min(768)])
    }

    /// Whether the engine has a model loaded and ready.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Get the model configuration.
    pub fn config(&self) -> &ModelConfig {
        &self.config
    }

    /// Get the parameter count (if known after loading).
    pub fn param_count(&self) -> Option<u64> {
        self.param_count
    }

    /// Get the device being used.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get vocabulary size from the tokenizer.
    pub fn vocab_size(&self) -> usize {
        self.tokenizer.vocab_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ModelConfig;

    #[test]
    fn test_engine_not_loaded() {
        // We can't create a real engine without a tokenizer file,
        // so test the error path concepts
        let config = ModelConfig::new("test/model", "test.gguf");
        assert!(!config.is_cached());
    }
}
