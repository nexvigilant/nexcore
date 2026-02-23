//! Semantic elements — the irreducible units of regulatory meaning.
//!
//! Chemistry analogue: atoms and compounds with conservation laws.

use nexcore_id::NexId;
use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

// =============================================================================
// SEMANTIC ELEMENT CLASSES
// =============================================================================
// Just as chemistry has element groups (noble gases, halogens, metals),
// semantic elements belong to fundamental classes.
//
// These ~8 classes compose into the full MedDRA hierarchy of ~80,000 terms.
// The claim: you need maybe 200-300 truly atomic concepts across these classes.

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElementClass {
    /// Organ systems: cardiac, hepatic, renal, neurological...
    OrganSystem,
    /// Causality markers: attributed, suspected, coincidental, unassessable...
    Causality,
    /// Temporal relations: acute, chronic, delayed, immediate, onset...
    Temporality,
    /// Severity grades: mild, moderate, severe, life-threatening, fatal...
    Severity,
    /// Observation types: reported_event, lab_finding, diagnosis, symptom...
    ObservationType,
    /// Modifiers: related, unrelated, expected, unexpected...
    Modifier,
    /// Actions: dose_reduced, drug_withdrawn, hospitalized...
    Action,
    /// Outcomes: recovered, recovering, not_recovered, fatal, unknown...
    Outcome,
}

impl ElementClass {
    /// Map to the dominant Lex Primitiva.
    ///
    /// This is a convergent mapping — the 8 element classes independently
    /// correspond to 8 distinct T1 primitives.
    pub fn dominant_primitive(&self) -> LexPrimitiva {
        match self {
            Self::OrganSystem => LexPrimitiva::Location, // λ — body locations
            Self::Causality => LexPrimitiva::Causality,  // → — causal links
            Self::Temporality => LexPrimitiva::Frequency, // ν — time/frequency domain
            Self::Severity => LexPrimitiva::Quantity,    // N — graded measurement
            Self::ObservationType => LexPrimitiva::Existence, // ∃ — what was observed to exist
            Self::Modifier => LexPrimitiva::Comparison,  // κ — comparison against baseline
            Self::Action => LexPrimitiva::Irreversibility, // ∝ — interventions are one-way
            Self::Outcome => LexPrimitiva::State,        // ς — state transitions
        }
    }

    /// All 8 element classes.
    pub fn all() -> &'static [ElementClass] {
        &[
            Self::OrganSystem,
            Self::Causality,
            Self::Temporality,
            Self::Severity,
            Self::ObservationType,
            Self::Modifier,
            Self::Action,
            Self::Outcome,
        ]
    }
}

// =============================================================================
// SEMANTIC ATOM
// =============================================================================
// The fundamental unit of meaning. Not a word — a crystallized concept
// with a fixed numerical identity, a position in the hierarchy lattice,
// and a spectral fingerprint for equivalence testing.

/// Dimensionality of the semantic vector space.
/// 256 is a reasonable starting point — enough to capture nuance,
/// small enough to compute efficiently.
pub const VECTOR_DIM: usize = 256;

/// A canonical semantic atom — the irreducible unit of meaning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atom {
    /// Unique, immutable identity. Once assigned, never changes.
    pub id: NexId,

    /// The canonical label (human-readable, but NOT the definition).
    /// "cardiac" is a label. The vector is the definition.
    pub label: String,

    /// Element class — which group in the periodic table.
    pub class: ElementClass,

    /// The semantic vector — this IS the atom's meaning.
    /// Two atoms with identical vectors are identical meanings.
    pub vector: Vec<OrderedFloat<f64>>,

    /// Semantic volatility: how much this atom's meaning shifts
    /// across contexts. Low = stable (like "drug"). High = context-
    /// dependent (like "unexpected"). Range: 0.0 - 1.0.
    ///
    /// Chemistry analogue: boiling point. Low volatility = high
    /// boiling point = separates last in distillation.
    pub volatility: OrderedFloat<f64>,

    /// Version — atoms can be updated, but every version is preserved.
    /// This enables spectral drift detection over time.
    pub version: u32,

    /// MedDRA code mapping (if derived from MedDRA seed data).
    pub meddra_code: Option<String>,
}

impl Atom {
    /// Create a new atom with a zero vector (for demonstration).
    /// In production, vectors would come from distributional analysis
    /// of regulatory corpora.
    pub fn new(label: &str, class: ElementClass, volatility: f64) -> Self {
        Self {
            id: NexId::v4(),
            label: label.to_string(),
            class,
            vector: vec![OrderedFloat(0.0); VECTOR_DIM],
            volatility: OrderedFloat(volatility),
            version: 1,
            meddra_code: None,
        }
    }

    /// Semantic mass — total information content of this atom.
    /// Conservation law: composition must preserve total semantic mass.
    ///
    /// Chemistry analogue: molecular weight.
    pub fn semantic_mass(&self) -> f64 {
        self.vector
            .iter()
            .map(|v| v.into_inner() * v.into_inner())
            .sum::<f64>()
            .sqrt()
    }
}

// =============================================================================
// COMPOUND EXPRESSION
// =============================================================================
// Atoms compose into compound expressions via composition rules.
// "Serious unexpected cardiac adverse reaction" is a compound of 5 atoms.
// The composition IS the proof — deterministic, auditable, reversible.

/// How two atoms relate in a composition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Relation {
    /// A modifies B: "serious" + "event" -> serious modifies event
    Modifies,
    /// A causes B: "drug" + "reaction" -> drug causes reaction
    Causes,
    /// A precedes B temporally: "exposure" then "onset"
    Precedes,
    /// A and B co-occur: "nausea" and "vomiting"
    CoOccurs,
    /// A is a subtype of B: "MI" is-a "cardiac event"
    IsA,
    /// A negates B: "no" + "improvement" -> absence of improvement
    Negates,
}

/// A bond between two atoms in a compound expression.
/// Chemistry analogue: chemical bond (covalent, ionic, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bond {
    pub source: NexId,
    pub target: NexId,
    pub relation: Relation,
    /// Bond strength: how tightly coupled are these atoms?
    /// Strong bond = hard to decompose. Weak bond = easily separated.
    pub strength: OrderedFloat<f64>,
}

/// A compound semantic expression — multiple atoms bonded together.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compound {
    pub id: NexId,
    pub atoms: Vec<Atom>,
    pub bonds: Vec<Bond>,
    /// The compound's own vector — computed from constituent atoms
    /// and their bonding pattern. NOT just the sum of atom vectors.
    pub vector: Vec<OrderedFloat<f64>>,
}

impl Compound {
    /// Semantic mass of the compound — must equal sum of constituent
    /// atom masses (conservation law / stoichiometry).
    pub fn semantic_mass(&self) -> f64 {
        self.atoms.iter().map(|a| a.semantic_mass()).sum()
    }

    /// Verify stoichiometric conservation: does the compound's mass
    /// equal the sum of its parts?
    pub fn verify_conservation(&self) -> StoichiometryResult {
        let component_mass: f64 = self.atoms.iter().map(|a| a.semantic_mass()).sum();
        let compound_mass = self
            .vector
            .iter()
            .map(|v| v.into_inner() * v.into_inner())
            .sum::<f64>()
            .sqrt();

        let delta = (compound_mass - component_mass).abs();
        let tolerance = 1e-6;

        if delta < tolerance {
            StoichiometryResult::Conserved { delta }
        } else if compound_mass > component_mass {
            StoichiometryResult::MeaningCreated {
                excess: delta,
                warning: "Composition added meaning not present in inputs".into(),
            }
        } else {
            StoichiometryResult::MeaningDestroyed {
                deficit: delta,
                warning: "Composition lost meaning from inputs".into(),
            }
        }
    }
}

/// Result of a stoichiometric conservation check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoichiometryResult {
    /// Mass conserved within tolerance — composition is valid.
    Conserved { delta: f64 },
    /// More mass out than in — meaning was created from nothing.
    MeaningCreated { excess: f64, warning: String },
    /// Less mass out than in — meaning was lost.
    MeaningDestroyed { deficit: f64, warning: String },
}
