use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    pub vocab_size: usize,
    pub seq_length: usize,
    pub batch_size: usize,
    pub hidden_dim: usize,
    pub num_layers: usize,
    pub num_heads: usize,
    pub ff_dim: usize,
    pub num_epochs: usize,
    pub learning_rate: f32,
    pub warmup_steps: usize,
    pub max_grad_norm: f32,
    pub log_interval: usize,
    pub max_batches_per_epoch: usize,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            vocab_size: 9, // DNA char-level: PAD,UNK,CLS,SEP,MASK,A,T,G,C
            seq_length: 64,
            batch_size: 4,
            hidden_dim: 128,
            num_layers: 2,
            num_heads: 4,
            ff_dim: 512,
            num_epochs: 10,
            learning_rate: 1e-4,
            warmup_steps: 10,
            max_grad_norm: 1.0,
            log_interval: 1,
            max_batches_per_epoch: 100,
        }
    }
}

impl RunConfig {
    pub fn load_or_default(path: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(config_path) = path {
            let path = Path::new(config_path);
            if path.exists() {
                let content = fs::read_to_string(path)?;
                let cfg = serde_json::from_str::<RunConfig>(&content)?;
                return Ok(cfg);
            }
        }
        Ok(Self::default())
    }
}
