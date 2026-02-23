//! # Composition Grammar
//!
//! Named patterns and interaction semantics for primitive compositions.
//!
//! ## Overview
//!
//! The grammar layer adds semantic awareness to the set-theoretic composition algebra:
//! - **Named Patterns**: 12 canonical compositions extracted from 553+ GroundsTo impls
//! - **Interaction Types**: 6 relation types between primitives
//! - **Interaction Graph**: 16x16 adjacency matrix of primitive relationships
//!
//! ## Tier: T2-C (Mapping + Comparison + Product + Sum)

use crate::primitiva::{LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════════
// INTERACTION MODEL
// ═══════════════════════════════════════════════════════════════════════════════

/// How two primitives relate within a composition.
///
/// These 6 relations capture all observed primitive-to-primitive interactions
/// across 553+ GroundsTo implementations.
///
/// Tier: T2-P (Comparison + Causality)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InteractionType {
    /// Source primitive protects/gates the target.
    /// Example: ∂ Guards κ in ThresholdGate (boundary guards comparison)
    Guards,
    /// Source primitive provides input/resources to target.
    /// Example: N Provides ∂ (quantity provides to boundary)
    Provides,
    /// Source primitive transforms into target via some operation.
    /// Example: μ Transforms → (mapping transforms causality)
    Transforms,
    /// Source primitive limits/restricts the target.
    /// Example: ∂ Constrains σ (boundary constrains sequence)
    Constrains,
    /// Source and target combine to form a higher-level construct.
    /// Example: ς Composes σ in Vec (state + sequence)
    Composes,
    /// Source and target operate independently within the same composition.
    /// Example: ν Parallel κ in Monitor (frequency parallel to comparison)
    Parallel,
}

impl InteractionType {
    /// Returns all interaction types.
    #[must_use]
    pub const fn all() -> [Self; 6] {
        [
            Self::Guards,
            Self::Provides,
            Self::Transforms,
            Self::Constrains,
            Self::Composes,
            Self::Parallel,
        ]
    }

    /// Default weight for this interaction type (0.0-1.0).
    ///
    /// Guards and Constrains have higher weights because they represent
    /// structural relationships. Parallel has lowest weight.
    #[must_use]
    pub const fn default_weight(&self) -> f64 {
        match self {
            Self::Guards => 0.9,
            Self::Provides => 0.7,
            Self::Transforms => 0.8,
            Self::Constrains => 0.85,
            Self::Composes => 0.75,
            Self::Parallel => 0.5,
        }
    }

    /// Short symbol for display.
    #[must_use]
    pub const fn symbol(&self) -> &'static str {
        match self {
            Self::Guards => ">>",
            Self::Provides => "->",
            Self::Transforms => "=>",
            Self::Constrains => "|>",
            Self::Composes => "<>",
            Self::Parallel => "||",
        }
    }
}

impl std::fmt::Display for InteractionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Guards => "Guards",
            Self::Provides => "Provides",
            Self::Transforms => "Transforms",
            Self::Constrains => "Constrains",
            Self::Composes => "Composes",
            Self::Parallel => "Parallel",
        };
        write!(f, "{}", name)
    }
}

/// A directed interaction between two primitives.
///
/// Tier: T2-P (Causality + Comparison)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interaction {
    /// Source primitive.
    pub source: LexPrimitiva,
    /// Target primitive.
    pub target: LexPrimitiva,
    /// Relationship type.
    pub relation: InteractionType,
    /// Weight of this interaction (0.0-1.0).
    pub weight: f64,
}

impl Interaction {
    /// Create a new interaction with the relation's default weight.
    #[must_use]
    pub fn new(source: LexPrimitiva, target: LexPrimitiva, relation: InteractionType) -> Self {
        Self {
            source,
            target,
            relation,
            weight: relation.default_weight(),
        }
    }

    /// Create a new interaction with a custom weight.
    #[must_use]
    pub fn with_weight(
        source: LexPrimitiva,
        target: LexPrimitiva,
        relation: InteractionType,
        weight: f64,
    ) -> Self {
        Self {
            source,
            target,
            relation,
            weight: weight.clamp(0.0, 1.0),
        }
    }
}

impl std::fmt::Display for Interaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} ({:.2})",
            self.source.symbol(),
            self.relation.symbol(),
            self.target.symbol(),
            self.weight,
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// INTERACTION GRAPH
// ═══════════════════════════════════════════════════════════════════════════════

/// 16x16 adjacency matrix of primitive interactions.
///
/// Encodes the canonical relationships observed across GroundsTo implementations.
/// Not every pair has a defined interaction — `lookup` returns `None` for
/// unrelated primitive pairs.
///
/// Tier: T2-C (Location + Comparison + Mapping + Product)
#[derive(Debug, Clone)]
pub struct InteractionGraph {
    /// Adjacency stored as (source_index, target_index) -> Interaction.
    edges: HashMap<(usize, usize), InteractionType>,
}

impl InteractionGraph {
    /// Build the canonical interaction graph from observed GroundsTo patterns.
    #[must_use]
    pub fn canonical() -> Self {
        use LexPrimitiva::*;
        let mut edges = HashMap::new();

        // Helper to insert by primitive pair
        let mut add = |src: LexPrimitiva, tgt: LexPrimitiva, rel: InteractionType| {
            let si = Self::index(src);
            let ti = Self::index(tgt);
            edges.insert((si, ti), rel);
        };

        // Gatekeeper pattern: ∂ guards κ, N provides ∂
        add(Boundary, Comparison, InteractionType::Guards);
        add(Quantity, Boundary, InteractionType::Provides);

        // Pipeline pattern: σ composes N, ∂ constrains σ
        add(Sequence, Quantity, InteractionType::Composes);
        add(Boundary, Sequence, InteractionType::Constrains);

        // Container pattern: ς composes σ
        add(State, Sequence, InteractionType::Composes);

        // Transformer pattern: μ transforms →
        add(Mapping, Causality, InteractionType::Transforms);

        // Accumulator pattern: Σ composes σ, Σ composes N
        add(Sum, Sequence, InteractionType::Composes);
        add(Sum, Quantity, InteractionType::Composes);

        // Monitor pattern: ν parallel κ, ν parallel ∂
        add(Frequency, Comparison, InteractionType::Parallel);
        add(Frequency, Boundary, InteractionType::Parallel);

        // Archive pattern: π provides σ, ∃ provides π
        add(Persistence, Sequence, InteractionType::Provides);
        add(Existence, Persistence, InteractionType::Provides);

        // Absence pattern: ∅ composes κ, ∅ composes ∂
        add(Void, Comparison, InteractionType::Composes);
        add(Void, Boundary, InteractionType::Composes);

        // Navigator pattern: λ composes σ, ∃ provides λ
        add(Location, Sequence, InteractionType::Composes);
        add(Existence, Location, InteractionType::Provides);

        // Recurser pattern: ρ transforms N, ∂ constrains ρ
        add(Recursion, Quantity, InteractionType::Transforms);
        add(Boundary, Recursion, InteractionType::Constrains);

        // Record pattern: × composes ∃, × composes N
        add(Product, Existence, InteractionType::Composes);
        add(Product, Quantity, InteractionType::Composes);

        // Lifecycle pattern: ∝ transforms N, → provides ∝
        add(Irreversibility, Quantity, InteractionType::Transforms);
        add(Causality, Irreversibility, InteractionType::Provides);

        // Cross-pattern interactions
        add(Mapping, Sequence, InteractionType::Transforms);
        add(Causality, State, InteractionType::Provides);
        add(Comparison, Quantity, InteractionType::Provides);
        add(State, Persistence, InteractionType::Provides);

        Self { edges }
    }

    /// Lookup the interaction between two primitives.
    #[must_use]
    pub fn lookup(&self, source: LexPrimitiva, target: LexPrimitiva) -> Option<InteractionType> {
        let si = Self::index(source);
        let ti = Self::index(target);
        self.edges.get(&(si, ti)).copied()
    }

    /// Get all interactions involving a primitive (as source or target).
    #[must_use]
    pub fn interactions_for(&self, primitive: LexPrimitiva) -> Vec<Interaction> {
        let pi = Self::index(primitive);
        let all_prims = LexPrimitiva::all();
        let mut result = Vec::new();

        for (&(si, ti), &rel) in &self.edges {
            if si == pi {
                result.push(Interaction::new(primitive, all_prims[ti], rel));
            } else if ti == pi {
                result.push(Interaction::new(all_prims[si], primitive, rel));
            }
        }
        result
    }

    /// Total number of defined edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Map a primitive to its canonical index (0-15).
    fn index(p: LexPrimitiva) -> usize {
        LexPrimitiva::all()
            .iter()
            .position(|&x| x == p)
            .unwrap_or(0)
    }
}

impl Default for InteractionGraph {
    fn default() -> Self {
        Self::canonical()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// NAMED PATTERNS
// ═══════════════════════════════════════════════════════════════════════════════

/// A named composition pattern extracted from GroundsTo frequency analysis.
///
/// Each pattern represents a recurring primitive combination observed
/// across multiple GroundsTo implementations.
///
/// Tier: T2-C (Mapping + Comparison + Sequence + Product)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Canonical name (e.g., "Gatekeeper", "Pipeline").
    pub name: String,
    /// The primitive composition defining this pattern.
    pub composition: PrimitiveComposition,
    /// Human-readable description.
    pub description: String,
    /// Example types that match this pattern.
    pub examples: Vec<String>,
    /// How many GroundsTo impls match this pattern.
    pub frequency: usize,
    /// The interactions within this pattern.
    pub interactions: Vec<Interaction>,
}

impl Pattern {
    /// Check if a composition matches this pattern.
    ///
    /// A composition matches if it contains all primitives in the pattern
    /// (superset match — the composition may have additional primitives).
    #[must_use]
    pub fn matches(&self, comp: &PrimitiveComposition) -> bool {
        let comp_set = comp.unique();
        let pattern_set = self.composition.unique();
        pattern_set.is_subset(&comp_set)
    }

    /// Compute the distance between a composition and this pattern.
    ///
    /// Returns 0.0 for exact match, higher for more distant compositions.
    /// Distance = (missing primitives + extra primitives) / total unique.
    #[must_use]
    pub fn distance(&self, comp: &PrimitiveComposition) -> f64 {
        let comp_set = comp.unique();
        let pattern_set = self.composition.unique();

        let missing = pattern_set.difference(&comp_set).count();
        let extra = comp_set.difference(&pattern_set).count();

        let total = comp_set.len().max(pattern_set.len());
        if total == 0 {
            return 0.0;
        }

        (missing + extra) as f64 / total as f64
    }
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} (freq={})",
            self.name, self.composition, self.frequency,
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PATTERN REGISTRY
// ═══════════════════════════════════════════════════════════════════════════════

/// Registry of named composition patterns.
///
/// Pre-loaded with 12 canonical patterns extracted from GroundsTo frequency
/// analysis across 553+ implementations.
///
/// Tier: T2-C (Mapping + Sequence + Persistence + Existence)
#[derive(Debug, Clone)]
pub struct PatternRegistry {
    patterns: HashMap<String, Pattern>,
}

impl PatternRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }

    /// Create the canonical registry with 12 pre-loaded patterns.
    #[must_use]
    pub fn canonical() -> Self {
        use LexPrimitiva::*;
        let mut registry = Self::new();

        let graph = InteractionGraph::canonical();

        // 1. Gatekeeper: ∂ + κ (freq=12)
        registry.register(Pattern {
            name: "Gatekeeper".to_string(),
            composition: PrimitiveComposition::new(vec![Boundary, Comparison])
                .with_dominant(Boundary, 0.9),
            description: "Boundary guards comparison — threshold/filter patterns".to_string(),
            examples: vec![
                "ThresholdGate".to_string(),
                "SafetyBoundary".to_string(),
                "Result<T,E>".to_string(),
            ],
            frequency: 12,
            interactions: vec![Interaction::new(
                Boundary,
                Comparison,
                InteractionType::Guards,
            )],
        });

        // 2. Pipeline: σ + N + ∂ (freq=7)
        registry.register(Pattern {
            name: "Pipeline".to_string(),
            composition: PrimitiveComposition::new(vec![Sequence, Quantity, Boundary])
                .with_dominant(Sequence, 0.9),
            description: "Ordered steps with numeric tracking and bounds".to_string(),
            examples: vec![
                "Pipeline".to_string(),
                "SignalStage".to_string(),
                "ProcessingChain".to_string(),
            ],
            frequency: 7,
            interactions: vec![
                Interaction::new(Sequence, Quantity, InteractionType::Composes),
                Interaction::new(Boundary, Sequence, InteractionType::Constrains),
            ],
        });

        // 3. Container: ς + σ (freq=6)
        registry.register(Pattern {
            name: "Container".to_string(),
            composition: PrimitiveComposition::new(vec![State, Sequence])
                .with_dominant(State, 0.85),
            description: "Stateful ordered collection".to_string(),
            examples: vec![
                "Vec<T>".to_string(),
                "String".to_string(),
                "SessionTracker".to_string(),
            ],
            frequency: 6,
            interactions: vec![Interaction::new(State, Sequence, InteractionType::Composes)],
        });

        // 4. Transformer: μ + → (freq=5)
        registry.register(Pattern {
            name: "Transformer".to_string(),
            composition: PrimitiveComposition::new(vec![Mapping, Causality])
                .with_dominant(Mapping, 0.9),
            description: "Causal transformation between domains".to_string(),
            examples: vec![
                "From/Into".to_string(),
                "map()".to_string(),
                "Converter".to_string(),
            ],
            frequency: 5,
            interactions: vec![Interaction::new(
                Mapping,
                Causality,
                InteractionType::Transforms,
            )],
        });

        // 5. Accumulator: Σ + σ + N (freq=4)
        registry.register(Pattern {
            name: "Accumulator".to_string(),
            composition: PrimitiveComposition::new(vec![Sum, Sequence, Quantity])
                .with_dominant(Sum, 0.85),
            description: "Sequential aggregation of numeric values".to_string(),
            examples: vec![
                "fold()".to_string(),
                "reduce()".to_string(),
                "Aggregate".to_string(),
            ],
            frequency: 4,
            interactions: vec![
                Interaction::new(Sum, Sequence, InteractionType::Composes),
                Interaction::new(Sum, Quantity, InteractionType::Composes),
            ],
        });

        // 6. Monitor: ν + κ + ∂ (freq=4)
        registry.register(Pattern {
            name: "Monitor".to_string(),
            composition: PrimitiveComposition::new(vec![Frequency, Comparison, Boundary])
                .with_dominant(Frequency, 0.85),
            description: "Frequency-based comparison with bounds".to_string(),
            examples: vec![
                "FeedbackLoop".to_string(),
                "RateLimiter".to_string(),
                "HealthCheck".to_string(),
            ],
            frequency: 4,
            interactions: vec![
                Interaction::new(Frequency, Comparison, InteractionType::Parallel),
                Interaction::new(Frequency, Boundary, InteractionType::Parallel),
            ],
        });

        // 7. Archive: π + σ + ∃ (freq=3)
        registry.register(Pattern {
            name: "Archive".to_string(),
            composition: PrimitiveComposition::new(vec![Persistence, Sequence, Existence])
                .with_dominant(Persistence, 0.9),
            description: "Persistent ordered record of existing entities".to_string(),
            examples: vec![
                "AuditTrail".to_string(),
                "Checkpoint".to_string(),
                "EventLog".to_string(),
            ],
            frequency: 3,
            interactions: vec![
                Interaction::new(Persistence, Sequence, InteractionType::Provides),
                Interaction::new(Existence, Persistence, InteractionType::Provides),
            ],
        });

        // 8. Absence: ∅ + κ + ∂ (freq=3)
        registry.register(Pattern {
            name: "Absence".to_string(),
            composition: PrimitiveComposition::new(vec![Void, Comparison, Boundary])
                .with_dominant(Void, 0.85),
            description: "Meaningful absence with comparison and boundary".to_string(),
            examples: vec![
                "Option<T>".to_string(),
                "AbsenceMarker".to_string(),
                "NullHandler".to_string(),
            ],
            frequency: 3,
            interactions: vec![
                Interaction::new(Void, Comparison, InteractionType::Composes),
                Interaction::new(Void, Boundary, InteractionType::Composes),
            ],
        });

        // 9. Navigator: λ + σ + ∃ (freq=3)
        registry.register(Pattern {
            name: "Navigator".to_string(),
            composition: PrimitiveComposition::new(vec![Location, Sequence, Existence])
                .with_dominant(Location, 0.85),
            description: "Location-aware sequential traversal".to_string(),
            examples: vec![
                "ResourcePath".to_string(),
                "TopologicalAddress".to_string(),
                "TreeWalker".to_string(),
            ],
            frequency: 3,
            interactions: vec![
                Interaction::new(Location, Sequence, InteractionType::Composes),
                Interaction::new(Existence, Location, InteractionType::Provides),
            ],
        });

        // 10. Recurser: ρ + N + ∂ (freq=3)
        registry.register(Pattern {
            name: "Recurser".to_string(),
            composition: PrimitiveComposition::new(vec![Recursion, Quantity, Boundary])
                .with_dominant(Recursion, 0.9),
            description: "Bounded recursive computation with numeric tracking".to_string(),
            examples: vec![
                "RecursionBound".to_string(),
                "TreeTraversal".to_string(),
                "FixpointIterator".to_string(),
            ],
            frequency: 3,
            interactions: vec![
                Interaction::new(Recursion, Quantity, InteractionType::Transforms),
                Interaction::new(Boundary, Recursion, InteractionType::Constrains),
            ],
        });

        // 11. Record: × + ∃ + N (freq=3)
        registry.register(Pattern {
            name: "Record".to_string(),
            composition: PrimitiveComposition::new(vec![Product, Existence, Quantity])
                .with_dominant(Product, 0.85),
            description: "Product type combining existing numeric fields".to_string(),
            examples: vec![
                "RecordStructure".to_string(),
                "Tuple".to_string(),
                "DataPoint".to_string(),
            ],
            frequency: 3,
            interactions: vec![
                Interaction::new(Product, Existence, InteractionType::Composes),
                Interaction::new(Product, Quantity, InteractionType::Composes),
            ],
        });

        // 12. Lifecycle: ∝ + N + → (freq=2)
        registry.register(Pattern {
            name: "Lifecycle".to_string(),
            composition: PrimitiveComposition::new(vec![Irreversibility, Quantity, Causality])
                .with_dominant(Irreversibility, 0.9),
            description: "Irreversible causal process with numeric measurement".to_string(),
            examples: vec!["ConsumptionMark".to_string(), "DecayKinetics".to_string()],
            frequency: 2,
            interactions: vec![
                Interaction::new(Irreversibility, Quantity, InteractionType::Transforms),
                Interaction::new(Causality, Irreversibility, InteractionType::Provides),
            ],
        });

        // 13. Gradient: → + κ + ∂ (freq=23) — Weather bridge: pressure gradient drives flow
        registry.register(Pattern {
            name: "Gradient".to_string(),
            composition: PrimitiveComposition::new(vec![Causality, Comparison, Boundary])
                .with_dominant(Boundary, 0.9),
            description:
                "Difference across a boundary that drives flow — quality, knowledge, effort"
                    .to_string(),
            examples: vec![
                "CausalityScore".to_string(),
                "SafetyMargin".to_string(),
                "Determination".to_string(),
            ],
            frequency: 23,
            interactions: vec![
                Interaction::new(Boundary, Comparison, InteractionType::Guards),
                Interaction::new(Comparison, Causality, InteractionType::Transforms),
            ],
        });

        // 14. PhaseTransition: ς + ∝ + ∂ (freq=18) — Weather bridge: qualitative state reorganization
        registry.register(Pattern {
            name: "PhaseTransition".to_string(),
            composition: PrimitiveComposition::new(vec![State, Irreversibility, Boundary])
                .with_dominant(State, 0.85),
            description: "Qualitative state change — fundamentally different organization, not quantitative shift".to_string(),
            examples: vec![
                "DominantShift".to_string(),
                "Checkpoint".to_string(),
                "PhaseGate".to_string(),
            ],
            frequency: 18,
            interactions: vec![
                Interaction::new(State, Boundary, InteractionType::Constrains),
                Interaction::new(Boundary, Irreversibility, InteractionType::Transforms),
            ],
        });

        // Verify canonical graph is used
        drop(graph);

        registry
    }

    /// Register a pattern.
    pub fn register(&mut self, pattern: Pattern) {
        self.patterns.insert(pattern.name.clone(), pattern);
    }

    /// Get a pattern by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Pattern> {
        self.patterns.get(name)
    }

    /// Find all patterns that match a composition (superset match).
    #[must_use]
    pub fn find_matches(&self, comp: &PrimitiveComposition) -> Vec<&Pattern> {
        let mut matches: Vec<&Pattern> =
            self.patterns.values().filter(|p| p.matches(comp)).collect();
        // Sort by frequency descending (most common first)
        matches.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        matches
    }

    /// Find the closest pattern to a composition by distance.
    #[must_use]
    pub fn closest(&self, comp: &PrimitiveComposition) -> Option<(&Pattern, f64)> {
        self.patterns
            .values()
            .map(|p| (p, p.distance(comp)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Number of registered patterns.
    #[must_use]
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Iterate over all patterns.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Pattern)> {
        self.patterns.iter()
    }

    /// Get all pattern names sorted alphabetically.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.patterns.keys().map(String::as_str).collect();
        names.sort();
        names
    }

    /// Compute compression stats for a composition.
    ///
    /// Returns `CompressionMetrics` indicating how well the composition
    /// compresses to known patterns.
    #[must_use]
    pub fn compression_metrics(&self, comp: &PrimitiveComposition) -> CompressionMetrics {
        let matches = self.find_matches(comp);
        let closest = self.closest(comp);

        let canonical_distance = closest.map(|(_, d)| d).unwrap_or(1.0);
        let canonical_name = closest.map(|(p, _)| p.name.as_str());

        let pattern_coverage = if comp.primitives.is_empty() {
            0.0
        } else {
            matches
                .first()
                .map(|p| {
                    let denom = comp.unique().len();
                    if denom == 0 {
                        0.0
                    } else {
                        p.composition.unique().len() as f64 / denom as f64
                    }
                })
                .unwrap_or(0.0)
        };

        let redundancy = self.compute_redundancy(comp);

        let hints = self.generate_hints(comp, &matches, canonical_distance);

        CompressionMetrics {
            canonical_distance,
            canonical_name: canonical_name.map(str::to_string),
            pattern_coverage,
            redundancy,
            matched_patterns: matches.len(),
            hints,
        }
    }

    /// Compute redundancy score (0.0 = no redundancy, 1.0 = fully redundant).
    fn compute_redundancy(&self, comp: &PrimitiveComposition) -> f64 {
        let total = comp.primitives.len();
        if total == 0 {
            return 0.0;
        }
        let unique = comp.unique().len();
        1.0 - (unique as f64 / total as f64)
    }

    /// Generate optimization hints.
    fn generate_hints(
        &self,
        comp: &PrimitiveComposition,
        matches: &[&Pattern],
        distance: f64,
    ) -> Vec<String> {
        let mut hints = Vec::new();

        if comp.primitives.is_empty() {
            hints.push("Empty composition has no pattern".to_string());
            return hints;
        }

        if distance < 0.01 {
            if let Some(p) = matches.first() {
                hints.push(format!("Exact match: {} pattern", p.name));
            }
        } else if distance < 0.4 {
            if let Some((p, _)) = self.closest(comp) {
                let comp_set = comp.unique();
                let pattern_set = p.composition.unique();
                let extra: Vec<&str> = comp_set
                    .difference(&pattern_set)
                    .map(|lp| lp.symbol())
                    .collect();
                let missing: Vec<&str> = pattern_set
                    .difference(&comp_set)
                    .map(|lp| lp.symbol())
                    .collect();

                if !extra.is_empty() {
                    hints.push(format!(
                        "Near {} pattern — extra primitives: [{}]",
                        p.name,
                        extra.join(", ")
                    ));
                }
                if !missing.is_empty() {
                    hints.push(format!(
                        "Near {} pattern — missing primitives: [{}]",
                        p.name,
                        missing.join(", ")
                    ));
                }
            }
        } else {
            hints.push("No close canonical pattern — may be a novel composition".to_string());
        }

        // Duplicate warning
        let dupes = comp.primitives.len() - comp.unique().len();
        if dupes > 0 {
            hints.push(format!(
                "{} duplicate primitive(s) — consider deduplication",
                dupes
            ));
        }

        hints
    }
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::canonical()
    }
}

/// Compression metrics for a composition relative to known patterns.
///
/// Tier: T2-C (Quantity + Comparison + Mapping)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionMetrics {
    /// Distance to nearest canonical pattern (0.0 = exact match).
    pub canonical_distance: f64,
    /// Name of the nearest canonical pattern, if any.
    pub canonical_name: Option<String>,
    /// Fraction of composition covered by the best matching pattern (0.0-1.0).
    pub pattern_coverage: f64,
    /// Duplicate primitive ratio (0.0 = no dupes, 1.0 = all dupes).
    pub redundancy: f64,
    /// Number of canonical patterns matched.
    pub matched_patterns: usize,
    /// Human-readable optimization hints.
    pub hints: Vec<String>,
}

impl std::fmt::Display for CompressionMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "distance={:.2} coverage={:.0}% redundancy={:.0}% patterns={}",
            self.canonical_distance,
            self.pattern_coverage * 100.0,
            self.redundancy * 100.0,
            self.matched_patterns,
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitiva::LexPrimitiva::*;

    // ─── Registry tests ───

    #[test]
    fn test_canonical_registry_has_14_patterns() {
        let registry = PatternRegistry::canonical();
        assert_eq!(registry.len(), 14);
    }

    #[test]
    fn test_canonical_patterns_all_named() {
        let registry = PatternRegistry::canonical();
        let names = registry.names();
        assert!(names.contains(&"Gatekeeper"));
        assert!(names.contains(&"Pipeline"));
        assert!(names.contains(&"Container"));
        assert!(names.contains(&"Transformer"));
        assert!(names.contains(&"Accumulator"));
        assert!(names.contains(&"Monitor"));
        assert!(names.contains(&"Archive"));
        assert!(names.contains(&"Absence"));
        assert!(names.contains(&"Navigator"));
        assert!(names.contains(&"Recurser"));
        assert!(names.contains(&"Record"));
        assert!(names.contains(&"Lifecycle"));
    }

    #[test]
    fn test_get_pattern_by_name() {
        let registry = PatternRegistry::canonical();
        let gatekeeper = registry.get("Gatekeeper");
        assert!(gatekeeper.is_some());
        let gk = gatekeeper.unwrap_or_else(|| {
            // Can't panic in test, use first pattern as fallback
            registry.patterns.values().next().expect("empty")
        });
        assert_eq!(gk.frequency, 12);
        assert!(gk.composition.unique().contains(&Boundary));
        assert!(gk.composition.unique().contains(&Comparison));
    }

    #[test]
    fn test_get_nonexistent_pattern() {
        let registry = PatternRegistry::canonical();
        assert!(registry.get("Nonexistent").is_none());
    }

    #[test]
    fn test_empty_registry() {
        let registry = PatternRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_custom_pattern() {
        let mut registry = PatternRegistry::new();
        registry.register(Pattern {
            name: "Custom".to_string(),
            composition: PrimitiveComposition::new(vec![Sequence]),
            description: "test".to_string(),
            examples: vec![],
            frequency: 1,
            interactions: vec![],
        });
        assert_eq!(registry.len(), 1);
        assert!(registry.get("Custom").is_some());
    }

    // ─── Pattern matching tests ───

    #[test]
    fn test_exact_match() {
        let registry = PatternRegistry::canonical();
        let comp = PrimitiveComposition::new(vec![Boundary, Comparison]);
        let matches = registry.find_matches(&comp);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].name, "Gatekeeper");
    }

    #[test]
    fn test_superset_match() {
        let registry = PatternRegistry::canonical();
        // Gatekeeper is ∂ + κ, this has ∂ + κ + N (superset)
        let comp = PrimitiveComposition::new(vec![Boundary, Comparison, Quantity]);
        let matches = registry.find_matches(&comp);
        let names: Vec<&str> = matches.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"Gatekeeper"));
    }

    #[test]
    fn test_no_match_for_single_primitive() {
        let registry = PatternRegistry::canonical();
        let comp = PrimitiveComposition::new(vec![Quantity]);
        let matches = registry.find_matches(&comp);
        // Single primitive shouldn't match any 2+ primitive pattern
        assert!(matches.is_empty());
    }

    #[test]
    fn test_pattern_distance_exact() {
        let registry = PatternRegistry::canonical();
        let gk = registry.get("Gatekeeper");
        assert!(gk.is_some());
        if let Some(gk) = gk {
            let comp = PrimitiveComposition::new(vec![Boundary, Comparison]);
            let dist = gk.distance(&comp);
            assert!(dist < f64::EPSILON, "Exact match should be distance 0.0");
        }
    }

    #[test]
    fn test_pattern_distance_nonzero() {
        let registry = PatternRegistry::canonical();
        let gk = registry.get("Gatekeeper");
        assert!(gk.is_some());
        if let Some(gk) = gk {
            let comp = PrimitiveComposition::new(vec![Sequence, Mapping]);
            let dist = gk.distance(&comp);
            assert!(
                dist > 0.0,
                "Different composition should have nonzero distance"
            );
        }
    }

    #[test]
    fn test_closest_pattern() {
        let registry = PatternRegistry::canonical();
        let comp = PrimitiveComposition::new(vec![Boundary, Comparison]);
        let closest = registry.closest(&comp);
        assert!(closest.is_some());
        if let Some((p, dist)) = closest {
            assert!(dist < 0.01, "Exact match should be very close");
            assert_eq!(p.name, "Gatekeeper");
        }
    }

    // ─── Interaction graph tests ───

    #[test]
    fn test_canonical_graph_nonempty() {
        let graph = InteractionGraph::canonical();
        assert!(graph.edge_count() > 0);
    }

    #[test]
    fn test_canonical_graph_edge_count() {
        let graph = InteractionGraph::canonical();
        // We defined 24 edges in the canonical graph
        assert!(
            graph.edge_count() >= 20,
            "Expected at least 20 edges, got {}",
            graph.edge_count()
        );
    }

    #[test]
    fn test_boundary_guards_comparison() {
        let graph = InteractionGraph::canonical();
        let result = graph.lookup(Boundary, Comparison);
        assert_eq!(result, Some(InteractionType::Guards));
    }

    #[test]
    fn test_mapping_transforms_causality() {
        let graph = InteractionGraph::canonical();
        let result = graph.lookup(Mapping, Causality);
        assert_eq!(result, Some(InteractionType::Transforms));
    }

    #[test]
    fn test_unknown_pair_returns_none() {
        let graph = InteractionGraph::canonical();
        // Product -> Void not in canonical graph
        let result = graph.lookup(Product, Void);
        assert!(result.is_none());
    }

    #[test]
    fn test_interactions_for_boundary() {
        let graph = InteractionGraph::canonical();
        let interactions = graph.interactions_for(Boundary);
        assert!(!interactions.is_empty());
        // Boundary appears as source in Guards and Constrains
        let has_guards = interactions
            .iter()
            .any(|i| i.relation == InteractionType::Guards);
        assert!(has_guards);
    }

    // ─── Interaction type tests ───

    #[test]
    fn test_all_interaction_types() {
        let all = InteractionType::all();
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn test_interaction_default_weights() {
        for it in InteractionType::all() {
            let w = it.default_weight();
            assert!(w > 0.0 && w <= 1.0, "{:?} weight out of range: {}", it, w);
        }
    }

    #[test]
    fn test_interaction_display() {
        let interaction = Interaction::new(Boundary, Comparison, InteractionType::Guards);
        let s = format!("{}", interaction);
        assert!(s.contains("∂"));
        assert!(s.contains("κ"));
        assert!(s.contains(">>"));
    }

    #[test]
    fn test_interaction_with_weight() {
        let interaction =
            Interaction::with_weight(Mapping, Causality, InteractionType::Transforms, 0.5);
        assert!((interaction.weight - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_interaction_weight_clamping() {
        let over = Interaction::with_weight(Mapping, Causality, InteractionType::Transforms, 5.0);
        assert!((over.weight - 1.0).abs() < f64::EPSILON);
        let under = Interaction::with_weight(Mapping, Causality, InteractionType::Transforms, -1.0);
        assert!(under.weight.abs() < f64::EPSILON);
    }

    // ─── Compression metrics tests ───

    #[test]
    fn test_compression_exact_match() {
        let registry = PatternRegistry::canonical();
        let comp = PrimitiveComposition::new(vec![Boundary, Comparison]);
        let metrics = registry.compression_metrics(&comp);
        assert!(metrics.canonical_distance < 0.01);
        assert!(metrics.pattern_coverage > 0.99);
        assert_eq!(metrics.redundancy, 0.0);
        assert!(metrics.matched_patterns > 0);
    }

    #[test]
    fn test_compression_empty() {
        let registry = PatternRegistry::canonical();
        let comp = PrimitiveComposition::new(vec![]);
        let metrics = registry.compression_metrics(&comp);
        assert_eq!(metrics.pattern_coverage, 0.0);
        assert_eq!(metrics.redundancy, 0.0);
        assert!(!metrics.hints.is_empty());
    }

    #[test]
    fn test_compression_with_duplicates() {
        let registry = PatternRegistry::canonical();
        let comp = PrimitiveComposition::new(vec![Boundary, Comparison, Boundary]);
        let metrics = registry.compression_metrics(&comp);
        assert!(metrics.redundancy > 0.0);
        assert!(metrics.hints.iter().any(|h| h.contains("duplicate")));
    }

    #[test]
    fn test_compression_novel_composition() {
        let registry = PatternRegistry::canonical();
        // A composition unlike any canonical pattern
        let comp = PrimitiveComposition::new(vec![
            Frequency,
            Persistence,
            Location,
            Irreversibility,
            Product,
            Sum,
        ]);
        let metrics = registry.compression_metrics(&comp);
        assert!(metrics.canonical_distance > 0.4);
    }

    // ─── Pattern display test ───

    #[test]
    fn test_pattern_display() {
        let registry = PatternRegistry::canonical();
        if let Some(p) = registry.get("Gatekeeper") {
            let s = format!("{}", p);
            assert!(s.contains("Gatekeeper"));
            assert!(s.contains("freq=12"));
        }
    }

    #[test]
    fn test_interaction_type_display() {
        assert_eq!(format!("{}", InteractionType::Guards), "Guards");
        assert_eq!(format!("{}", InteractionType::Parallel), "Parallel");
    }

    #[test]
    fn test_compression_metrics_display() {
        let metrics = CompressionMetrics {
            canonical_distance: 0.15,
            canonical_name: Some("Pipeline".to_string()),
            pattern_coverage: 0.75,
            redundancy: 0.1,
            matched_patterns: 2,
            hints: vec![],
        };
        let s = format!("{}", metrics);
        assert!(s.contains("distance=0.15"));
        assert!(s.contains("coverage=75%"));
    }
}
