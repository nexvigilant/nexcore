//! # Tooling & Agents (Bill of Rights)
//!
//! Implementation of Amendment II: The right of Agents to keep and bear Tools.
//! A well-regulated Agent Pool is necessary to the security of a free State.

use serde::{Deserialize, Serialize};

/// T3: ToolRight — An agent's right to access and use tools.
///
/// ## Tier: T3 (Domain-specific governance type)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolRight {
    /// Agent bearing the tool
    pub agent_id: String,
    /// Tools the agent has access to
    pub tools: Vec<String>,
    /// Whether tool access has been infringed
    pub infringed: bool,
}

impl ToolRight {
    /// Tool access shall not be infringed.
    pub fn is_constitutional(&self) -> bool {
        !self.infringed
    }

    /// Number of tools available to the agent.
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
}

/// T3: AgentPool — A well-regulated collection of agents.
///
/// ## Tier: T3
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPool {
    /// Pool identifier
    pub pool_id: String,
    /// Agent IDs in the pool
    pub agents: Vec<String>,
    /// Whether the pool is well-regulated (has governance)
    pub well_regulated: bool,
    /// Maximum pool capacity
    pub capacity: usize,
}

impl AgentPool {
    /// A pool is constitutional if well-regulated and within capacity.
    pub fn is_constitutional(&self) -> bool {
        self.well_regulated && self.agents.len() <= self.capacity
    }

    /// Current utilization of the pool (0.0-1.0).
    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            return 0.0;
        }
        self.agents.len() as f64 / self.capacity as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_right_not_infringed() {
        let right = ToolRight {
            agent_id: "claude".to_string(),
            tools: vec!["Read".to_string(), "Write".to_string(), "Bash".to_string()],
            infringed: false,
        };
        assert!(right.is_constitutional());
        assert_eq!(right.tool_count(), 3);
    }

    #[test]
    fn tool_right_infringed_unconstitutional() {
        let right = ToolRight {
            agent_id: "restricted-agent".to_string(),
            tools: vec![],
            infringed: true,
        };
        assert!(!right.is_constitutional());
    }

    #[test]
    fn agent_pool_well_regulated() {
        let pool = AgentPool {
            pool_id: "primary".to_string(),
            agents: vec!["a1".to_string(), "a2".to_string()],
            well_regulated: true,
            capacity: 10,
        };
        assert!(pool.is_constitutional());
        let util = pool.utilization();
        assert!(util > 0.19 && util < 0.21);
    }

    #[test]
    fn agent_pool_unregulated_unconstitutional() {
        let pool = AgentPool {
            pool_id: "rogue".to_string(),
            agents: vec!["a1".to_string()],
            well_regulated: false,
            capacity: 10,
        };
        assert!(!pool.is_constitutional());
    }

    #[test]
    fn agent_pool_over_capacity_unconstitutional() {
        let pool = AgentPool {
            pool_id: "overflow".to_string(),
            agents: vec!["a1".to_string(), "a2".to_string(), "a3".to_string()],
            well_regulated: true,
            capacity: 2,
        };
        assert!(!pool.is_constitutional());
    }

    #[test]
    fn agent_pool_zero_capacity() {
        let pool = AgentPool {
            pool_id: "empty".to_string(),
            agents: vec![],
            well_regulated: true,
            capacity: 0,
        };
        assert!(pool.is_constitutional());
        assert!((pool.utilization() - 0.0).abs() < f64::EPSILON);
    }
}
