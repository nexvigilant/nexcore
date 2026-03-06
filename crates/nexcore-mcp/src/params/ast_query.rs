//! AST query parameters for structural code search.

use schemars::JsonSchema;
use serde::Deserialize;

/// Parse a single Rust source file and return all extracted items (types, enums, traits, functions, impl blocks).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AstQueryFileParams {
    /// Absolute path to the Rust source file.
    pub path: String,
}

/// Search a crate's source files for items matching a name pattern.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AstQuerySearchParams {
    /// Crate name (resolved to `~/Projects/Active/nexcore/crates/{crate_name}/`).
    pub crate_name: String,
    /// Name pattern to search for (case-insensitive substring match).
    pub pattern: String,
    /// Filter by item types: "struct", "enum", "trait", "fn", "impl" (optional — all if omitted).
    pub item_types: Option<Vec<String>>,
}

/// Find all implementors of a trait within a crate.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AstQueryImplementorsParams {
    /// Crate name (resolved to `~/Projects/Active/nexcore/crates/{crate_name}/`).
    pub crate_name: String,
    /// Trait name to search for (case-insensitive substring match on trait_name field).
    pub trait_name: String,
}
