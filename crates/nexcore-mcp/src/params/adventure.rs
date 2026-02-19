//! Adventure HUD Parameters (Experience Tracking)
//!
//! Adventures, tasks, skills, measures, and milestones.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for starting an adventure.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AdventureStartParams {
    /// Adventure name.
    pub name: String,
}

/// Parameters for tracking a task.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AdventureTaskParams {
    /// Task ID.
    pub id: String,
    /// Description.
    pub subject: String,
    /// Status.
    pub status: String,
}

/// Parameters for logging skill usage.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AdventureSkillParams {
    /// Skill name.
    pub skill: String,
}

/// Parameters for tracking a metric.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AdventureMeasureParams {
    /// Metric name.
    pub name: String,
    /// Metric value.
    pub value: f64,
}

/// Parameters for recording a milestone.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AdventureMilestoneParams {
    /// Milestone description.
    pub milestone: String,
}
