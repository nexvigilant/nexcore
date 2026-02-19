//! Git Tool Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Parameters for git status, diff, log, commit, branch, checkout, push, stash.

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

fn default_false() -> bool {
    false
}

/// Parameters for git status
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct GitStatusParams {
    /// Path to the git repository. Defaults to current directory.
    #[serde(default)]
    pub path: Option<String>,
}

/// Parameters for git diff
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct GitDiffParams {
    /// Path to the git repository. Defaults to current directory.
    #[serde(default)]
    pub path: Option<String>,
    /// Show staged changes (--staged/--cached)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub staged: bool,
    /// Specific file to diff
    #[serde(default)]
    pub file: Option<String>,
    /// Diff against a specific ref (branch, commit, tag)
    #[serde(default)]
    pub ref_spec: Option<String>,
}

/// Parameters for git log
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct GitLogParams {
    /// Path to the git repository. Defaults to current directory.
    #[serde(default)]
    pub path: Option<String>,
    /// Number of commits to show (default: 10)
    #[serde(default = "default_log_count")]
    pub count: u32,
    /// Use one-line format
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub oneline: bool,
}

fn default_log_count() -> u32 {
    10
}

/// Parameters for git commit
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct GitCommitParams {
    /// Path to the git repository. Defaults to current directory.
    #[serde(default)]
    pub path: Option<String>,
    /// Commit message (required)
    pub message: String,
    /// Files to stage before committing. If empty, commits currently staged changes.
    #[serde(default)]
    pub files: Vec<String>,
}

/// Parameters for git branch
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct GitBranchParams {
    /// Path to the git repository. Defaults to current directory.
    #[serde(default)]
    pub path: Option<String>,
    /// List all branches (local and remote with -a)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub list: bool,
    /// Create a new branch with this name
    #[serde(default)]
    pub create: Option<String>,
    /// Delete a branch with this name
    #[serde(default)]
    pub delete: Option<String>,
}

/// Parameters for git checkout
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct GitCheckoutParams {
    /// Path to the git repository. Defaults to current directory.
    #[serde(default)]
    pub path: Option<String>,
    /// Branch, tag, or commit to checkout
    pub target: String,
    /// Create a new branch (-b flag)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub create: bool,
}

/// Parameters for git push
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct GitPushParams {
    /// Path to the git repository. Defaults to current directory.
    #[serde(default)]
    pub path: Option<String>,
    /// Remote name (default: origin)
    #[serde(default)]
    pub remote: Option<String>,
    /// Branch to push
    #[serde(default)]
    pub branch: Option<String>,
    /// Set upstream tracking (-u flag)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub set_upstream: bool,
    /// Force push (DANGEROUS — requires explicit confirmation)
    #[serde(
        default = "default_false",
        deserialize_with = "deserialize_bool_lenient"
    )]
    pub force: bool,
}

/// Stash action type
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub enum StashAction {
    /// Save current changes to stash
    #[serde(rename = "push")]
    Push,
    /// Apply and remove the top stash entry
    #[serde(rename = "pop")]
    Pop,
    /// List all stash entries
    #[serde(rename = "list")]
    List,
    /// Drop the top stash entry
    #[serde(rename = "drop")]
    Drop,
}

/// Parameters for git stash
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct GitStashParams {
    /// Path to the git repository. Defaults to current directory.
    #[serde(default)]
    pub path: Option<String>,
    /// Stash action: push, pop, list, or drop
    pub action: StashAction,
    /// Optional message for stash push
    #[serde(default)]
    pub message: Option<String>,
}
