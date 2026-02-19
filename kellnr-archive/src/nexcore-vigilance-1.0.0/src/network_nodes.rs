//! # Network Node Finding & Isolation
//!
//! Grounds structural node identification in T1 primitives:
//! - ∃ (Existence): The node as a distinct entity
//! - μ (Mapping): The node's connectivity and signal profile
//! - ∂ (Boundary): The isolation mechanism

use crate::lex_primitiva::{GroundsTo, LexPrimitiva, PrimitiveComposition};
use crate::vdag::prelude::NodeId;
use serde::{Deserialize, Serialize};

/// A signal emitted by a node in the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSignal {
    pub pattern_id: String,
    pub intensity: f64,
    pub timestamp: f64,
}

/// A network entity capable of emitting signals and maintaining connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    pub id: NodeId,
    pub signals: Vec<NodeSignal>,
    pub neighbors: Vec<NodeId>,
    pub is_isolated: bool,
}

impl GroundsTo for NetworkNode {
    fn dominant_primitive() -> Option<LexPrimitiva> {
        Some(LexPrimitiva::Mapping) // Nodes are defined by their mapping to others
    }

    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // The node itself
            LexPrimitiva::Mapping,   // Connectivity
            LexPrimitiva::Boundary,  // Isolation capability
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.9)
    }
}

/// Disproportionality metrics for node identification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentification {
    pub prr: f64,
    pub chi_square: f64,
    pub confidence: f64,
    pub is_match: bool,
}

impl NetworkNode {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            signals: Vec::new(),
            neighbors: Vec::new(),
            is_isolated: false,
        }
    }

    /// Calculate if this node disproportionately matches a signal characteristic.
    pub fn identify(
        &self,
        target_pattern: &str,
        background_total_cases: u32,
        total_network_reports: u32,
    ) -> NodeIdentification {
        let a = self
            .signals
            .iter()
            .filter(|s| s.pattern_id == target_pattern)
            .count() as f64;
        let b = (self.signals.len() as f64) - a;
        let c = (background_total_cases as f64) - a;
        let d = (total_network_reports as f64) - a - b - c;

        // PRR = (a / (a+b)) / (c / (c+d))
        let prr = if (c / (c + d)) > 0.0 && (a + b) > 0.0 {
            (a / (a + b)) / (c / (c + d))
        } else {
            0.0
        };

        NodeIdentification {
            prr,
            chi_square: 0.0, // TODO: Implement Chi-square
            confidence: 0.9,
            is_match: prr >= 2.0 && a >= 3.0,
        }
    }

    /// Isolate the node by establishing a logical boundary (∂).
    pub fn isolate(&mut self) {
        self.is_isolated = true;
        // In a real system, this would drop all network sockets/handles
    }
}

/// A scanner that monitors multiple nodes for disproportionate signal patterns.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeSignalScanner {
    pub nodes: Vec<NetworkNode>,
    pub total_reports: u32,
}

impl NodeSignalScanner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: NetworkNode) {
        self.nodes.push(node);
    }

    /// Find all nodes that match a specific signal characteristic.
    pub fn find_nodes(&self, target_pattern: &str) -> Vec<(NodeId, NodeIdentification)> {
        // Calculate background frequency for the pattern
        let background_cases: u32 = self
            .nodes
            .iter()
            .map(|n| {
                n.signals
                    .iter()
                    .filter(|s| s.pattern_id == target_pattern)
                    .count() as u32
            })
            .sum();

        self.nodes
            .iter()
            .map(|n| {
                (
                    n.id.clone(),
                    n.identify(target_pattern, background_cases, self.total_reports),
                )
            })
            .filter(|(_, id)| id.is_match)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_identification() {
        let mut node = NetworkNode::new("NODE_001".to_string());
        // 10 signals, 5 are "ANOMALY"
        for _ in 0..5 {
            node.signals.push(NodeSignal {
                pattern_id: "ANOMALY".to_string(),
                intensity: 1.0,
                timestamp: 0.0,
            });
            node.signals.push(NodeSignal {
                pattern_id: "NORMAL".to_string(),
                intensity: 0.1,
                timestamp: 0.0,
            });
        }

        // Background: 100 total ANOMALY cases in 10,000 reports
        let id = node.identify("ANOMALY", 100, 10000);

        assert!(id.prr > 2.0);
        assert!(id.is_match);
    }

    #[test]
    fn test_node_isolation() {
        let mut node = NetworkNode::new("NODE_002".to_string());
        assert!(!node.is_isolated);
        node.isolate();
        assert!(node.is_isolated);
    }
}
