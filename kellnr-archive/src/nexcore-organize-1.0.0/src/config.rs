//! Configuration types for the ORGANIZE pipeline.
//!
//! Tier: T2-C (μ Mapping + ∂ Boundary + ς State)
//!
//! Supports both TOML file loading and programmatic construction.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{OrganizeError, OrganizeResult};

// ============================================================================
// Top-Level Config
// ============================================================================

/// Top-level configuration for the ORGANIZE pipeline.
///
/// Tier: T2-C (μ Mapping — maps rules to pipeline behavior)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizeConfig {
    /// Root directory to organize.
    pub root: PathBuf,

    /// Whether to execute mutations or just report.
    #[serde(default = "default_dry_run")]
    pub dry_run: bool,

    /// Maximum directory depth to walk (0 = unlimited).
    #[serde(default)]
    pub max_depth: usize,

    /// Patterns to exclude from observation (glob patterns).
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// Ranking configuration.
    #[serde(default)]
    pub ranking: RankingConfig,

    /// Grouping rules (name → rule).
    #[serde(default)]
    pub groups: HashMap<String, GroupRule>,

    /// Default action for ungrouped entries.
    #[serde(default)]
    pub default_action: FileOp,

    /// Naming configuration.
    #[serde(default)]
    pub naming: NamingConfig,

    /// State snapshot path for drift detection.
    #[serde(default)]
    pub state_path: Option<PathBuf>,
}

fn default_dry_run() -> bool {
    true
}

impl OrganizeConfig {
    /// Create a minimal config for the given root directory.
    ///
    /// Defaults to dry-run mode with standard grouping rules.
    pub fn default_for(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            root,
            dry_run: true,
            max_depth: 0,
            exclude_patterns: vec![
                ".git".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
                ".DS_Store".to_string(),
            ],
            ranking: RankingConfig::default(),
            groups: default_groups(),
            default_action: FileOp::Keep,
            naming: NamingConfig::default(),
            state_path: None,
        }
    }

    /// Load configuration from a TOML file.
    pub fn from_toml(path: &Path) -> OrganizeResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| OrganizeError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Serialize configuration to TOML string.
    pub fn to_toml(&self) -> OrganizeResult<String> {
        toml::to_string_pretty(self).map_err(|e| OrganizeError::Config(e.to_string()))
    }
}

// ============================================================================
// Ranking Config
// ============================================================================

/// Weights for the ranking step.
///
/// Tier: T2-P (κ Comparison — controls scoring weights)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingConfig {
    /// Weight for recency (0.0..=1.0).
    pub recency_weight: f64,
    /// Weight for file size (0.0..=1.0).
    pub size_weight: f64,
    /// Weight for extension relevance (0.0..=1.0).
    pub relevance_weight: f64,
    /// Weight for directory depth (0.0..=1.0).
    pub depth_weight: f64,
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            recency_weight: 0.3,
            size_weight: 0.2,
            relevance_weight: 0.35,
            depth_weight: 0.15,
        }
    }
}

// ============================================================================
// Group Rules
// ============================================================================

/// Rule that determines membership in a group.
///
/// Tier: T2-P (μ Mapping — maps file attributes to group membership)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRule {
    /// File extensions that match this group (e.g., ["rs", "toml"]).
    #[serde(default)]
    pub extensions: Vec<String>,

    /// Filename patterns (glob-style) that match this group.
    #[serde(default)]
    pub patterns: Vec<String>,

    /// Minimum file size in bytes (inclusive).
    #[serde(default)]
    pub min_size: Option<u64>,

    /// Maximum file size in bytes (inclusive).
    #[serde(default)]
    pub max_size: Option<u64>,

    /// Maximum age in days before this rule applies.
    #[serde(default)]
    pub max_age_days: Option<u64>,

    /// Action to take for entries in this group.
    pub action: FileOp,

    /// Target directory for Move/Archive actions (relative to root).
    #[serde(default)]
    pub target_dir: Option<PathBuf>,
}

// ============================================================================
// Actions
// ============================================================================

/// File organization operation (Move/Archive/Delete/Keep/Review).
///
/// Tier: T2-P (σ — sequence of file operations)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileOp {
    /// Move to a target directory.
    Move,
    /// Archive (compress and move).
    Archive,
    /// Delete permanently.
    Delete,
    /// Keep in place (no-op).
    Keep,
    /// Flag for human review.
    Review,
}

/// Backward-compatible alias.
#[deprecated(note = "use FileOp — F2 equivocation fix")]
pub type Action = FileOp;

impl Default for FileOp {
    fn default() -> Self {
        Self::Keep
    }
}

impl std::fmt::Display for FileOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Move => write!(f, "Move"),
            Self::Archive => write!(f, "Archive"),
            Self::Delete => write!(f, "Delete"),
            Self::Keep => write!(f, "Keep"),
            Self::Review => write!(f, "Review"),
        }
    }
}

// ============================================================================
// Naming Config
// ============================================================================

/// Configuration for the naming/rename step.
///
/// Tier: T2-P (∂ Boundary — naming constraints and collision rules)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingConfig {
    /// Whether to lowercase filenames.
    #[serde(default)]
    pub lowercase: bool,

    /// Whether to replace spaces with underscores.
    #[serde(default = "default_true")]
    pub replace_spaces: bool,

    /// Maximum filename length (0 = unlimited).
    #[serde(default)]
    pub max_length: usize,

    /// Strategy for handling name collisions.
    #[serde(default)]
    pub collision_strategy: CollisionStrategy,
}

fn default_true() -> bool {
    true
}

impl Default for NamingConfig {
    fn default() -> Self {
        Self {
            lowercase: false,
            replace_spaces: true,
            max_length: 0,
            collision_strategy: CollisionStrategy::Suffix,
        }
    }
}

/// Strategy for resolving naming collisions.
///
/// Tier: T2-P (∂ Boundary — boundary resolution strategy)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CollisionStrategy {
    /// Append numeric suffix: file_1.txt, file_2.txt.
    Suffix,
    /// Skip the conflicting file.
    Skip,
    /// Return an error.
    Error,
}

impl Default for CollisionStrategy {
    fn default() -> Self {
        Self::Suffix
    }
}

// ============================================================================
// Default Groups
// ============================================================================

/// Create a standard set of grouping rules.
fn default_groups() -> HashMap<String, GroupRule> {
    let mut groups = HashMap::new();

    groups.insert(
        "rust".to_string(),
        GroupRule {
            extensions: vec!["rs".to_string(), "toml".to_string()],
            patterns: vec!["Cargo.*".to_string()],
            min_size: None,
            max_size: None,
            max_age_days: None,
            action: FileOp::Keep,
            target_dir: None,
        },
    );

    groups.insert(
        "documents".to_string(),
        GroupRule {
            extensions: vec![
                "md".to_string(),
                "txt".to_string(),
                "pdf".to_string(),
                "doc".to_string(),
                "docx".to_string(),
            ],
            patterns: vec![],
            min_size: None,
            max_size: None,
            max_age_days: None,
            action: FileOp::Keep,
            target_dir: Some(PathBuf::from("documents")),
        },
    );

    groups.insert(
        "archives".to_string(),
        GroupRule {
            extensions: vec![
                "zip".to_string(),
                "tar".to_string(),
                "gz".to_string(),
                "bz2".to_string(),
                "xz".to_string(),
                "7z".to_string(),
            ],
            patterns: vec![],
            min_size: None,
            max_size: None,
            max_age_days: None,
            action: FileOp::Archive,
            target_dir: Some(PathBuf::from("archives")),
        },
    );

    groups.insert(
        "images".to_string(),
        GroupRule {
            extensions: vec![
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "svg".to_string(),
                "webp".to_string(),
            ],
            patterns: vec![],
            min_size: None,
            max_size: None,
            max_age_days: None,
            action: FileOp::Move,
            target_dir: Some(PathBuf::from("images")),
        },
    );

    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OrganizeConfig::default_for("/tmp/test");
        assert!(config.dry_run);
        assert_eq!(config.max_depth, 0);
        assert!(!config.groups.is_empty());
        assert_eq!(config.default_action, FileOp::Keep);
    }

    #[test]
    fn test_action_display() {
        assert_eq!(FileOp::Move.to_string(), "Move");
        assert_eq!(FileOp::Delete.to_string(), "Delete");
        assert_eq!(FileOp::Keep.to_string(), "Keep");
        assert_eq!(FileOp::Review.to_string(), "Review");
        assert_eq!(FileOp::Archive.to_string(), "Archive");
    }

    #[test]
    fn test_default_groups_populated() {
        let groups = default_groups();
        assert!(groups.contains_key("rust"));
        assert!(groups.contains_key("documents"));
        assert!(groups.contains_key("archives"));
        assert!(groups.contains_key("images"));
    }

    #[test]
    fn test_ranking_config_weights_sum() {
        let rc = RankingConfig::default();
        let sum = rc.recency_weight + rc.size_weight + rc.relevance_weight + rc.depth_weight;
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_collision_strategy_default() {
        assert_eq!(CollisionStrategy::default(), CollisionStrategy::Suffix);
    }

    #[test]
    fn test_config_roundtrip_toml() {
        let config = OrganizeConfig::default_for("/tmp/test");
        let toml_str = config.to_toml();
        assert!(toml_str.is_ok());
    }
}
