//! # ASR (Automated Skill Router) Configuration
//!
//! Type-safe configuration for skill routing, flywheel cycles, and Definition of Done.
//!
//! Integrates with:
//! - `nexcore-vigilance::pv::thresholds::SignalCriteria` for signal thresholds
//! - `nexcore-vigilance::skills::taxonomy` for SMST scores
//! - `nexcore-brain` for session state persistence

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Flywheel cycle stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum FlywheelStage {
    /// Active skill execution
    #[default]
    SkillExecution,
    /// Weekly review (Friday)
    FridayReview,
    /// Sprint planning
    WeeklySprint,
}

impl FlywheelStage {
    /// Get the next stage in the cycle
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::SkillExecution => Self::FridayReview,
            Self::FridayReview => Self::WeeklySprint,
            Self::WeeklySprint => Self::SkillExecution,
        }
    }

    /// Get all stages in order
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::SkillExecution, Self::FridayReview, Self::WeeklySprint]
    }
}

/// Definition of Done checklist item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoDItem {
    /// Item description
    pub description: String,
    /// Whether this item is completed
    #[serde(default)]
    pub completed: bool,
    /// Optional requirement (e.g., "≥3" for test count)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirement: Option<String>,
}

impl DoDItem {
    /// Create a new DoD item
    #[must_use]
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            completed: false,
            requirement: None,
        }
    }

    /// Create a DoD item with a requirement
    #[must_use]
    pub fn with_requirement(
        description: impl Into<String>,
        requirement: impl Into<String>,
    ) -> Self {
        Self {
            description: description.into(),
            completed: false,
            requirement: Some(requirement.into()),
        }
    }
}

/// Standard 6-point Definition of Done checklist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoDChecklist {
    pub items: Vec<DoDItem>,
}

impl Default for DoDChecklist {
    fn default() -> Self {
        Self {
            items: vec![
                DoDItem::new("Algorithm section extracted"),
                DoDItem::with_requirement("Decision tree nodes generated", "complete"),
                DoDItem::with_requirement("Unit tests", "≥3 cases"),
                DoDItem::new("All tests passing"),
                DoDItem::with_requirement("Coverage", "≥80%"),
                DoDItem::new("No regression"),
            ],
        }
    }
}

impl DoDChecklist {
    /// Check if all items are completed
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.items.iter().all(|item| item.completed)
    }

    /// Get completion percentage
    #[must_use]
    pub fn completion_percentage(&self) -> f64 {
        if self.items.is_empty() {
            return 100.0;
        }
        let completed = self.items.iter().filter(|i| i.completed).count();
        ((completed as f64) / (self.items.len() as f64)) * 100.0
    }
}

/// Model selection for routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Model {
    Haiku,
    #[default]
    Sonnet,
    Opus,
}

/// ASR routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Default model when confidence is insufficient
    #[serde(default)]
    pub default_model: Model,

    /// Minimum confidence for auto-routing (0.0-1.0)
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f64,

    /// Log routing decisions
    #[serde(default = "default_true")]
    pub log_decisions: bool,

    /// Path to routing log
    #[serde(default = "default_log_path")]
    pub log_path: PathBuf,
}

fn default_confidence_threshold() -> f64 {
    0.8
}

fn default_true() -> bool {
    true
}

fn default_log_path() -> PathBuf {
    PathBuf::from("~/.claude/logs/asr_routing.jsonl")
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            default_model: Model::default(),
            confidence_threshold: default_confidence_threshold(),
            log_decisions: true,
            log_path: default_log_path(),
        }
    }
}

/// Flywheel cycle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelConfig {
    /// Current stage
    #[serde(default)]
    pub current_stage: FlywheelStage,

    /// State file path
    #[serde(default = "default_state_file")]
    pub state_file: PathBuf,

    /// Review cooldown in hours
    #[serde(default = "default_cooldown")]
    pub review_cooldown_hours: u32,

    /// Sprint duration in days
    #[serde(default = "default_sprint_duration")]
    pub sprint_duration_days: u32,

    /// Maximum skills per sprint
    #[serde(default = "default_max_skills")]
    pub max_skills_per_sprint: u32,
}

fn default_state_file() -> PathBuf {
    PathBuf::from("~/.claude/brain/flywheel_state.json")
}

fn default_cooldown() -> u32 {
    24
}

fn default_sprint_duration() -> u32 {
    7
}

fn default_max_skills() -> u32 {
    5
}

impl Default for FlywheelConfig {
    fn default() -> Self {
        Self {
            current_stage: FlywheelStage::default(),
            state_file: default_state_file(),
            review_cooldown_hours: default_cooldown(),
            sprint_duration_days: default_sprint_duration(),
            max_skills_per_sprint: default_max_skills(),
        }
    }
}

/// Complete ASR configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrConfig {
    /// Routing configuration
    #[serde(default)]
    pub routing: RoutingConfig,

    /// Flywheel configuration
    #[serde(default)]
    pub flywheel: FlywheelConfig,

    /// Definition of Done checklist
    #[serde(default)]
    pub dod: DoDChecklist,

    /// Skills registry path
    #[serde(default = "default_skills_path")]
    pub skills_path: PathBuf,

    /// Minimum SMST score for skill activation
    #[serde(default = "default_min_smst")]
    pub min_smst_score: u32,
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            routing: RoutingConfig::default(),
            flywheel: FlywheelConfig::default(),
            dod: DoDChecklist::default(),
            skills_path: default_skills_path(),
            min_smst_score: default_min_smst(),
        }
    }
}

fn default_skills_path() -> PathBuf {
    PathBuf::from("~/.claude/skills")
}

fn default_min_smst() -> u32 {
    75 // Platinum minimum
}

impl AsrConfig {
    /// Load from TOML file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> nexcore_error::Result<Self> {
        use nexcore_error::Context;
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read ASR config: {}", path.display()))?;
        let config: Self = toml::from_str(&content)
            .context(format!("Failed to parse ASR config: {}", path.display()))?;
        Ok(config)
    }

    /// Load from default path (~/.claude/config/asr.toml)
    pub fn load_default() -> nexcore_error::Result<Self> {
        use nexcore_error::Context;
        let home = std::env::var("HOME").context("HOME env not set")?;
        let path = format!("{}/.claude/config/asr.toml", home);

        if std::path::Path::new(&path).exists() {
            Self::from_file(&path)
        } else {
            Ok(Self::default())
        }
    }

    /// Write to TOML file
    pub fn write_toml(&self, path: impl AsRef<std::path::Path>) -> nexcore_error::Result<()> {
        use nexcore_error::Context;
        let path = path.as_ref();
        let content = toml::to_string_pretty(self).context("Failed to serialize ASR config")?;
        std::fs::write(path, &content)
            .context(format!("Failed to write ASR config: {}", path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flywheel_cycle() {
        assert_eq!(
            FlywheelStage::SkillExecution.next(),
            FlywheelStage::FridayReview
        );
        assert_eq!(
            FlywheelStage::FridayReview.next(),
            FlywheelStage::WeeklySprint
        );
        assert_eq!(
            FlywheelStage::WeeklySprint.next(),
            FlywheelStage::SkillExecution
        );
    }

    #[test]
    fn test_dod_completion() {
        let mut dod = DoDChecklist::default();
        assert!(!dod.is_complete());
        assert_eq!(dod.completion_percentage(), 0.0);

        for item in &mut dod.items {
            item.completed = true;
        }
        assert!(dod.is_complete());
        assert_eq!(dod.completion_percentage(), 100.0);
    }

    #[test]
    fn test_default_config() {
        let config = AsrConfig::default();
        assert_eq!(config.routing.confidence_threshold, 0.8);
        assert_eq!(config.min_smst_score, 75);
        assert_eq!(config.dod.items.len(), 6);
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = AsrConfig::default();
        let toml = toml::to_string_pretty(&config).unwrap();
        let parsed: AsrConfig = toml::from_str(&toml).unwrap();
        assert_eq!(
            parsed.routing.confidence_threshold,
            config.routing.confidence_threshold
        );
    }
}
