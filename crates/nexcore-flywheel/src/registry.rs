//! Node registry: tracks which flywheel nodes exist and their status.

use crate::node::{FlywheelTier, NodeDescriptor, NodeStatus};
use serde::{Deserialize, Serialize};

/// Registry of all flywheel nodes and their lifecycle status.
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
    /// Creates the default seven-node registry across Live and Staging tiers.
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

    /// Returns a slice of all registered nodes.
    pub fn nodes(&self) -> &[NodeDescriptor] {
        &self.nodes
    }

    /// Returns all nodes belonging to the given tier.
    pub fn nodes_in_tier(&self, tier: FlywheelTier) -> Vec<&NodeDescriptor> {
        self.nodes.iter().filter(|n| n.tier == tier).collect()
    }

    /// Returns a reference to the node with the given name, if it exists.
    pub fn find(&self, name: &str) -> Option<&NodeDescriptor> {
        self.nodes.iter().find(|n| n.name == name)
    }

    /// Returns a mutable reference to the node with the given name, if it exists.
    pub fn find_mut(&mut self, name: &str) -> Option<&mut NodeDescriptor> {
        self.nodes.iter_mut().find(|n| n.name == name)
    }

    /// Promotes a node to the next tier, returning `true` if the promotion occurred.
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

    /// Returns node counts as (live, staging, draft).
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

    /// Returns the total number of registered nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    /// Returns `true` if the registry contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_seven_nodes() {
        let reg = NodeRegistry::default();
        assert_eq!(reg.len(), 7);
        assert!(!reg.is_empty());
    }

    #[test]
    fn default_tier_counts() {
        let reg = NodeRegistry::default();
        let (live, staging, draft) = reg.count_by_tier();
        assert_eq!(live, 4);
        assert_eq!(staging, 3);
        assert_eq!(draft, 0);
    }

    #[test]
    fn find_homeostasis() {
        let reg = NodeRegistry::default();
        let node = reg.find("homeostasis");
        assert!(node.is_some());
        assert_eq!(node.map(|n| n.tier), Some(FlywheelTier::Live));
    }

    #[test]
    fn find_nonexistent_returns_none() {
        let reg = NodeRegistry::default();
        assert!(reg.find("nonexistent").is_none());
    }

    #[test]
    fn nodes_in_live_tier() {
        let reg = NodeRegistry::default();
        let live = reg.nodes_in_tier(FlywheelTier::Live);
        assert_eq!(live.len(), 4);
    }

    #[test]
    fn nodes_in_staging_tier() {
        let reg = NodeRegistry::default();
        let staging = reg.nodes_in_tier(FlywheelTier::Staging);
        assert_eq!(staging.len(), 3);
    }

    #[test]
    fn promote_staging_to_live() {
        let mut reg = NodeRegistry::default();
        assert!(reg.promote("skill-maturation"));
        let node = reg.find("skill-maturation").expect("exists");
        assert_eq!(node.tier, FlywheelTier::Live);
        assert_eq!(node.status, NodeStatus::Active);
    }

    #[test]
    fn promote_live_stays_live() {
        let mut reg = NodeRegistry::default();
        assert!(!reg.promote("homeostasis"));
    }

    #[test]
    fn promote_nonexistent_returns_false() {
        let mut reg = NodeRegistry::default();
        assert!(!reg.promote("ghost"));
    }

    #[test]
    fn find_mut_updates_status() {
        let mut reg = NodeRegistry::default();
        if let Some(node) = reg.find_mut("insight") {
            node.status = NodeStatus::Active;
        }
        assert_eq!(
            reg.find("insight").map(|n| n.status),
            Some(NodeStatus::Active)
        );
    }

    #[test]
    fn serialization_roundtrip() {
        let reg = NodeRegistry::default();
        let json = serde_json::to_string(&reg).expect("ser");
        let back: NodeRegistry = serde_json::from_str(&json).expect("de");
        assert_eq!(back.len(), 7);
    }
}
