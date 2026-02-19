//! GitHub CLI (gh) Tool Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Parameters for gh pr create/view/list, issue view, and generic API calls.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{self, Deserialize, Deserializer};

/// Lenient bool deserializer
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

fn default_false() -> bool {
    false
}

/// Parameters for gh pr create
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct GhPrCreateParams {
    /// PR title (required)
    pub title: String,
    /// PR body/description
    #[serde(default)]
    pub body: Option<String>,
    /// Base branch (default: repo default branch)
    #[serde(default)]
    pub base: Option<String>,
    /// Create as draft PR
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub draft: bool,
    /// Path to the git repository
    #[serde(default)]
    pub path: Option<String>,
}

/// Parameters for gh pr view
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct GhPrViewParams {
    /// PR number. If not specified, views the PR for the current branch.
    #[serde(default)]
    pub number: Option<u32>,
    /// Path to the git repository
    #[serde(default)]
    pub path: Option<String>,
}

/// Parameters for gh pr list
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct GhPrListParams {
    /// Filter by state: open, closed, merged, all (default: open)
    #[serde(default)]
    pub state: Option<String>,
    /// Maximum number of PRs to list (default: 30)
    #[serde(default = "default_pr_limit")]
    pub limit: u32,
    /// Path to the git repository
    #[serde(default)]
    pub path: Option<String>,
}

fn default_pr_limit() -> u32 {
    30
}

/// Parameters for gh issue view
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct GhIssueViewParams {
    /// Issue number (required)
    pub number: u32,
    /// Path to the git repository
    #[serde(default)]
    pub path: Option<String>,
}

/// Parameters for gh api
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct GhApiParams {
    /// API endpoint (e.g., "repos/owner/repo/pulls/123/comments")
    pub endpoint: String,
    /// HTTP method (default: GET). DELETE is blocked by default.
    #[serde(default)]
    pub method: Option<String>,
    /// Request body as JSON string
    #[serde(default)]
    pub body: Option<String>,
    /// Allow DELETE method (requires explicit confirmation)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub allow_delete: bool,
}
