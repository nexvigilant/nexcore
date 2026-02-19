//! Learning Loops Implementation
//!
//! Single, double, and triple loop learning based on Argyris & Schön.

use serde::{Deserialize, Serialize};

/// Type of learning loop
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopType {
    /// Single-loop: Error → Fix → Continue
    Single,
    /// Double-loop: Error → Question assumption → Revise model
    Double,
    /// Triple-loop: Pattern → Question methodology → Evolve
    Triple,
}

impl LoopType {
    /// Returns the core question for this loop type
    pub fn question(&self) -> &'static str {
        match self {
            Self::Single => "Did we execute correctly?",
            Self::Double => "Did we build the right thing?",
            Self::Triple => "Are we learning correctly?",
        }
    }

    /// Returns the trigger condition
    pub fn trigger(&self) -> &'static str {
        match self {
            Self::Single => "Any node failure",
            Self::Double => "Phase 4 entry OR failure rate > 20%",
            Self::Triple => "Every 5th execution",
        }
    }
}

/// A pattern extracted from execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Unique name
    pub name: String,
    /// When this pattern applies
    pub trigger: String,
    /// What action to take
    pub action: String,
    /// ConfidenceLevel level
    pub confidence: ConfidenceLevel,
    /// Number of times observed
    pub observations: u32,
    /// Timestamp of last observation
    pub last_seen: f64,
}

impl Pattern {
    /// Creates a new pattern
    pub fn new(name: &str, trigger: &str, action: &str) -> Self {
        Self {
            name: name.to_string(),
            trigger: trigger.to_string(),
            action: action.to_string(),
            confidence: ConfidenceLevel::Low,
            observations: 1,
            last_seen: crate::ctvp::now(),
        }
    }

    /// Increases observation count and updates confidence
    pub fn observe(&mut self) {
        self.observations += 1;
        self.last_seen = crate::ctvp::now();
        self.confidence = match self.observations {
            1..=2 => ConfidenceLevel::Low,
            3..=5 => ConfidenceLevel::Medium,
            _ => ConfidenceLevel::High,
        };
    }
}

/// ConfidenceLevel level for patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// Observed 1-2 times
    Low,
    /// Observed 3-5 times
    Medium,
    /// Observed 6+ times
    High,
}

impl ConfidenceLevel {
    /// Returns emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Low => "🔵",
            Self::Medium => "🟡",
            Self::High => "🟢",
        }
    }
}

/// Result of a learning loop execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopResult {
    /// Type of loop executed
    pub loop_type: LoopType,
    /// Assumptions examined (for double/triple)
    pub assumptions_examined: Vec<Assumption>,
    /// Patterns extracted
    pub patterns: Vec<Pattern>,
    /// Model revisions proposed
    pub revisions: Vec<String>,
    /// Whether methodology change is recommended
    pub methodology_change: bool,
}

impl LoopResult {
    /// Creates a new single-loop result
    pub fn single(fix_applied: &str, success: bool) -> Self {
        Self {
            loop_type: LoopType::Single,
            assumptions_examined: Vec::new(),
            patterns: Vec::new(),
            revisions: vec![format!(
                "Fix: {} ({})",
                fix_applied,
                if success { "succeeded" } else { "failed" }
            )],
            methodology_change: false,
        }
    }

    /// Creates a new double-loop result
    pub fn double(assumptions: Vec<Assumption>, patterns: Vec<Pattern>) -> Self {
        let revisions: Vec<String> = assumptions
            .iter()
            .filter(|a| !a.valid)
            .map(|a| a.revision.clone())
            .collect();

        Self {
            loop_type: LoopType::Double,
            assumptions_examined: assumptions,
            patterns,
            revisions,
            methodology_change: false,
        }
    }

    /// Creates a new triple-loop result
    pub fn triple(methodology_change: bool, reason: &str) -> Self {
        Self {
            loop_type: LoopType::Triple,
            assumptions_examined: Vec::new(),
            patterns: Vec::new(),
            revisions: vec![reason.to_string()],
            methodology_change,
        }
    }
}

/// An assumption examined during double-loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assumption {
    /// Category of assumption
    pub category: AssumptionCategory,
    /// What was assumed
    pub statement: String,
    /// Evidence of what actually happened
    pub evidence: String,
    /// Whether the assumption was valid
    pub valid: bool,
    /// Proposed revision if invalid
    pub revision: String,
}

impl Assumption {
    /// Creates a new assumption
    pub fn new(category: AssumptionCategory, statement: &str) -> Self {
        Self {
            category,
            statement: statement.to_string(),
            evidence: String::new(),
            valid: true,
            revision: String::new(),
        }
    }

    /// Sets the evidence
    pub fn with_evidence(mut self, evidence: &str) -> Self {
        self.evidence = evidence.to_string();
        self
    }

    /// Marks as invalid with revision
    pub fn invalidate(mut self, revision: &str) -> Self {
        self.valid = false;
        self.revision = revision.to_string();
        self
    }
}

/// Categories of assumptions to examine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssumptionCategory {
    /// SMART transformation assumptions
    SmartTransformation,
    /// DAG structure assumptions
    DagGranularity,
    /// Parallel grouping assumptions
    ParallelGroups,
    /// Evidence threshold assumptions
    EvidenceThresholds,
    /// Execution strategy assumptions
    ExecutionStrategy,
}

impl AssumptionCategory {
    /// Returns all categories
    pub fn all() -> [AssumptionCategory; 5] {
        [
            Self::SmartTransformation,
            Self::DagGranularity,
            Self::ParallelGroups,
            Self::EvidenceThresholds,
            Self::ExecutionStrategy,
        ]
    }

    /// Returns the default question for this category
    pub fn question(&self) -> &'static str {
        match self {
            Self::SmartTransformation => "Was SMART transformation optimal?",
            Self::DagGranularity => "Was DAG granularity correct?",
            Self::ParallelGroups => "Were parallel groups accurate?",
            Self::EvidenceThresholds => "Were evidence thresholds calibrated?",
            Self::ExecutionStrategy => "Was execution strategy effective?",
        }
    }
}

/// Learning loop executor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningLoop {
    /// Execution count (for triple-loop triggering)
    pub execution_count: u32,
    /// Last triple-loop timestamp
    pub last_triple_loop: Option<f64>,
    /// Accumulated patterns
    pub patterns: Vec<Pattern>,
    /// Pattern application count
    pub patterns_applied: u32,
}

impl LearningLoop {
    /// Creates a new learning loop tracker
    pub fn new() -> Self {
        Self {
            execution_count: 0,
            last_triple_loop: None,
            patterns: Vec::new(),
            patterns_applied: 0,
        }
    }

    /// Records an execution
    pub fn record_execution(&mut self) {
        self.execution_count += 1;
    }

    /// Returns true if triple-loop should be triggered
    pub fn should_trigger_triple(&self) -> bool {
        self.execution_count > 0 && self.execution_count % 5 == 0
    }

    /// Returns true if double-loop should be triggered based on failure rate
    pub fn should_trigger_double(&self, failure_rate: f64) -> bool {
        failure_rate > crate::vdag::DOUBLE_LOOP_FAILURE_RATE
    }

    /// Adds a pattern (or updates if exists)
    pub fn add_pattern(&mut self, pattern: Pattern) {
        if let Some(existing) = self.patterns.iter_mut().find(|p| p.name == pattern.name) {
            existing.observe();
        } else {
            self.patterns.push(pattern);
        }
    }

    /// Records a pattern application
    pub fn record_application(&mut self) {
        self.patterns_applied += 1;
    }

    /// Returns matching patterns for a given context
    pub fn matching_patterns(&self, context: &str) -> Vec<&Pattern> {
        self.patterns
            .iter()
            .filter(|p| context.contains(&p.trigger) || p.trigger.contains(context))
            .collect()
    }

    /// Generates a status report
    pub fn report(&self) -> String {
        let mut r = String::new();
        r.push_str("\n╔══════════════════════════════════════════════════════╗\n");
        r.push_str("║  🔄 LEARNING LOOP STATUS                              ║\n");
        r.push_str("╠══════════════════════════════════════════════════════╣\n");
        r.push_str(&format!(
            "║  Executions: {}                                       ║\n",
            self.execution_count
        ));
        r.push_str(&format!(
            "║  Patterns extracted: {}                               ║\n",
            self.patterns.len()
        ));
        r.push_str(&format!(
            "║  Patterns applied: {}                                 ║\n",
            self.patterns_applied
        ));
        r.push_str(&format!(
            "║  Next triple-loop: execution #{}                      ║\n",
            ((self.execution_count / 5) + 1) * 5
        ));
        r.push_str("╚══════════════════════════════════════════════════════╝\n");
        r
    }
}

impl Default for LearningLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triple_loop_trigger() {
        let mut ll = LearningLoop::new();

        for i in 1..=10 {
            ll.record_execution();
            if i % 5 == 0 {
                assert!(ll.should_trigger_triple(), "Should trigger at {}", i);
            } else {
                assert!(!ll.should_trigger_triple(), "Should not trigger at {}", i);
            }
        }
    }

    #[test]
    fn test_double_loop_trigger() {
        let ll = LearningLoop::new();

        assert!(!ll.should_trigger_double(0.15));
        assert!(ll.should_trigger_double(0.25));
    }

    #[test]
    fn test_pattern_observation() {
        let mut pattern = Pattern::new("test", "trigger", "action");
        // Initial observation count is 1
        assert_eq!(pattern.confidence, ConfidenceLevel::Low);
        assert_eq!(pattern.observations, 1);

        pattern.observe(); // Now 2
        assert_eq!(pattern.confidence, ConfidenceLevel::Low);

        pattern.observe(); // Now 3
        assert_eq!(pattern.confidence, ConfidenceLevel::Medium);

        for _ in 0..4 {
            pattern.observe(); // Now 7
        }
        assert_eq!(pattern.confidence, ConfidenceLevel::High);
    }
}
