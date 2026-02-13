//! VDAG Pipeline Execution
//!
//! Orchestrates the 5-phase pipeline with state management.

use super::learning::LearningLoop;
use super::node::{Node, NodeId, NodeStatus};
use super::reality::RealityGradient;
use super::smart::SmartGoal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pipeline phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelinePhase {
    /// Phase 1: Goal Validation
    GoalValidation = 1,
    /// Phase 2: DAG Generation
    DagGeneration = 2,
    /// Phase 3: Execution
    Execution = 3,
    /// Phase 4: Reflection
    Reflection = 4,
    /// Phase 5: Socialization
    Socialization = 5,
}

impl PipelinePhase {
    /// Returns all phases in order
    pub fn all() -> [PipelinePhase; 5] {
        [
            Self::GoalValidation,
            Self::DagGeneration,
            Self::Execution,
            Self::Reflection,
            Self::Socialization,
        ]
    }

    /// Returns the next phase
    pub fn next(&self) -> Option<PipelinePhase> {
        match self {
            Self::GoalValidation => Some(Self::DagGeneration),
            Self::DagGeneration => Some(Self::Execution),
            Self::Execution => Some(Self::Reflection),
            Self::Reflection => Some(Self::Socialization),
            Self::Socialization => None,
        }
    }

    /// Returns the CEP stages for this phase
    pub fn cep_stages(&self) -> &'static str {
        match self {
            Self::GoalValidation => "SEE + SPEAK",
            Self::DagGeneration => "DECOMPOSE + COMPOSE",
            Self::Execution => "VALIDATE + DEPLOY",
            Self::Reflection => "REFLECT + EVOLVE",
            Self::Socialization => "SOCIALIZE",
        }
    }

    /// Returns the SECI quadrant for this phase
    pub fn seci_quadrant(&self) -> &'static str {
        match self {
            Self::GoalValidation => "S→E (Externalization)",
            Self::DagGeneration => "E→E (Combination)",
            Self::Execution => "E→T (Internalization)",
            Self::Reflection => "T→T (Internal learning)",
            Self::Socialization => "T→S (Socialization)",
        }
    }
}

/// Pipeline execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineState {
    /// Current phase
    pub phase: PipelinePhase,
    /// Current node being executed (if any)
    pub current_node: Option<NodeId>,
    /// Completed node IDs
    pub completed_nodes: Vec<NodeId>,
    /// Failed node IDs
    pub failed_nodes: Vec<NodeId>,
    /// Whether pipeline is paused
    pub paused: bool,
    /// Timestamp of last update
    pub updated_at: f64,
}

impl PipelineState {
    /// Creates initial state
    pub fn new() -> Self {
        Self {
            phase: PipelinePhase::GoalValidation,
            current_node: None,
            completed_nodes: Vec::new(),
            failed_nodes: Vec::new(),
            paused: false,
            updated_at: crate::ctvp::now(),
        }
    }

    /// Returns failure rate
    pub fn failure_rate(&self) -> f64 {
        let total = self.completed_nodes.len() + self.failed_nodes.len();
        if total == 0 {
            0.0
        } else {
            self.failed_nodes.len() as f64 / total as f64
        }
    }

    /// Advances to next phase
    pub fn advance(&mut self) -> bool {
        if let Some(next) = self.phase.next() {
            self.phase = next;
            self.updated_at = crate::ctvp::now();
            true
        } else {
            false
        }
    }
}

impl Default for PipelineState {
    fn default() -> Self {
        Self::new()
    }
}

/// The VDAG Pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    /// Pipeline ID
    pub id: String,
    /// The validated goal
    pub goal: SmartGoal,
    /// All nodes in the DAG
    pub nodes: HashMap<NodeId, Node>,
    /// Execution order (topologically sorted)
    pub execution_order: Vec<NodeId>,
    /// Parallel groups (level -> nodes)
    pub parallel_groups: HashMap<u32, Vec<NodeId>>,
    /// Current state
    pub state: PipelineState,
    /// Reality gradient
    pub reality_gradient: RealityGradient,
    /// Learning loop tracker
    pub learning: LearningLoop,
}

impl Pipeline {
    /// Creates a new pipeline from a validated goal
    pub fn from_goal(goal: SmartGoal) -> Result<Self, PipelineError> {
        if !goal.is_valid() {
            return Err(PipelineError::InvalidGoal(
                "Goal validation failed".to_string(),
            ));
        }

        Ok(Self {
            id: uuid(),
            goal,
            nodes: HashMap::new(),
            execution_order: Vec::new(),
            parallel_groups: HashMap::new(),
            state: PipelineState::new(),
            reality_gradient: RealityGradient::default(),
            learning: LearningLoop::new(),
        })
    }

    /// Adds a node to the pipeline
    pub fn add_node(&mut self, node: Node) {
        let group = node.parallel_group;
        let id = node.id.clone();

        self.nodes.insert(id.clone(), node);

        self.parallel_groups.entry(group).or_default().push(id);
    }

    /// Computes execution order (topological sort using Kahn's algorithm)
    pub fn compute_order(&mut self) -> Result<(), PipelineError> {
        use std::collections::{HashMap, VecDeque};

        // Build in-degree map
        let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
        for id in self.nodes.keys() {
            in_degree.insert(id.clone(), 0);
        }

        for node in self.nodes.values() {
            for dep in &node.depends_on {
                if !self.nodes.contains_key(dep) {
                    return Err(PipelineError::NodeNotFound(dep.clone()));
                }
            }
            // This node depends on others, so increment in-degree for this node per dependency
            in_degree.insert(node.id.clone(), node.depends_on.len());
        }

        // Queue nodes with no dependencies
        let mut queue: VecDeque<NodeId> = in_degree
            .iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut order = Vec::new();

        while let Some(id) = queue.pop_front() {
            order.push(id.clone());

            // For each node that depends on this one, decrease its in-degree
            for node in self.nodes.values() {
                if node.depends_on.contains(&id) {
                    if let Some(deg) = in_degree.get_mut(&node.id) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(node.id.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if order.len() != self.nodes.len() {
            // Find a node that's part of a cycle
            for (id, &deg) in &in_degree {
                if deg > 0 {
                    return Err(PipelineError::CyclicDependency(id.clone()));
                }
            }
        }

        self.execution_order = order;
        Ok(())
    }

    #[allow(dead_code)] // Topological sort helper - may be needed for DAG validation
    fn visit(
        &self,
        id: &NodeId,
        visited: &mut std::collections::HashSet<NodeId>,
        temp: &mut std::collections::HashSet<NodeId>,
        order: &mut Vec<NodeId>,
    ) -> Result<(), PipelineError> {
        if temp.contains(id) {
            return Err(PipelineError::CyclicDependency(id.clone()));
        }
        if visited.contains(id) {
            return Ok(());
        }

        temp.insert(id.clone());

        if let Some(node) = self.nodes.get(id) {
            for dep in &node.depends_on {
                self.visit(dep, visited, temp, order)?;
            }
        }

        temp.remove(id);
        visited.insert(id.clone());
        order.push(id.clone());
        Ok(())
    }

    /// Returns nodes ready to execute
    pub fn ready_nodes(&self) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|n| n.is_pending() && n.can_execute(&self.state.completed_nodes))
            .collect()
    }

    /// Marks a node as completed
    pub fn complete_node(&mut self, id: &NodeId, output: String, duration_ms: u64) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.status = NodeStatus::Completed {
                output,
                duration_ms,
            };
            self.state.completed_nodes.push(id.clone());
            self.state.current_node = None;
            self.state.updated_at = crate::ctvp::now();
        }
    }

    /// Marks a node as failed
    pub fn fail_node(&mut self, id: &NodeId, error: String, retries: u32) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.status = NodeStatus::Failed { error, retries };
            self.state.failed_nodes.push(id.clone());
            self.state.current_node = None;
            self.state.updated_at = crate::ctvp::now();
        }
    }

    /// Returns true if all nodes are complete
    pub fn is_complete(&self) -> bool {
        self.nodes.values().all(|n| n.status.is_complete())
    }

    /// Returns true if pipeline is blocked by reality gradient
    pub fn is_blocked(&self) -> bool {
        self.reality_gradient.is_blocked()
    }

    /// Generates execution report
    pub fn report(&self) -> String {
        let mut r = String::new();
        r.push_str("\n╔══════════════════════════════════════════════════════╗\n");
        r.push_str("║  📋 VDAG PIPELINE STATUS                              ║\n");
        r.push_str("╠══════════════════════════════════════════════════════╣\n");
        r.push_str(&format!(
            "║  ID: {}                              ║\n",
            &self.id[..8]
        ));
        r.push_str(&format!(
            "║  Phase: {:?}                                    ║\n",
            self.state.phase
        ));
        r.push_str(&format!(
            "║  Nodes: {}/{}                                        ║\n",
            self.state.completed_nodes.len(),
            self.nodes.len()
        ));
        r.push_str(&format!(
            "║  Failed: {}                                          ║\n",
            self.state.failed_nodes.len()
        ));
        r.push_str(&format!(
            "║  Reality Score: {:.2}                               ║\n",
            self.reality_gradient.score
        ));
        r.push_str("╠══════════════════════════════════════════════════════╣\n");

        // Node status
        for id in &self.execution_order {
            if let Some(node) = self.nodes.get(id) {
                r.push_str(&format!(
                    "║  {} {} {:<40} ║\n",
                    node.status.emoji(),
                    id,
                    truncate(&node.name, 40)
                ));
            }
        }

        r.push_str("╚══════════════════════════════════════════════════════╝\n");
        r
    }
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Pipeline ID
    pub pipeline_id: String,
    /// Final reality gradient
    pub reality_gradient: RealityGradient,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Number of nodes executed
    pub nodes_executed: usize,
    /// Number of nodes failed
    pub nodes_failed: usize,
    /// Whether execution was blocked
    pub blocked: bool,
    /// Learning loop state
    pub learning: LearningLoop,
}

impl ExecutionResult {
    /// Returns the reality score
    pub fn reality_score(&self) -> f64 {
        self.reality_gradient.score
    }
}

/// Pipeline error
#[derive(Debug, Clone)]
pub enum PipelineError {
    /// Goal validation failed
    InvalidGoal(String),
    /// Cyclic dependency detected
    CyclicDependency(NodeId),
    /// Node not found
    NodeNotFound(NodeId),
    /// Execution blocked by reality gradient
    Blocked(f64),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidGoal(msg) => write!(f, "Invalid goal: {}", msg),
            Self::CyclicDependency(id) => write!(f, "Cyclic dependency at node: {}", id),
            Self::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            Self::Blocked(score) => write!(f, "Blocked by Reality Gradient: {:.2}", score),
        }
    }
}

impl std::error::Error for PipelineError {}

/// Generates a UUID using nexcore-id
fn uuid() -> String {
    nexcore_id::NexId::v4().to_string()
}

/// Truncates a string to max length
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vdag::smart::SmartGoal;

    fn test_goal() -> SmartGoal {
        SmartGoal::builder()
            .raw("Test")
            .specific("Do X")
            .measurable("100%")
            .achievable("Yes")
            .relevant("Required")
            .time_bound("1 week")
            .build()
            .expect("should build")
    }

    #[test]
    fn test_pipeline_creation() {
        let goal = test_goal();
        let pipeline = Pipeline::from_goal(goal).expect("should create");

        assert_eq!(pipeline.state.phase, PipelinePhase::GoalValidation);
        assert!(pipeline.nodes.is_empty());
    }

    #[test]
    fn test_add_nodes() {
        let goal = test_goal();
        let mut pipeline = Pipeline::from_goal(goal).expect("should create");

        let node1 = Node::builder("N001").name("First").build();
        let node2 = Node::builder("N002")
            .name("Second")
            .depends_on("N001")
            .build();

        pipeline.add_node(node1);
        pipeline.add_node(node2);

        assert_eq!(pipeline.nodes.len(), 2);
    }

    #[test]
    fn test_topological_order() {
        let goal = test_goal();
        let mut pipeline = Pipeline::from_goal(goal).expect("should create");

        let node1 = Node::builder("N001").name("First").build();
        let node2 = Node::builder("N002").depends_on("N001").build();
        let node3 = Node::builder("N003").depends_on("N001").build();
        let node4 = Node::builder("N004")
            .depends_on("N002")
            .depends_on("N003")
            .build();

        pipeline.add_node(node1);
        pipeline.add_node(node2);
        pipeline.add_node(node3);
        pipeline.add_node(node4);

        pipeline.compute_order().expect("should compute");

        // N001 must come before N002, N003
        // N002 and N003 must come before N004
        let pos = |id: &str| {
            pipeline
                .execution_order
                .iter()
                .position(|x| x == id)
                .unwrap()
        };
        assert!(pos("N001") < pos("N002"));
        assert!(pos("N001") < pos("N003"));
        assert!(pos("N002") < pos("N004"));
        assert!(pos("N003") < pos("N004"));
    }

    #[test]
    fn test_cyclic_detection() {
        let goal = test_goal();
        let mut pipeline = Pipeline::from_goal(goal).expect("should create");

        let node1 = Node::builder("N001").depends_on("N002").build();
        let node2 = Node::builder("N002").depends_on("N001").build();

        pipeline.add_node(node1);
        pipeline.add_node(node2);

        let result = pipeline.compute_order();
        assert!(matches!(result, Err(PipelineError::CyclicDependency(_))));
    }

    #[test]
    fn test_phase_advancement() {
        let mut state = PipelineState::new();

        assert_eq!(state.phase, PipelinePhase::GoalValidation);
        assert!(state.advance());
        assert_eq!(state.phase, PipelinePhase::DagGeneration);
        assert!(state.advance());
        assert_eq!(state.phase, PipelinePhase::Execution);
        assert!(state.advance());
        assert_eq!(state.phase, PipelinePhase::Reflection);
        assert!(state.advance());
        assert_eq!(state.phase, PipelinePhase::Socialization);
        assert!(!state.advance()); // No more phases
    }
}
