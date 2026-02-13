//! # MESH (Medical Subject Headings) Module
//!
//! NLM MESH terminology integration for primitive validation, hierarchy mapping,
//! and cross-reference bridging.
//!
//! ## Overview
//!
//! MESH is the National Library of Medicine's controlled vocabulary thesaurus.
//! It provides:
//! - Hierarchical term classification via tree numbers
//! - Standardized descriptors with scope notes
//! - Qualifiers for refining descriptor meaning
//! - Entry terms (synonyms) for improved search recall
//!
//! ## Tier Classification
//!
//! MESH tree depth maps to primitive tiers:
//! - Depth 0 (e.g., "C"): T1 Universal
//! - Depth 1-2 (e.g., "C01.221"): T2 Primitive
//! - Depth 3-4 (e.g., "C01.221.812.640"): T2 Composite
//! - Depth 5+ (e.g., "C01.221.812.640.500.100"): T3 Domain-Specific

use serde::{Deserialize, Serialize};

/// MESH Descriptor - primary indexing unit
///
/// A descriptor is the main entry in MESH, representing a concept
/// with a unique identifier, preferred name, and hierarchical classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshDescriptor {
    /// Unique descriptor identifier (e.g., "D015242" for Ofloxacin)
    pub descriptor_ui: String,

    /// Preferred name (canonical term)
    pub name: String,

    /// Tree numbers indicating hierarchical classification
    /// A descriptor can appear in multiple places in the hierarchy
    pub tree_numbers: Vec<String>,

    /// Scope note (definition/description)
    pub scope_note: Option<String>,

    /// Entry terms (synonyms, alternate names)
    pub entry_terms: Vec<String>,

    /// Concepts within this descriptor
    pub concepts: Vec<MeshConcept>,

    /// Year descriptor was introduced or last modified
    pub year: u16,

    /// Whether this descriptor allows qualifiers
    pub allows_qualifiers: bool,
}

impl MeshDescriptor {
    /// Get the primitive tier based on tree depth
    ///
    /// Maps MESH hierarchy depth to primitive classification:
    /// - Top-level categories (A, B, C...): T1 Universal
    /// - 1-2 levels deep: T2 Primitive (cross-domain)
    /// - 3-4 levels deep: T2 Composite
    /// - 5+ levels: T3 Domain-Specific
    #[must_use]
    pub fn primitive_tier(&self) -> PrimitiveTier {
        self.tree_numbers
            .first()
            .map(|tn| tree_to_primitive_tier(tn))
            .unwrap_or(PrimitiveTier::T3DomainSpecific)
    }

    /// Get the primary tree path (first tree number)
    #[must_use]
    pub fn primary_tree(&self) -> Option<MeshTreePath> {
        self.tree_numbers.first().map(|tn| MeshTreePath::parse(tn))
    }

    /// Check if descriptor is in a specific tree branch
    #[must_use]
    pub fn is_in_branch(&self, branch: &str) -> bool {
        self.tree_numbers.iter().any(|tn| tn.starts_with(branch))
    }

    /// Get depth in hierarchy (shortest path)
    #[must_use]
    pub fn min_depth(&self) -> usize {
        self.tree_numbers
            .iter()
            .map(|tn| tn.matches('.').count())
            .min()
            .unwrap_or(0)
    }
}

/// MESH Concept - semantic unit within a descriptor
///
/// Each descriptor contains one or more concepts. The preferred
/// concept has the same name as the descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConcept {
    /// Unique concept identifier
    pub concept_ui: String,

    /// Concept name
    pub name: String,

    /// Whether this is the preferred concept for the descriptor
    pub preferred: bool,

    /// Semantic type from UMLS Semantic Network
    pub semantic_types: Vec<String>,

    /// Terms (lexical variants) for this concept
    pub terms: Vec<MeshTerm>,

    /// Registry numbers (CAS, EC, etc.) if applicable
    pub registry_numbers: Vec<String>,
}

/// MESH Term - lexical variant within a concept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshTerm {
    /// Unique term identifier
    pub term_ui: String,

    /// Term string
    pub string: String,

    /// Whether this is the preferred term for the concept
    pub preferred: bool,
}

/// MESH Qualifier - refines descriptor meaning
///
/// Qualifiers (subheadings) are used with descriptors to specify
/// a particular aspect of the subject.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshQualifier {
    /// Unique qualifier identifier (e.g., "Q000009" for adverse effects)
    pub qualifier_ui: String,

    /// Qualifier name
    pub name: String,

    /// Abbreviation (e.g., "AE" for adverse effects)
    pub abbreviation: String,

    /// Scope note
    pub scope_note: Option<String>,

    /// Tree number for qualifier hierarchy
    pub tree_number: String,
}

/// MESH Supplementary Concept Record
///
/// SCRs are for substances (drugs, chemicals) not in the main vocabulary.
/// They map to descriptors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshSupplementary {
    /// Unique SCR identifier
    pub scr_ui: String,

    /// Supplementary concept name
    pub name: String,

    /// Heading mapped to (descriptor)
    pub heading_mapped_to: Vec<String>,

    /// Registry numbers (CAS, etc.)
    pub registry_numbers: Vec<String>,

    /// Indexing information
    pub indexing_information: Option<String>,
}

/// Parsed MESH tree path with navigation capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshTreePath {
    /// Full tree number (e.g., "C01.221.812")
    pub full_path: String,

    /// Top-level category letter
    pub category: char,

    /// Path segments after category
    pub segments: Vec<String>,
}

impl MeshTreePath {
    /// Parse a tree number string into structured path
    #[must_use]
    pub fn parse(tree_number: &str) -> Self {
        let category = tree_number.chars().next().unwrap_or('Z');
        let segments: Vec<String> = if tree_number.len() > 1 {
            tree_number[1..]
                .trim_start_matches('.')
                .split('.')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect()
        } else {
            Vec::new()
        };

        Self {
            full_path: tree_number.to_string(),
            category,
            segments,
        }
    }

    /// Get depth in hierarchy (0 = top category)
    #[must_use]
    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    /// Get parent tree path (None if at root)
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        if self.segments.is_empty() {
            None
        } else {
            let mut parent_segments = self.segments.clone();
            parent_segments.pop();
            let parent_path = if parent_segments.is_empty() {
                self.category.to_string()
            } else {
                format!("{}.{}", self.category, parent_segments.join("."))
            };
            Some(Self::parse(&parent_path))
        }
    }

    /// Check if this path is ancestor of another
    #[must_use]
    pub fn is_ancestor_of(&self, other: &Self) -> bool {
        other.full_path.starts_with(&self.full_path) && other.full_path != self.full_path
    }

    /// Get MESH category name
    #[must_use]
    pub fn category_name(&self) -> &'static str {
        match self.category {
            'A' => "Anatomy",
            'B' => "Organisms",
            'C' => "Diseases",
            'D' => "Chemicals and Drugs",
            'E' => "Analytical, Diagnostic and Therapeutic Techniques, and Equipment",
            'F' => "Psychiatry and Psychology",
            'G' => "Phenomena and Processes",
            'H' => "Disciplines and Occupations",
            'I' => "Anthropology, Education, Sociology, and Social Phenomena",
            'J' => "Technology, Industry, and Agriculture",
            'K' => "Humanities",
            'L' => "Information Science",
            'M' => "Named Groups",
            'N' => "Health Care",
            'V' => "Publication Characteristics",
            'Z' => "Geographicals",
            _ => "Unknown",
        }
    }
}

/// Primitive tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveTier {
    /// T1: Universal primitives (top-level concepts)
    T1Universal,
    /// T2-P: Cross-domain primitives
    T2Primitive,
    /// T2-C: Cross-domain composites
    T2Composite,
    /// T3: Domain-specific concepts
    T3DomainSpecific,
}

impl PrimitiveTier {
    /// Get confidence multiplier for this tier
    #[must_use]
    pub fn confidence(&self) -> f64 {
        match self {
            PrimitiveTier::T1Universal => 1.0,
            PrimitiveTier::T2Primitive => 0.95,
            PrimitiveTier::T2Composite => 0.85,
            PrimitiveTier::T3DomainSpecific => 0.75,
        }
    }

    /// Human-readable tier name
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            PrimitiveTier::T1Universal => "T1: Universal",
            PrimitiveTier::T2Primitive => "T2-P: Cross-Domain Primitive",
            PrimitiveTier::T2Composite => "T2-C: Cross-Domain Composite",
            PrimitiveTier::T3DomainSpecific => "T3: Domain-Specific",
        }
    }
}

/// Map MESH tree depth to primitive tier
#[must_use]
pub fn tree_to_primitive_tier(tree_number: &str) -> PrimitiveTier {
    // MESH depth:
    // Level 0: "C" (len 1, 0 dots)
    // Level 1: "C01" (len > 1, 0 dots)
    // Level 2: "C01.221" (1 dot)
    // Level 3: "C01.221.812" (2 dots)
    let depth = if tree_number.len() <= 1 {
        0
    } else {
        tree_number.matches('.').count() + 1
    };

    match depth {
        0 => PrimitiveTier::T1Universal,
        1..=2 => PrimitiveTier::T2Primitive,
        3..=4 => PrimitiveTier::T2Composite,
        _ => PrimitiveTier::T3DomainSpecific,
    }
}

/// Brief descriptor format for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshDescriptorBrief {
    /// Unique identifier
    pub descriptor_ui: String,
    /// Preferred name
    pub name: String,
    /// Primary tree number
    pub tree_number: Option<String>,
    /// Computed primitive tier
    pub tier: PrimitiveTier,
}

impl From<&MeshDescriptor> for MeshDescriptorBrief {
    fn from(desc: &MeshDescriptor) -> Self {
        Self {
            descriptor_ui: desc.descriptor_ui.clone(),
            name: desc.name.clone(),
            tree_number: desc.tree_numbers.first().cloned(),
            tier: desc.primitive_tier(),
        }
    }
}

/// Direction for tree navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TreeDirection {
    /// Navigate to parent nodes
    Ancestors,
    /// Navigate to child nodes
    Descendants,
    /// Navigate to sibling nodes (same parent)
    Siblings,
}

/// Result of tree navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNavigationResult {
    /// Source descriptor
    pub source: MeshDescriptorBrief,
    /// Navigation direction
    pub direction: TreeDirection,
    /// Depth limit used
    pub depth: usize,
    /// Related descriptors found
    pub results: Vec<MeshDescriptorBrief>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_path_parsing() {
        let path = MeshTreePath::parse("C01.221.812");
        assert_eq!(path.category, 'C');
        assert_eq!(path.segments, vec!["01", "221", "812"]);
        assert_eq!(path.depth(), 3);
    }

    #[test]
    fn test_tree_path_parent() {
        let path = MeshTreePath::parse("C01.221.812");
        let parent = path.parent().expect("Should have parent");
        assert_eq!(parent.full_path, "C.01.221");
    }

    #[test]
    fn test_primitive_tier_mapping() {
        assert_eq!(tree_to_primitive_tier("C"), PrimitiveTier::T1Universal);
        assert_eq!(tree_to_primitive_tier("C01"), PrimitiveTier::T2Primitive);
        assert_eq!(
            tree_to_primitive_tier("C01.221"),
            PrimitiveTier::T2Primitive
        );
        assert_eq!(
            tree_to_primitive_tier("C01.221.812"),
            PrimitiveTier::T2Composite
        );
        assert_eq!(
            tree_to_primitive_tier("C01.221.812.640"),
            PrimitiveTier::T2Composite
        );
        assert_eq!(
            tree_to_primitive_tier("C01.221.812.640.500"),
            PrimitiveTier::T3DomainSpecific
        );
    }

    #[test]
    fn test_category_names() {
        assert_eq!(MeshTreePath::parse("C").category_name(), "Diseases");
        assert_eq!(
            MeshTreePath::parse("D01").category_name(),
            "Chemicals and Drugs"
        );
    }

    #[test]
    fn test_is_ancestor_of() {
        let parent = MeshTreePath::parse("C01.221");
        let child = MeshTreePath::parse("C01.221.812");
        assert!(parent.is_ancestor_of(&child));
        assert!(!child.is_ancestor_of(&parent));
        assert!(!parent.is_ancestor_of(&parent));
    }
}
