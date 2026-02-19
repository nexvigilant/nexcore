//! DAG Node Definitions
//!
//! Defines nodes for the execution DAG with CTVP and CEP alignment.

use serde::{Deserialize, Serialize};

/// Unique node identifier
pub type NodeId = String;

/// Status of a node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Not yet executed
    Pending,
    /// Currently executing
    Running,
    /// Successfully completed
    Completed {
        /// Execution output
        output: String,
        /// Duration in milliseconds
        duration_ms: u64,
    },
    /// Failed execution
    Failed {
        /// Error message
        error: String,
        /// Number of retries attempted
        retries: u32,
    },
    /// Skipped (dependency failed or not applicable)
    Skipped {
        /// Reason for skipping
        reason: String,
    },
}

impl NodeStatus {
    /// Returns true if node is complete (success or skip)
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Completed { .. } | Self::Skipped { .. })
    }

    /// Returns true if node failed
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    /// Returns emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Pending => "⏳",
            Self::Running => "🔄",
            Self::Completed { .. } => "✅",
            Self::Failed { .. } => "❌",
            Self::Skipped { .. } => "⏭️",
        }
    }
}

/// CTVP phase alignment for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CtvpPhase {
    /// Phase 0: Preclinical (unit tests)
    Preclinical = 0,
    /// Phase 1: Safety (fault injection)
    Safety = 1,
    /// Phase 2: Efficacy (real data)
    Efficacy = 2,
    /// Phase 3: Confirmation (scale)
    Confirmation = 3,
    /// Phase 4: Surveillance (monitoring)
    Surveillance = 4,
}

/// CEP stage alignment for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CepStage {
    /// Stage 1: SEE (observation)
    See = 1,
    /// Stage 2: SPEAK (articulation)
    Speak = 2,
    /// Stage 3: DECOMPOSE (analysis)
    Decompose = 3,
    /// Stage 4: COMPOSE (synthesis)
    Compose = 4,
    /// Stage 5: VALIDATE (verification)
    Validate = 5,
    /// Stage 6: DEPLOY (application)
    Deploy = 6,
    /// Stage 7: REFLECT (double-loop)
    Reflect = 7,
    /// Stage 8: EVOLVE (triple-loop)
    Evolve = 8,
    /// Stage 9: SOCIALIZE (sharing)
    Socialize = 9,
}

/// SECI quadrant alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeciQuadrant {
    /// S→E: Socialization to Externalization
    Externalization,
    /// E→E: Combination
    Combination,
    /// E→T: Internalization
    Internalization,
    /// T→S: Socialization
    Socialization,
}

/// Bloom's knowledge dimension
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BloomDimension {
    /// Facts and terminology
    Factual,
    /// Categories and principles
    Conceptual,
    /// Techniques and methods
    Procedural,
    /// Self-knowledge and strategy
    Metacognitive,
}

/// Safety configuration for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSafety {
    /// What could go wrong
    pub failure_mode: String,
    /// How to recover
    pub recovery_strategy: String,
    /// When to halt the entire DAG
    pub circuit_breaker: String,
}

impl Default for NodeSafety {
    fn default() -> Self {
        Self {
            failure_mode: "Unknown".to_string(),
            recovery_strategy: "Retry".to_string(),
            circuit_breaker: "3 consecutive failures".to_string(),
        }
    }
}

/// Epistemology configuration for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEpistemology {
    /// CEP stage
    pub cep_stage: CepStage,
    /// SECI quadrant
    pub seci_quadrant: SeciQuadrant,
    /// Bloom dimension
    pub bloom_dimension: BloomDimension,
}

impl Default for NodeEpistemology {
    fn default() -> Self {
        Self {
            cep_stage: CepStage::Compose,
            seci_quadrant: SeciQuadrant::Combination,
            bloom_dimension: BloomDimension::Procedural,
        }
    }
}

/// A node in the execution DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier
    pub id: NodeId,
    /// Human-readable name
    pub name: String,
    /// Description of what this node does
    pub description: String,

    // Execution
    /// Command to execute
    pub command: String,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Delay between retries in milliseconds
    pub retry_delay_ms: u64,

    // Dependencies
    /// Node IDs this depends on
    pub depends_on: Vec<NodeId>,
    /// Parallel execution group (nodes in same group can run concurrently)
    pub parallel_group: u32,

    // SMART alignment
    /// Which SMART dimension this node addresses
    pub smart_dimension: Option<String>,
    /// Success criteria
    pub success_criteria: String,

    // CTVP
    /// CTVP phase this contributes to
    pub ctvp_phase: CtvpPhase,
    /// Type of evidence this produces
    pub evidence_type: String,

    // Safety
    /// Safety configuration
    pub safety: NodeSafety,

    // Epistemology
    /// Epistemology configuration
    pub epistemology: NodeEpistemology,

    // Runtime state
    /// Current status
    pub status: NodeStatus,
}

impl Node {
    /// Creates a new node builder
    pub fn builder(id: &str) -> NodeBuilder {
        NodeBuilder::new(id)
    }

    /// Returns true if this node can execute (all dependencies complete)
    pub fn can_execute(&self, completed: &[NodeId]) -> bool {
        self.depends_on.iter().all(|dep| completed.contains(dep))
    }

    /// Returns true if node is ready to run
    pub fn is_pending(&self) -> bool {
        matches!(self.status, NodeStatus::Pending)
    }
}

/// Builder for Node
#[derive(Debug)]
pub struct NodeBuilder {
    id: NodeId,
    name: String,
    description: String,
    command: String,
    timeout_ms: u64,
    retry_count: u32,
    retry_delay_ms: u64,
    depends_on: Vec<NodeId>,
    parallel_group: u32,
    smart_dimension: Option<String>,
    success_criteria: String,
    ctvp_phase: CtvpPhase,
    evidence_type: String,
    safety: NodeSafety,
    epistemology: NodeEpistemology,
}

impl NodeBuilder {
    /// Creates a new builder with the given ID
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: id.to_string(),
            description: String::new(),
            command: String::new(),
            timeout_ms: 30000,
            retry_count: 3,
            retry_delay_ms: 1000,
            depends_on: Vec::new(),
            parallel_group: 1,
            smart_dimension: None,
            success_criteria: String::new(),
            ctvp_phase: CtvpPhase::Preclinical,
            evidence_type: "unit_test".to_string(),
            safety: NodeSafety::default(),
            epistemology: NodeEpistemology::default(),
        }
    }

    /// Sets the name
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Sets the description
    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Sets the command
    pub fn command(mut self, cmd: &str) -> Self {
        self.command = cmd.to_string();
        self
    }

    /// Sets the timeout
    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Adds a dependency
    pub fn depends_on(mut self, node_id: &str) -> Self {
        self.depends_on.push(node_id.to_string());
        self
    }

    /// Sets the parallel group
    pub fn parallel_group(mut self, group: u32) -> Self {
        self.parallel_group = group;
        self
    }

    /// Sets the SMART dimension
    pub fn smart_dimension(mut self, dim: &str) -> Self {
        self.smart_dimension = Some(dim.to_string());
        self
    }

    /// Sets success criteria
    pub fn success_criteria(mut self, criteria: &str) -> Self {
        self.success_criteria = criteria.to_string();
        self
    }

    /// Sets the CTVP phase
    pub fn ctvp_phase(mut self, phase: CtvpPhase) -> Self {
        self.ctvp_phase = phase;
        self
    }

    /// Sets the evidence type
    pub fn evidence_type(mut self, t: &str) -> Self {
        self.evidence_type = t.to_string();
        self
    }

    /// Sets safety configuration
    pub fn safety(mut self, safety: NodeSafety) -> Self {
        self.safety = safety;
        self
    }

    /// Sets epistemology configuration
    pub fn epistemology(mut self, ep: NodeEpistemology) -> Self {
        self.epistemology = ep;
        self
    }

    /// Builds the node
    pub fn build(self) -> Node {
        Node {
            id: self.id,
            name: self.name,
            description: self.description,
            command: self.command,
            timeout_ms: self.timeout_ms,
            retry_count: self.retry_count,
            retry_delay_ms: self.retry_delay_ms,
            depends_on: self.depends_on,
            parallel_group: self.parallel_group,
            smart_dimension: self.smart_dimension,
            success_criteria: self.success_criteria,
            ctvp_phase: self.ctvp_phase,
            evidence_type: self.evidence_type,
            safety: self.safety,
            epistemology: self.epistemology,
            status: NodeStatus::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_builder() {
        let node = Node::builder("N001")
            .name("Test Node")
            .command("echo test")
            .depends_on("N000")
            .parallel_group(2)
            .build();

        assert_eq!(node.id, "N001");
        assert_eq!(node.name, "Test Node");
        assert_eq!(node.depends_on, vec!["N000"]);
        assert!(node.is_pending());
    }

    #[test]
    fn test_can_execute() {
        let node = Node::builder("N002")
            .depends_on("N001")
            .depends_on("N000")
            .build();

        assert!(!node.can_execute(&["N001".to_string()]));
        assert!(node.can_execute(&["N000".to_string(), "N001".to_string()]));
    }
}
