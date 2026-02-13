//! # Decision Tree Skill Router
//!
//! Data-driven skill routing using a trained CART decision tree.
//! Learns from historical (task_features → skill_name) pairs.
//!
//! Tier: T2-C (composes T1 mapping with T3 skill types)
//! Grounds to: T1::Mapping (μ) — task features → skill selection

use nexcore_dtree::prelude::*;

use super::routing::RoutingResult;

// ============================================================================
// Task feature extraction
// ============================================================================

/// Feature indices for task characterization
pub mod feature_index {
    /// Number of items/entities in the task
    pub const ITEM_COUNT: usize = 0;
    /// Whether the task is repetitive (0.0 or 1.0)
    pub const IS_REPETITIVE: usize = 1;
    /// Whether the task has structure (0.0 or 1.0)
    pub const HAS_STRUCTURE: usize = 2;
    /// Whether the task needs reasoning (0.0 or 1.0)
    pub const NEEDS_REASONING: usize = 3;
    /// Query length in words (log2-scaled)
    pub const QUERY_LENGTH_LOG2: usize = 4;
    /// Total feature count
    pub const COUNT: usize = 5;
}

/// Feature names for explainability
const FEATURE_NAMES: [&str; feature_index::COUNT] = [
    "item_count",
    "is_repetitive",
    "has_structure",
    "needs_reasoning",
    "query_length_log2",
];

/// Characteristics of a task for routing decisions.
///
/// Tier: T2-C — composed task descriptor
#[derive(Debug, Clone)]
pub struct TaskCharacteristics {
    /// Number of items/entities involved
    pub item_count: usize,
    /// Whether the task is repetitive
    pub is_repetitive: bool,
    /// Whether the task has inherent structure
    pub has_structure: bool,
    /// Whether the task requires reasoning
    pub needs_reasoning: bool,
    /// The query text length in words
    pub query_word_count: usize,
}

/// Extract features from task characteristics.
///
/// Tier: T1 Mapping (μ) — domain → numeric
#[must_use]
pub fn extract_features(task: &TaskCharacteristics) -> Vec<Feature> {
    let query_log2 = if task.query_word_count > 0 {
        (task.query_word_count as f64).log2()
    } else {
        0.0
    };

    vec![
        Feature::Continuous(task.item_count as f64),
        Feature::Continuous(if task.is_repetitive { 1.0 } else { 0.0 }),
        Feature::Continuous(if task.has_structure { 1.0 } else { 0.0 }),
        Feature::Continuous(if task.needs_reasoning { 1.0 } else { 0.0 }),
        Feature::Continuous(query_log2),
    ]
}

// ============================================================================
// DTree Router
// ============================================================================

/// Prediction result from the dtree router
#[derive(Debug, Clone)]
pub struct DtreeRoutingResult {
    /// Routing result (compatible with existing system)
    pub routing: RoutingResult,
    /// Decision path for explainability
    pub path: Vec<String>,
    /// Number of training samples in the matching leaf
    pub leaf_samples: usize,
}

/// Decision tree-backed skill router.
///
/// Trained on historical (task → skill) pairs, provides data-driven
/// routing as a complement to fuzzy/tag-based strategies.
///
/// Tier: T2-C (composed mapping + state)
pub struct DtreeRouter {
    /// The trained decision tree
    tree: DecisionTree,
    /// Minimum confidence to return a routing result
    min_confidence: f64,
}

impl DtreeRouter {
    /// Train from historical task/skill pairs.
    ///
    /// # Errors
    /// Returns `Err` if training data is empty or training fails.
    pub fn train(
        tasks: &[TaskCharacteristics],
        skill_names: &[&str],
        config: TreeConfig,
    ) -> Result<Self, nexcore_dtree::train::TrainError> {
        if tasks.is_empty() || tasks.len() != skill_names.len() {
            return Err(nexcore_dtree::train::TrainError::EmptyData);
        }

        let features: Vec<Vec<Feature>> = tasks.iter().map(|t| extract_features(t)).collect();

        let labels: Vec<String> = skill_names.iter().map(|s| (*s).to_string()).collect();

        let mut tree = fit(&features, &labels, config)?;
        tree.set_feature_names(FEATURE_NAMES.iter().map(|s| (*s).to_string()).collect());

        Ok(Self {
            tree,
            min_confidence: 0.5,
        })
    }

    /// Set minimum confidence threshold.
    #[must_use]
    pub fn with_min_confidence(mut self, threshold: f64) -> Self {
        self.min_confidence = threshold.clamp(0.0, 1.0);
        self
    }

    /// Route a task to a skill.
    ///
    /// Returns `None` if confidence is below threshold.
    #[must_use]
    pub fn route(&self, task: &TaskCharacteristics) -> Option<DtreeRoutingResult> {
        let features = extract_features(task);
        let result = predict(&self.tree, &features).ok()?;

        if result.confidence.value() < self.min_confidence {
            return None;
        }

        let path: Vec<String> = result.path.iter().map(|step| format!("{step}")).collect();

        Some(DtreeRoutingResult {
            routing: RoutingResult {
                skill_name: result.prediction,
                score: result.confidence.value(),
                strategy: super::routing::RoutingStrategy::DtreeBased,
            },
            path,
            leaf_samples: result.leaf_samples,
        })
    }

    /// Get feature importance scores.
    #[must_use]
    pub fn importance(&self) -> Vec<FeatureImportance> {
        feature_importance(&self.tree)
    }

    /// Get the underlying tree reference.
    #[must_use]
    pub fn tree(&self) -> &DecisionTree {
        &self.tree
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn task(
        items: usize,
        rep: bool,
        struc: bool,
        reason: bool,
        words: usize,
    ) -> TaskCharacteristics {
        TaskCharacteristics {
            item_count: items,
            is_repetitive: rep,
            has_structure: struc,
            needs_reasoning: reason,
            query_word_count: words,
        }
    }

    fn training_set() -> (Vec<TaskCharacteristics>, Vec<&'static str>) {
        let tasks = vec![
            // forge: complex, structured, reasoning
            task(10, false, true, true, 20),
            task(8, false, true, true, 15),
            // data-transformer: repetitive, structured
            task(100, true, true, false, 5),
            task(50, true, true, false, 8),
            // explore: simple queries, no structure
            task(1, false, false, false, 3),
            task(2, false, false, false, 4),
        ];
        let skills = vec![
            "forge",
            "forge",
            "data-transformer",
            "data-transformer",
            "explore",
            "explore",
        ];
        (tasks, skills)
    }

    #[test]
    fn extract_features_count() {
        let t = task(5, true, false, true, 10);
        let features = extract_features(&t);
        assert_eq!(features.len(), feature_index::COUNT);
    }

    #[test]
    fn train_and_route_forge() {
        let (tasks, skills) = training_set();
        let router = DtreeRouter::train(&tasks, &skills, TreeConfig::default())
            .ok()
            .expect("train ok");

        let complex = task(12, false, true, true, 25);
        let result = router.route(&complex);
        assert!(result.is_some());
        assert_eq!(result.expect("routed").routing.skill_name, "forge");
    }

    #[test]
    fn train_and_route_explore() {
        let (tasks, skills) = training_set();
        let router = DtreeRouter::train(&tasks, &skills, TreeConfig::default())
            .ok()
            .expect("train ok");

        let simple = task(1, false, false, false, 2);
        let result = router.route(&simple);
        assert!(result.is_some());
        assert_eq!(result.expect("routed").routing.skill_name, "explore");
    }

    #[test]
    fn importance_populated() {
        let (tasks, skills) = training_set();
        let router = DtreeRouter::train(&tasks, &skills, TreeConfig::default())
            .ok()
            .expect("train ok");

        let imp = router.importance();
        assert!(!imp.is_empty());
        assert!(imp.iter().any(|fi| fi.importance > 0.0));
    }
}
