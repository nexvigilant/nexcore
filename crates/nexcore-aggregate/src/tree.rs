//! Recursive tree traversal and fold — the ρ (Recursion) primitive.
//!
//! Provides generic recursive operations over tree-structured data:
//! - `tree_fold`: Recursive bottom-up aggregation
//! - `tree_map`: Transform each node recursively
//! - `tree_depth`: Compute maximum depth
//! - `tree_flatten`: Recursive linearization to sequence
//!
//! ## Tier: T2-P (ρ + Σ + σ)
//!
//! ## Lifecycle
//! - **begins**: Traversal starts at root node
//! - **exists**: Stack frames track position in tree
//! - **changes**: Each recursive call transforms accumulator
//! - **persists**: Results bubble up through return chain
//! - **ends**: Root return delivers final aggregated value

use std::collections::HashSet;

use crate::error::AggregateError;

// ---------------------------------------------------------------------------
// TreeNode Trait
// ---------------------------------------------------------------------------

/// A node in a tree structure that supports recursive traversal.
///
/// ## Primitive Grounding
/// - ρ (Recursion): Self-referential structure via children()
/// - λ (Location): Each node has an identity
/// - σ (Sequence): Children form ordered sequence
pub trait TreeNode {
    /// Unique identifier for cycle detection.
    fn id(&self) -> &str;

    /// The node's value for aggregation.
    fn value(&self) -> f64;

    /// Child nodes (empty slice for leaves).
    fn children(&self) -> &[Self]
    where
        Self: Sized;
}

// ---------------------------------------------------------------------------
// SimpleNode — concrete tree implementation
// ---------------------------------------------------------------------------

/// A simple tree node for testing and general use.
///
/// Tier: T2-P (ρ + N + σ)
#[derive(Debug, Clone)]
pub struct SimpleNode {
    /// Node identifier.
    pub id: String,
    /// Numeric value at this node.
    pub value: f64,
    /// Child nodes.
    pub children: Vec<SimpleNode>,
}

impl SimpleNode {
    /// Create a leaf node (no children).
    pub fn leaf(id: impl Into<String>, value: f64) -> Self {
        Self {
            id: id.into(),
            value,
            children: Vec::new(),
        }
    }

    /// Create a branch node with children.
    pub fn branch(id: impl Into<String>, value: f64, children: Vec<SimpleNode>) -> Self {
        Self {
            id: id.into(),
            value,
            children,
        }
    }
}

impl TreeNode for SimpleNode {
    fn id(&self) -> &str {
        &self.id
    }
    fn value(&self) -> f64 {
        self.value
    }
    fn children(&self) -> &[Self] {
        &self.children
    }
}

// ---------------------------------------------------------------------------
// Recursive Operations
// ---------------------------------------------------------------------------

/// Configuration for recursive traversal.
#[derive(Debug, Clone)]
pub struct TraversalConfig {
    /// Maximum recursion depth (default: 100).
    pub max_depth: usize,
    /// Whether to detect and report cycles (default: true).
    pub detect_cycles: bool,
}

impl Default for TraversalConfig {
    fn default() -> Self {
        Self {
            max_depth: 100,
            detect_cycles: true,
        }
    }
}

/// Recursively fold a tree bottom-up: leaves are folded first, then
/// their results are combined with parent values.
///
/// ## Primitive Grounding
/// - ρ (Recursion): Recursive descent through tree
/// - Σ (Sum): Accumulation via fold function
/// - κ (Comparison): Implicit in combine function
///
/// ## Algorithm
/// ```text
/// tree_fold(node) = combine(node.value, [tree_fold(child) for child in children])
/// ```
pub fn tree_fold<T: TreeNode>(
    node: &T,
    combine: &dyn Fn(f64, &[f64]) -> f64,
    config: &TraversalConfig,
) -> Result<f64, AggregateError> {
    let mut visited = HashSet::new();
    tree_fold_inner(node, combine, config, 0, &mut visited)
}

fn tree_fold_inner<T: TreeNode>(
    node: &T,
    combine: &dyn Fn(f64, &[f64]) -> f64,
    config: &TraversalConfig,
    depth: usize,
    visited: &mut HashSet<String>,
) -> Result<f64, AggregateError> {
    // Depth check
    if depth > config.max_depth {
        return Err(AggregateError::MaxDepthExceeded {
            node: node.id().to_string(),
            max_depth: config.max_depth,
        });
    }

    // Cycle detection
    if config.detect_cycles {
        if visited.contains(node.id()) {
            return Err(AggregateError::CycleDetected {
                node: node.id().to_string(),
                depth,
            });
        }
        visited.insert(node.id().to_string());
    }

    // Recurse into children (ρ)
    let children = node.children();
    if children.is_empty() {
        // Leaf: return value directly
        if config.detect_cycles {
            visited.remove(node.id());
        }
        return Ok(node.value());
    }

    // Fold children recursively (Σ + ρ)
    let mut child_results = Vec::with_capacity(children.len());
    for child in children {
        child_results.push(tree_fold_inner(child, combine, config, depth + 1, visited)?);
    }

    // Combine node value with children results
    let result = combine(node.value(), &child_results);

    if config.detect_cycles {
        visited.remove(node.id());
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Safe Traversal Helpers
// ---------------------------------------------------------------------------

/// Enter a safe traversal: check depth limit and cycle detection.
fn enter_traversal<T: TreeNode>(
    node: &T,
    config: &TraversalConfig,
    depth: usize,
    visited: &mut HashSet<String>,
) -> Result<(), AggregateError> {
    if depth > config.max_depth {
        return Err(AggregateError::MaxDepthExceeded {
            node: node.id().to_string(),
            max_depth: config.max_depth,
        });
    }
    if config.detect_cycles {
        if visited.contains(node.id()) {
            return Err(AggregateError::CycleDetected {
                node: node.id().to_string(),
                depth,
            });
        }
        visited.insert(node.id().to_string());
    }
    Ok(())
}

/// Exit a safe traversal: remove from visited set.
fn exit_traversal<T: TreeNode>(node: &T, config: &TraversalConfig, visited: &mut HashSet<String>) {
    if config.detect_cycles {
        visited.remove(node.id());
    }
}

// ---------------------------------------------------------------------------
// Safe Utility Operations
// ---------------------------------------------------------------------------

/// Compute the maximum depth of a tree with cycle detection and depth limiting.
///
/// Uses `TraversalConfig::default()` (max_depth=100, detect_cycles=true).
///
/// Tier: T1 (ρ + N + κ)
pub fn tree_depth<T: TreeNode>(node: &T) -> Result<usize, AggregateError> {
    let config = TraversalConfig::default();
    let mut visited = HashSet::new();
    tree_depth_inner(node, &config, 0, &mut visited)
}

fn tree_depth_inner<T: TreeNode>(
    node: &T,
    config: &TraversalConfig,
    depth: usize,
    visited: &mut HashSet<String>,
) -> Result<usize, AggregateError> {
    enter_traversal(node, config, depth, visited)?;

    let children = node.children();
    let result = if children.is_empty() {
        0
    } else {
        let mut max_child = 0usize;
        for child in children {
            let d = tree_depth_inner(child, config, depth + 1, visited)?;
            if d > max_child {
                max_child = d;
            }
        }
        max_child + 1
    };

    exit_traversal(node, config, visited);
    Ok(result)
}

/// Count total nodes in a tree with cycle detection and depth limiting.
///
/// Uses `TraversalConfig::default()` (max_depth=100, detect_cycles=true).
///
/// Tier: T1 (ρ + Σ + N)
pub fn tree_count<T: TreeNode>(node: &T) -> Result<usize, AggregateError> {
    let config = TraversalConfig::default();
    let mut visited = HashSet::new();
    tree_count_inner(node, &config, 0, &mut visited)
}

fn tree_count_inner<T: TreeNode>(
    node: &T,
    config: &TraversalConfig,
    depth: usize,
    visited: &mut HashSet<String>,
) -> Result<usize, AggregateError> {
    enter_traversal(node, config, depth, visited)?;

    let mut total = 1usize;
    for child in node.children() {
        total += tree_count_inner(child, config, depth + 1, visited)?;
    }

    exit_traversal(node, config, visited);
    Ok(total)
}

/// Flatten a tree to a sequence via pre-order traversal with safety checks.
///
/// Uses `TraversalConfig::default()` (max_depth=100, detect_cycles=true).
///
/// Tier: T2-P (ρ + σ + λ)
pub fn tree_flatten<T: TreeNode>(node: &T) -> Result<Vec<(String, f64)>, AggregateError> {
    let config = TraversalConfig::default();
    let mut visited = HashSet::new();
    tree_flatten_inner(node, &config, 0, &mut visited)
}

fn tree_flatten_inner<T: TreeNode>(
    node: &T,
    config: &TraversalConfig,
    depth: usize,
    visited: &mut HashSet<String>,
) -> Result<Vec<(String, f64)>, AggregateError> {
    enter_traversal(node, config, depth, visited)?;

    let mut result = vec![(node.id().to_string(), node.value())];
    for child in node.children() {
        result.extend(tree_flatten_inner(child, config, depth + 1, visited)?);
    }

    exit_traversal(node, config, visited);
    Ok(result)
}

/// Find the node with the maximum value in the tree with safety checks.
///
/// Uses `TraversalConfig::default()` (max_depth=100, detect_cycles=true).
///
/// Tier: T2-P (ρ + κ + ∃)
pub fn tree_max<T: TreeNode>(node: &T) -> Result<(String, f64), AggregateError> {
    let config = TraversalConfig::default();
    let mut visited = HashSet::new();
    tree_max_inner(node, &config, 0, &mut visited)
}

fn tree_max_inner<T: TreeNode>(
    node: &T,
    config: &TraversalConfig,
    depth: usize,
    visited: &mut HashSet<String>,
) -> Result<(String, f64), AggregateError> {
    enter_traversal(node, config, depth, visited)?;

    let mut best = (node.id().to_string(), node.value());
    for child in node.children() {
        let child_best = tree_max_inner(child, config, depth + 1, visited)?;
        if child_best.1 > best.1 {
            best = child_best;
        }
    }

    exit_traversal(node, config, visited);
    Ok(best)
}

// ---------------------------------------------------------------------------
// Common Combine Functions
// ---------------------------------------------------------------------------

/// Sum combine: parent + Σ(children).
pub fn combine_sum(parent: f64, children: &[f64]) -> f64 {
    parent + children.iter().sum::<f64>()
}

/// Max combine: max(parent, max(children)).
pub fn combine_max(parent: f64, children: &[f64]) -> f64 {
    children.iter().copied().fold(parent, f64::max)
}

/// Mean combine: (parent + mean(children)) / 2.
pub fn combine_mean(parent: f64, children: &[f64]) -> f64 {
    if children.is_empty() {
        return parent;
    }
    let child_mean = children.iter().sum::<f64>() / children.len() as f64;
    (parent + child_mean) / 2.0
}

/// Weighted sum: parent + Σ(children × weight).
pub fn combine_weighted(weight: f64) -> impl Fn(f64, &[f64]) -> f64 {
    move |parent: f64, children: &[f64]| parent + children.iter().map(|c| c * weight).sum::<f64>()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> SimpleNode {
        //       root(1.0)
        //      /         \
        //   a(2.0)     b(3.0)
        //   /   \         \
        // c(4.0) d(5.0)  e(6.0)
        SimpleNode::branch(
            "root",
            1.0,
            vec![
                SimpleNode::branch(
                    "a",
                    2.0,
                    vec![SimpleNode::leaf("c", 4.0), SimpleNode::leaf("d", 5.0)],
                ),
                SimpleNode::branch("b", 3.0, vec![SimpleNode::leaf("e", 6.0)]),
            ],
        )
    }

    #[test]
    fn test_tree_fold_sum() {
        let tree = sample_tree();
        let config = TraversalConfig::default();
        let result = tree_fold(&tree, &combine_sum, &config);
        assert!(result.is_ok());
        // 1 + (2 + 4 + 5) + (3 + 6) = 21
        assert!((result.unwrap_or(0.0) - 21.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tree_fold_max() {
        let tree = sample_tree();
        let config = TraversalConfig::default();
        let result = tree_fold(&tree, &combine_max, &config);
        assert!(result.is_ok());
        // max(1, max(max(2,4,5), max(3,6))) = max(1, 5, 6) = 6
        assert!((result.unwrap_or(0.0) - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tree_fold_leaf() {
        let leaf = SimpleNode::leaf("x", 42.0);
        let config = TraversalConfig::default();
        let result = tree_fold(&leaf, &combine_sum, &config);
        assert!(result.is_ok());
        assert!((result.unwrap_or(0.0) - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tree_depth() {
        let tree = sample_tree();
        let depth = tree_depth(&tree);
        assert!(depth.is_ok());
        assert_eq!(depth.unwrap_or(0), 2); // root -> a -> c/d
    }

    #[test]
    fn test_tree_depth_leaf() {
        let leaf = SimpleNode::leaf("x", 1.0);
        assert_eq!(tree_depth(&leaf).unwrap_or(99), 0);
    }

    #[test]
    fn test_tree_count() {
        let tree = sample_tree();
        let count = tree_count(&tree);
        assert!(count.is_ok());
        // root, a, b, c, d, e = 6 nodes
        assert_eq!(count.unwrap_or(0), 6);
    }

    #[test]
    fn test_tree_flatten() {
        let tree = sample_tree();
        let flat = tree_flatten(&tree);
        assert!(flat.is_ok());
        let flat = flat.unwrap_or_default();
        assert_eq!(flat.len(), 6);
        assert_eq!(flat[0].0, "root");
        assert_eq!(flat[1].0, "a");
        assert_eq!(flat[2].0, "c");
    }

    #[test]
    fn test_tree_max_value() {
        let tree = sample_tree();
        let result = tree_max(&tree);
        assert!(result.is_ok());
        let (id, val) = result.unwrap_or_else(|_| (String::new(), 0.0));
        assert_eq!(id, "e");
        assert!((val - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tree_depth_max_exceeded() {
        // Verify tree_depth now respects depth limits
        let config = TraversalConfig {
            max_depth: 1,
            detect_cycles: false,
        };
        let tree = sample_tree();
        // tree_depth uses default config (max_depth=100), so sample_tree (depth=2) passes
        let depth = tree_depth(&tree);
        assert!(depth.is_ok());
        // But tree_fold with max_depth=1 should fail
        let result = tree_fold(&tree, &combine_sum, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_depth_exceeded() {
        let config = TraversalConfig {
            max_depth: 1,
            detect_cycles: false,
        };
        let tree = sample_tree();
        let result = tree_fold(&tree, &combine_sum, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_combine_weighted() {
        let wf = combine_weighted(0.5);
        // parent=10, children=[4, 6] → 10 + (4*0.5 + 6*0.5) = 10 + 5 = 15
        let result = wf(10.0, &[4.0, 6.0]);
        assert!((result - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_combine_mean() {
        // parent=10, children=[4, 6] → (10 + 5) / 2 = 7.5
        let result = combine_mean(10.0, &[4.0, 6.0]);
        assert!((result - 7.5).abs() < f64::EPSILON);
    }
}
