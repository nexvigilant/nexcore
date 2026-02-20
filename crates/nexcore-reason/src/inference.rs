// Inference engine

//! Causal inference engine for The Foundry's REASON station (A3).
//!
//! The engine consumes a [`CausalDag`] and produces an [`IntelligenceReport`]
//! by traversing all root-to-leaf causal chains, scoring each path by the
//! product of its link strengths, and synthesising findings and recommendations
//! from the highest-scoring paths.
//!
//! # Design
//!
//! Path scoring uses the multiplicative composition of causal strengths along a
//! chain. A chain `A →(0.9)→ B →(0.8)→ C` has a composite score of `0.72`.
//! This models the intuition that evidence weakens as it passes through
//! intermediate factors — each link in the chain attenuates the overall signal.
//!
//! The [`RiskLevel`] is mapped from the highest observed path score, so a
//! single strong chain is sufficient to escalate the overall risk classification
//! even if most other chains are weak.
//!
//! # Pipeline position
//!
//! ```text
//! Inference bridge  →  A3 (InferenceEngine::infer)  →  IntelligenceReport
//! ```
//!
//! # Examples
//!
//! ```
//! use nexcore_reason::dag::{CausalDag, CausalLink, CausalNode, NodeType};
//! use nexcore_reason::inference::InferenceEngine;
//!
//! let mut dag = CausalDag::default();
//!
//! dag.add_node(CausalNode {
//!     id: "missing_tests".to_string(),
//!     label: "Missing test coverage".to_string(),
//!     node_type: NodeType::Root,
//! });
//! dag.add_node(CausalNode {
//!     id: "regression_risk".to_string(),
//!     label: "Regression risk".to_string(),
//!     node_type: NodeType::Risk,
//! });
//! dag.add_link(CausalLink {
//!     from: "missing_tests".to_string(),
//!     to: "regression_risk".to_string(),
//!     strength: 0.85,
//! });
//!
//! let engine = InferenceEngine::new(dag);
//! let report = engine.infer().expect("inference must succeed");
//!
//! assert!(!report.findings.is_empty());
//! assert!(report.confidence > 0.0);
//! ```

use serde::{Deserialize, Serialize};

use crate::dag::{CausalDag, CausalLink, CausalNode, NodeId, NodeType};
use nexcore_foundry::analyst::{CausalEdge, CausalGraph, IntelligenceReport, RiskLevel};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration that governs inference behaviour.
///
/// All thresholds operate on normalised `[0.0, 1.0]` scores derived from
/// the product of causal link strengths along a path.
///
/// # Examples
///
/// ```
/// use nexcore_reason::inference::InferenceConfig;
///
/// let cfg = InferenceConfig::default();
/// assert_eq!(cfg.risk_threshold, 0.7);
/// assert_eq!(cfg.confidence_floor, 0.5);
/// assert_eq!(cfg.max_findings, 10);
/// assert_eq!(cfg.max_recommendations, 5);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Composite path score above which a path is treated as a risk finding.
    ///
    /// Paths whose product score exceeds this value contribute to the
    /// findings list in the resulting [`IntelligenceReport`].
    /// Default: `0.7`.
    pub risk_threshold: f64,

    /// Minimum composite path score required for a path to be included in the
    /// report at all.
    ///
    /// Paths scoring below this value are discarded before findings and
    /// recommendations are generated.
    /// Default: `0.5`.
    pub confidence_floor: f64,

    /// Maximum number of findings to include in the [`IntelligenceReport`].
    ///
    /// Findings are ordered by descending path score; only the top
    /// `max_findings` are retained.
    /// Default: `10`.
    pub max_findings: usize,

    /// Maximum number of recommendations to include in the
    /// [`IntelligenceReport`].
    ///
    /// Recommendations derive from [`NodeType::Recommendation`] nodes
    /// reachable via high-scoring paths; only the top `max_recommendations`
    /// are retained.
    /// Default: `5`.
    pub max_recommendations: usize,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            risk_threshold: 0.7,
            confidence_floor: 0.5,
            max_findings: 10,
            max_recommendations: 5,
        }
    }
}

// ---------------------------------------------------------------------------
// Finding (intermediate representation)
// ---------------------------------------------------------------------------

/// An intermediate finding synthesised from high-scoring causal paths before
/// the final [`IntelligenceReport`] is assembled.
///
/// `Finding` instances are internal to the inference engine; they are
/// converted to plain strings when the report is produced.
///
/// # Examples
///
/// ```
/// use nexcore_reason::inference::Finding;
///
/// let finding = Finding {
///     description: "High complexity drives regression risk".to_string(),
///     severity: 0.82,
///     supporting_paths: vec![vec!["complexity".to_string(), "regression_risk".to_string()]],
/// };
/// assert!(finding.severity > 0.5);
/// assert_eq!(finding.supporting_paths.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Human-readable description of the causal finding.
    pub description: String,

    /// Severity of this finding in the range `[0.0, 1.0]`.
    ///
    /// Derived from the composite path score of the supporting causal chain.
    pub severity: f64,

    /// One or more causal chains (sequences of [`NodeId`]) that provide
    /// evidence for this finding.
    pub supporting_paths: Vec<Vec<NodeId>>,
}

// ---------------------------------------------------------------------------
// InferenceEngine
// ---------------------------------------------------------------------------

/// Causal inference engine for the REASON station (A3).
///
/// Traverses a [`CausalDag`], scores all root-to-leaf paths, and synthesises
/// an [`IntelligenceReport`] with risk classification, findings, and
/// recommendations.
///
/// Construct with [`InferenceEngine::new`] for default configuration or
/// [`InferenceEngine::with_config`] when you need fine-grained control.
///
/// # Examples
///
/// ```
/// use nexcore_reason::dag::{CausalDag, CausalLink, CausalNode, NodeType};
/// use nexcore_reason::inference::{InferenceConfig, InferenceEngine};
///
/// let mut dag = CausalDag::default();
/// dag.add_node(CausalNode {
///     id: "root".to_string(),
///     label: "Root cause".to_string(),
///     node_type: NodeType::Root,
/// });
/// dag.add_node(CausalNode {
///     id: "risk".to_string(),
///     label: "Risk outcome".to_string(),
///     node_type: NodeType::Risk,
/// });
/// dag.add_link(CausalLink {
///     from: "root".to_string(),
///     to: "risk".to_string(),
///     strength: 0.9,
/// });
///
/// let engine = InferenceEngine::new(dag);
/// let report = engine.infer().expect("inference must succeed");
/// assert_eq!(report.risk_level, nexcore_foundry::analyst::RiskLevel::High);
/// ```
#[derive(Debug, Clone)]
pub struct InferenceEngine {
    dag: CausalDag,
    config: InferenceConfig,
}

impl InferenceEngine {
    /// Creates a new [`InferenceEngine`] with default [`InferenceConfig`].
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_reason::dag::CausalDag;
    /// use nexcore_reason::inference::InferenceEngine;
    ///
    /// let engine = InferenceEngine::new(CausalDag::default());
    /// let report = engine.infer().expect("empty DAG produces a valid report");
    /// assert!(report.findings.is_empty());
    /// ```
    #[must_use]
    pub fn new(dag: CausalDag) -> Self {
        Self {
            dag,
            config: InferenceConfig::default(),
        }
    }

    /// Creates a new [`InferenceEngine`] with a custom [`InferenceConfig`].
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_reason::dag::CausalDag;
    /// use nexcore_reason::inference::{InferenceConfig, InferenceEngine};
    ///
    /// let config = InferenceConfig {
    ///     risk_threshold: 0.6,
    ///     confidence_floor: 0.3,
    ///     max_findings: 5,
    ///     max_recommendations: 3,
    /// };
    /// let engine = InferenceEngine::with_config(CausalDag::default(), config);
    /// let report = engine.infer().expect("inference must succeed");
    /// assert!(report.findings.is_empty());
    /// ```
    #[must_use]
    pub fn with_config(dag: CausalDag, config: InferenceConfig) -> Self {
        Self { dag, config }
    }

    /// Runs causal inference over the DAG and produces an [`IntelligenceReport`].
    ///
    /// # Algorithm
    ///
    /// 1. Enumerate all root-to-leaf causal chains via depth-first traversal.
    /// 2. Score each chain as the product of its link strengths.
    /// 3. Discard chains whose score falls below [`InferenceConfig::confidence_floor`].
    /// 4. Classify overall [`RiskLevel`] from the maximum observed chain score.
    /// 5. Generate [`Finding`]s from chains above [`InferenceConfig::risk_threshold`].
    /// 6. Generate recommendations from [`NodeType::Recommendation`] nodes
    ///    reachable via above-threshold chains.
    /// 7. Compute `confidence` as the arithmetic mean of retained chain scores.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if the DAG contains a node referenced by a
    /// link that does not exist in the node list (a referential integrity
    /// violation).
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_reason::dag::{CausalDag, CausalLink, CausalNode, NodeType};
    /// use nexcore_reason::inference::InferenceEngine;
    ///
    /// let engine = InferenceEngine::new(CausalDag::default());
    /// let report = engine.infer().expect("empty DAG must produce Low risk");
    /// assert_eq!(report.risk_level, nexcore_foundry::analyst::RiskLevel::Low);
    /// assert_eq!(report.confidence, 0.0);
    /// ```
    pub fn infer(&self) -> Result<IntelligenceReport, anyhow::Error> {
        // Validate referential integrity: every link endpoint must be a
        // known node identifier.
        self.validate_links()?;

        // Step 1 & 2: find all paths and their composite scores.
        let chains = self.find_causal_chains();

        // Step 3: filter by confidence floor.
        let retained: Vec<(Vec<&NodeId>, f64)> = chains
            .into_iter()
            .filter(|(_, score)| *score >= self.config.confidence_floor)
            .collect();

        // Step 4: overall risk level from the highest score.
        let max_score = retained
            .iter()
            .map(|(_, s)| *s)
            .fold(0.0_f64, f64::max);
        let risk_level = self.classify_risk(max_score);

        // Step 5: findings from high-scoring paths that terminate at Risk nodes.
        let findings = self.build_findings(&retained);

        // Step 6: recommendations from Recommendation nodes in strong paths.
        let recommendations = self.build_recommendations(&retained);

        // Step 7: confidence as arithmetic mean of retained scores.
        let confidence = if retained.is_empty() {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let count = retained.len() as f64;
            retained.iter().map(|(_, s)| *s).sum::<f64>() / count
        };

        Ok(IntelligenceReport {
            findings,
            recommendations,
            risk_level,
            confidence,
        })
    }

    /// Maps a maximum path score to a [`RiskLevel`].
    ///
    /// | Score range | [`RiskLevel`]        |
    /// |-------------|----------------------|
    /// | `[0.0, 0.3)` | [`RiskLevel::Low`]  |
    /// | `[0.3, 0.6)` | [`RiskLevel::Moderate`] |
    /// | `[0.6, 0.8)` | [`RiskLevel::High`]  |
    /// | `[0.8, 1.0]` | [`RiskLevel::Critical`] |
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_reason::dag::CausalDag;
    /// use nexcore_reason::inference::InferenceEngine;
    /// use nexcore_foundry::analyst::RiskLevel;
    ///
    /// let engine = InferenceEngine::new(CausalDag::default());
    /// assert_eq!(engine.classify_risk(0.0), RiskLevel::Low);
    /// assert_eq!(engine.classify_risk(0.29), RiskLevel::Low);
    /// assert_eq!(engine.classify_risk(0.3), RiskLevel::Moderate);
    /// assert_eq!(engine.classify_risk(0.59), RiskLevel::Moderate);
    /// assert_eq!(engine.classify_risk(0.6), RiskLevel::High);
    /// assert_eq!(engine.classify_risk(0.79), RiskLevel::High);
    /// assert_eq!(engine.classify_risk(0.8), RiskLevel::Critical);
    /// assert_eq!(engine.classify_risk(1.0), RiskLevel::Critical);
    /// ```
    #[must_use]
    pub fn classify_risk(&self, max_path_score: f64) -> RiskLevel {
        if max_path_score < 0.3 {
            RiskLevel::Low
        } else if max_path_score < 0.6 {
            RiskLevel::Moderate
        } else if max_path_score < 0.8 {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        }
    }

    /// Enumerates all root-to-leaf causal chains in the DAG with their
    /// composite scores.
    ///
    /// The composite score of a chain is the product of all link strengths
    /// along the path. A chain `A →(0.9)→ B →(0.8)→ C` scores `0.72`.
    ///
    /// Chains are returned in no guaranteed order. Nodes with no outgoing
    /// links are treated as leaves regardless of their [`NodeType`].
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_reason::dag::{CausalDag, CausalLink, CausalNode, NodeType};
    /// use nexcore_reason::inference::InferenceEngine;
    ///
    /// let mut dag = CausalDag::default();
    /// dag.add_node(CausalNode { id: "a".to_string(), label: "A".to_string(), node_type: NodeType::Root });
    /// dag.add_node(CausalNode { id: "b".to_string(), label: "B".to_string(), node_type: NodeType::Risk });
    /// dag.add_link(CausalLink { from: "a".to_string(), to: "b".to_string(), strength: 0.8 });
    ///
    /// let engine = InferenceEngine::new(dag);
    /// let chains = engine.find_causal_chains();
    /// assert_eq!(chains.len(), 1);
    ///
    /// let (path, score) = &chains[0];
    /// assert_eq!(path.len(), 2);
    /// assert!((score - 0.8).abs() < f64::EPSILON);
    /// ```
    #[must_use]
    pub fn find_causal_chains(&self) -> Vec<(Vec<&NodeId>, f64)> {
        let mut results = Vec::new();

        for root in self.dag.roots() {
            let mut current_path: Vec<&NodeId> = vec![&root.id];
            self.dfs_paths(
                &root.id,
                &mut current_path,
                1.0_f64,
                &mut results,
            );
        }

        results
    }

    /// Converts the DAG's link set into a [`CausalGraph`] in the format
    /// expected by the `nexcore_foundry` analyst pipeline.
    ///
    /// Each [`CausalLink`] in the DAG maps to one [`CausalEdge`]. Node
    /// labels are used as the `from` / `to` strings; if a node referenced
    /// by a link has no matching node record, the raw [`NodeId`] is used
    /// instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_reason::dag::{CausalDag, CausalLink, CausalNode, NodeType};
    /// use nexcore_reason::inference::InferenceEngine;
    ///
    /// let mut dag = CausalDag::default();
    /// dag.add_node(CausalNode { id: "a".to_string(), label: "Factor A".to_string(), node_type: NodeType::Root });
    /// dag.add_node(CausalNode { id: "b".to_string(), label: "Risk B".to_string(), node_type: NodeType::Risk });
    /// dag.add_link(CausalLink { from: "a".to_string(), to: "b".to_string(), strength: 0.75 });
    ///
    /// let engine = InferenceEngine::new(dag);
    /// let graph = engine.to_causal_graph();
    /// assert_eq!(graph.edges.len(), 1);
    /// assert_eq!(graph.edges[0].from, "Factor A");
    /// assert_eq!(graph.edges[0].to, "Risk B");
    /// assert!((graph.edges[0].strength - 0.75).abs() < f64::EPSILON);
    /// ```
    #[must_use]
    pub fn to_causal_graph(&self) -> CausalGraph {
        let edges = self
            .dag
            .links()
            .iter()
            .map(|link| self.link_to_edge(link))
            .collect();

        CausalGraph { edges }
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Validates that every endpoint of every link resolves to a known node.
    fn validate_links(&self) -> Result<(), anyhow::Error> {
        for link in self.dag.links() {
            if self.dag.node(&link.from).is_none() {
                return Err(anyhow::anyhow!(
                    "causal link references unknown source node `{}`",
                    link.from
                ));
            }
            if self.dag.node(&link.to).is_none() {
                return Err(anyhow::anyhow!(
                    "causal link references unknown target node `{}`",
                    link.to
                ));
            }
        }
        Ok(())
    }

    /// Depth-first traversal that accumulates all root-to-leaf paths.
    ///
    /// `current_id` is the node currently being visited; `path` holds the
    /// chain of node IDs visited so far (including `current_id`);
    /// `running_score` is the product of link strengths accumulated so far
    /// along the current path.
    fn dfs_paths<'dag>(
        &'dag self,
        current_id: &str,
        path: &mut Vec<&'dag NodeId>,
        running_score: f64,
        results: &mut Vec<(Vec<&'dag NodeId>, f64)>,
    ) {
        let children = self.dag.children(current_id);

        if children.is_empty() {
            // Leaf node — record this path.
            results.push((path.clone(), running_score));
            return;
        }

        for child in children {
            let edge_strength = self
                .dag
                .link_between(current_id, &child.id)
                .map_or(1.0, |l| l.strength);

            let new_score = running_score * edge_strength;
            path.push(&child.id);
            self.dfs_paths(&child.id, path, new_score, results);
            path.pop();
        }
    }

    /// Converts a [`CausalLink`] to a [`CausalEdge`], resolving node IDs to
    /// their human-readable labels where possible.
    fn link_to_edge(&self, link: &CausalLink) -> CausalEdge {
        let from_label = self
            .dag
            .node(&link.from)
            .map_or_else(|| link.from.clone(), |n| n.label.clone());

        let to_label = self
            .dag
            .node(&link.to)
            .map_or_else(|| link.to.clone(), |n| n.label.clone());

        CausalEdge {
            from: from_label,
            to: to_label,
            strength: link.strength,
        }
    }

    /// Builds [`Finding`] descriptions from paths that terminate at
    /// [`NodeType::Risk`] nodes and whose score exceeds the risk threshold.
    fn build_findings(&self, chains: &[(Vec<&NodeId>, f64)]) -> Vec<String> {
        let mut findings: Vec<Finding> = Vec::new();

        for (path, score) in chains {
            let score = *score;
            if score < self.config.risk_threshold {
                continue;
            }

            // Only create a finding when the terminal node is a Risk node.
            let terminal_id = match path.last() {
                Some(id) => id,
                None => continue,
            };

            let terminal_node = match self.dag.node(terminal_id) {
                Some(n) => n,
                None => continue,
            };

            if terminal_node.node_type != NodeType::Risk {
                continue;
            }

            let description = self.describe_finding(path, score, terminal_node);
            let owned_path: Vec<NodeId> = path.iter().map(|&id| id.clone()).collect();

            // Merge with an existing finding for the same terminal node, or
            // create a new one.
            if let Some(existing) = findings
                .iter_mut()
                .find(|f| f.description == description)
            {
                existing.supporting_paths.push(owned_path);
                if score > existing.severity {
                    existing.severity = score;
                }
            } else {
                findings.push(Finding {
                    description,
                    severity: score,
                    supporting_paths: vec![owned_path],
                });
            }
        }

        // Sort by descending severity and truncate to max_findings.
        findings.sort_by(|a, b| {
            b.severity
                .partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        findings.truncate(self.config.max_findings);

        findings
            .into_iter()
            .map(|f| f.description)
            .collect()
    }

    /// Builds recommendation strings from [`NodeType::Recommendation`] nodes
    /// reachable via above-threshold chains.
    fn build_recommendations(&self, chains: &[(Vec<&NodeId>, f64)]) -> Vec<String> {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut recommendations: Vec<(String, f64)> = Vec::new();

        for (path, score) in chains {
            let score = *score;
            if score < self.config.risk_threshold {
                continue;
            }

            for node_id in path.iter() {
                let node = match self.dag.node(node_id) {
                    Some(n) => n,
                    None => continue,
                };

                if node.node_type != NodeType::Recommendation {
                    continue;
                }

                if seen.insert(node.id.clone()) {
                    recommendations.push((node.label.clone(), score));
                }
            }
        }

        // Sort by descending score (highest-confidence recommendations first).
        recommendations.sort_by(|(_, a), (_, b)| {
            b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)
        });
        recommendations.truncate(self.config.max_recommendations);

        recommendations.into_iter().map(|(label, _)| label).collect()
    }

    /// Formats a human-readable description for a single causal finding.
    fn describe_finding(
        &self,
        path: &[&NodeId],
        score: f64,
        terminal: &CausalNode,
    ) -> String {
        let chain_labels: Vec<String> = path
            .iter()
            .map(|id| {
                self.dag
                    .node(id)
                    .map_or_else(|| (*id).clone(), |n| n.label.clone())
            })
            .collect();

        format!(
            "Causal chain ({score:.2}) leads to risk \"{risk}\": {chain}",
            score = score,
            risk = terminal.label,
            chain = chain_labels.join(" → "),
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use nexcore_foundry::analyst::RiskLevel;

    use crate::dag::{CausalDag, CausalLink, CausalNode, NodeType};

    use super::{InferenceConfig, InferenceEngine};

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn node(id: &str, label: &str, node_type: NodeType) -> CausalNode {
        CausalNode {
            id: id.to_string(),
            label: label.to_string(),
            node_type,
        }
    }

    fn link(from: &str, to: &str, strength: f64) -> CausalLink {
        CausalLink {
            from: from.to_string(),
            to: to.to_string(),
            strength,
        }
    }

    // -----------------------------------------------------------------------
    // Test 1: empty DAG
    // -----------------------------------------------------------------------

    /// An empty DAG has no chains, yields no findings, and produces
    /// [`RiskLevel::Low`] with zero confidence.
    #[test]
    fn empty_dag_produces_low_risk_report() {
        let engine = InferenceEngine::new(CausalDag::default());
        let report = engine.infer().expect("empty DAG inference must succeed");

        assert!(report.findings.is_empty(), "no findings for empty dag");
        assert!(
            report.recommendations.is_empty(),
            "no recommendations for empty dag"
        );
        assert_eq!(report.risk_level, RiskLevel::Low);
        assert_eq!(report.confidence, 0.0);
    }

    // -----------------------------------------------------------------------
    // Test 2: single root → risk path
    // -----------------------------------------------------------------------

    /// A single root → risk chain with strength 0.85 should produce one
    /// finding, [`RiskLevel::High`] (0.85 ≥ 0.8 → Critical by default), and
    /// confidence ≈ 0.85.
    ///
    /// Note: 0.85 ≥ 0.8 maps to Critical.
    #[test]
    fn single_path_dag_produces_critical_risk() {
        let mut dag = CausalDag::default();
        dag.add_node(node("root", "Root cause", NodeType::Root));
        dag.add_node(node("risk", "Risk outcome", NodeType::Risk));
        dag.add_link(link("root", "risk", 0.85));

        let engine = InferenceEngine::new(dag);
        let report = engine.infer().expect("single-path inference must succeed");

        assert_eq!(report.risk_level, RiskLevel::Critical);
        assert!(!report.findings.is_empty(), "should have at least one finding");
        assert!(
            (report.confidence - 0.85).abs() < 1e-9,
            "confidence should be 0.85"
        );
    }

    // -----------------------------------------------------------------------
    // Test 3: multi-path DAG with risk classification
    // -----------------------------------------------------------------------

    /// A DAG with two paths: one weak (0.4) and one strong (0.75).
    /// The strong path triggers High risk.  The weak path is above the
    /// confidence_floor (0.5 default is NOT met by 0.4), so it is dropped.
    /// Only the strong path appears in findings.
    #[test]
    fn multi_path_risk_classification_uses_highest_score() {
        let mut dag = CausalDag::default();

        // Path 1: root1 → risk1 (score 0.75 — above threshold 0.7 → High)
        dag.add_node(node("root1", "High complexity", NodeType::Root));
        dag.add_node(node("risk1", "Regression risk", NodeType::Risk));
        dag.add_link(link("root1", "risk1", 0.75));

        // Path 2: root2 → risk2 (score 0.40 — below confidence floor 0.5)
        dag.add_node(node("root2", "Missing docs", NodeType::Root));
        dag.add_node(node("risk2", "Onboarding delay", NodeType::Risk));
        dag.add_link(link("root2", "risk2", 0.40));

        let engine = InferenceEngine::new(dag);
        let report = engine.infer().expect("multi-path inference must succeed");

        // Risk level driven by the strong path (0.75 < 0.8 → High).
        assert_eq!(report.risk_level, RiskLevel::High);

        // Only the strong path (above threshold) produces a finding.
        assert_eq!(report.findings.len(), 1);
        assert!(
            report.findings[0].contains("Regression risk"),
            "finding should mention the risk node label"
        );
    }

    // -----------------------------------------------------------------------
    // Test 4: config overrides
    // -----------------------------------------------------------------------

    /// A lower `confidence_floor` admits weaker paths; a lower
    /// `risk_threshold` allows them to produce findings too.
    #[test]
    fn config_overrides_affect_report() {
        let mut dag = CausalDag::default();

        // Weak path: score 0.3 — below default floor of 0.5 but above
        // the override floor of 0.2.
        dag.add_node(node("root", "Weak root", NodeType::Root));
        dag.add_node(node("risk", "Minor risk", NodeType::Risk));
        dag.add_link(link("root", "risk", 0.3));

        let config = InferenceConfig {
            confidence_floor: 0.2,
            risk_threshold: 0.25,
            max_findings: 10,
            max_recommendations: 5,
        };
        let engine = InferenceEngine::with_config(dag, config);
        let report = engine.infer().expect("config override inference must succeed");

        // 0.3 < 0.6 → Moderate (not Low, because floor is met and score = 0.3).
        assert_eq!(report.risk_level, RiskLevel::Moderate);

        // The weak path is above the overridden threshold (0.25) → finding.
        assert!(!report.findings.is_empty(), "should find the weak path");
        assert!(
            (report.confidence - 0.3).abs() < 1e-9,
            "confidence should equal the single retained path score"
        );
    }

    // -----------------------------------------------------------------------
    // Test 5: causal graph export
    // -----------------------------------------------------------------------

    /// `to_causal_graph` converts DAG links to `CausalEdge`s using node
    /// labels as endpoint names and preserving link strengths exactly.
    #[test]
    fn causal_graph_export_uses_labels_and_preserves_strength() {
        let mut dag = CausalDag::default();
        dag.add_node(node("n1", "Factor Alpha", NodeType::Root));
        dag.add_node(node("n2", "Risk Beta", NodeType::Risk));
        dag.add_link(link("n1", "n2", 0.62));

        let engine = InferenceEngine::new(dag);
        let graph = engine.to_causal_graph();

        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].from, "Factor Alpha");
        assert_eq!(graph.edges[0].to, "Risk Beta");
        assert!(
            (graph.edges[0].strength - 0.62).abs() < f64::EPSILON,
            "strength must be preserved exactly"
        );
    }

    // -----------------------------------------------------------------------
    // Test 6: recommendation nodes surface in report
    // -----------------------------------------------------------------------

    /// A path through a [`NodeType::Recommendation`] node that scores above
    /// the risk threshold should surface that node's label as a recommendation.
    #[test]
    fn recommendation_nodes_appear_in_report() {
        let mut dag = CausalDag::default();
        dag.add_node(node("root", "Missing coverage", NodeType::Root));
        dag.add_node(node("factor", "Untested paths", NodeType::Factor));
        dag.add_node(node("rec", "Add integration tests", NodeType::Recommendation));
        dag.add_link(link("root", "factor", 0.9));
        dag.add_link(link("factor", "rec", 0.9));

        let engine = InferenceEngine::new(dag);
        let report = engine.infer().expect("recommendation inference must succeed");

        assert!(
            report.recommendations.iter().any(|r| r == "Add integration tests"),
            "recommendation label should appear in report; got: {:?}",
            report.recommendations
        );
    }

    // -----------------------------------------------------------------------
    // Test 7: integrity error on dangling link
    // -----------------------------------------------------------------------

    /// A DAG with a link that references a node not in the node list must
    /// return an error from `infer`.
    #[test]
    fn dangling_link_produces_error() {
        let mut dag = CausalDag::default();
        dag.add_node(node("exists", "Existing node", NodeType::Root));
        // Link references "missing" which has no node record.
        dag.add_link(link("exists", "missing", 0.5));

        let engine = InferenceEngine::new(dag);
        let result = engine.infer();
        assert!(result.is_err(), "dangling link should produce an error");
    }

    // -----------------------------------------------------------------------
    // Test 8: multi-hop path scores correctly
    // -----------------------------------------------------------------------

    /// A three-hop chain A →(0.9)→ B →(0.8)→ C →(0.9)→ D should score
    /// 0.9 × 0.8 × 0.9 = 0.648.
    #[test]
    fn multi_hop_path_score_is_product_of_strengths() {
        let mut dag = CausalDag::default();
        dag.add_node(node("a", "A", NodeType::Root));
        dag.add_node(node("b", "B", NodeType::Factor));
        dag.add_node(node("c", "C", NodeType::Factor));
        dag.add_node(node("d", "D", NodeType::Risk));
        dag.add_link(link("a", "b", 0.9));
        dag.add_link(link("b", "c", 0.8));
        dag.add_link(link("c", "d", 0.9));

        let engine = InferenceEngine::new(dag);
        let chains = engine.find_causal_chains();

        assert_eq!(chains.len(), 1);
        let expected = 0.9_f64 * 0.8 * 0.9;
        assert!(
            (chains[0].1 - expected).abs() < 1e-9,
            "score was {}, expected {}",
            chains[0].1,
            expected
        );
    }

    // -----------------------------------------------------------------------
    // Test 9: classify_risk boundary values
    // -----------------------------------------------------------------------

    #[test]
    fn classify_risk_boundary_values() {
        let engine = InferenceEngine::new(CausalDag::default());

        assert_eq!(engine.classify_risk(0.0), RiskLevel::Low);
        assert_eq!(engine.classify_risk(0.299), RiskLevel::Low);
        assert_eq!(engine.classify_risk(0.3), RiskLevel::Moderate);
        assert_eq!(engine.classify_risk(0.599), RiskLevel::Moderate);
        assert_eq!(engine.classify_risk(0.6), RiskLevel::High);
        assert_eq!(engine.classify_risk(0.799), RiskLevel::High);
        assert_eq!(engine.classify_risk(0.8), RiskLevel::Critical);
        assert_eq!(engine.classify_risk(1.0), RiskLevel::Critical);
    }
}
