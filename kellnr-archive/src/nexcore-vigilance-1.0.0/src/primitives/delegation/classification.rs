//! Classification Tree - T2-Primitive
//!
//! Ordered predicate evaluation mapping to actions.
//! Decomposes to: Sequence + Mapping + Recursion

/// Result of predicate evaluation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredicateResult {
    Match,
    NoMatch,
    Continue,
}

/// A predicate function that evaluates task characteristics
pub type Predicate<T> = Box<dyn Fn(&T) -> PredicateResult + Send + Sync>;

/// Classification tree for task routing
/// T1 composition: Sequence (ordered) + Mapping (predicate→action) + Recursion (tree)
#[derive(Default)]
pub struct ClassificationTree<T, A> {
    nodes: Vec<ClassificationNode<T, A>>,
    default_action: Option<A>,
}

struct ClassificationNode<T, A> {
    predicate: Predicate<T>,
    action: A,
    priority: u8,
}

impl<T, A: Clone> ClassificationTree<T, A> {
    /// Create new classification tree
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            default_action: None,
        }
    }

    /// Add a classification rule with priority
    pub fn add_rule<P>(&mut self, predicate: P, action: A, priority: u8)
    where
        P: Fn(&T) -> PredicateResult + Send + Sync + 'static,
    {
        self.nodes.push(ClassificationNode {
            predicate: Box::new(predicate),
            action,
            priority,
        });
        // Keep sorted by priority (highest first) - T1: Sequence
        self.nodes.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Set default action when no rules match
    pub fn set_default(&mut self, action: A) {
        self.default_action = Some(action);
    }

    /// Classify input and return action - T1: Mapping
    pub fn classify(&self, input: &T) -> Option<A> {
        // T1: Sequence - evaluate in priority order
        for node in &self.nodes {
            match (node.predicate)(input) {
                PredicateResult::Match => return Some(node.action.clone()),
                PredicateResult::NoMatch => continue,
                PredicateResult::Continue => continue,
            }
        }
        self.default_action.clone()
    }
}

/// Builder for fluent classification tree construction
#[derive(Default)]
pub struct ClassificationBuilder<T, A> {
    tree: ClassificationTree<T, A>,
}

impl<T, A: Clone> ClassificationBuilder<T, A> {
    pub fn new() -> Self {
        Self {
            tree: ClassificationTree::new(),
        }
    }

    pub fn when<P>(mut self, predicate: P, action: A, priority: u8) -> Self
    where
        P: Fn(&T) -> PredicateResult + Send + Sync + 'static,
    {
        self.tree.add_rule(predicate, action, priority);
        self
    }

    pub fn otherwise(mut self, action: A) -> Self {
        self.tree.set_default(action);
        self
    }

    pub fn build(self) -> ClassificationTree<T, A> {
        self.tree
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification() {
        let tree = ClassificationBuilder::<u32, &str>::new()
            .when(
                |n| {
                    if *n > 100 {
                        PredicateResult::Match
                    } else {
                        PredicateResult::NoMatch
                    }
                },
                "large",
                10,
            )
            .when(
                |n| {
                    if *n > 10 {
                        PredicateResult::Match
                    } else {
                        PredicateResult::NoMatch
                    }
                },
                "medium",
                5,
            )
            .otherwise("small")
            .build();

        assert_eq!(tree.classify(&150), Some("large"));
        assert_eq!(tree.classify(&50), Some("medium"));
        assert_eq!(tree.classify(&5), Some("small"));
    }
}
