//! # Lineage — Artifact Genealogy and Provenance
//!
//! Tracks the ancestry of artifacts, sessions, and deployments across
//! generations. Every artifact has a parent, and lineage enables tracing
//! from any offspring back to its origin.
//!
//! ## Biological Mapping
//!
//! | Concept | Biological | Claude Code |
//! |---------|-----------|-------------|
//! | Lineage | Family tree | Artifact version history |
//! | Ancestor | Progenitor organism | Original session/commit |
//! | Descendant | Offspring | Derived artifact/fork |
//! | Generation | Generational distance | Session hops / version count |
//! | Pedigree | Breeding record | Full provenance chain |

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// A node in the lineage tree.
///
/// **T1 grounding**: ∃(Existence) + λ(Location) + π(Persistence)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageNode {
    /// Unique identifier for this node.
    pub id: String,
    /// What kind of entity this is.
    pub kind: EntityKind,
    /// Parent node ID (None for root/progenitor).
    pub parent_id: Option<String>,
    /// Generation number (0 = progenitor).
    pub generation: u32,
    /// When this entity was created.
    pub created_at: DateTime,
    /// Metadata tags for this entity.
    pub tags: Vec<String>,
}

/// The kind of entity in the lineage tree.
///
/// **T1 grounding**: Σ(Sum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityKind {
    /// A Claude Code session.
    Session,
    /// A brain artifact.
    Artifact,
    /// A code commit.
    Commit,
    /// A crate release.
    Release,
    /// A deployment.
    Deployment,
    /// A skill.
    Skill,
}

/// Complete lineage tree.
///
/// **T1 grounding**: ρ(Recursion) + σ(Sequence) + π(Persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lineage {
    /// All nodes in the tree.
    nodes: Vec<LineageNode>,
}

impl Lineage {
    /// Creates a new empty lineage.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Adds a progenitor (root) node.
    pub fn add_progenitor(&mut self, id: String, kind: EntityKind) -> &LineageNode {
        let node = LineageNode {
            id,
            kind,
            parent_id: None,
            generation: 0,
            created_at: DateTime::now(),
            tags: Vec::new(),
        };
        self.nodes.push(node);
        // SAFETY: just pushed an element, so last() is always Some
        #[allow(clippy::expect_used)]
        self.nodes.last().expect("just pushed")
    }

    /// Adds a descendant node with a parent.
    ///
    /// Returns `None` if the parent doesn't exist.
    pub fn add_descendant(
        &mut self,
        id: String,
        kind: EntityKind,
        parent_id: &str,
    ) -> Option<&LineageNode> {
        let parent_gen = self.nodes.iter().find(|n| n.id == parent_id)?.generation;

        let node = LineageNode {
            id,
            kind,
            parent_id: Some(parent_id.to_string()),
            generation: parent_gen + 1,
            created_at: DateTime::now(),
            tags: Vec::new(),
        };
        self.nodes.push(node);
        self.nodes.last()
    }

    /// Finds a node by ID.
    pub fn find(&self, id: &str) -> Option<&LineageNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Returns the full ancestry chain from a node back to the progenitor.
    pub fn ancestry(&self, id: &str) -> Vec<&LineageNode> {
        let mut chain = Vec::new();
        let mut current_id = Some(id.to_string());

        while let Some(ref cid) = current_id {
            if let Some(node) = self.find(cid) {
                chain.push(node);
                current_id = node.parent_id.clone();
            } else {
                break;
            }
        }

        chain
    }

    /// Returns all direct children of a node.
    pub fn children(&self, id: &str) -> Vec<&LineageNode> {
        self.nodes
            .iter()
            .filter(|n| n.parent_id.as_deref() == Some(id))
            .collect()
    }

    /// Returns all descendants (recursive children) of a node.
    pub fn descendants(&self, id: &str) -> Vec<&LineageNode> {
        let mut result = Vec::new();
        let mut stack = vec![id.to_string()];

        while let Some(current) = stack.pop() {
            for child in self.children(&current) {
                result.push(child);
                stack.push(child.id.clone());
            }
        }

        result
    }

    /// Returns the maximum generation depth in the tree.
    pub fn max_generation(&self) -> u32 {
        self.nodes.iter().map(|n| n.generation).max().unwrap_or(0)
    }

    /// Returns all nodes at a specific generation.
    pub fn at_generation(&self, generation: u32) -> Vec<&LineageNode> {
        self.nodes
            .iter()
            .filter(|n| n.generation == generation)
            .collect()
    }

    /// Returns the total number of nodes.
    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    /// Returns all nodes of a specific kind.
    pub fn by_kind(&self, kind: EntityKind) -> Vec<&LineageNode> {
        self.nodes.iter().filter(|n| n.kind == kind).collect()
    }

    /// Calculates the "reproductive fitness" of a node —
    /// how many descendants it has relative to its generation peers.
    pub fn fitness(&self, id: &str) -> f64 {
        let node = match self.find(id) {
            Some(n) => n,
            None => return 0.0,
        };

        let descendants = self.descendants(id).len();
        let peers = self.at_generation(node.generation).len();

        if peers <= 1 {
            return descendants as f64;
        }

        let peer_descendants: usize = self
            .at_generation(node.generation)
            .iter()
            .map(|p| self.descendants(&p.id).len())
            .sum();

        let avg_peer_descendants = peer_descendants as f64 / peers as f64;

        if avg_peer_descendants == 0.0 {
            return descendants as f64;
        }

        descendants as f64 / avg_peer_descendants
    }
}

impl Default for Lineage {
    fn default() -> Self {
        Self::new()
    }
}

/// A pedigree record — provenance certificate for an artifact.
///
/// **T1 grounding**: π(Persistence) + ∝(Irreversibility) + σ(Sequence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pedigree {
    /// The entity this pedigree belongs to.
    pub entity_id: String,
    /// Full ancestry chain (oldest first).
    pub ancestors: Vec<String>,
    /// Generation depth.
    pub depth: u32,
    /// Whether the lineage is complete (no broken links).
    pub complete: bool,
    /// When the pedigree was computed.
    pub computed_at: DateTime,
}

impl Pedigree {
    /// Computes a pedigree from a lineage tree.
    pub fn compute(lineage: &Lineage, entity_id: &str) -> Option<Self> {
        let ancestry = lineage.ancestry(entity_id);
        if ancestry.is_empty() {
            return None;
        }

        let depth = ancestry.first().map(|n| n.generation).unwrap_or(0);
        let complete = ancestry
            .last()
            .map(|n| n.parent_id.is_none())
            .unwrap_or(false);

        let ancestors: Vec<String> = ancestry.iter().rev().map(|n| n.id.clone()).collect();

        Some(Self {
            entity_id: entity_id.into(),
            ancestors,
            depth,
            complete,
            computed_at: DateTime::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_lineage() -> Lineage {
        let mut l = Lineage::new();
        l.add_progenitor("root".into(), EntityKind::Session);
        l.add_descendant("child-1".into(), EntityKind::Artifact, "root");
        l.add_descendant("child-2".into(), EntityKind::Commit, "root");
        l.add_descendant("grandchild-1".into(), EntityKind::Release, "child-1");
        l.add_descendant("grandchild-2".into(), EntityKind::Deployment, "child-1");
        l.add_descendant("great-grandchild".into(), EntityKind::Skill, "grandchild-1");
        l
    }

    #[test]
    fn test_lineage_structure() {
        let l = build_lineage();
        assert_eq!(l.size(), 6);
        assert_eq!(l.max_generation(), 3);
    }

    #[test]
    fn test_ancestry_chain() {
        let l = build_lineage();
        let ancestry = l.ancestry("great-grandchild");
        assert_eq!(ancestry.len(), 4);
        assert_eq!(ancestry[0].id, "great-grandchild");
        assert_eq!(ancestry[1].id, "grandchild-1");
        assert_eq!(ancestry[2].id, "child-1");
        assert_eq!(ancestry[3].id, "root");
    }

    #[test]
    fn test_children() {
        let l = build_lineage();
        let children = l.children("root");
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_descendants() {
        let l = build_lineage();
        let desc = l.descendants("child-1");
        assert_eq!(desc.len(), 3); // grandchild-1, grandchild-2, great-grandchild
    }

    #[test]
    fn test_at_generation() {
        let l = build_lineage();
        assert_eq!(l.at_generation(0).len(), 1); // root
        assert_eq!(l.at_generation(1).len(), 2); // child-1, child-2
        assert_eq!(l.at_generation(2).len(), 2); // grandchild-1, grandchild-2
        assert_eq!(l.at_generation(3).len(), 1); // great-grandchild
    }

    #[test]
    fn test_by_kind() {
        let l = build_lineage();
        assert_eq!(l.by_kind(EntityKind::Session).len(), 1);
        assert_eq!(l.by_kind(EntityKind::Artifact).len(), 1);
        assert_eq!(l.by_kind(EntityKind::Skill).len(), 1);
    }

    #[test]
    fn test_fitness() {
        let l = build_lineage();
        // child-1 has 3 descendants, child-2 has 0
        // avg peer descendants = (3+0)/2 = 1.5
        // child-1 fitness = 3/1.5 = 2.0
        let fitness = l.fitness("child-1");
        assert!((fitness - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pedigree() {
        let l = build_lineage();
        let pedigree = Pedigree::compute(&l, "great-grandchild");
        assert!(pedigree.is_some());

        let p = pedigree.expect("pedigree should exist");
        assert_eq!(p.entity_id, "great-grandchild");
        assert_eq!(p.depth, 3);
        assert!(p.complete);
        assert_eq!(p.ancestors.len(), 4);
        // Ancestors oldest-first
        assert_eq!(p.ancestors[0], "root");
        assert_eq!(p.ancestors[3], "great-grandchild");
    }

    #[test]
    fn test_nonexistent_pedigree() {
        let l = Lineage::new();
        assert!(Pedigree::compute(&l, "ghost").is_none());
    }
}
