//! Parameter types for compilation space MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// A compilation point description (position in 7D space).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompilationPointInput {
    /// Abstraction level: "execution", "binary", "ir", "ast", "tokens", "source", "spec", "intent".
    pub abstraction: String,
    /// Language identifier (e.g., "rust", "prima", "pvdsl", "javascript").
    pub language: String,
    /// Optional revision number for temporal coordinate.
    pub revision: Option<u64>,
    /// Optional evaluation state: "symbolic", "partial", "concrete".
    pub eval_state: Option<String>,
    /// Optional reflection depth (0 = ground, 1 = meta, 2 = meta-meta).
    pub reflection_depth: Option<u32>,
    /// Optional dimensionality: "scalar", "linear", "tree", "graph".
    pub dimensionality: Option<String>,
}

/// Compare two compilation points.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompilationPointCompareParams {
    /// First compilation point.
    pub a: CompilationPointInput,
    /// Second compilation point.
    pub b: CompilationPointInput,
}

/// Get summary of a compilation point.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompilationPointSummaryParams {
    /// The compilation point to summarize.
    pub point: CompilationPointInput,
}

/// List preset compilation points.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompilationPointPresetsParams {}

/// Look up a named transform from the catalog.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompilationCatalogLookupParams {
    /// Transform name (e.g., "lex", "parse", "lower", "codegen", "transpile",
    /// "const_eval", "macro_expand", "serialize", "intent_compile", "refactor",
    /// "prima_compile", "pvdsl_execute", etc.).
    pub transform: String,
    /// Language for transforms that need one (e.g., "rust", "prima").
    pub language: Option<String>,
    /// Target language for cross-language transforms (transpile, ir_translate).
    pub target_language: Option<String>,
    /// Revision range start for temporal transforms (diff, patch, migrate).
    pub from_rev: Option<u64>,
    /// Revision range end for temporal transforms.
    pub to_rev: Option<u64>,
    /// Feature name for feature_gate transform.
    pub feature: Option<String>,
}

/// Validate a transform chain for step connectivity.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompilationChainValidateParams {
    /// Ordered list of catalog transform names to form a chain.
    pub transforms: Vec<CompilationCatalogLookupParams>,
}

/// Get a preset transform chain.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompilationChainPresetsParams {
    /// Chain name: "compile" (language needed), "nexcore_ai", "rust_compile".
    pub chain: String,
    /// Language for compile chain.
    pub language: Option<String>,
}

/// List all 7 compilation space axes.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompilationAxesCatalogParams {}

/// List all 8 abstraction levels in order.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompilationAbstractionLevelsParams {}

/// Compute spatial distance between two compilation points.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompilationDistanceParams {
    /// First compilation point.
    pub a: CompilationPointInput,
    /// Second compilation point.
    pub b: CompilationPointInput,
}
