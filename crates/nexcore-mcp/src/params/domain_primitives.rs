//! Domain primitives analysis params
use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesListParams {
    /// Optional taxonomy filter
    #[serde(default)]
    pub taxonomy: Option<String>,
    /// Optional tier filter
    #[serde(default)]
    pub tier: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesTransferParams {
    /// Optional taxonomy filter
    #[serde(default)]
    pub taxonomy: Option<String>,
    /// Primitive name filter
    #[serde(default)]
    pub primitive_name: Option<String>,
    /// Target domain filter
    #[serde(default)]
    pub target_domain: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesDecomposeParams {
    /// Optional taxonomy filter
    #[serde(default)]
    pub taxonomy: Option<String>,
    /// Primitive name to decompose
    pub primitive_name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesBottlenecksParams {
    /// Optional taxonomy filter
    #[serde(default)]
    pub taxonomy: Option<String>,
    /// Maximum results
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesCompareParams {
    /// First taxonomy
    #[serde(default)]
    pub taxonomy_a: Option<String>,
    /// Second taxonomy
    #[serde(default)]
    pub taxonomy_b: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesCriticalPathsParams {
    /// Optional taxonomy filter
    #[serde(default)]
    pub taxonomy: Option<String>,
    /// Maximum results
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesRegistryParams {
    /// Optional domain filter
    #[serde(default)]
    pub domain: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesSaveParams {
    /// File path to save
    pub path: String,
    /// Taxonomy name to save
    pub taxonomy: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesLoadParams {
    /// File path to load
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesTopoSortParams {
    /// Optional taxonomy filter
    #[serde(default)]
    pub taxonomy: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesTransferMatrixParams {
    /// Domains to include
    #[serde(default)]
    pub domains: Option<Vec<String>>,
    /// Maximum results
    #[serde(default)]
    pub limit: Option<usize>,
}
