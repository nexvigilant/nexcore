//! Rust Development Tool Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Batch 1: rust_dev_error_type, rust_dev_derive_advisor,
//!          rust_dev_match_generate, rust_dev_borrow_explain
//! Batch 2: rust_dev_clippy_explain, rust_dev_rustc_explain,
//!          rust_dev_unsafe_audit
//! Batch 3: rust_dev_cargo_expand, rust_dev_cargo_bloat,
//!          rust_dev_cargo_miri, rust_dev_edition_migrate,
//!          rust_dev_invocations

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{self, Deserialize, Deserializer};

/// Lenient bool deserializer: accepts `true`, `"true"`, `"1"`, `false`, `"false"`, `"0"`.
fn deserialize_bool_lenient<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(crate = "rmcp::serde", untagged)]
    enum BoolOrString {
        Bool(bool),
        Str(String),
    }

    match BoolOrString::deserialize(deserializer)? {
        BoolOrString::Bool(b) => Ok(b),
        BoolOrString::Str(s) => match s.to_lowercase().as_str() {
            "true" | "1" | "yes" => Ok(true),
            "false" | "0" | "no" | "" => Ok(false),
            other => Err(serde::de::Error::custom(format!(
                "expected boolean or bool-like string, got: {other}"
            ))),
        },
    }
}

fn default_true() -> bool {
    true
}

// ============================================================================
// rust_dev_error_type
// ============================================================================

/// A single error variant specification.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct ErrorVariantSpec {
    /// Variant name (e.g. "NotFound", "ParseFailed")
    pub name: String,
    /// Display message template (e.g. "resource not found: {0}" or "parse error: {field}")
    pub message: String,
    /// Named or positional fields (e.g. ["String"] for tuple, ["path: String", "line: usize"] for struct)
    #[serde(default)]
    pub fields: Vec<String>,
    /// Source error type for `#[from]` conversion (e.g. "std::io::Error")
    pub from: Option<String>,
}

/// Parameters for rust_dev_error_type: generate a complete thiserror enum.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RustDevErrorTypeParams {
    /// Module or type name prefix (used for the error type, e.g. "MyService" -> MyServiceError)
    pub name: String,
    /// Error variant specifications
    pub variants: Vec<ErrorVariantSpec>,
    /// Include thiserror derives (default: true). If false, generates manual Display impl.
    #[serde(
        default = "default_true",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub use_thiserror: bool,
}

// ============================================================================
// rust_dev_derive_advisor
// ============================================================================

/// Parameters for rust_dev_derive_advisor: analyze a type definition and recommend derives.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RustDevDeriveAdvisorParams {
    /// Rust type definition source code (struct or enum)
    pub type_definition: String,
}

// ============================================================================
// rust_dev_match_generate
// ============================================================================

/// Parameters for rust_dev_match_generate: generate exhaustive match arms for an enum.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RustDevMatchGenerateParams {
    /// Enum definition (Rust source code)
    pub enum_definition: String,
    /// Variable name to match on (default: "value")
    pub match_var: Option<String>,
    /// Generate todo!() bodies vs empty blocks (default: true)
    #[serde(
        default = "default_true",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub with_todo: bool,
}

// ============================================================================
// rust_dev_borrow_explain
// ============================================================================

/// Parameters for rust_dev_borrow_explain: parse a borrow checker error and explain it.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RustDevBorrowExplainParams {
    /// The compiler error message (full text from cargo check/build)
    pub error_message: String,
    /// Optional surrounding source code for context-aware fix suggestions
    pub source_code: Option<String>,
}

// ============================================================================
// rust_dev_clippy_explain
// ============================================================================

/// Parameters for rust_dev_clippy_explain: parse a clippy lint → explain + suggest fix.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RustDevClippyExplainParams {
    /// The clippy lint name (e.g. "clippy::unwrap_used") or full warning message
    pub lint: String,
    /// Optional source code that triggered the lint
    pub source_code: Option<String>,
}

// ============================================================================
// rust_dev_rustc_explain
// ============================================================================

/// Parameters for rust_dev_rustc_explain: get rustc's official explanation for an error code.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RustDevRustcExplainParams {
    /// Error code (e.g. "E0382", "E0277"). The "E" prefix is optional.
    pub error_code: String,
}

// ============================================================================
// rust_dev_unsafe_audit
// ============================================================================

/// Parameters for rust_dev_unsafe_audit: scan source code for unsafe blocks and classify them.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RustDevUnsafeAuditParams {
    /// Rust source code to audit (file contents)
    pub source_code: String,
    /// File path (for reporting purposes only)
    pub file_path: Option<String>,
}

// ============================================================================
// rust_dev_cargo_expand (Batch 3)
// ============================================================================

/// Parameters for rust_dev_cargo_expand: expand macros in a crate or item.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RustDevCargoExpandParams {
    /// Path to crate directory. Defaults to nexcore workspace root.
    pub path: Option<String>,
    /// Specific package to expand (for workspace builds). Maps to -p flag.
    pub package: Option<String>,
    /// Specific item to expand (e.g. "MyStruct" or "my_module::MyStruct")
    pub item: Option<String>,
    /// Expand only a specific theme: "derive" or "attr" or "proc_macro"
    pub theme: Option<String>,
}

// ============================================================================
// rust_dev_cargo_bloat (Batch 3)
// ============================================================================

/// Parameters for rust_dev_cargo_bloat: analyze binary size by function/crate.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RustDevCargoBloatParams {
    /// Path to crate directory. Defaults to nexcore workspace root.
    pub path: Option<String>,
    /// Specific package to analyze. Maps to -p flag.
    pub package: Option<String>,
    /// Show crate-level breakdown instead of function-level (--crates flag)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub crates: bool,
    /// Number of top entries to show (default: 20)
    pub top: Option<u32>,
    /// Build in release mode (default: true for accurate size data)
    #[serde(
        default = "default_true",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub release: bool,
}

fn default_false() -> bool {
    false
}

// ============================================================================
// rust_dev_cargo_miri (Batch 3)
// ============================================================================

/// Parameters for rust_dev_cargo_miri: run Miri for undefined behavior detection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RustDevCargoMiriParams {
    /// Path to crate directory.
    pub path: Option<String>,
    /// Specific package to test. Maps to -p flag.
    pub package: Option<String>,
    /// Test name filter (passed after --)
    pub test_filter: Option<String>,
    /// Run only lib tests (--lib)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub lib_only: bool,
}

// ============================================================================
// rust_dev_edition_migrate (Batch 3)
// ============================================================================

/// Parameters for rust_dev_edition_migrate: get edition migration guidance.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RustDevEditionMigrateParams {
    /// Source edition (e.g. "2015", "2018", "2021")
    pub from_edition: String,
    /// Target edition (e.g. "2021", "2024")
    pub to_edition: String,
}

// ============================================================================
// rust_dev_invocations (Batch 3)
// ============================================================================

/// Parameters for rust_dev_invocations: get metadata about the rust_dev tool suite.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct RustDevInvocationsParams {
    /// Filter to a specific tool (e.g. "error_type", "borrow_explain"). Omit for all tools.
    pub tool: Option<String>,
}
