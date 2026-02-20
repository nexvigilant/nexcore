//! Counterfactual analysis engine for The Foundry's REASON station (A3).
//!
//! Answers "what-if" questions about the causal graph by applying
//! [`Intervention`]s to an isolated clone of the original [`CausalDag`] and
//! comparing the structural outcome against the baseline.
//!
//! # Workflow
//!
//! 1. Construct a [`CounterfactualEngine`] around an existing [`CausalDag`].
//! 2. Describe a hypothetical change as an [`Intervention`].
//! 3. Call [`CounterfactualEngine::evaluate`] to obtain a
//!    [`CounterfactualResult`] that quantifies which nodes and paths were
//!    affected.
//!
//! The original DAG is **never mutated**; every evaluation works on a
//! private clone.
//!
//! # Example — removing a node
//!
//! ```
//! use nexcore_reason::dag::{CausalDag, CausalLink, CausalNode, NodeId, NodeType};
//! use nexcore_reason::counterfactual::{CounterfactualEngine, Intervention};
//!
//! let mut dag = CausalDag::new();
//! dag.add_node(CausalNode { id: "a".to_string(), label: "A".to_string(), node_type: NodeType::Root });
//! dag.add_node(CausalNode { id: "b".to_string(), label: "B".to_string(), node_type: NodeType::Factor });
//! dag.add_node(CausalNode { id: "c".to_string(), label: "C".to_string(), node_type: NodeType::Risk });
//! dag.add_link(CausalLink { from: "a".to_string(), to: "b".to_string(), strength: 0.9 });
//! dag.add_link(CausalLink { from: "b".to_string(), to: "c".to_string(), strength: 0.8 });
//!
//! let engine = CounterfactualEngine::new(dag);
//! let result = engine.evaluate(&Intervention::RemoveNode("b".to_string())).unwrap();
//!
//! // Removing the bridge node "b" severs the a → c path.
//! assert!(result.broken_paths.contains(&("a".to_string(), "c".to_string())));
//! assert!(result.impact_score > 0.0);
//! ```

use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};

use crate::dag::{CausalDag, CausalLink, CausalNode, NodeId, NodeType};

// ---------------------------------------------------------------------------
// Intervention
// ---------------------------------------------------------------------------

/// A hypothetical change applied to the [`CausalDag`] for counterfactual
/// analysis.
///
/// Each variant represents a distinct structural or parametric mutation.
/// The original DAG is never touched; interventions operate on a clone.
///
/// # Example
///
/// ```
/// use nexcore_reason::counterfactual::Intervention;
///
/// let iv = Intervention::RemoveNode("drug_exposure".to_string());
/// // Pattern-match to inspect the variant.
/// assert!(matches!(iv, Intervention::RemoveNode(_)));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Intervention {
    /// Remove a node and all causal links that touch it.
    ///
    /// After the removal any node that was only reachable through the removed
    /// node becomes disconnected from its former ancestors.
    RemoveNode(NodeId),

    /// Remove the specific directed link from `from` to `to`.
    ///
    /// Both endpoint nodes remain in the graph; only the edge is severed.
    RemoveLink {
        /// Source of the link to remove.
        from: NodeId,
        /// Target of the link to remove.
        to: NodeId,
    },

    /// Change the causal strength of the link from `from` to `to`.
    ///
    /// This is a parametric (non-structural) intervention; no nodes or links
    /// are added or removed.  The impact score is proportional to the
    /// magnitude of the change relative to the original strength.
    AdjustStrength {
        /// Source of the link whose strength changes.
        from: NodeId,
        /// Target of the link whose strength changes.
        to: NodeId,
        /// Replacement strength value in `[0.0, 1.0]`.  Values outside that
        /// range are clamped by the DAG's mutation helpers.
        new_strength: f64,
    },

    /// Inject a hypothetical [`CausalNode`] into the graph.
    ///
    /// The node is appended as an isolated vertex (no links are added
    /// automatically).  The caller is responsible for adding connecting links
    /// as additional [`AdjustStrength`] or follow-up interventions if needed.
    ///
    /// [`AdjustStrength`]: Intervention::AdjustStrength
    InjectNode(CausalNode),
}

// ---------------------------------------------------------------------------
// CounterfactualResult
// ---------------------------------------------------------------------------

/// The outcome of applying a single [`Intervention`] to the causal graph.
///
/// All fields compare the **modified** graph against the **original** to
/// characterise the scope and severity of the change.
///
/// # Example
///
/// ```
/// use nexcore_reason::dag::{CausalDag, CausalLink, CausalNode, NodeType};
/// use nexcore_reason::counterfactual::{CounterfactualEngine, Intervention};
///
/// let mut dag = CausalDag::new();
/// dag.add_node(CausalNode { id: "x".to_string(), label: "X".to_string(), node_type: NodeType::Root });
/// dag.add_node(CausalNode { id: "y".to_string(), label: "Y".to_string(), node_type: NodeType::Risk });
/// dag.add_link(CausalLink { from: "x".to_string(), to: "y".to_string(), strength: 0.5 });
///
/// let engine = CounterfactualEngine::new(dag);
/// let result = engine
///     .evaluate(&Intervention::RemoveLink {
///         from: "x".to_string(),
///         to: "y".to_string(),
///     })
///     .unwrap();
///
/// assert_eq!(result.broken_paths, vec![("x".to_string(), "y".to_string())]);
/// assert!((result.impact_score - 1.0).abs() < 1e-6);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualResult {
    /// The intervention that produced this result.
    pub intervention: Intervention,

    /// Nodes whose reachability or structural role (root / leaf) changed.
    pub affected_nodes: Vec<NodeId>,

    /// Source–target pairs that were connected in the original graph but are
    /// disconnected after the intervention.
    pub broken_paths: Vec<(NodeId, NodeId)>,

    /// Nodes that became roots (no incoming edges) after the intervention that
    /// were **not** roots before.
    pub new_roots: Vec<NodeId>,

    /// Nodes that became leaves (no outgoing edges) after the intervention that
    /// were **not** leaves before.
    pub new_leaves: Vec<NodeId>,

    /// Fraction of the original graph that was affected, in `[0.0, 1.0]`.
    ///
    /// Computed as `affected_nodes.len() / original_node_count`, clamped to
    /// `[0.0, 1.0]`.  For [`Intervention::AdjustStrength`] the value is
    /// derived from the normalised strength delta rather than a structural
    /// node count.
    pub impact_score: f64,
}

// ---------------------------------------------------------------------------
// Internal graph helpers
// ---------------------------------------------------------------------------

/// Returns all node IDs present in a DAG as a `HashSet<NodeId>`.
fn node_id_set(dag: &CausalDag) -> HashSet<NodeId> {
    dag.nodes().iter().map(|n| n.id.clone()).collect()
}

/// Returns `true` when `dag` contains a directed path from `from` to `to`.
///
/// BFS over the link list.  O(V + E).
fn can_reach(dag: &CausalDag, from: &NodeId, to: &NodeId) -> bool {
    if from == to {
        return true;
    }
    // Build an outgoing adjacency map on the fly from the link slice.
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for link in dag.links() {
        adj.entry(link.from.as_str()).or_default().push(link.to.as_str());
    }

    let mut visited: HashSet<&str> = HashSet::new();
    let mut queue: VecDeque<&str> = VecDeque::new();
    queue.push_back(from.as_str());
    visited.insert(from.as_str());

    while let Some(current) = queue.pop_front() {
        if let Some(neighbors) = adj.get(current) {
            for &neighbor in neighbors {
                if neighbor == to.as_str() {
                    return true;
                }
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }
    }
    false
}

/// Returns the set of node IDs that have at least one incoming link.
fn nodes_with_incoming(dag: &CausalDag) -> HashSet<NodeId> {
    dag.links().iter().map(|l| l.to.clone()).collect()
}

/// Returns the set of node IDs that have at least one outgoing link.
fn nodes_with_outgoing(dag: &CausalDag) -> HashSet<NodeId> {
    dag.links().iter().map(|l| l.from.clone()).collect()
}

/// Returns node IDs that are structural roots: present in the graph but
/// **not** the target of any link.
fn structural_roots(dag: &CausalDag) -> HashSet<NodeId> {
    let all = node_id_set(dag);
    let with_incoming = nodes_with_incoming(dag);
    all.difference(&with_incoming).cloned().collect()
}

/// Returns node IDs that are structural leaves: present in the graph but
/// **not** the source of any link.
fn structural_leaves(dag: &CausalDag) -> HashSet<NodeId> {
    let all = node_id_set(dag);
    let with_outgoing = nodes_with_outgoing(dag);
    all.difference(&with_outgoing).cloned().collect()
}

/// Removes `node_id` and all links that touch it from `dag`.
fn remove_node_from(dag: &mut CausalDag, node_id: &NodeId) {
    // Reconstruct the dag in-place: replace its vec-based storage with
    // filtered copies.  The public API only exposes `&[CausalNode]` and
    // `&[CausalLink]` for reading, so we must use `add_node`/`add_link` on a
    // fresh instance and swap it in.
    let surviving_nodes: Vec<CausalNode> = dag
        .nodes()
        .iter()
        .filter(|n| &n.id != node_id)
        .cloned()
        .collect();
    let surviving_links: Vec<CausalLink> = dag
        .links()
        .iter()
        .filter(|l| &l.from != node_id && &l.to != node_id)
        .cloned()
        .collect();

    let mut fresh = CausalDag::new();
    for n in surviving_nodes {
        fresh.add_node(n);
    }
    for l in surviving_links {
        fresh.add_link(l);
    }
    *dag = fresh;
}

/// Removes the first link matching `from → to` from `dag`.
fn remove_link_from(dag: &mut CausalDag, from: &NodeId, to: &NodeId) {
    let mut removed = false;
    let surviving_links: Vec<CausalLink> = dag
        .links()
        .iter()
        .filter(|l| {
            if !removed && &l.from == from && &l.to == to {
                removed = true;
                false
            } else {
                true
            }
        })
        .cloned()
        .collect();

    let nodes: Vec<CausalNode> = dag.nodes().to_vec();
    let mut fresh = CausalDag::new();
    for n in nodes {
        fresh.add_node(n);
    }
    for l in surviving_links {
        fresh.add_link(l);
    }
    *dag = fresh;
}

/// Adjusts the strength of the first link matching `from → to` in `dag`.
fn adjust_strength_in(dag: &mut CausalDag, from: &NodeId, to: &NodeId, new_strength: f64) {
    let updated_links: Vec<CausalLink> = dag
        .links()
        .iter()
        .map(|l| {
            if &l.from == from && &l.to == to {
                CausalLink {
                    from: l.from.clone(),
                    to: l.to.clone(),
                    strength: new_strength.clamp(0.0, 1.0),
                }
            } else {
                l.clone()
            }
        })
        .collect();

    let nodes: Vec<CausalNode> = dag.nodes().to_vec();
    let mut fresh = CausalDag::new();
    for n in nodes {
        fresh.add_node(n);
    }
    for l in updated_links {
        fresh.add_link(l);
    }
    *dag = fresh;
}

// ---------------------------------------------------------------------------
// CounterfactualEngine
// ---------------------------------------------------------------------------

/// Evaluates counterfactual interventions against a [`CausalDag`].
///
/// The engine holds an owned copy of the baseline graph.  Every call to
/// [`evaluate`] works on a private clone, so the original remains unchanged
/// and multiple independent evaluations can be performed without interference.
///
/// [`evaluate`]: CounterfactualEngine::evaluate
///
/// # Example
///
/// ```
/// use nexcore_reason::dag::{CausalDag, CausalLink, CausalNode, NodeType};
/// use nexcore_reason::counterfactual::{CounterfactualEngine, Intervention};
///
/// let mut dag = CausalDag::new();
/// dag.add_node(CausalNode { id: "cause".to_string(), label: "Cause".to_string(), node_type: NodeType::Root });
/// dag.add_node(CausalNode { id: "effect".to_string(), label: "Effect".to_string(), node_type: NodeType::Risk });
/// dag.add_link(CausalLink { from: "cause".to_string(), to: "effect".to_string(), strength: 0.75 });
///
/// let engine = CounterfactualEngine::new(dag);
/// let batch = engine
///     .evaluate_batch(&[
///         Intervention::RemoveNode("cause".to_string()),
///         Intervention::AdjustStrength {
///             from: "cause".to_string(),
///             to: "effect".to_string(),
///             new_strength: 0.1,
///         },
///     ])
///     .unwrap();
///
/// assert_eq!(batch.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct CounterfactualEngine {
    dag: CausalDag,
}

impl CounterfactualEngine {
    /// Creates a new engine backed by `dag`.
    ///
    /// The engine takes ownership of the graph so that it can clone it
    /// efficiently for each evaluation without requiring the caller to keep
    /// a separate reference.
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_reason::dag::CausalDag;
    /// use nexcore_reason::counterfactual::CounterfactualEngine;
    ///
    /// let engine = CounterfactualEngine::new(CausalDag::new());
    /// ```
    #[must_use]
    pub fn new(dag: CausalDag) -> Self {
        Self { dag }
    }

    /// Applies `intervention` to a clone of the baseline DAG and returns a
    /// [`CounterfactualResult`] describing the structural change.
    ///
    /// # Errors
    ///
    /// Returns an error when:
    ///
    /// - A [`RemoveNode`] or [`RemoveLink`] targets an id that does not exist
    ///   in the baseline graph.
    /// - An [`AdjustStrength`] targets a link that does not exist.
    /// - An [`InjectNode`] supplies a node whose id already exists.
    ///
    /// [`RemoveNode`]: Intervention::RemoveNode
    /// [`RemoveLink`]: Intervention::RemoveLink
    /// [`AdjustStrength`]: Intervention::AdjustStrength
    /// [`InjectNode`]: Intervention::InjectNode
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_reason::dag::{CausalDag, CausalLink, CausalNode, NodeType};
    /// use nexcore_reason::counterfactual::{CounterfactualEngine, Intervention};
    ///
    /// let mut dag = CausalDag::new();
    /// dag.add_node(CausalNode { id: "p".to_string(), label: "P".to_string(), node_type: NodeType::Root });
    /// dag.add_node(CausalNode { id: "q".to_string(), label: "Q".to_string(), node_type: NodeType::Risk });
    /// dag.add_link(CausalLink { from: "p".to_string(), to: "q".to_string(), strength: 0.6 });
    ///
    /// let engine = CounterfactualEngine::new(dag);
    /// let result = engine.evaluate(&Intervention::RemoveNode("p".to_string())).unwrap();
    ///
    /// // "p" is gone, so "q" becomes a new root.
    /// assert!(result.new_roots.contains(&"q".to_string()));
    /// ```
    pub fn evaluate(&self, intervention: &Intervention) -> Result<CounterfactualResult> {
        let original_node_count = self.dag.nodes().len();

        // Snapshot structural properties of the baseline.
        let original_roots: HashSet<NodeId> = structural_roots(&self.dag);
        let original_leaves: HashSet<NodeId> = structural_leaves(&self.dag);
        let original_ids: HashSet<NodeId> = node_id_set(&self.dag);

        // All pairs that are reachable in the original (excluding self-pairs).
        let original_reachable: HashSet<(NodeId, NodeId)> = {
            let ids: Vec<&NodeId> = self.dag.nodes().iter().map(|n| &n.id).collect();
            let mut set = HashSet::new();
            for &a in &ids {
                for &b in &ids {
                    if a != b && can_reach(&self.dag, a, b) {
                        set.insert((a.clone(), b.clone()));
                    }
                }
            }
            set
        };

        // Clone and apply the intervention.
        let mut modified = self.dag.clone();
        self.apply_intervention(&mut modified, intervention, &original_ids)?;

        // --- Compute diff ---

        let modified_roots: HashSet<NodeId> = structural_roots(&modified);
        let modified_leaves: HashSet<NodeId> = structural_leaves(&modified);
        let modified_ids: HashSet<NodeId> = node_id_set(&modified);

        // New roots / leaves: only consider nodes that still exist.
        let new_roots: Vec<NodeId> = modified_roots
            .difference(&original_roots)
            .filter(|id| modified_ids.contains(*id))
            .cloned()
            .collect();

        let new_leaves: Vec<NodeId> = modified_leaves
            .difference(&original_leaves)
            .filter(|id| modified_ids.contains(*id))
            .cloned()
            .collect();

        // Broken paths: pairs reachable before but not after.
        let broken_paths: Vec<(NodeId, NodeId)> = original_reachable
            .iter()
            .filter(|(a, b)| {
                // Both endpoints must still exist for the path to be broken
                // rather than simply removed.
                modified_ids.contains(a)
                    && modified_ids.contains(b)
                    && !can_reach(&modified, a, b)
            })
            .map(|(a, b)| (a.clone(), b.clone()))
            .collect();

        // Affected nodes: nodes whose reachability set changed, or that
        // changed root/leaf status.
        let affected_nodes: Vec<NodeId> = self.compute_affected_nodes(
            &original_roots,
            &original_leaves,
            &modified_roots,
            &modified_leaves,
            &original_reachable,
            &modified,
            &modified_ids,
            intervention,
        );

        // Impact score.
        let impact_score = self.compute_impact_score(
            intervention,
            &affected_nodes,
            original_node_count,
        );

        Ok(CounterfactualResult {
            intervention: intervention.clone(),
            affected_nodes,
            broken_paths,
            new_roots,
            new_leaves,
            impact_score,
        })
    }

    /// Evaluates a sequence of interventions in order, returning one
    /// [`CounterfactualResult`] per intervention.
    ///
    /// Each evaluation is independent; interventions do not compose.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered, aborting the remaining
    /// evaluations.
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_reason::dag::{CausalDag, CausalLink, CausalNode, NodeType};
    /// use nexcore_reason::counterfactual::{CounterfactualEngine, Intervention};
    ///
    /// let dag = CausalDag::new();
    /// let engine = CounterfactualEngine::new(dag);
    /// let results = engine.evaluate_batch(&[]).unwrap();
    /// assert!(results.is_empty());
    /// ```
    pub fn evaluate_batch(
        &self,
        interventions: &[Intervention],
    ) -> Result<Vec<CounterfactualResult>> {
        interventions
            .iter()
            .map(|iv| self.evaluate(iv))
            .collect::<Result<Vec<_>>>()
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Mutates `modified` by applying `intervention`.
    fn apply_intervention(
        &self,
        modified: &mut CausalDag,
        intervention: &Intervention,
        original_ids: &HashSet<NodeId>,
    ) -> Result<()> {
        match intervention {
            Intervention::RemoveNode(id) => {
                anyhow::ensure!(
                    original_ids.contains(id),
                    "RemoveNode: node '{}' does not exist in the DAG",
                    id
                );
                remove_node_from(modified, id);
            }

            Intervention::RemoveLink { from, to } => {
                anyhow::ensure!(
                    modified.link_between(from, to).is_some(),
                    "RemoveLink: no link from '{}' to '{}' exists in the DAG",
                    from,
                    to
                );
                remove_link_from(modified, from, to);
            }

            Intervention::AdjustStrength {
                from,
                to,
                new_strength,
            } => {
                anyhow::ensure!(
                    modified.link_between(from, to).is_some(),
                    "AdjustStrength: no link from '{}' to '{}' exists in the DAG",
                    from,
                    to
                );
                adjust_strength_in(modified, from, to, *new_strength);
            }

            Intervention::InjectNode(node) => {
                anyhow::ensure!(
                    !original_ids.contains(&node.id),
                    "InjectNode: node '{}' already exists in the DAG",
                    node.id
                );
                modified.add_node(node.clone());
            }
        }
        Ok(())
    }

    /// Determines which nodes were meaningfully affected by the intervention.
    ///
    /// A node is "affected" when any of the following are true:
    ///
    /// - Its reachability set changed (it can now reach fewer or more nodes).
    /// - It became a root or leaf where it was not before.
    /// - It is the direct target of the intervention (the removed / adjusted
    ///   node or link endpoints).
    #[allow(clippy::too_many_arguments)]
    fn compute_affected_nodes(
        &self,
        original_roots: &HashSet<NodeId>,
        original_leaves: &HashSet<NodeId>,
        modified_roots: &HashSet<NodeId>,
        modified_leaves: &HashSet<NodeId>,
        original_reachable: &HashSet<(NodeId, NodeId)>,
        modified: &CausalDag,
        modified_ids: &HashSet<NodeId>,
        intervention: &Intervention,
    ) -> Vec<NodeId> {
        let mut affected: HashSet<NodeId> = HashSet::new();

        // Nodes that changed root/leaf status.
        for id in modified_roots.symmetric_difference(original_roots) {
            if modified_ids.contains(id) {
                affected.insert(id.clone());
            }
        }
        for id in modified_leaves.symmetric_difference(original_leaves) {
            if modified_ids.contains(id) {
                affected.insert(id.clone());
            }
        }

        // Nodes whose reachability set shrank.
        for id in modified_ids {
            let original_reach_count = original_reachable
                .iter()
                .filter(|(a, _)| a == id)
                .count();
            let modified_reach_count = modified
                .nodes()
                .iter()
                .filter(|n| &n.id != id && can_reach(modified, id, &n.id))
                .count();
            if original_reach_count != modified_reach_count {
                affected.insert(id.clone());
            }
        }

        // Direct intervention targets.
        match intervention {
            Intervention::RemoveLink { from, to }
            | Intervention::AdjustStrength { from, to, .. } => {
                // Endpoints of the modified link are affected.
                if modified_ids.contains(from) {
                    affected.insert(from.clone());
                }
                if modified_ids.contains(to) {
                    affected.insert(to.clone());
                }
            }
            Intervention::InjectNode(node) => {
                // The newly injected node is itself affected.
                affected.insert(node.id.clone());
            }
            Intervention::RemoveNode(_) => {
                // The removed node is gone; its former in-neighbours are
                // captured by the reachability diff above.
            }
        }

        let mut result: Vec<NodeId> = affected.into_iter().collect();
        result.sort();
        result
    }

    /// Computes an impact score in `[0.0, 1.0]`.
    ///
    /// - For structural interventions (RemoveNode, RemoveLink, InjectNode):
    ///   `affected_count / original_node_count`.
    /// - For [`AdjustStrength`]: the normalised absolute strength delta
    ///   (`|old - new| / 1.0`), with the number of affected endpoint nodes
    ///   layered on top as a secondary weight.
    ///
    /// [`AdjustStrength`]: Intervention::AdjustStrength
    fn compute_impact_score(
        &self,
        intervention: &Intervention,
        affected_nodes: &[NodeId],
        original_node_count: usize,
    ) -> f64 {
        if original_node_count == 0 {
            return 0.0;
        }

        match intervention {
            Intervention::AdjustStrength { from, to, new_strength } => {
                // Derive impact from the magnitude of the strength change.
                let old_strength = self
                    .dag
                    .link_between(from, to)
                    .map(|l| l.strength)
                    .unwrap_or(0.0);
                let delta = (old_strength - new_strength.clamp(0.0, 1.0)).abs();
                // Weight the delta by the fraction of affected nodes so that
                // a tiny delta on a highly connected link still reflects some
                // structural significance.
                let node_fraction =
                    affected_nodes.len() as f64 / original_node_count as f64;
                (delta + node_fraction * 0.1_f64).clamp(0.0, 1.0)
            }
            _ => {
                let fraction =
                    affected_nodes.len() as f64 / original_node_count as f64;
                fraction.clamp(0.0, 1.0)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::{CausalDag, CausalLink, CausalNode, NodeType};

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn node(id: &str, nt: NodeType) -> CausalNode {
        CausalNode {
            id: id.to_string(),
            label: id.to_string(),
            node_type: nt,
        }
    }

    fn link(from: &str, to: &str, s: f64) -> CausalLink {
        CausalLink {
            from: from.to_string(),
            to: to.to_string(),
            strength: s,
        }
    }

    /// Builds a linear three-node chain: a → b → c.
    fn chain_dag() -> CausalDag {
        let mut dag = CausalDag::new();
        dag.add_node(node("a", NodeType::Root));
        dag.add_node(node("b", NodeType::Factor));
        dag.add_node(node("c", NodeType::Risk));
        dag.add_link(link("a", "b", 0.9));
        dag.add_link(link("b", "c", 0.8));
        dag
    }

    /// Builds a diamond: a → b, a → c, b → d, c → d.
    fn diamond_dag() -> CausalDag {
        let mut dag = CausalDag::new();
        dag.add_node(node("a", NodeType::Root));
        dag.add_node(node("b", NodeType::Factor));
        dag.add_node(node("c", NodeType::Factor));
        dag.add_node(node("d", NodeType::Risk));
        dag.add_link(link("a", "b", 0.9));
        dag.add_link(link("a", "c", 0.7));
        dag.add_link(link("b", "d", 0.8));
        dag.add_link(link("c", "d", 0.6));
        dag
    }

    // -----------------------------------------------------------------------
    // RemoveNode
    // -----------------------------------------------------------------------

    #[test]
    fn remove_node_breaks_transitive_path() {
        let engine = CounterfactualEngine::new(chain_dag());
        let result = engine
            .evaluate(&Intervention::RemoveNode("b".to_string()))
            .expect("evaluation failed");

        assert!(
            result
                .broken_paths
                .contains(&("a".to_string(), "c".to_string())),
            "expected a→c to be broken, got {:?}",
            result.broken_paths
        );
    }

    #[test]
    fn remove_node_makes_downstream_new_root() {
        let engine = CounterfactualEngine::new(chain_dag());
        let result = engine
            .evaluate(&Intervention::RemoveNode("a".to_string()))
            .expect("evaluation failed");

        // Removing "a" leaves "b" with no incoming links — it becomes a root.
        assert!(
            result.new_roots.contains(&"b".to_string()),
            "expected b to become a new root, got {:?}",
            result.new_roots
        );
    }

    #[test]
    fn remove_nonexistent_node_returns_error() {
        let engine = CounterfactualEngine::new(chain_dag());
        let err = engine.evaluate(&Intervention::RemoveNode("ghost".to_string()));
        assert!(err.is_err(), "expected error for missing node");
    }

    #[test]
    fn remove_node_impact_score_nonzero() {
        let engine = CounterfactualEngine::new(chain_dag());
        let result = engine
            .evaluate(&Intervention::RemoveNode("b".to_string()))
            .expect("evaluation failed");

        assert!(
            result.impact_score > 0.0,
            "impact score should be positive, got {}",
            result.impact_score
        );
        assert!(
            result.impact_score <= 1.0,
            "impact score must be ≤ 1.0, got {}",
            result.impact_score
        );
    }

    // -----------------------------------------------------------------------
    // RemoveLink
    // -----------------------------------------------------------------------

    #[test]
    fn remove_link_severs_direct_path() {
        let engine = CounterfactualEngine::new(chain_dag());
        let result = engine
            .evaluate(&Intervention::RemoveLink {
                from: "a".to_string(),
                to: "b".to_string(),
            })
            .expect("evaluation failed");

        assert!(
            result
                .broken_paths
                .contains(&("a".to_string(), "b".to_string())),
            "expected a→b to be broken, got {:?}",
            result.broken_paths
        );
        assert!(
            result
                .broken_paths
                .contains(&("a".to_string(), "c".to_string())),
            "expected a→c to be broken too, got {:?}",
            result.broken_paths
        );
    }

    #[test]
    fn remove_link_leaves_alternate_path_intact() {
        // In the diamond, removing a→b should NOT break a→d because a→c→d
        // still exists.
        let engine = CounterfactualEngine::new(diamond_dag());
        let result = engine
            .evaluate(&Intervention::RemoveLink {
                from: "a".to_string(),
                to: "b".to_string(),
            })
            .expect("evaluation failed");

        assert!(
            !result
                .broken_paths
                .contains(&("a".to_string(), "d".to_string())),
            "a→d should still be reachable via c, broken_paths: {:?}",
            result.broken_paths
        );
    }

    #[test]
    fn remove_nonexistent_link_returns_error() {
        let engine = CounterfactualEngine::new(chain_dag());
        let err = engine.evaluate(&Intervention::RemoveLink {
            from: "c".to_string(),
            to: "a".to_string(),
        });
        assert!(err.is_err(), "expected error for missing link");
    }

    // -----------------------------------------------------------------------
    // AdjustStrength
    // -----------------------------------------------------------------------

    #[test]
    fn adjust_strength_does_not_change_structure() {
        let engine = CounterfactualEngine::new(chain_dag());
        let result = engine
            .evaluate(&Intervention::AdjustStrength {
                from: "a".to_string(),
                to: "b".to_string(),
                new_strength: 0.1,
            })
            .expect("evaluation failed");

        // No paths should be structurally broken.
        assert!(
            result.broken_paths.is_empty(),
            "AdjustStrength should not break paths, got {:?}",
            result.broken_paths
        );
    }

    #[test]
    fn adjust_strength_impact_reflects_delta() {
        let engine = CounterfactualEngine::new(chain_dag());

        let large_delta = engine
            .evaluate(&Intervention::AdjustStrength {
                from: "a".to_string(),
                to: "b".to_string(),
                new_strength: 0.0,
            })
            .expect("evaluation failed");

        let small_delta = engine
            .evaluate(&Intervention::AdjustStrength {
                from: "a".to_string(),
                to: "b".to_string(),
                new_strength: 0.85,
            })
            .expect("evaluation failed");

        assert!(
            large_delta.impact_score > small_delta.impact_score,
            "larger strength delta should produce higher impact score: {} vs {}",
            large_delta.impact_score,
            small_delta.impact_score
        );
    }

    #[test]
    fn adjust_nonexistent_link_returns_error() {
        let engine = CounterfactualEngine::new(chain_dag());
        let err = engine.evaluate(&Intervention::AdjustStrength {
            from: "z".to_string(),
            to: "a".to_string(),
            new_strength: 0.5,
        });
        assert!(err.is_err(), "expected error for missing link");
    }

    // -----------------------------------------------------------------------
    // InjectNode
    // -----------------------------------------------------------------------

    #[test]
    fn inject_node_appears_as_new_root_and_leaf() {
        let engine = CounterfactualEngine::new(chain_dag());
        let result = engine
            .evaluate(&Intervention::InjectNode(node("hypo", NodeType::Hypothetical)))
            .expect("evaluation failed");

        // The injected isolated node has no links, so it is both a root and a
        // leaf in the modified graph but was not in the original.
        assert!(
            result.new_roots.contains(&"hypo".to_string()),
            "injected node should be a new root, got {:?}",
            result.new_roots
        );
        assert!(
            result.new_leaves.contains(&"hypo".to_string()),
            "injected node should be a new leaf, got {:?}",
            result.new_leaves
        );
    }

    #[test]
    fn inject_duplicate_node_returns_error() {
        let engine = CounterfactualEngine::new(chain_dag());
        let err = engine.evaluate(&Intervention::InjectNode(node("a", NodeType::Hypothetical)));
        assert!(err.is_err(), "expected error for duplicate node id");
    }

    // -----------------------------------------------------------------------
    // evaluate_batch
    // -----------------------------------------------------------------------

    #[test]
    fn batch_evaluates_all_interventions() {
        let engine = CounterfactualEngine::new(chain_dag());
        let interventions = vec![
            Intervention::RemoveNode("a".to_string()),
            Intervention::RemoveLink {
                from: "a".to_string(),
                to: "b".to_string(),
            },
            Intervention::AdjustStrength {
                from: "b".to_string(),
                to: "c".to_string(),
                new_strength: 0.3,
            },
        ];
        let results = engine
            .evaluate_batch(&interventions)
            .expect("batch evaluation failed");

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn batch_on_empty_slice_returns_empty_vec() {
        let engine = CounterfactualEngine::new(chain_dag());
        let results = engine.evaluate_batch(&[]).expect("empty batch failed");
        assert!(results.is_empty());
    }

    // -----------------------------------------------------------------------
    // Impact score bounds
    // -----------------------------------------------------------------------

    #[test]
    fn impact_score_is_always_in_unit_range() {
        let engine = CounterfactualEngine::new(diamond_dag());
        let interventions = [
            Intervention::RemoveNode("b".to_string()),
            Intervention::RemoveLink {
                from: "a".to_string(),
                to: "b".to_string(),
            },
            Intervention::AdjustStrength {
                from: "a".to_string(),
                to: "c".to_string(),
                new_strength: 0.0,
            },
        ];
        for iv in &interventions {
            let result = engine.evaluate(iv).expect("evaluation failed");
            assert!(
                result.impact_score >= 0.0 && result.impact_score <= 1.0,
                "impact_score out of range for {:?}: {}",
                iv,
                result.impact_score
            );
        }
    }
}
