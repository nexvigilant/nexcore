//! Wolfram Alpha Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Natural language query and computational knowledge engine parameters.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for Wolfram Alpha full query
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframQueryParams {
    /// Natural language query or mathematical expression
    pub query: String,
    /// Preferred unit system for results
    #[serde(default = "default_metric")]
    pub units: String,
    /// Location context for geographic queries (city, country, or coordinates)
    #[serde(default)]
    pub location: Option<String>,
    /// Include images, sources, and available drill-down options
    #[serde(default)]
    pub verbose: bool,
}

fn default_metric() -> String {
    "metric".to_string()
}

/// Parameters for short/spoken answer
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframShortParams {
    /// Question or calculation to answer
    pub query: String,
    /// Preferred unit system
    #[serde(default = "default_metric")]
    pub units: String,
}

/// Parameters for calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframCalculateParams {
    /// Mathematical expression or equation to evaluate
    pub expression: String,
}

/// Parameters for step-by-step solution
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframStepByStepParams {
    /// Math problem to solve with steps shown
    pub problem: String,
}

/// Parameters for plotting
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframPlotParams {
    /// Function or expression to plot
    pub expression: String,
    /// Optional range specification (e.g., 'x from -10 to 10')
    #[serde(default)]
    pub range: Option<String>,
}

/// Parameters for unit conversion
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframConvertParams {
    /// Numeric value to convert
    pub value: f64,
    /// Source unit (e.g., 'miles', 'kg', 'fahrenheit')
    pub from_unit: String,
    /// Target unit (e.g., 'km', 'pounds', 'celsius')
    pub to_unit: String,
}

/// Parameters for chemistry lookup
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframChemistryParams {
    /// Chemical compound name, formula, SMILES, or CAS number
    pub compound: String,
    /// Specific property to look up (e.g., 'boiling point', 'density', 'structure')
    #[serde(default)]
    pub property: Option<String>,
}

/// Parameters for physics query
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframPhysicsParams {
    /// Physics question, constant, or calculation
    pub query: String,
}

/// Parameters for astronomy query
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframAstronomyParams {
    /// Astronomical query
    pub query: String,
    /// Observer location for rise/set times and visibility
    #[serde(default)]
    pub location: Option<String>,
}

/// Parameters for statistics query
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframStatisticsParams {
    /// Statistical query or dataset
    pub query: String,
}

/// Parameters for data lookup
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframDataLookupParams {
    /// Data query
    pub query: String,
}

/// Parameters for query with assumption
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframQueryWithAssumptionParams {
    /// The query to interpret
    pub query: String,
    /// Assumption string from previous query (e.g., '*C.Mercury-_*Planet-')
    pub assumption: String,
}

/// Parameters for filtered query
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframQueryFilteredParams {
    /// Query to send
    pub query: String,
    /// Pod IDs to include (only these will be returned)
    #[serde(default)]
    pub include_pods: Option<Vec<String>>,
    /// Pod IDs to exclude from results
    #[serde(default)]
    pub exclude_pods: Option<Vec<String>>,
}

/// Parameters for image result
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframImageParams {
    /// Query to visualize
    pub query: String,
}

/// Parameters for datetime query
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframDatetimeParams {
    /// Date/time query or calculation
    pub query: String,
    /// Location for time zone context
    #[serde(default)]
    pub location: Option<String>,
}

/// Parameters for nutrition lookup
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframNutritionParams {
    /// Food item or meal to look up
    pub food: String,
    /// Optional quantity (e.g., '100g', '1 cup', '1 serving')
    #[serde(default)]
    pub amount: Option<String>,
}

/// Parameters for finance query
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframFinanceParams {
    /// Financial query or calculation
    pub query: String,
}

/// Parameters for linguistics query
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframLinguisticsParams {
    /// Linguistics query
    pub query: String,
}
