//! # Standard Operating Procedures (SOP)
//!
//! SMART goal framework and SOP models for deterministic skill execution.
//!
//! ## SMART Goals
//!
//! - **S**pecific: Clear outcome with defined scope
//! - **M**easurable: Quantifiable metrics and tracking
//! - **A**chievable: Realistic targets with enablers/blockers
//! - **R**elevant: Aligned with strategic objectives
//! - **T**ime-bound: Clear deadlines and milestones

use nexcore_chrono::Date;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Impact category for goal relevance assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactCategory {
    /// Patient safety and clinical outcomes
    PatientSafety,
    /// Regulatory compliance requirements
    RegulatoryCompliance,
    /// Business continuity and operations
    BusinessContinuity,
    /// Efficiency and cost optimization
    EfficiencyCost,
    /// Data integrity and quality
    DataIntegrity,
    /// Stakeholder trust and reputation
    StakeholderTrust,
}

/// Source of target/benchmark values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetSource {
    /// Regulatory requirement
    Regulatory,
    /// Industry benchmark
    IndustryBenchmark,
    /// Baseline improvement target
    BaselineImprovement,
    /// Expert judgment
    ExpertJudgment,
}

/// Performance level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceLevel {
    /// Learning/beginner level
    Novice,
    /// Competent level
    Proficient,
    /// Mastery level
    Expert,
}

/// Type of metric measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    /// Percentage rate (0-100%)
    RatePercent,
    /// Count/frequency
    Count,
    /// Time duration
    Time,
    /// Composite score
    Score,
    /// Statistical measure (mean/median)
    MeanMedian,
    /// Percentile ranking
    Percentile,
}

/// Specific component of a SMART goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificComponent {
    /// Desired outcome statement
    pub outcome: String,
    /// Scope and boundary definition
    pub scope_boundary: String,
    /// Quality threshold criteria
    pub quality_threshold: String,
    /// Explicit exclusions
    pub exclusions: Vec<String>,
}

/// Metric specification for measurability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSpecification {
    /// Metric name
    pub name: String,
    /// Metric definition
    pub definition: String,
    /// Calculation formula
    pub formula: String,
    /// Type of measurement
    pub metric_type: MetricType,
    /// Data source
    pub data_source: String,
    /// Measurement frequency
    pub frequency: String,
    /// Responsible owner
    pub owner: String,
    /// Baseline value (if available)
    pub baseline: Option<String>,
    /// Target value
    pub target: String,
    /// Escalation threshold
    pub escalation_threshold: Option<String>,
}

/// Measurable component of a SMART goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurableComponent {
    /// Primary metric for goal measurement
    pub primary_metric: MetricSpecification,
    /// Secondary/supporting metrics
    pub secondary_metrics: Vec<MetricSpecification>,
    /// Method for tracking progress
    pub tracking_method: String,
}

/// Achievable component of a SMART goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievableComponent {
    /// Source of target value
    pub target_source: TargetSource,
    /// Justification for target
    pub target_justification: String,
    /// Factors enabling success
    pub enablers: Vec<String>,
    /// Potential blockers
    pub blockers: Vec<String>,
    /// Mitigation strategies
    pub mitigations: Vec<String>,
    /// Confidence level (0-100%)
    pub confidence_percent: u8,
}

/// Relevant component of a SMART goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevantComponent {
    /// Impact category
    pub impact_category: ImpactCategory,
    /// Impact statement
    pub impact_statement: String,
    /// Strategic alignment
    pub strategic_alignment: String,
    /// Opportunity cost (if applicable)
    pub opportunity_cost: Option<String>,
}

/// Milestone for time-bound tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    /// Checkpoint description
    pub checkpoint: String,
    /// Target date
    pub target_date: Date,
    /// Percentage complete at this milestone
    pub percentage_complete: u8,
}

/// Time-bound component of a SMART goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBoundComponent {
    /// Final deadline
    pub deadline: Date,
    /// Operational target date (if different)
    pub operational_target: Option<Date>,
    /// Date to trigger escalation
    pub escalation_trigger_date: Option<Date>,
    /// Intermediate milestones
    pub milestones: Vec<Milestone>,
    /// Event that starts the clock
    pub clock_start_event: String,
}

/// Complete SMART goal specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartGoal {
    /// Goal statement (one sentence)
    pub statement: String,
    /// Specific: What, who, where, why
    pub specific: SpecificComponent,
    /// Measurable: Metrics and tracking
    pub measurable: MeasurableComponent,
    /// Achievable: Feasibility assessment
    pub achievable: AchievableComponent,
    /// Relevant: Strategic alignment
    pub relevant: RelevantComponent,
    /// Time-bound: Deadlines and milestones
    pub time_bound: TimeBoundComponent,
    /// Explicit anti-goals (what NOT to do)
    pub anti_goals: Vec<String>,
    /// Leading indicators for early warning
    pub leading_indicators: Vec<String>,
}

/// Standard Operating Procedure with SMART goals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartSop {
    /// Unique SOP identifier
    pub sop_id: String,
    /// SOP title
    pub title: String,
    /// Version string
    pub version: String,
    /// Effective date
    pub effective_date: Date,
    /// Purpose statement
    pub purpose: String,
    /// Scope description
    pub scope: String,
    /// Role-to-responsibility mapping
    pub responsibilities: HashMap<String, String>,
    /// Primary SMART goal
    pub smart_goal: SmartGoal,
    /// Procedure-level goals
    pub procedure_goals: Vec<SmartGoal>,
    /// Document author
    pub author: String,
    /// Approver (if applicable)
    pub approver: Option<String>,
    /// Review cycle in months
    pub review_cycle_months: u32,
    /// Related document references
    pub related_documents: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impact_category_serialization() {
        let category = ImpactCategory::PatientSafety;
        let json = serde_json::to_string(&category).unwrap();
        assert_eq!(json, "\"patient_safety\"");
    }

    #[test]
    fn test_metric_type_variants() {
        assert_eq!(
            serde_json::to_string(&MetricType::RatePercent).unwrap(),
            "\"rate_percent\""
        );
        assert_eq!(
            serde_json::to_string(&MetricType::Count).unwrap(),
            "\"count\""
        );
    }

    #[test]
    fn test_performance_level_ordering() {
        // Verify enum variants exist
        let _ = PerformanceLevel::Novice;
        let _ = PerformanceLevel::Proficient;
        let _ = PerformanceLevel::Expert;
    }
}
