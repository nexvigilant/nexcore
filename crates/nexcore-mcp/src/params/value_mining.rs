//! Value mining / signal detection params
use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValueSignalTypesParams {
    /// Optional filter by type name
    #[serde(default)]
    pub type_filter: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValueSignalDetectParams {
    /// Signal type: sentiment, trend, engagement, virality, controversy
    pub signal_type: String,
    /// Observed value
    pub observed: f64,
    /// Baseline value
    pub baseline: f64,
    /// Sample size
    pub sample_size: usize,
    /// Entity name
    pub entity: String,
    /// Source name
    pub source: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValueBaselineCreateParams {
    /// Source name
    pub source: String,
    /// Positive sentiment rate
    pub positive_rate: f64,
    /// Negative sentiment rate
    pub negative_rate: f64,
    /// Average engagement
    pub avg_engagement: f64,
    /// Posts per hour
    pub posts_per_hour: f64,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValuePvMappingParams {
    /// Optional signal type filter
    #[serde(default)]
    pub signal_type: Option<String>,
}
