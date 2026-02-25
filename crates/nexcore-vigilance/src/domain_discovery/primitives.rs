//! # Primitive Types
//!
//! Type-safe representations of domain primitives extracted via the
//! Domain Discovery Framework (Phase 3: EXTRACT_PRIMITIVES).
//!
//! ## Tier Classification
//!
//! | Tier | Coverage | Description |
//! |------|----------|-------------|
//! | T1 | 10+ domains | Universal ontological bedrock |
//! | T2 | 2-9 domains | Cross-domain transferable concepts |
//! | T3 | 1 domain | Domain-specific primitives |

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Primitive tier classification based on domain coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimitiveTier {
    /// T1: Universal primitives appearing in 10+ domains.
    /// Examples: entity, state, time, constraint, observation, parameter, manifold
    #[serde(rename = "T1_Universal")]
    T1Universal,

    /// T2: Cross-domain primitives appearing in 2-9 domains.
    /// Examples: level, hierarchy, emergence, safety_manifold, perturbation
    #[serde(rename = "T2_CrossDomain")]
    T2CrossDomain,

    /// T3: Domain-specific primitives appearing in 1 domain only.
    /// Examples: harm_type, safety_margin, vigilance_system, attenuation
    #[serde(rename = "T3_DomainSpecific")]
    T3DomainSpecific,
}

impl PrimitiveTier {
    /// Returns the minimum domain coverage for this tier.
    #[must_use]
    pub const fn min_coverage(&self) -> usize {
        match self {
            Self::T1Universal => 10,
            Self::T2CrossDomain => 2,
            Self::T3DomainSpecific => 1,
        }
    }

    /// Returns human-readable tier name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::T1Universal => "Universal",
            Self::T2CrossDomain => "Cross-Domain",
            Self::T3DomainSpecific => "Domain-Specific",
        }
    }
}

/// Unique identifier for a primitive.
///
/// Format: `PRIM-{TIER}-{NUMBER}` where:
/// - TIER: UNIV (T1), XDOM (T2), or domain code (T3)
/// - NUMBER: 4-digit zero-padded sequence
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrimitiveId(pub String);

impl PrimitiveId {
    /// Creates a new primitive ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Extracts the tier from the ID prefix.
    #[must_use]
    pub fn tier(&self) -> Option<PrimitiveTier> {
        if self.0.starts_with("PRIM-UNIV-") {
            Some(PrimitiveTier::T1Universal)
        } else if self.0.starts_with("PRIM-XDOM-") {
            Some(PrimitiveTier::T2CrossDomain)
        } else if self.0.starts_with("PRIM-") {
            Some(PrimitiveTier::T3DomainSpecific)
        } else {
            None
        }
    }
}

impl std::fmt::Display for PrimitiveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Domain coverage list for a primitive.
pub type DomainCoverage = Vec<String>;

/// A domain primitive - an irreducible concept extracted from a domain.
///
/// # Example
///
/// ```yaml
/// id: PRIM-UNIV-0001
/// name: entity
/// definition: "A distinct, identifiable thing that exists"
/// tier: T1_Universal
/// confidence: 1.0
/// depends_on: []
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Primitive {
    /// Unique identifier (e.g., "PRIM-UNIV-0001").
    pub id: PrimitiveId,

    /// Short name (e.g., "entity", "harm_type").
    pub name: String,

    /// Human-readable definition.
    pub definition: String,

    /// Tier classification (T1/T2/T3).
    pub tier: PrimitiveTier,

    /// Domains where this primitive appears.
    #[serde(default)]
    pub domain_coverage: DomainCoverage,

    /// Confidence score in [0.0, 1.0].
    pub confidence: f64,

    /// Names of primitives this one depends on.
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Example instantiations.
    #[serde(default)]
    pub examples: Vec<String>,

    /// Domain-specific instantiation description.
    #[serde(default)]
    pub tov_instantiation: Option<String>,
}

impl Primitive {
    /// Creates a new primitive with required fields.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        definition: impl Into<String>,
        tier: PrimitiveTier,
        confidence: f64,
    ) -> Self {
        Self {
            id: PrimitiveId::new(id),
            name: name.into(),
            definition: definition.into(),
            tier,
            domain_coverage: Vec::new(),
            confidence: confidence.clamp(0.0, 1.0),
            depends_on: Vec::new(),
            examples: Vec::new(),
            tov_instantiation: None,
        }
    }

    /// Adds domain coverage.
    #[must_use]
    pub fn with_coverage(mut self, domains: Vec<String>) -> Self {
        self.domain_coverage = domains;
        self
    }

    /// Adds dependencies.
    #[must_use]
    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.depends_on = deps;
        self
    }

    /// Checks if this is a root primitive (no dependencies).
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.depends_on.is_empty()
    }

    /// Returns the number of domains covered.
    #[must_use]
    pub fn coverage_count(&self) -> usize {
        self.domain_coverage.len()
    }
}

/// Counts of primitives per tier.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TierCounts {
    /// Count of T1 universal primitives.
    pub t1_universal: usize,
    /// Count of T2 cross-domain primitives.
    pub t2_cross_domain: usize,
    /// Count of T3 domain-specific primitives.
    pub t3_domain_specific: usize,
}

impl TierCounts {
    /// Total primitive count.
    #[must_use]
    pub fn total(&self) -> usize {
        self.t1_universal + self.t2_cross_domain + self.t3_domain_specific
    }
}

/// Result of primitive extraction (Phase 3 output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitivesResult {
    /// Domain name (e.g., "theory_of_vigilance").
    pub domain: String,

    /// Domain version (e.g., "8.0.0").
    pub version: String,

    /// Extraction mode (e.g., "Standard", "Deep").
    #[serde(default)]
    pub mode: Option<String>,

    /// Extraction algorithm description.
    #[serde(default)]
    pub extraction_algorithm: Option<String>,

    /// Primitives organized by tier.
    pub primitives: PrimitivesByTier,

    /// Tier counts summary.
    #[serde(default)]
    pub tier_counts: TierCounts,
}

/// Primitives organized by tier.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrimitivesByTier {
    /// T1 Universal primitives.
    #[serde(rename = "T1_Universal", default)]
    pub t1_universal: Vec<Primitive>,

    /// T2 Cross-Domain primitives.
    #[serde(rename = "T2_CrossDomain", default)]
    pub t2_cross_domain: Vec<Primitive>,

    /// T3 Domain-Specific primitives.
    #[serde(rename = "T3_DomainSpecific", default)]
    pub t3_domain_specific: Vec<Primitive>,
}

impl PrimitivesByTier {
    /// Returns all primitives as a flat iterator.
    pub fn all(&self) -> impl Iterator<Item = &Primitive> {
        self.t1_universal
            .iter()
            .chain(self.t2_cross_domain.iter())
            .chain(self.t3_domain_specific.iter())
    }

    /// Finds a primitive by name across all tiers.
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&Primitive> {
        self.all().find(|p| p.name == name)
    }

    /// Finds a primitive by ID across all tiers.
    #[must_use]
    pub fn find_by_id(&self, id: &str) -> Option<&Primitive> {
        self.all().find(|p| p.id.0 == id)
    }

    /// Returns tier counts.
    #[must_use]
    pub fn counts(&self) -> TierCounts {
        TierCounts {
            t1_universal: self.t1_universal.len(),
            t2_cross_domain: self.t2_cross_domain.len(),
            t3_domain_specific: self.t3_domain_specific.len(),
        }
    }
}

impl PrimitivesResult {
    /// Loads primitives from a YAML file.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or parsed.
    pub fn from_file(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let result: Self = serde_yml::from_str(&content)?;
        Ok(result)
    }

    /// Returns total primitive count.
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.primitives.counts().total()
    }

    /// Finds a primitive by name.
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&Primitive> {
        self.primitives.find_by_name(name)
    }

    /// Returns all root primitives (no dependencies).
    pub fn roots(&self) -> impl Iterator<Item = &Primitive> {
        self.primitives.all().filter(|p| p.is_root())
    }

    /// Builds a dependency map: primitive name → list of dependents.
    #[must_use]
    pub fn dependency_map(&self) -> IndexMap<String, Vec<String>> {
        let mut map: IndexMap<String, Vec<String>> = IndexMap::new();
        for p in self.primitives.all() {
            for dep in &p.depends_on {
                map.entry(dep.clone()).or_default().push(p.name.clone());
            }
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_tier_min_coverage() {
        assert_eq!(PrimitiveTier::T1Universal.min_coverage(), 10);
        assert_eq!(PrimitiveTier::T2CrossDomain.min_coverage(), 2);
        assert_eq!(PrimitiveTier::T3DomainSpecific.min_coverage(), 1);
    }

    #[test]
    fn test_primitive_id_tier_extraction() {
        let t1 = PrimitiveId::new("PRIM-UNIV-0001");
        assert_eq!(t1.tier(), Some(PrimitiveTier::T1Universal));

        let t2 = PrimitiveId::new("PRIM-XDOM-0001");
        assert_eq!(t2.tier(), Some(PrimitiveTier::T2CrossDomain));

        let t3 = PrimitiveId::new("PRIM-TOV-0001");
        assert_eq!(t3.tier(), Some(PrimitiveTier::T3DomainSpecific));
    }

    #[test]
    fn test_primitive_creation() {
        let p = Primitive::new(
            "PRIM-UNIV-0001",
            "entity",
            "A distinct thing",
            PrimitiveTier::T1Universal,
            1.0,
        )
        .with_dependencies(vec![]);

        assert!(p.is_root());
        assert_eq!(p.name, "entity");
    }

    #[test]
    fn test_tier_counts() {
        let counts = TierCounts {
            t1_universal: 7,
            t2_cross_domain: 11,
            t3_domain_specific: 12,
        };
        assert_eq!(counts.total(), 30);
    }
}
