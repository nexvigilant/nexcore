//! # Classification Tags
//!
//! What gets tagged and the tag assignment itself.
//!
//! ## Primitive Grounding
//! - **TagTarget**: T2-P, Dominant: μ Mapping (maps names to targets)
//! - **ClassificationTag**: T2-C, Dominant: μ Mapping (μ + ς + π)

use crate::level::ClassificationLevel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// What kind of asset is being classified.
///
/// ## Tier: T2-P
/// ## Dominant: μ Mapping
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TagTarget {
    /// An entire project directory.
    Project(String),
    /// A specific crate.
    Crate(String),
    /// A specific file path.
    File(String),
    /// A skill name.
    Skill(String),
    /// An MCP tool name.
    McpTool(String),
    /// A cloud region or deployment.
    Region(String),
    /// A data category (e.g., PHI, PII).
    DataCategory(String),
}

impl TagTarget {
    /// Returns the target kind as a static string.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Project(_) => "project",
            Self::Crate(_) => "crate",
            Self::File(_) => "file",
            Self::Skill(_) => "skill",
            Self::McpTool(_) => "mcp_tool",
            Self::Region(_) => "region",
            Self::DataCategory(_) => "data_category",
        }
    }

    /// Returns the target name/identifier.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Project(n)
            | Self::Crate(n)
            | Self::File(n)
            | Self::Skill(n)
            | Self::McpTool(n)
            | Self::Region(n)
            | Self::DataCategory(n) => n,
        }
    }
}

impl fmt::Display for TagTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.kind(), self.name())
    }
}

/// A classification assignment to a specific target.
///
/// ## Tier: T2-C
/// ## Dominant: μ Mapping (maps target → level with metadata)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassificationTag {
    /// What is being classified.
    pub target: TagTarget,
    /// The assigned classification level.
    pub level: ClassificationLevel,
    /// Who assigned this classification.
    pub assigned_by: String,
    /// When the classification was assigned.
    pub timestamp: DateTime<Utc>,
    /// Reason for this classification.
    pub reason: String,
    /// Whether downgrade is permitted without dual-auth.
    pub downgrade_permitted: bool,
}

impl ClassificationTag {
    /// Create a new classification tag.
    #[must_use]
    pub fn new(
        target: TagTarget,
        level: ClassificationLevel,
        assigned_by: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            target,
            level,
            assigned_by: assigned_by.into(),
            timestamp: Utc::now(),
            reason: reason.into(),
            downgrade_permitted: false,
        }
    }

    /// Create with downgrade permission.
    #[must_use]
    pub fn with_downgrade_permitted(mut self, permitted: bool) -> Self {
        self.downgrade_permitted = permitted;
        self
    }

    /// Whether this tag's level is restricted.
    #[must_use]
    pub fn is_restricted(&self) -> bool {
        self.level.is_restricted()
    }
}

impl fmt::Display for ClassificationTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} by {}",
            self.level, self.target, self.assigned_by
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_target_project() {
        let t = TagTarget::Project("nexcore".into());
        assert_eq!(t.kind(), "project");
        assert_eq!(t.name(), "nexcore");
    }

    #[test]
    fn tag_target_file() {
        let t = TagTarget::File("src/secret.rs".into());
        assert_eq!(t.kind(), "file");
        assert_eq!(t.name(), "src/secret.rs");
    }

    #[test]
    fn tag_target_skill() {
        let t = TagTarget::Skill("forge".into());
        assert_eq!(t.kind(), "skill");
    }

    #[test]
    fn tag_target_mcp_tool() {
        let t = TagTarget::McpTool("pv_signal_complete".into());
        assert_eq!(t.kind(), "mcp_tool");
    }

    #[test]
    fn tag_target_display() {
        let t = TagTarget::Region("us-east1".into());
        assert_eq!(t.to_string(), "region:us-east1");
    }

    #[test]
    fn classification_tag_creation() {
        let tag = ClassificationTag::new(
            TagTarget::Project("nexcore".into()),
            ClassificationLevel::Internal,
            "Matthew Campion, PharmD",
            "Team-only codebase",
        );
        assert_eq!(tag.level, ClassificationLevel::Internal);
        assert!(!tag.downgrade_permitted);
    }

    #[test]
    fn classification_tag_downgrade_default_false() {
        let tag = ClassificationTag::new(
            TagTarget::File("key.pem".into()),
            ClassificationLevel::TopSecret,
            "admin",
            "crypto key",
        );
        assert!(!tag.downgrade_permitted);
    }

    #[test]
    fn classification_tag_with_downgrade() {
        let tag = ClassificationTag::new(
            TagTarget::Crate("nexcore-labs".into()),
            ClassificationLevel::Confidential,
            "admin",
            "experimental",
        )
        .with_downgrade_permitted(true);
        assert!(tag.downgrade_permitted);
    }

    #[test]
    fn classification_tag_is_restricted() {
        let tag = ClassificationTag::new(
            TagTarget::Project("algo".into()),
            ClassificationLevel::Secret,
            "admin",
            "proprietary",
        );
        assert!(tag.is_restricted());
    }

    #[test]
    fn serde_roundtrip() {
        let tag = ClassificationTag::new(
            TagTarget::DataCategory("PHI".into()),
            ClassificationLevel::TopSecret,
            "admin",
            "protected health info",
        );
        let json = serde_json::to_string(&tag).unwrap_or_default();
        let parsed: Result<ClassificationTag, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok());
    }
}
