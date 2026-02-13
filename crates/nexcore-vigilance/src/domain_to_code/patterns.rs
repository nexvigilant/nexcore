//! # Pattern AST
//!
//! Abstract syntax tree for domain patterns.
//! Uses T1 recursion primitive (enum with Box) for tree structures.

use super::languages::DomainLanguage;
use serde::{Deserialize, Serialize};

/// A domain pattern - the core AST node type.
///
/// Uses T1 recursion primitive: `enum` with `Box<Self>` for nesting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainPattern {
    /// A single concept (leaf node).
    Concept {
        /// Concept name.
        name: String,
        /// Associated language.
        language: DomainLanguage,
    },

    /// A composite pattern (internal node).
    Composite {
        /// Pattern name.
        name: String,
        /// Child patterns (T1 recursion via Box).
        children: Vec<Box<DomainPattern>>,
        /// Combination semantics.
        combination: CombinationSemantics,
    },

    /// A transformation pattern.
    Transform {
        /// Source pattern.
        source: Box<DomainPattern>,
        /// Target pattern.
        target: Box<DomainPattern>,
        /// Transformation type.
        transform_type: TransformType,
    },

    /// A conditional pattern.
    Conditional {
        /// Condition pattern.
        condition: Box<DomainPattern>,
        /// Pattern if true.
        then_branch: Box<DomainPattern>,
        /// Pattern if false (T1 void: Option).
        else_branch: Option<Box<DomainPattern>>,
    },

    /// A sequence pattern (T1 sequence).
    Sequence {
        /// Ordered patterns.
        elements: Vec<Box<DomainPattern>>,
        /// Whether order matters.
        ordered: bool,
    },
}

/// How composite patterns combine their children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombinationSemantics {
    /// All children must be satisfied (product type).
    All,
    /// Any child can satisfy (sum type).
    Any,
    /// Children form a sequence (ordered product).
    Sequence,
    /// Children are alternatives with priority.
    Priority,
}

/// Type of transformation between patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformType {
    /// Direct mapping (From/Into).
    Mapping,
    /// Lossy conversion (TryFrom).
    Lossy,
    /// Enriching conversion (adds information).
    Enriching,
    /// Reducing conversion (loses information).
    Reducing,
}

impl DomainPattern {
    /// Creates a concept pattern.
    #[must_use]
    pub fn concept(name: impl Into<String>, language: DomainLanguage) -> Self {
        Self::Concept {
            name: name.into(),
            language,
        }
    }

    /// Creates a composite pattern.
    #[must_use]
    pub fn composite(
        name: impl Into<String>,
        children: Vec<DomainPattern>,
        combination: CombinationSemantics,
    ) -> Self {
        Self::Composite {
            name: name.into(),
            children: children.into_iter().map(Box::new).collect(),
            combination,
        }
    }

    /// Creates a transform pattern.
    #[must_use]
    pub fn transform(
        source: DomainPattern,
        target: DomainPattern,
        transform_type: TransformType,
    ) -> Self {
        Self::Transform {
            source: Box::new(source),
            target: Box::new(target),
            transform_type,
        }
    }

    /// Creates a sequence pattern.
    #[must_use]
    pub fn sequence(elements: Vec<DomainPattern>, ordered: bool) -> Self {
        Self::Sequence {
            elements: elements.into_iter().map(Box::new).collect(),
            ordered,
        }
    }

    /// Returns the depth of the pattern tree.
    #[must_use]
    pub fn depth(&self) -> usize {
        match self {
            Self::Concept { .. } => 1,
            Self::Composite { children, .. } => {
                1 + children.iter().map(|c| c.depth()).max().unwrap_or(0)
            }
            Self::Transform { source, target, .. } => 1 + source.depth().max(target.depth()),
            Self::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                let else_depth = else_branch.as_ref().map_or(0, |e| e.depth());
                1 + condition.depth().max(then_branch.depth()).max(else_depth)
            }
            Self::Sequence { elements, .. } => {
                1 + elements.iter().map(|e| e.depth()).max().unwrap_or(0)
            }
        }
    }

    /// Collects all concept names in the pattern tree.
    #[must_use]
    pub fn concept_names(&self) -> Vec<&str> {
        match self {
            Self::Concept { name, .. } => vec![name.as_str()],
            Self::Composite { children, .. } => {
                children.iter().flat_map(|c| c.concept_names()).collect()
            }
            Self::Transform { source, target, .. } => {
                let mut names = source.concept_names();
                names.extend(target.concept_names());
                names
            }
            Self::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut names = condition.concept_names();
                names.extend(then_branch.concept_names());
                if let Some(eb) = else_branch {
                    names.extend(eb.concept_names());
                }
                names
            }
            Self::Sequence { elements, .. } => {
                elements.iter().flat_map(|e| e.concept_names()).collect()
            }
        }
    }

    /// Returns the pattern name (if applicable).
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Concept { name, .. } | Self::Composite { name, .. } => Some(name.as_str()),
            _ => None,
        }
    }
}

/// Semantic annotations for patterns.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternSemantics {
    /// Is this pattern pure (no side effects)?
    pub is_pure: bool,
    /// Is this pattern total (defined for all inputs)?
    pub is_total: bool,
    /// Estimated computational complexity.
    pub complexity: Option<String>,
    /// Invariants this pattern maintains.
    pub invariants: Vec<String>,
}

/// Pattern AST wrapper with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAst {
    /// The root pattern.
    pub root: DomainPattern,
    /// Semantic annotations.
    pub semantics: PatternSemantics,
    /// Source domain.
    pub source_domain: String,
    /// Pattern version.
    pub version: String,
}

impl PatternAst {
    /// Creates a new pattern AST.
    #[must_use]
    pub fn new(root: DomainPattern, domain: impl Into<String>) -> Self {
        Self {
            root,
            semantics: PatternSemantics::default(),
            source_domain: domain.into(),
            version: "1.0.0".to_string(),
        }
    }

    /// Adds semantics.
    #[must_use]
    pub fn with_semantics(mut self, semantics: PatternSemantics) -> Self {
        self.semantics = semantics;
        self
    }

    /// Returns the pattern depth.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.root.depth()
    }

    /// Returns all concept names.
    #[must_use]
    pub fn concept_names(&self) -> Vec<&str> {
        self.root.concept_names()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_pattern() {
        let p = DomainPattern::concept("risk_score", DomainLanguage::Risk);
        assert_eq!(p.depth(), 1);
        assert_eq!(p.concept_names(), vec!["risk_score"]);
    }

    #[test]
    fn test_composite_pattern() {
        let p = DomainPattern::composite(
            "combined",
            vec![
                DomainPattern::concept("a", DomainLanguage::Risk),
                DomainPattern::concept("b", DomainLanguage::Network),
            ],
            CombinationSemantics::All,
        );
        assert_eq!(p.depth(), 2);
        assert_eq!(p.concept_names().len(), 2);
    }

    #[test]
    fn test_transform_pattern() {
        let p = DomainPattern::transform(
            DomainPattern::concept("input", DomainLanguage::Information),
            DomainPattern::concept("output", DomainLanguage::Risk),
            TransformType::Mapping,
        );
        assert_eq!(p.depth(), 2);
        assert_eq!(p.concept_names().len(), 2);
    }

    #[test]
    fn test_sequence_pattern() {
        let p = DomainPattern::sequence(
            vec![
                DomainPattern::concept("step1", DomainLanguage::Resource),
                DomainPattern::concept("step2", DomainLanguage::Resource),
                DomainPattern::concept("step3", DomainLanguage::Resource),
            ],
            true,
        );
        assert_eq!(p.depth(), 2);
        assert_eq!(p.concept_names().len(), 3);
    }

    #[test]
    fn test_pattern_ast() {
        let ast = PatternAst::new(
            DomainPattern::concept("test", DomainLanguage::Emergence),
            "test_domain",
        );
        assert_eq!(ast.source_domain, "test_domain");
        assert_eq!(ast.depth(), 1);
    }
}
