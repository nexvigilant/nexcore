//! Claude FS Parameters (N-space Filesystem)
//!
//! List, read, write, delete, search, tail, diff, and stat operations under ~/.claude/.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for claude_fs_list.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsListParams {
    /// Relative path under ~/.claude/
    #[serde(default = "default_dot")]
    pub path: String,
}

/// Parameters for claude_fs_read.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsReadParams {
    /// Relative path under ~/.claude/
    pub path: String,
}

/// Parameters for claude_fs_write.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsWriteParams {
    /// Relative path under ~/.claude/
    pub path: String,
    /// Content to write.
    pub content: String,
    /// Whether to create parent directories.
    pub create_dirs: Option<bool>,
}

/// Parameters for claude_fs_delete.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsDeleteParams {
    /// Relative path under ~/.claude/
    pub path: String,
}

/// Parameters for claude_fs_search.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsSearchParams {
    /// Substring to search for.
    pub query: String,
    /// Root directory to search under.
    pub root: Option<String>,
    /// Maximum results to return.
    pub max_results: Option<usize>,
}

/// Parameters for claude_fs_tail.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsTailParams {
    /// Relative path under ~/.claude/
    pub path: String,
    /// Number of lines from the end.
    pub lines: Option<usize>,
}

/// Parameters for claude_fs_diff.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsDiffParams {
    /// First file path.
    pub path_a: String,
    /// Second file path.
    pub path_b: String,
}

/// Parameters for claude_fs_stat.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsStatParams {
    /// Relative path under ~/.claude/
    pub path: String,
}

fn default_dot() -> String {
    ".".to_string()
}
