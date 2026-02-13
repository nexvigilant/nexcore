//! SMART Goal Validation
//!
//! Implements the 35-variable SMART goal framework with validation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SMART dimension categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SmartDimension {
    /// Specific: What exactly will be done?
    Specific,
    /// Measurable: How will success be quantified?
    Measurable,
    /// Achievable: Is this realistic?
    Achievable,
    /// Relevant: Why does this matter?
    Relevant,
    /// TimeBound: What is the deadline?
    TimeBound,
}

impl SmartDimension {
    /// Returns all dimensions
    pub fn all() -> [SmartDimension; 5] {
        [
            Self::Specific,
            Self::Measurable,
            Self::Achievable,
            Self::Relevant,
            Self::TimeBound,
        ]
    }

    /// Returns the variable prefix for this dimension
    pub fn variable_prefix(&self) -> &'static str {
        match self {
            Self::Specific => "V00",
            Self::Measurable => "V01",
            Self::Achievable => "V02",
            Self::Relevant => "V03",
            Self::TimeBound => "V04",
        }
    }
}

/// The 7 layers of goal achievement (35 variables total)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Layer {
    /// Layer 0: Quality (SMART dimensions)
    Quality = 0,
    /// Layer 1: Resources (time, money, skill, tool, energy)
    Resource = 1,
    /// Layer 2: Psychology (motivation, confidence, fear, habit, identity)
    Psychology = 2,
    /// Layer 3: Strategy (plan, milestone, feedback, contingency, priority)
    Strategy = 3,
    /// Layer 4: Dependencies (prereq, blocker, approval, sequence, focus)
    Dependencies = 4,
    /// Layer 5: Environment (support, competition, timing, space, accountability)
    Environment = 5,
    /// Layer 6: Knowledge (domain, execution, network, gap, learning)
    Knowledge = 6,
}

/// A single variable in the 35-variable framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    /// Variable ID (e.g., "V00", "V12")
    pub id: String,
    /// Layer this belongs to
    pub layer: u8,
    /// Position within layer (0-4)
    pub position: u8,
    /// Name of the variable
    pub name: String,
    /// Current status
    pub status: bool,
    /// Notes/evidence
    pub notes: String,
}

impl Variable {
    /// Creates a new variable
    pub fn new(layer: u8, position: u8, name: &str) -> Self {
        Self {
            id: format!("V{}{}", layer, position),
            layer,
            position,
            name: name.to_string(),
            status: false,
            notes: String::new(),
        }
    }

    /// Sets the status
    pub fn with_status(mut self, status: bool) -> Self {
        self.status = status;
        self
    }

    /// Sets notes
    pub fn with_notes(mut self, notes: &str) -> Self {
        self.notes = notes.to_string();
        self
    }
}

/// A validated SMART goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartGoal {
    /// Original raw goal statement
    pub raw: String,
    /// Specific dimension
    pub specific: String,
    /// Measurable dimension
    pub measurable: String,
    /// Achievable dimension
    pub achievable: String,
    /// Relevant dimension
    pub relevant: String,
    /// Time-bound dimension
    pub time_bound: String,
    /// All 35 variables
    pub variables: HashMap<String, Variable>,
    /// Validation result
    pub validation: SmartValidation,
}

impl SmartGoal {
    /// Creates a builder for a new goal
    pub fn builder() -> SmartGoalBuilder {
        SmartGoalBuilder::new()
    }

    /// Returns true if all SMART dimensions are valid
    pub fn is_valid(&self) -> bool {
        self.validation.is_valid
    }

    /// Returns the blocking variables (FALSE status that blocks execution)
    pub fn blockers(&self) -> Vec<&Variable> {
        self.variables
            .values()
            .filter(|v| !v.status && is_critical_variable(&v.id))
            .collect()
    }

    /// Returns formatted SMART statement
    pub fn formatted(&self) -> String {
        format!(
            "SMART Goal:\n\
             - Specific: {}\n\
             - Measurable: {}\n\
             - Achievable: {}\n\
             - Relevant: {}\n\
             - Time-bound: {}",
            self.specific, self.measurable, self.achievable, self.relevant, self.time_bound
        )
    }
}

/// Builder for SmartGoal
#[derive(Debug, Default)]
pub struct SmartGoalBuilder {
    raw: Option<String>,
    specific: Option<String>,
    measurable: Option<String>,
    achievable: Option<String>,
    relevant: Option<String>,
    time_bound: Option<String>,
    variables: HashMap<String, Variable>,
}

impl SmartGoalBuilder {
    /// Creates a new builder
    pub fn new() -> Self {
        let mut builder = Self::default();
        builder.initialize_variables();
        builder
    }

    fn initialize_variables(&mut self) {
        // Layer 0: Quality (SMART)
        let layer0 = [
            "specific",
            "measurable",
            "achievable",
            "relevant",
            "time_bound",
        ];
        for (i, name) in layer0.iter().enumerate() {
            let v = Variable::new(0, i as u8, name);
            self.variables.insert(v.id.clone(), v);
        }

        // Layer 1: Resource
        let layer1 = ["time", "money", "skill", "tool", "energy"];
        for (i, name) in layer1.iter().enumerate() {
            let v = Variable::new(1, i as u8, name);
            self.variables.insert(v.id.clone(), v);
        }

        // Layer 2: Psychology
        let layer2 = ["motivation", "confidence", "fear", "habit", "identity"];
        for (i, name) in layer2.iter().enumerate() {
            let v = Variable::new(2, i as u8, name);
            self.variables.insert(v.id.clone(), v);
        }

        // Layer 3: Strategy
        let layer3 = ["plan", "milestone", "feedback", "contingency", "priority"];
        for (i, name) in layer3.iter().enumerate() {
            let v = Variable::new(3, i as u8, name);
            self.variables.insert(v.id.clone(), v);
        }

        // Layer 4: Dependencies
        let layer4 = ["prereq", "blocker", "approval", "sequence", "focus"];
        for (i, name) in layer4.iter().enumerate() {
            let v = Variable::new(4, i as u8, name);
            self.variables.insert(v.id.clone(), v);
        }

        // Layer 5: Environment
        let layer5 = [
            "support",
            "competition",
            "timing",
            "space",
            "accountability",
        ];
        for (i, name) in layer5.iter().enumerate() {
            let v = Variable::new(5, i as u8, name);
            self.variables.insert(v.id.clone(), v);
        }

        // Layer 6: Knowledge
        let layer6 = ["domain", "execution", "network", "gap", "learning"];
        for (i, name) in layer6.iter().enumerate() {
            let v = Variable::new(6, i as u8, name);
            self.variables.insert(v.id.clone(), v);
        }
    }

    /// Sets the raw goal
    pub fn raw(mut self, raw: &str) -> Self {
        self.raw = Some(raw.to_string());
        self
    }

    /// Sets the specific dimension
    pub fn specific(mut self, specific: &str) -> Self {
        self.specific = Some(specific.to_string());
        if let Some(v) = self.variables.get_mut("V00") {
            v.status = !specific.is_empty();
            v.notes = specific.to_string();
        }
        self
    }

    /// Sets the measurable dimension
    pub fn measurable(mut self, measurable: &str) -> Self {
        self.measurable = Some(measurable.to_string());
        if let Some(v) = self.variables.get_mut("V01") {
            v.status = !measurable.is_empty();
            v.notes = measurable.to_string();
        }
        self
    }

    /// Sets the achievable dimension
    pub fn achievable(mut self, achievable: &str) -> Self {
        self.achievable = Some(achievable.to_string());
        if let Some(v) = self.variables.get_mut("V02") {
            v.status = !achievable.is_empty();
            v.notes = achievable.to_string();
        }
        self
    }

    /// Sets the relevant dimension
    pub fn relevant(mut self, relevant: &str) -> Self {
        self.relevant = Some(relevant.to_string());
        if let Some(v) = self.variables.get_mut("V03") {
            v.status = !relevant.to_string().is_empty();
            v.notes = relevant.to_string();
        }
        self
    }

    /// Sets the time_bound dimension
    pub fn time_bound(mut self, time_bound: &str) -> Self {
        self.time_bound = Some(time_bound.to_string());
        if let Some(v) = self.variables.get_mut("V04") {
            v.status = !time_bound.is_empty();
            v.notes = time_bound.to_string();
        }
        self
    }

    /// Sets a variable status
    pub fn set_variable(mut self, id: &str, status: bool, notes: &str) -> Self {
        if let Some(v) = self.variables.get_mut(id) {
            v.status = status;
            v.notes = notes.to_string();
        }
        self
    }

    /// Builds the SmartGoal
    pub fn build(self) -> Result<SmartGoal, SmartValidationError> {
        let raw = self.raw.unwrap_or_default();
        let specific = self.specific.unwrap_or_default();
        let measurable = self.measurable.unwrap_or_default();
        let achievable = self.achievable.unwrap_or_default();
        let relevant = self.relevant.unwrap_or_default();
        let time_bound = self.time_bound.unwrap_or_default();

        // Validate SMART dimensions
        let mut errors = Vec::new();
        if specific.is_empty() {
            errors.push("Specific dimension is required".to_string());
        }
        if measurable.is_empty() {
            errors.push("Measurable dimension is required".to_string());
        }
        if achievable.is_empty() {
            errors.push("Achievable dimension is required".to_string());
        }
        if relevant.is_empty() {
            errors.push("Relevant dimension is required".to_string());
        }
        if time_bound.is_empty() {
            errors.push("Time-bound dimension is required".to_string());
        }

        // Check for blockers (V41 = blocker, TRUE means no blocker)
        // By default, V41 is TRUE (no blocker present)
        let blockers: Vec<String> = self
            .variables
            .values()
            .filter(|v| v.id == "V41" && v.status) // V41=true means blocker IS present
            .map(|v| v.id.clone())
            .collect();

        let is_valid = errors.is_empty() && blockers.is_empty();

        let validation = SmartValidation {
            is_valid,
            errors,
            blockers,
            score: if is_valid { 1.0 } else { 0.0 },
        };

        Ok(SmartGoal {
            raw,
            specific,
            measurable,
            achievable,
            relevant,
            time_bound,
            variables: self.variables,
            validation,
        })
    }
}

/// SMART validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartValidation {
    /// Whether the goal is valid
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Blocking variables
    pub blockers: Vec<String>,
    /// Validation score (0.0 - 1.0)
    pub score: f64,
}

/// SMART validation error
#[derive(Debug, Clone)]
pub struct SmartValidationError {
    /// Error message
    pub message: String,
    /// Which variables failed
    pub failed_variables: Vec<String>,
}

impl std::fmt::Display for SmartValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SMART validation failed: {}", self.message)
    }
}

impl std::error::Error for SmartValidationError {}

/// Returns true if a variable is critical (blocks execution if FALSE)
fn is_critical_variable(id: &str) -> bool {
    matches!(
        id,
        "V00" | "V01" | "V02" | "V03" | "V04" | // SMART core
        "V12" | // skill
        "V21" | // confidence
        "V41" // blocker
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_goal_builder() {
        let goal = SmartGoal::builder()
            .raw("Test goal")
            .specific("Do X")
            .measurable("100% complete")
            .achievable("Have resources")
            .relevant("Needed for Y")
            .time_bound("1 week")
            .build()
            .expect("should build");

        assert!(goal.is_valid());
        assert_eq!(goal.specific, "Do X");
    }

    #[test]
    fn test_missing_dimension() {
        let goal = SmartGoal::builder()
            .raw("Test goal")
            .specific("Do X")
            // Missing measurable
            .achievable("Have resources")
            .relevant("Needed for Y")
            .time_bound("1 week")
            .build()
            .expect("should build with validation errors");

        assert!(!goal.is_valid());
        assert!(!goal.validation.errors.is_empty());
    }

    #[test]
    fn test_variable_initialization() {
        let goal = SmartGoal::builder()
            .raw("Test")
            .specific("X")
            .measurable("Y")
            .achievable("Z")
            .relevant("W")
            .time_bound("1d")
            .build()
            .expect("should build");

        // Should have 35 variables (7 layers × 5 each)
        assert_eq!(goal.variables.len(), 35);
    }
}
