//! # Dossier Generation
//!
//! Generate analysis reports for primitives and compositions.
//!
//! ## Tier: T2-C (Persistence + Sequence + State)

use crate::bedrock::BedrockAtom;
use crate::graph::{DependencyGraph, MathFoundation};
use crate::primitiva::{LexPrimitiva, PrimitiveComposition};
use crate::tier::Tier;
use serde::{Deserialize, Serialize};

/// A complete dossier for a primitive or composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Dossier {
    /// Title of the dossier.
    pub title: String,
    /// Subject being analyzed.
    pub subject: DossierSubject,
    /// Summary section.
    pub summary: Summary,
    /// Structural analysis.
    pub structure: StructureAnalysis,
    /// Grounding analysis.
    pub grounding: GroundingAnalysis,
    /// Generated at timestamp (ISO 8601).
    pub generated_at: String,
}

/// Subject of a dossier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DossierSubject {
    /// Single primitive.
    Primitive(LexPrimitiva),
    /// Composition of primitives.
    Composition(PrimitiveComposition),
    /// Bedrock atom.
    Atom(BedrockAtom),
}

/// Summary section of a dossier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Summary {
    /// One-line description.
    pub headline: String,
    /// Tier classification.
    pub tier: Tier,
    /// Transfer confidence multiplier.
    pub transfer_multiplier: f64,
    /// Primary constant.
    pub primary_constant: String,
    /// Key characteristics.
    pub characteristics: Vec<String>,
}

/// Structural analysis section.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StructureAnalysis {
    /// Symbol(s) used.
    pub symbols: Vec<String>,
    /// Dependencies (derives-from).
    pub dependencies: Vec<String>,
    /// Dependents (derived-by).
    pub dependents: Vec<String>,
    /// Depth in DAG.
    pub depth: usize,
    /// Is this a root?
    pub is_root: bool,
}

/// Grounding analysis section.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GroundingAnalysis {
    /// Bedrock atoms.
    pub atoms: Vec<AtomEntry>,
    /// Mathematical foundations.
    pub foundations: Vec<String>,
    /// Terminal constants reached.
    pub constants: Vec<String>,
    /// Traces to constants.
    pub traces: Vec<String>,
}

/// Entry for a bedrock atom.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AtomEntry {
    /// Atom name.
    pub name: String,
    /// Primary constant.
    pub constant: String,
}

/// Generator for dossiers.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct DossierGenerator {
    /// Whether to include full traces.
    pub include_traces: bool,
}

impl DossierGenerator {
    /// Create a new generator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            include_traces: true,
        }
    }

    /// Create generator without traces (compact mode).
    #[must_use]
    pub fn compact() -> Self {
        Self {
            include_traces: false,
        }
    }

    /// Generate dossier for a primitive.
    #[must_use]
    pub fn for_primitive(&self, primitive: LexPrimitiva) -> Dossier {
        Dossier {
            title: format!("{} ({}) Dossier", primitive.name(), primitive.symbol()),
            subject: DossierSubject::Primitive(primitive),
            summary: self.primitive_summary(primitive),
            structure: self.primitive_structure(primitive),
            grounding: self.primitive_grounding(primitive),
            generated_at: Self::timestamp(),
        }
    }

    /// Generate dossier for a composition.
    #[must_use]
    pub fn for_composition(&self, composition: &PrimitiveComposition) -> Dossier {
        let tier = Tier::classify(composition);
        Dossier {
            title: format!("{} Composition Dossier", tier.code()),
            subject: DossierSubject::Composition(composition.clone()),
            summary: self.composition_summary(composition),
            structure: self.composition_structure(composition),
            grounding: self.composition_grounding(composition),
            generated_at: Self::timestamp(),
        }
    }

    /// Generate dossier for an atom.
    #[must_use]
    pub fn for_atom(&self, atom: BedrockAtom) -> Dossier {
        let parent = atom.parent_primitive();
        Dossier {
            title: format!("{} Atom Dossier", atom.name()),
            subject: DossierSubject::Atom(atom),
            summary: self.atom_summary(atom),
            structure: self.atom_structure(atom, parent),
            grounding: self.atom_grounding(atom),
            generated_at: Self::timestamp(),
        }
    }

    fn primitive_summary(&self, p: LexPrimitiva) -> Summary {
        Summary {
            headline: p.description().to_string(),
            tier: Tier::T1Universal,
            transfer_multiplier: Tier::T1Universal.transfer_multiplier(),
            primary_constant: p.primary_constant().symbol.to_string(),
            characteristics: vec![
                format!("Symbol: {}", p.symbol()),
                format!("Rust: {}", p.rust_manifestation()),
                format!("Root: {}", if p.is_root() { "Yes" } else { "No" }),
            ],
        }
    }

    fn primitive_structure(&self, p: LexPrimitiva) -> StructureAnalysis {
        let deps: Vec<String> = p
            .derives_from()
            .iter()
            .map(|d| d.symbol().to_string())
            .collect();
        let dependents = self.find_dependents(p);
        let depth = Self::compute_depth(p);
        StructureAnalysis {
            symbols: vec![p.symbol().to_string()],
            dependencies: deps,
            dependents,
            depth,
            is_root: p.is_root(),
        }
    }

    fn primitive_grounding(&self, p: LexPrimitiva) -> GroundingAnalysis {
        let atoms: Vec<AtomEntry> = BedrockAtom::for_primitive(p)
            .iter()
            .map(|a| AtomEntry {
                name: a.name().to_string(),
                constant: a.primary_constant().symbol.to_string(),
            })
            .collect();
        let foundations: Vec<String> = crate::graph::foundations_for_primitive(p)
            .iter()
            .map(|f| f.name().to_string())
            .collect();
        let constants: Vec<String> = DependencyGraph::constants_for_primitive(p)
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let traces = if self.include_traces {
            DependencyGraph::trace(p)
                .iter()
                .map(|t| format!("{}", t))
                .collect()
        } else {
            Vec::new()
        };
        GroundingAnalysis {
            atoms,
            foundations,
            constants,
            traces,
        }
    }

    fn composition_summary(&self, comp: &PrimitiveComposition) -> Summary {
        let tier = Tier::classify(comp);
        let unique_count = comp.unique().len();
        let dominant_str = comp
            .dominant
            .map(|d| d.symbol().to_string())
            .unwrap_or_else(|| "None".to_string());
        Summary {
            headline: format!("{} primitives, {} tier", unique_count, tier.code()),
            tier,
            transfer_multiplier: tier.transfer_multiplier(),
            primary_constant: comp
                .dominant
                .map(|d| d.primary_constant().symbol)
                .unwrap_or("1")
                .to_string(),
            characteristics: vec![
                format!("Unique primitives: {}", unique_count),
                format!("Dominant: {}", dominant_str),
                format!("Confidence: {:.2}", comp.confidence),
            ],
        }
    }

    fn composition_structure(&self, comp: &PrimitiveComposition) -> StructureAnalysis {
        let symbols: Vec<String> = comp
            .primitives
            .iter()
            .map(|p| p.symbol().to_string())
            .collect();
        let mut dep_set = std::collections::BTreeSet::new();
        for p in &comp.primitives {
            for d in p.derives_from() {
                dep_set.insert(d.symbol().to_string());
            }
        }
        let all_deps: Vec<String> = dep_set.into_iter().collect();
        let max_depth = comp
            .primitives
            .iter()
            .map(|p| Self::compute_depth(*p))
            .max()
            .unwrap_or(0);
        StructureAnalysis {
            symbols,
            dependencies: all_deps,
            dependents: Vec::new(), // N/A for compositions
            depth: max_depth,
            is_root: comp.primitives.iter().all(|p| p.is_root()),
        }
    }

    fn composition_grounding(&self, comp: &PrimitiveComposition) -> GroundingAnalysis {
        let mut all_constants = std::collections::BTreeSet::new();
        let mut all_foundations = std::collections::BTreeSet::new();
        for p in &comp.primitives {
            for c in DependencyGraph::constants_for_primitive(*p) {
                all_constants.insert(c.to_string());
            }
            for f in crate::graph::foundations_for_primitive(*p) {
                all_foundations.insert(f.name().to_string());
            }
        }
        GroundingAnalysis {
            atoms: Vec::new(), // Too many for compositions
            foundations: all_foundations.into_iter().collect(),
            constants: all_constants.into_iter().collect(),
            traces: Vec::new(),
        }
    }

    fn atom_summary(&self, atom: BedrockAtom) -> Summary {
        let parent = atom.parent_primitive();
        Summary {
            headline: format!("Bedrock atom of {} ({})", parent.name(), parent.symbol()),
            tier: Tier::T1Universal,
            transfer_multiplier: 1.0,
            primary_constant: atom.primary_constant().symbol.to_string(),
            characteristics: vec![format!("Parent: {} ({})", parent.name(), parent.symbol())],
        }
    }

    fn atom_structure(&self, atom: BedrockAtom, parent: LexPrimitiva) -> StructureAnalysis {
        StructureAnalysis {
            symbols: vec![atom.name().to_string()],
            dependencies: vec![parent.symbol().to_string()],
            dependents: Vec::new(),
            depth: 1,
            is_root: false,
        }
    }

    fn atom_grounding(&self, atom: BedrockAtom) -> GroundingAnalysis {
        GroundingAnalysis {
            atoms: vec![AtomEntry {
                name: atom.name().to_string(),
                constant: atom.primary_constant().symbol.to_string(),
            }],
            foundations: Vec::new(),
            constants: vec![atom.primary_constant().symbol.to_string()],
            traces: Vec::new(),
        }
    }

    fn find_dependents(&self, p: LexPrimitiva) -> Vec<String> {
        LexPrimitiva::all()
            .iter()
            .filter(|other| other.derives_from().contains(&p))
            .map(|d| d.symbol().to_string())
            .collect()
    }

    fn compute_depth(p: LexPrimitiva) -> usize {
        primitive_depth(p)
    }

    fn timestamp() -> String {
        // Simple timestamp without chrono dependency
        "2026-02-04T00:00:00Z".to_string()
    }
}

/// Compute derivation depth for a primitive (free function to avoid self-only-in-recursion lint).
fn primitive_depth(p: LexPrimitiva) -> usize {
    if p.is_root() {
        return 0;
    }
    p.derives_from()
        .iter()
        .map(|d| primitive_depth(*d))
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

impl Dossier {
    /// Render as markdown.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", self.title));
        md.push_str(&format!("**Generated:** {}\n\n", self.generated_at));
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- **{}**\n", self.summary.headline));
        md.push_str(&format!(
            "- Tier: {} (×{:.1})\n",
            self.summary.tier.code(),
            self.summary.transfer_multiplier
        ));
        md.push_str(&format!(
            "- Primary Constant: `{}`\n",
            self.summary.primary_constant
        ));
        for c in &self.summary.characteristics {
            md.push_str(&format!("- {}\n", c));
        }
        md.push_str("\n## Structure\n\n");
        md.push_str(&format!(
            "- Symbols: `{}`\n",
            self.structure.symbols.join(", ")
        ));
        md.push_str(&format!("- Depth: {}\n", self.structure.depth));
        md.push_str(&format!("- Root: {}\n", self.structure.is_root));
        if !self.structure.dependencies.is_empty() {
            md.push_str(&format!(
                "- Dependencies: {}\n",
                self.structure.dependencies.join(", ")
            ));
        }
        if !self.structure.dependents.is_empty() {
            md.push_str(&format!(
                "- Dependents: {}\n",
                self.structure.dependents.join(", ")
            ));
        }
        md.push_str("\n## Grounding\n\n");
        if !self.grounding.atoms.is_empty() {
            md.push_str("### Bedrock Atoms\n\n");
            for atom in &self.grounding.atoms {
                md.push_str(&format!("- {} → `{}`\n", atom.name, atom.constant));
            }
        }
        if !self.grounding.constants.is_empty() {
            md.push_str(&format!(
                "\n**Constants:** {}\n",
                self.grounding.constants.join(", ")
            ));
        }
        md
    }

    /// Render as JSON.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| String::from("{}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_dossier() {
        let generator = DossierGenerator::new();
        let dossier = generator.for_primitive(LexPrimitiva::Quantity);
        assert!(dossier.title.contains("Quantity"));
        assert_eq!(dossier.summary.tier, Tier::T1Universal);
    }

    #[test]
    fn test_composition_dossier() {
        let generator = DossierGenerator::new();
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Mapping]);
        let dossier = generator.for_composition(&comp);
        assert!(dossier.title.contains("Composition"));
    }

    #[test]
    fn test_atom_dossier() {
        let generator = DossierGenerator::new();
        let dossier = generator.for_atom(BedrockAtom::Count);
        assert!(dossier.title.contains("Count"));
    }

    #[test]
    fn test_markdown_output() {
        let generator = DossierGenerator::new();
        let dossier = generator.for_primitive(LexPrimitiva::Causality);
        let md = dossier.to_markdown();
        assert!(md.contains("# Causality"));
        assert!(md.contains("## Summary"));
    }

    #[test]
    fn test_json_output() {
        let generator = DossierGenerator::new();
        let dossier = generator.for_primitive(LexPrimitiva::Void);
        let json = dossier.to_json();
        assert!(json.contains("Void"));
    }

    #[test]
    fn test_compact_mode() {
        let generator = DossierGenerator::compact();
        let dossier = generator.for_primitive(LexPrimitiva::Sequence);
        assert!(dossier.grounding.traces.is_empty());
    }

    #[test]
    fn test_depth_calculation() {
        let generator = DossierGenerator::new();
        let root_dossier = generator.for_primitive(LexPrimitiva::Quantity);
        assert_eq!(root_dossier.structure.depth, 0);
        let derived_dossier = generator.for_primitive(LexPrimitiva::Frequency);
        assert!(derived_dossier.structure.depth > 0);
    }
}
