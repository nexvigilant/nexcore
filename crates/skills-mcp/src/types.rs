use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EmptyParams {}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillRunParams {
    /// Program name (e.g. "clean", "audit", "smart", "brain", "guard", "pulse", "forge", "scope", "craft", "trace", "rivet", "glean", "pixel")
    pub program: String,
    /// Phase letter or "ALL" for full pipeline (e.g. "C", "L", "E", "A", "N", "ALL")
    pub phase: String,
    /// Optional target directory for project-scoped programs
    pub dir: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveDecomposeParams {
    /// A concept, problem, code pattern, or domain term to decompose into T1 primitives.
    /// Examples: "rate limiter", "cache invalidation", "login authentication", "HashMap<K,V>"
    pub concept: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveComposeParams {
    /// List of T1 primitive names to compose together.
    /// Examples: ["Sequence", "Mapping", "Boundary"] or ["σ", "μ", "∂"]
    pub primitives: Vec<String>,
}
