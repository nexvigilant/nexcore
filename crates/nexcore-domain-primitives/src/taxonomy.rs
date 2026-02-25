//! Domain taxonomy: a collection of extracted primitives with dependency graph.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::transfer::{DomainTransfer, TransferScore};

/// Tier classification for extracted primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Tier {
    /// Universal — appears identically in all domains.
    T1,
    /// Cross-domain primitive — atomic, transfers to multiple domains.
    T2P,
    /// Cross-domain composite — composed from T1/T2-P, multiple domains.
    T2C,
    /// Domain-specific — meaningful in source domain only.
    T3,
}

impl Tier {
    /// Numeric ordering for sorting (lower = more universal).
    pub fn rank(self) -> u8 {
        match self {
            Self::T1 => 0,
            Self::T2P => 1,
            Self::T2C => 2,
            Self::T3 => 3,
        }
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::T1 => "T1-Universal",
            Self::T2P => "T2-P-CrossDomainPrimitive",
            Self::T2C => "T2-C-CrossDomainComposite",
            Self::T3 => "T3-DomainSpecific",
        }
    }

    /// All tier variants in universal-to-specific order.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::T1, Self::T2P, Self::T2C, Self::T3]
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// A single extracted primitive.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Primitive {
    /// Short kebab-case identifier (e.g., "sensor-fusion").
    pub name: String,
    /// One-sentence definition suitable for the primitive test.
    pub definition: String,
    /// Tier classification.
    pub tier: Tier,
    /// Names of primitives this depends on (must exist in same taxonomy).
    pub dependencies: Vec<String>,
    /// Example domains where this primitive appears.
    pub domain_examples: Vec<String>,
}

impl Primitive {
    /// Builder-style constructor.
    pub fn new(name: impl Into<String>, definition: impl Into<String>, tier: Tier) -> Self {
        Self {
            name: name.into(),
            definition: definition.into(),
            tier,
            dependencies: Vec::new(),
            domain_examples: Vec::new(),
        }
    }

    /// Add dependency names.
    pub fn with_deps(mut self, deps: &[&str]) -> Self {
        self.dependencies = deps.iter().map(|&s| s.to_string()).collect();
        self
    }

    /// Add domain examples.
    pub fn with_domains(mut self, domains: &[&str]) -> Self {
        self.domain_examples = domains.iter().map(|&s| s.to_string()).collect();
        self
    }
}

/// Complete extraction result for a domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DomainTaxonomy {
    /// Name of the source domain (e.g., "Golden Dome").
    pub name: String,
    /// Description of what was extracted.
    pub description: String,
    /// All extracted primitives.
    pub primitives: Vec<Primitive>,
    /// Pre-computed cross-domain transfers.
    pub transfers: Vec<DomainTransfer>,
}

impl DomainTaxonomy {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            primitives: Vec::new(),
            transfers: Vec::new(),
        }
    }

    /// Filter primitives by tier.
    pub fn by_tier(&self, tier: Tier) -> Vec<&Primitive> {
        self.primitives.iter().filter(|p| p.tier == tier).collect()
    }

    /// Count primitives per tier.
    pub fn tier_counts(&self) -> HashMap<Tier, usize> {
        let mut counts = HashMap::new();
        for p in &self.primitives {
            *counts.entry(p.tier).or_insert(0) += 1;
        }
        counts
    }

    /// Find a primitive by name.
    pub fn get(&self, name: &str) -> Option<&Primitive> {
        self.primitives.iter().find(|p| p.name == name)
    }

    /// Adjacency list: primitive name → names it depends on.
    pub fn dependency_graph(&self) -> HashMap<&str, Vec<&str>> {
        self.primitives
            .iter()
            .map(|p| {
                (
                    p.name.as_str(),
                    p.dependencies.iter().map(String::as_str).collect(),
                )
            })
            .collect()
    }

    /// Reverse adjacency: primitive name → names that depend on it.
    pub fn dependents_graph(&self) -> HashMap<&str, Vec<&str>> {
        let mut rev: HashMap<&str, Vec<&str>> = HashMap::new();
        for p in &self.primitives {
            // Ensure every node is in the map
            rev.entry(p.name.as_str()).or_default();
            for dep in &p.dependencies {
                rev.entry(dep.as_str()).or_default().push(p.name.as_str());
            }
        }
        rev
    }

    /// Recursively decompose a primitive into its T1 foundation.
    /// Returns a tree of (name, tier, children).
    pub fn decompose(&self, name: &str) -> Option<DecompositionNode> {
        let prim = self.get(name)?;
        let children = prim
            .dependencies
            .iter()
            .filter_map(|dep| self.decompose(dep))
            .collect();
        Some(DecompositionNode {
            name: prim.name.clone(),
            tier: prim.tier,
            children,
        })
    }

    /// Average transfer confidence for a given tier across all target domains.
    pub fn avg_transfer_confidence(&self, tier: Tier) -> f64 {
        let tier_names: Vec<&str> = self.by_tier(tier).iter().map(|p| p.name.as_str()).collect();
        if tier_names.is_empty() {
            return 0.0;
        }
        let relevant: Vec<f64> = self
            .transfers
            .iter()
            .filter(|t| tier_names.contains(&t.primitive_name.as_str()))
            .map(|t| t.confidence())
            .collect();
        if relevant.is_empty() {
            return 0.0;
        }
        relevant.iter().sum::<f64>() / relevant.len() as f64
    }

    /// Minimal spanning set: T1 + T2-P primitives sufficient to reconstruct all higher tiers.
    pub fn irreducible_atoms(&self) -> Vec<&Primitive> {
        self.primitives
            .iter()
            .filter(|p| matches!(p.tier, Tier::T1 | Tier::T2P))
            .collect()
    }
}

/// Recursive decomposition tree node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DecompositionNode {
    pub name: String,
    pub tier: Tier,
    pub children: Vec<DecompositionNode>,
}

impl DecompositionNode {
    /// Depth of the deepest leaf.
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            0
        } else {
            1 + self.children.iter().map(Self::depth).max().unwrap_or(0)
        }
    }

    /// Collect all leaf names (T1 atoms).
    pub fn leaves(&self) -> Vec<&str> {
        if self.children.is_empty() {
            vec![self.name.as_str()]
        } else {
            self.children.iter().flat_map(Self::leaves).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_taxonomy() -> DomainTaxonomy {
        let mut tax = DomainTaxonomy::new("test", "test taxonomy");
        tax.primitives.push(
            Primitive::new("detection", "Signal recognition", Tier::T1)
                .with_domains(&["cyber", "medicine"]),
        );
        tax.primitives.push(
            Primitive::new("tracking", "Continuous state estimation", Tier::T1)
                .with_domains(&["logistics", "finance"]),
        );
        tax.primitives.push(
            Primitive::new(
                "sensor-fusion",
                "Integration of heterogeneous observations",
                Tier::T2P,
            )
            .with_deps(&["detection", "tracking"])
            .with_domains(&["autonomous-vehicles", "diagnosis"]),
        );
        tax.primitives.push(
            Primitive::new(
                "fire-control-loop",
                "Detect-Track-Decide-Engage cycle",
                Tier::T2C,
            )
            .with_deps(&["detection", "tracking", "sensor-fusion"])
            .with_domains(&["OODA", "PID-controllers"]),
        );
        tax
    }

    #[test]
    fn by_tier_filters() {
        let tax = sample_taxonomy();
        assert_eq!(tax.by_tier(Tier::T1).len(), 2);
        assert_eq!(tax.by_tier(Tier::T2P).len(), 1);
        assert_eq!(tax.by_tier(Tier::T2C).len(), 1);
        assert_eq!(tax.by_tier(Tier::T3).len(), 0);
    }

    #[test]
    fn tier_counts() {
        let tax = sample_taxonomy();
        let counts = tax.tier_counts();
        assert_eq!(*counts.get(&Tier::T1).unwrap_or(&0), 2);
        assert_eq!(*counts.get(&Tier::T2P).unwrap_or(&0), 1);
    }

    #[test]
    fn get_by_name() {
        let tax = sample_taxonomy();
        assert!(tax.get("detection").is_some());
        assert!(tax.get("nonexistent").is_none());
    }

    #[test]
    fn dependency_graph_structure() {
        let tax = sample_taxonomy();
        let graph = tax.dependency_graph();
        assert!(graph.get("detection").is_some());
        let empty: Vec<&str> = vec![];
        let sf_deps = graph.get("sensor-fusion").unwrap_or(&empty);
        assert!(sf_deps.contains(&"detection"));
        assert!(sf_deps.contains(&"tracking"));
    }

    #[test]
    fn dependents_graph_reverse() {
        let tax = sample_taxonomy();
        let rev = tax.dependents_graph();
        let empty2: Vec<&str> = vec![];
        let detection_dependents = rev.get("detection").unwrap_or(&empty2);
        assert!(detection_dependents.contains(&"sensor-fusion"));
        assert!(detection_dependents.contains(&"fire-control-loop"));
    }

    #[test]
    fn decompose_leaf() {
        let tax = sample_taxonomy();
        let node = tax.decompose("detection");
        assert!(node.is_some());
        let node = node.unwrap_or_else(|| DecompositionNode {
            name: String::new(),
            tier: Tier::T1,
            children: vec![],
        });
        assert_eq!(node.depth(), 0);
        assert_eq!(node.leaves(), vec!["detection"]);
    }

    #[test]
    fn decompose_composite() {
        let tax = sample_taxonomy();
        let node = tax.decompose("fire-control-loop");
        assert!(node.is_some());
        let node = node.unwrap_or_else(|| DecompositionNode {
            name: String::new(),
            tier: Tier::T1,
            children: vec![],
        });
        assert_eq!(node.tier, Tier::T2C);
        assert_eq!(node.depth(), 2); // T2C → T2P → T1
    }

    #[test]
    fn irreducible_atoms_count() {
        let tax = sample_taxonomy();
        let atoms = tax.irreducible_atoms();
        assert_eq!(atoms.len(), 3); // 2 T1 + 1 T2P
    }

    #[test]
    fn tier_display() {
        assert_eq!(format!("{}", Tier::T1), "T1-Universal");
        assert_eq!(format!("{}", Tier::T2P), "T2-P-CrossDomainPrimitive");
    }

    #[test]
    fn tier_rank_ordering() {
        assert!(Tier::T1.rank() < Tier::T2P.rank());
        assert!(Tier::T2P.rank() < Tier::T2C.rank());
        assert!(Tier::T2C.rank() < Tier::T3.rank());
    }
}
