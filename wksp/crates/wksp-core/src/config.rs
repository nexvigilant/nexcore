use serde::{ Deserialize, Serialize };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WkspConfig {
    pub api_base_url: String,
    pub environment: String,
    pub debug_mode: bool,
}

impl Default for WkspConfig {
    fn default() -> Self {
        Self {
            api_base_url: "http://localhost:3030".to_string(),
            environment: "development".to_string(),
            debug_mode: true,
        }
    }
}
