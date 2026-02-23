//! Parameter structs for Compound Registry MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// Resolve a compound by name (cache → PubChem → ChEMBL pipeline).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompoundResolveParams {
    /// Compound name to resolve (e.g. "aspirin", "ibuprofen")
    pub name: String,
}

/// Batch resolve multiple compounds by name.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompoundResolveBatchParams {
    /// List of compound names to resolve
    pub names: Vec<String>,
}

/// Search cached compounds by partial name match.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompoundCacheSearchParams {
    /// Partial name query (case-insensitive LIKE search)
    pub query: String,
    /// Max results (default 20)
    pub limit: Option<usize>,
}

/// Get a specific compound from cache by exact name.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompoundCacheGetParams {
    /// Exact compound name (case-insensitive)
    pub name: String,
}

/// Count total compounds in the local cache.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompoundCacheCountParams {}
