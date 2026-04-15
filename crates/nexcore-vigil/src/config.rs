use nexcore_error::{NexError, Result};
use serde::Deserialize;
use std::path::PathBuf;

/// Convert config errors to NexError.
fn cfg_err(e: config::ConfigError) -> NexError {
    NexError::new(e.to_string())
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub anthropic_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
    #[serde(default = "default_llm_provider")]
    pub llm_provider: String,
    pub qdrant_url: String,
    pub ksb_root: PathBuf,
    pub data_dir: PathBuf,
    pub event_bus_size: usize,
    pub webhook_port: u16,
    pub friday_api_key: String,
}

fn default_llm_provider() -> String {
    "claude".to_string()
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let s = config::Config::builder()
            .add_source(config::Environment::default())
            .set_default("qdrant_url", "http://localhost:6333")
            .map_err(cfg_err)?
            .set_default("ksb_root", "./ksb")
            .map_err(cfg_err)?
            .set_default("data_dir", "./data")
            .map_err(cfg_err)?
            .set_default("event_bus_size", 1000)
            .map_err(cfg_err)?
            .set_default("webhook_port", 8080)
            .map_err(cfg_err)?
            .set_default("friday_api_key", "secret-key")
            .map_err(cfg_err)?
            .set_default("llm_provider", "claude")
            .map_err(cfg_err)?
            .build()
            .map_err(cfg_err)?;

        s.try_deserialize().map_err(cfg_err)
    }
}
