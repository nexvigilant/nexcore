//! Node registry: tracks which flywheel nodes exist and their status.

use crate::node::{FlywheelTier, NodeDescriptor, NodeStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRegistry {
    nodes: Vec<NodeDescriptor>,
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::default_three_node()
    }
}

impl NodeRegistry {
    pub fn default_three_node() -> Self {
        Self {
            nodes: vec![
                NodeDescriptor::new(
                    FlywheelTier::Live,
                    "homeostasis",
                    vec![
                        "nexcore-homeostasis".into(),
                        "nexcore-homeostasis-memory".into(),
                        "nexcore-homeostasis-primitives".into(),
                        "nexcore-homeostasis-storm".into(),
                        "nexcore-guardian-engine".into(),
                        "nexcore-cytokine".into(),
                        "nexcore-hormones".into(),
                    ],
                    NodeStatus::Active,
                ),
                NodeDescriptor::new(
                    FlywheelTier::Live,
                    "pv-signal",
                    vec![
                        "nexcore-pv-core".into(),
                        "nexcore-vigilance".into(),
                        "nexcore-faers-etl".into(),
                        "nexcore-qbr".into(),
                        "nexcore-pvos".into(),
                        "nexcore-pvdsl".into(),
                    ],
                    NodeStatus::Active,
                ),
                NodeDescriptor::new(
                    FlywheelTier::Live,
                    "immunity",
                    vec![
                        "nexcore-immunity".into(),
                        "nexcore-antibodies".into(),
                        "nexcore-spliceosome".into(),
                        "nexcore-ribosome".into(),
                        "nexcore-transcriptase".into(),
                    ],
                    NodeStatus::Active,
                ),
                NodeDescriptor::new(
                    FlywheelTier::Live,
                    "trust",
                    vec![
                        "nexcore-trust".into(),
                        "nexcore-proof-of-meaning".into(),
                        "nexcore-tov".into(),
                    ],
                    NodeStatus::Active,
                ),
                NodeDescriptor::new(
                    FlywheelTier::Staging,
                    "skill-maturation",
                    vec![
                        "nexcore-skills-engine".into(),
                        "nexcore-skill-compiler".into(),
                        "nexcore-skill-exec".into(),
                    ],
                    NodeStatus::Wiring,
                ),
                NodeDescriptor::new(
                    FlywheelTier::Staging,
                    "insight",
                    vec!["nexcore-insight".into()],
                    NodeStatus::Wiring,
                ),
                NodeDescriptor::new(
                    FlywheelTier::Staging,
                    "cep-primitives",
                    vec![
                        "nexcore-lex-primitiva".into(),
                        "nexcore-primitives".into(),
                        "nexcore-transform".into(),
                    ],
                    NodeStatus::Wiring,
                ),
            ],
        }
    }

    pub fn nodes(&self) -> &[NodeDescriptor] {
        &self.nodes
    }

    pub fn nodes_in_tier(&self, tier: FlywheelTier) -> Vec<&NodeDescriptor> {
        self.nodes.iter().filter(|n| n.tier == tier).collect()
    }

    pub fn find(&self, name: &str) -> Option<&NodeDescriptor> {
        self.nodes.iter().find(|n| n.name == name)
    }

    pub fn find_mut(&mut self, name: &str) -> Option<&mut NodeDescriptor> {
        self.nodes.iter_mut().find(|n| n.name == name)
    }

    pub fn promote(&mut self, name: &str) -> bool {
        if let Some(node) = self.find_mut(name) {
            let new_tier = node.tier.promote();
            if new_tier != node.tier {
                node.tier = new_tier;
                node.status = match new_tier {
                    FlywheelTier::Live => NodeStatus::Active,
                    FlywheelTier::Staging => NodeStatus::Wiring,
                    FlywheelTier::Draft => NodeStatus::Dormant,
                };
                return true;
            }
        }
        false
    }

    pub fn count_by_tier(&self) -> (usize, usize, usize) {
        let live = self
            .nodes
            .iter()
            .filter(|n| n.tier == FlywheelTier::Live)
            .count();
        let staging = self
            .nodes
            .iter()
            .filter(|n| n.tier == FlywheelTier::Staging)
            .count();
        let draft = self
            .nodes
            .iter()
            .filter(|n| n.tier == FlywheelTier::Draft)
            .count();
        (live, staging, draft)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
