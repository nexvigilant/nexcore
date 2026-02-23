use nexcore_error::Result;
use serde::Deserialize;
use std::path::PathBuf;

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
            .set_default("qdrant_url", "http://localhost:6333")?
            .set_default("ksb_root", "./ksb")?
            .set_default("data_dir", "./data")?
            .set_default("event_bus_size", 1000)?
            .set_default("webhook_port", 8080)?
            .set_default("friday_api_key", "secret-key")?
            .set_default("llm_provider", "claude")?
            .build()?;

        s.try_deserialize().map_err(Into::into)
    }
}
