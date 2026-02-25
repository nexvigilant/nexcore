//! DataFrame MCP tool parameters.
//!
//! Sovereign DataFrame engine exposure — Phase 5 of Directive 006A.
//! Every tool accepts either inline `data` (JSON array) or `path` (file path).

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

// =============================================================================
// Shared input: data OR path
// =============================================================================

/// A filter condition for query operations.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DataframeFilterCondition {
    /// Column name to filter on.
    pub column: String,
    /// Comparison operator: "eq", "ne", "gt", "ge", "lt", "le", "contains".
    pub op: String,
    /// Value to compare against (number, string, or boolean).
    pub value: serde_json::Value,
}

/// A column definition for constructing DataFrames.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DataframeColumnDef {
    /// Column name.
    pub name: String,
    /// Data type: "bool", "i64", "u64", "f64", "string".
    pub dtype: String,
    /// Array of values (must match dtype).
    pub values: Vec<serde_json::Value>,
}

/// Aggregation specification.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DataframeAggSpec {
    /// Aggregation function: "sum", "mean", "min", "max", "count", "first", "last", "n_unique".
    pub func: String,
    /// Column to aggregate (not needed for "count").
    #[serde(default)]
    pub column: Option<String>,
}

/// Rename specification.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DataframeRename {
    /// Current column name.
    pub from: String,
    /// New column name.
    pub to: String,
}

// =============================================================================
// Tool params
// =============================================================================

/// Describe a DataFrame: schema, row count, per-column statistics, sample rows.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DataframeDescribeParams {
    /// Inline JSON array of row objects.
    #[serde(default)]
    pub data: Option<Vec<serde_json::Value>>,
    /// Path to a JSON file containing row array.
    #[serde(default)]
    pub path: Option<String>,
    /// Number of sample rows to include (default: 5).
    #[serde(default)]
    pub sample_rows: Option<usize>,
}

/// Query a DataFrame: filter, sort, and limit rows.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DataframeQueryParams {
    /// Inline JSON array of row objects.
    #[serde(default)]
    pub data: Option<Vec<serde_json::Value>>,
    /// Path to a JSON file containing row array.
    #[serde(default)]
    pub path: Option<String>,
    /// Columns to include in output (default: all).
    #[serde(default)]
    pub columns: Option<Vec<String>>,
    /// Filter conditions (all must match — AND logic).
    #[serde(default)]
    pub filters: Option<Vec<DataframeFilterCondition>>,
    /// Column to sort by.
    #[serde(default)]
    pub sort_by: Option<String>,
    /// Sort descending (default: false).
    #[serde(default)]
    pub descending: Option<bool>,
    /// Maximum rows to return.
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Aggregate a DataFrame: group by columns and apply aggregation functions.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DataframeAggregateParams {
    /// Inline JSON array of row objects.
    #[serde(default)]
    pub data: Option<Vec<serde_json::Value>>,
    /// Path to a JSON file containing row array.
    #[serde(default)]
    pub path: Option<String>,
    /// Columns to group by.
    pub group_by: Vec<String>,
    /// Aggregation functions to apply.
    pub aggs: Vec<DataframeAggSpec>,
}

/// Count occurrences by key columns using the optimized Counter type.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DataframeCounterParams {
    /// Inline JSON array of row objects.
    #[serde(default)]
    pub data: Option<Vec<serde_json::Value>>,
    /// Path to a JSON file containing row array.
    #[serde(default)]
    pub path: Option<String>,
    /// Columns to count by (the key columns).
    pub key_columns: Vec<String>,
    /// Minimum count threshold (default: 1).
    #[serde(default)]
    pub min_count: Option<u64>,
}

/// Get detailed statistics for a single column.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DataframeColumnStatsParams {
    /// Inline JSON array of row objects.
    #[serde(default)]
    pub data: Option<Vec<serde_json::Value>>,
    /// Path to a JSON file containing row array.
    #[serde(default)]
    pub path: Option<String>,
    /// Column name to analyze.
    pub column: String,
}

/// Construct a DataFrame from typed column definitions.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DataframeConstructParams {
    /// Column definitions with name, type, and values.
    pub columns: Vec<DataframeColumnDef>,
}

/// Transform a DataFrame: select, drop, or rename columns.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DataframeTransformParams {
    /// Inline JSON array of row objects.
    #[serde(default)]
    pub data: Option<Vec<serde_json::Value>>,
    /// Path to a JSON file containing row array.
    #[serde(default)]
    pub path: Option<String>,
    /// Columns to select (keep only these).
    #[serde(default)]
    pub select: Option<Vec<String>>,
    /// Columns to drop.
    #[serde(default)]
    pub drop: Option<Vec<String>>,
    /// Columns to rename.
    #[serde(default)]
    pub rename: Option<Vec<DataframeRename>>,
}

/// Join two DataFrames on key columns.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DataframeJoinParams {
    /// Left DataFrame: inline JSON array of row objects.
    #[serde(default)]
    pub left_data: Option<Vec<serde_json::Value>>,
    /// Left DataFrame: path to JSON file.
    #[serde(default)]
    pub left_path: Option<String>,
    /// Right DataFrame: inline JSON array of row objects.
    #[serde(default)]
    pub right_data: Option<Vec<serde_json::Value>>,
    /// Right DataFrame: path to JSON file.
    #[serde(default)]
    pub right_path: Option<String>,
    /// Shared key column names (used when both tables have the same key names).
    #[serde(default)]
    pub on: Option<Vec<String>>,
    /// Left key column names (for asymmetric keys).
    #[serde(default)]
    pub left_on: Option<Vec<String>>,
    /// Right key column names (for asymmetric keys).
    #[serde(default)]
    pub right_on: Option<Vec<String>>,
    /// Join type: "inner" (default), "left", "right", "outer", "semi", "anti".
    #[serde(default)]
    pub how: Option<String>,
}

/// Save a DataFrame to a JSON file.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DataframeSaveParams {
    /// Inline JSON array of row objects.
    #[serde(default)]
    pub data: Option<Vec<serde_json::Value>>,
    /// Path to a JSON file to read from (if data not provided).
    #[serde(default)]
    pub source_path: Option<String>,
    /// Output file path (required).
    pub output_path: String,
}
