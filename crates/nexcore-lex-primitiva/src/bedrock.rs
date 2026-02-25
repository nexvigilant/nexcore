//! # Bedrock Atoms
//!
//! The intermediate layer between Lex Primitiva and mathematical constants.
//! Each primitive decomposes into 5 bedrock atoms (80 total).
//!
//! ## Tier: T1-Universal to T2-P
// complexity_override: approved by user for enum definition

use crate::constants::MathConstant;
use crate::primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};

/// A bedrock atom - the sub-primitive building block.
///
/// Each of the 16 Lex Primitiva decomposes into exactly 5 bedrock atoms,
/// yielding 80 total atoms that form the irreducible vocabulary of computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BedrockAtom {
    // ═══════════════════════════════════════════════════════════════════
    // Quantity (N) — numeric measurement and aggregation
    // ═══════════════════════════════════════════════════════════════════
    /// Size or scale of a measured value.
    Magnitude,
    /// Reference standard for measurement (meters, seconds, etc.).
    Unit,
    /// Discrete enumeration of elements.
    Count,
    /// Function mapping observations to numeric values.
    MeasureFunction,
    /// Combining multiple measurements (sum, mean, max).
    Aggregation,

    // ═══════════════════════════════════════════════════════════════════
    // Causality (→) — directed influence and intervention
    // ═══════════════════════════════════════════════════════════════════
    /// Asymmetric A→B relationship (cause precedes effect).
    DirectedRelation,
    /// External manipulation that changes system state.
    Intervention,
    /// The process by which cause produces effect.
    Mechanism,
    /// Cause must occur before effect in time.
    TemporalPrecedence,
    /// "What would have happened if X had not occurred?"
    Counterfactual,

    // ═══════════════════════════════════════════════════════════════════
    // Comparison (κ) — equivalence, ordering, and decision
    // ═══════════════════════════════════════════════════════════════════
    /// Binary relation for comparing two values.
    Comparator,
    /// Reflexive, symmetric, transitive equality relation.
    Equivalence,
    /// Total or partial order (<, ≤, >, ≥).
    Ordering,
    /// Distance function satisfying triangle inequality.
    Metric,
    /// Threshold-based selection criterion.
    DecisionRule,

    // ═══════════════════════════════════════════════════════════════════
    // Sequence (σ) — ordered collections and iteration
    // ═══════════════════════════════════════════════════════════════════
    /// Well-ordering on sequence elements.
    OrderRelation,
    /// Position-based element access (a[i]).
    Indexing,
    /// Navigation between adjacent elements.
    SuccessorPredecessor,
    /// Joining sequences end-to-end.
    Concatenation,
    /// Distinguished starting element (empty sequence, zero index).
    BaseElement,

    // ═══════════════════════════════════════════════════════════════════
    // Mapping (μ) — functions and transformations
    // ═══════════════════════════════════════════════════════════════════
    /// Set of valid input values.
    Domain,
    /// Set of possible output values.
    Codomain,
    /// The x ↦ f(x) correspondence.
    AssignmentRule,
    /// Chaining functions: (g ∘ f)(x) = g(f(x)).
    Composition,
    /// The do-nothing transformation: id(x) = x.
    Identity,

    // ═══════════════════════════════════════════════════════════════════
    // State (ς) — mutable storage and time evolution
    // ═══════════════════════════════════════════════════════════════════
    /// Current content of a state variable.
    Value,
    /// Discrete moment in system evolution.
    TimeStep,
    /// Transition function s' = f(s, input).
    UpdateRule,
    /// Memory location holding state.
    Storage,
    /// Starting value before any updates (s₀).
    InitialCondition,

    // ═══════════════════════════════════════════════════════════════════
    // Recursion (ρ) — self-reference and induction
    // ═══════════════════════════════════════════════════════════════════
    /// Non-recursive termination condition.
    BaseCase,
    /// Rule expressing f(n) in terms of f(n-1).
    RecursiveRule,
    /// Structure referring to itself (φ in φ = 1 + 1/φ).
    SelfReference,
    /// Decreasing measure proving termination (ω-chain).
    TerminationMeasure,
    /// Data type defined by its own constructors.
    InductiveStructure,

    // ═══════════════════════════════════════════════════════════════════
    // Void (∅) — emptiness and absence
    // ═══════════════════════════════════════════════════════════════════
    /// Set with no elements: {}.
    EmptySet,
    /// Type with no inhabitants (Rust's `!`).
    UninhabitedType,
    /// Proposition that cannot be proven true.
    FalseProposition,
    /// Category-theoretic initial object (unique morphism to all).
    InitialObject,
    /// Explicit representation of "nothing here" (None, null).
    AbsenceMarker,

    // ═══════════════════════════════════════════════════════════════════
    // Boundary (∂) — constraints and feasibility
    // ═══════════════════════════════════════════════════════════════════
    /// Boolean function defining valid region.
    ConstraintPredicate,
    /// Set of values satisfying all constraints.
    FeasibleSet,
    /// Inequality or limit condition (x ≤ max).
    LimitInequality,
    /// Runtime assertion preventing invalid transitions.
    GuardCondition,
    /// Signal emitted when constraint is breached.
    ViolationSignal,

    // ═══════════════════════════════════════════════════════════════════
    // Frequency (f) — periodicity and sampling
    // ═══════════════════════════════════════════════════════════════════
    /// Repeating pattern returning to initial state.
    Cycle,
    /// Duration of one complete cycle (T = 1/f).
    Period,
    /// Events per unit time (frequency, throughput).
    Rate,
    /// Position within a cycle (0 to 2π).
    Phase,
    /// Discrete observation interval (Nyquist).
    SamplingInterval,

    // ═══════════════════════════════════════════════════════════════════
    // Existence (∃) — witnesses and instantiation
    // ═══════════════════════════════════════════════════════════════════
    /// Concrete example proving ∃x.P(x).
    Witness,
    /// Creating a specific instance from a type.
    Instantiation,
    /// Property that at least one element exists.
    NonEmptiness,
    /// Proof by explicit construction (not contradiction).
    ConstructiveProof,
    /// Lexical region where binding is valid.
    Scope,

    // ═══════════════════════════════════════════════════════════════════
    // Persistence (π) — durability and recovery
    // ═══════════════════════════════════════════════════════════════════
    /// Non-volatile storage surviving restarts.
    StableStorage,
    /// Guarantee that committed data survives failures.
    Durability,
    /// Point-in-time capture of system state.
    Snapshot,
    /// Append-only sequential record (WAL, event log).
    LogAppend,
    /// Restoring state from persistent records.
    Recovery,

    // ═══════════════════════════════════════════════════════════════════
    // Location (λ) — addressing and distance
    // ═══════════════════════════════════════════════════════════════════
    /// Pointer or handle to a storage location.
    AddressReference,
    /// Position in a coordinate system (x, y, z).
    Coordinate,
    /// ε-ball around a point (topological locality).
    Neighborhood,
    /// Separation between two locations.
    Distance,
    /// Sequence of steps connecting two locations.
    Path,

    // ═══════════════════════════════════════════════════════════════════
    // Irreversibility (∝) — one-way operations
    // ═══════════════════════════════════════════════════════════════════
    /// Function without inverse (hash, projection).
    NonInvertibleMap,
    /// Transformation destroying recoverable data.
    InformationLoss,
    /// Second law: isolated systems increase disorder.
    EntropyIncrease,
    /// Resource consumed upon use (linear types, move semantics).
    CommitConsume,
    /// Operation that cannot be undone.
    NoUndo,

    // ═══════════════════════════════════════════════════════════════════
    // Sum (Σ) — discriminated unions and case analysis
    // ═══════════════════════════════════════════════════════════════════
    /// Set of possible variant types.
    VariantSet,
    /// Constructor embedding value into sum type.
    Injection,
    /// Discriminant identifying which variant is active.
    Tag,
    /// Match expression handling each variant.
    CaseAnalysis,
    /// Guarantee that variants don't overlap.
    Disjointness,

    // ═══════════════════════════════════════════════════════════════════
    // Product (×) — conjunctive combination and record types
    // ═══════════════════════════════════════════════════════════════════
    /// Named or positional field in a product type.
    ComponentField,
    /// Extracting one component from a product (π₁, π₂).
    Projection,
    /// Combining values into a product: `(a, b)`.
    Pairing,
    /// Field access pattern (`s.field`, `.0`).
    RecordAccess,
    /// Number of components in the product (tuple width).
    Arity,
}

// Per-primitive atom arrays (5 each)
const QUANTITY_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::Magnitude,
    BedrockAtom::Unit,
    BedrockAtom::Count,
    BedrockAtom::MeasureFunction,
    BedrockAtom::Aggregation,
];
const CAUSALITY_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::DirectedRelation,
    BedrockAtom::Intervention,
    BedrockAtom::Mechanism,
    BedrockAtom::TemporalPrecedence,
    BedrockAtom::Counterfactual,
];
const COMPARISON_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::Comparator,
    BedrockAtom::Equivalence,
    BedrockAtom::Ordering,
    BedrockAtom::Metric,
    BedrockAtom::DecisionRule,
];
const SEQUENCE_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::OrderRelation,
    BedrockAtom::Indexing,
    BedrockAtom::SuccessorPredecessor,
    BedrockAtom::Concatenation,
    BedrockAtom::BaseElement,
];
const MAPPING_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::Domain,
    BedrockAtom::Codomain,
    BedrockAtom::AssignmentRule,
    BedrockAtom::Composition,
    BedrockAtom::Identity,
];
const STATE_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::Value,
    BedrockAtom::TimeStep,
    BedrockAtom::UpdateRule,
    BedrockAtom::Storage,
    BedrockAtom::InitialCondition,
];
const RECURSION_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::BaseCase,
    BedrockAtom::RecursiveRule,
    BedrockAtom::SelfReference,
    BedrockAtom::TerminationMeasure,
    BedrockAtom::InductiveStructure,
];
const VOID_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::EmptySet,
    BedrockAtom::UninhabitedType,
    BedrockAtom::FalseProposition,
    BedrockAtom::InitialObject,
    BedrockAtom::AbsenceMarker,
];
const BOUNDARY_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::ConstraintPredicate,
    BedrockAtom::FeasibleSet,
    BedrockAtom::LimitInequality,
    BedrockAtom::GuardCondition,
    BedrockAtom::ViolationSignal,
];
const FREQUENCY_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::Cycle,
    BedrockAtom::Period,
    BedrockAtom::Rate,
    BedrockAtom::Phase,
    BedrockAtom::SamplingInterval,
];
const EXISTENCE_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::Witness,
    BedrockAtom::Instantiation,
    BedrockAtom::NonEmptiness,
    BedrockAtom::ConstructiveProof,
    BedrockAtom::Scope,
];
const PERSISTENCE_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::StableStorage,
    BedrockAtom::Durability,
    BedrockAtom::Snapshot,
    BedrockAtom::LogAppend,
    BedrockAtom::Recovery,
];
const LOCATION_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::AddressReference,
    BedrockAtom::Coordinate,
    BedrockAtom::Neighborhood,
    BedrockAtom::Distance,
    BedrockAtom::Path,
];
const IRREVERSIBILITY_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::NonInvertibleMap,
    BedrockAtom::InformationLoss,
    BedrockAtom::EntropyIncrease,
    BedrockAtom::CommitConsume,
    BedrockAtom::NoUndo,
];
const SUM_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::VariantSet,
    BedrockAtom::Injection,
    BedrockAtom::Tag,
    BedrockAtom::CaseAnalysis,
    BedrockAtom::Disjointness,
];
const PRODUCT_ATOMS: [BedrockAtom; 5] = [
    BedrockAtom::ComponentField,
    BedrockAtom::Projection,
    BedrockAtom::Pairing,
    BedrockAtom::RecordAccess,
    BedrockAtom::Arity,
];

impl BedrockAtom {
    /// Returns the parent primitive for this atom.
    #[must_use]
    pub const fn parent_primitive(&self) -> LexPrimitiva {
        use BedrockAtom::*;
        match self {
            Magnitude | Unit | Count | MeasureFunction | Aggregation => LexPrimitiva::Quantity,
            DirectedRelation | Intervention | Mechanism | TemporalPrecedence | Counterfactual => {
                LexPrimitiva::Causality
            }
            Comparator | Equivalence | Ordering | Metric | DecisionRule => LexPrimitiva::Comparison,
            OrderRelation | Indexing | SuccessorPredecessor | Concatenation | BaseElement => {
                LexPrimitiva::Sequence
            }
            Domain | Codomain | AssignmentRule | Composition | Identity => LexPrimitiva::Mapping,
            Value | TimeStep | UpdateRule | Storage | InitialCondition => LexPrimitiva::State,
            BaseCase | RecursiveRule | SelfReference | TerminationMeasure | InductiveStructure => {
                LexPrimitiva::Recursion
            }
            EmptySet | UninhabitedType | FalseProposition | InitialObject | AbsenceMarker => {
                LexPrimitiva::Void
            }
            ConstraintPredicate | FeasibleSet | LimitInequality | GuardCondition
            | ViolationSignal => LexPrimitiva::Boundary,
            Cycle | Period | Rate | Phase | SamplingInterval => LexPrimitiva::Frequency,
            Witness | Instantiation | NonEmptiness | ConstructiveProof | Scope => {
                LexPrimitiva::Existence
            }
            StableStorage | Durability | Snapshot | LogAppend | Recovery => {
                LexPrimitiva::Persistence
            }
            AddressReference | Coordinate | Neighborhood | Distance | Path => {
                LexPrimitiva::Location
            }
            NonInvertibleMap | InformationLoss | EntropyIncrease | CommitConsume | NoUndo => {
                LexPrimitiva::Irreversibility
            }
            VariantSet | Injection | Tag | CaseAnalysis | Disjointness => LexPrimitiva::Sum,
            ComponentField | Projection | Pairing | RecordAccess | Arity => LexPrimitiva::Product,
        }
    }

    /// Returns atoms for a specific primitive.
    #[must_use]
    pub const fn for_primitive(primitive: LexPrimitiva) -> &'static [Self; 5] {
        match primitive {
            LexPrimitiva::Quantity => &QUANTITY_ATOMS,
            LexPrimitiva::Causality => &CAUSALITY_ATOMS,
            LexPrimitiva::Comparison => &COMPARISON_ATOMS,
            LexPrimitiva::Sequence => &SEQUENCE_ATOMS,
            LexPrimitiva::Mapping => &MAPPING_ATOMS,
            LexPrimitiva::State => &STATE_ATOMS,
            LexPrimitiva::Recursion => &RECURSION_ATOMS,
            LexPrimitiva::Void => &VOID_ATOMS,
            LexPrimitiva::Boundary => &BOUNDARY_ATOMS,
            LexPrimitiva::Frequency => &FREQUENCY_ATOMS,
            LexPrimitiva::Existence => &EXISTENCE_ATOMS,
            LexPrimitiva::Persistence => &PERSISTENCE_ATOMS,
            LexPrimitiva::Location => &LOCATION_ATOMS,
            LexPrimitiva::Irreversibility => &IRREVERSIBILITY_ATOMS,
            LexPrimitiva::Sum => &SUM_ATOMS,
            LexPrimitiva::Product => &PRODUCT_ATOMS,
        }
    }

    /// Returns the primary mathematical constant this atom grounds to.
    #[must_use]
    pub const fn primary_constant(&self) -> MathConstant {
        use BedrockAtom::*;
        match self {
            Count | EmptySet | UninhabitedType | FalseProposition | InitialObject
            | AbsenceMarker | Comparator | Equivalence | Ordering | Metric | OrderRelation
            | Indexing | Concatenation | BaseElement | InitialCondition | BaseCase
            | TemporalPrecedence | Counterfactual | AddressReference | Coordinate | Distance
            | Path | NonInvertibleMap | NoUndo | Tag | Disjointness | MeasureFunction
            | Projection | Arity => MathConstant::ZERO,

            Magnitude | Unit | DirectedRelation | Intervention | Mechanism | DecisionRule
            | SuccessorPredecessor | Domain | Codomain | AssignmentRule | Composition
            | Identity | Value | TimeStep | UpdateRule | Storage | RecursiveRule
            | InductiveStructure | ConstraintPredicate | FeasibleSet | GuardCondition | Period
            | Rate | SamplingInterval | Witness | Instantiation | NonEmptiness
            | ConstructiveProof | Scope | Snapshot | Recovery | CommitConsume | VariantSet
            | Injection | CaseAnalysis | ComponentField | Pairing | RecordAccess => {
                MathConstant::ONE
            }

            Aggregation | ViolationSignal | StableStorage | Durability => MathConstant::INFINITY,
            SelfReference => MathConstant::PHI,
            TerminationMeasure => MathConstant::OMEGA,
            LimitInequality | Neighborhood => MathConstant::EPSILON,
            Cycle | Phase => MathConstant::PI,
            LogAppend | InformationLoss => MathConstant::LN_2,
            EntropyIncrease => MathConstant::K_BOLTZMANN,
        }
    }

    /// Returns the name of this atom.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        use BedrockAtom::*;
        match self {
            Magnitude => "Magnitude",
            Unit => "Unit",
            Count => "Count",
            MeasureFunction => "Measure Function",
            Aggregation => "Aggregation",
            DirectedRelation => "Directed Relation",
            Intervention => "Intervention",
            Mechanism => "Mechanism",
            TemporalPrecedence => "Temporal Precedence",
            Counterfactual => "Counterfactual",
            Comparator => "Comparator",
            Equivalence => "Equivalence",
            Ordering => "Ordering",
            Metric => "Metric",
            DecisionRule => "Decision Rule",
            OrderRelation => "Order Relation",
            Indexing => "Indexing",
            SuccessorPredecessor => "Successor/Predecessor",
            Concatenation => "Concatenation",
            BaseElement => "Base Element",
            Domain => "Domain",
            Codomain => "Codomain",
            AssignmentRule => "Assignment Rule",
            Composition => "Composition",
            Identity => "Identity",
            Value => "Value",
            TimeStep => "Time Step",
            UpdateRule => "Update Rule",
            Storage => "Storage",
            InitialCondition => "Initial Condition",
            BaseCase => "Base Case",
            RecursiveRule => "Recursive Rule",
            SelfReference => "Self-Reference",
            TerminationMeasure => "Termination Measure",
            InductiveStructure => "Inductive Structure",
            EmptySet => "Empty Set",
            UninhabitedType => "Uninhabited Type",
            FalseProposition => "False Proposition",
            InitialObject => "Initial Object",
            AbsenceMarker => "Absence Marker",
            ConstraintPredicate => "Constraint Predicate",
            FeasibleSet => "Feasible Set",
            LimitInequality => "Limit/Inequality",
            GuardCondition => "Guard Condition",
            ViolationSignal => "Violation Signal",
            Cycle => "Cycle",
            Period => "Period",
            Rate => "Rate",
            Phase => "Phase",
            SamplingInterval => "Sampling Interval",
            Witness => "Witness",
            Instantiation => "Instantiation",
            NonEmptiness => "Non-emptiness",
            ConstructiveProof => "Constructive Proof",
            Scope => "Scope",
            StableStorage => "Stable Storage",
            Durability => "Durability",
            Snapshot => "Snapshot",
            LogAppend => "Log/Append",
            Recovery => "Recovery",
            AddressReference => "Address/Reference",
            Coordinate => "Coordinate",
            Neighborhood => "Neighborhood",
            Distance => "Distance",
            Path => "Path",
            NonInvertibleMap => "Non-invertible Map",
            InformationLoss => "Information Loss",
            EntropyIncrease => "Entropy Increase",
            CommitConsume => "Commit/Consume",
            NoUndo => "No Undo",
            VariantSet => "Variant Set",
            Injection => "Injection",
            Tag => "Tag",
            CaseAnalysis => "Case Analysis",
            Disjointness => "Disjointness",
            ComponentField => "Component Field",
            Projection => "Projection",
            Pairing => "Pairing",
            RecordAccess => "Record Access",
            Arity => "Arity",
        }
    }
}

impl std::fmt::Display for BedrockAtom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atoms_per_primitive() {
        for primitive in LexPrimitiva::all() {
            let atoms = BedrockAtom::for_primitive(primitive);
            assert_eq!(atoms.len(), 5);
        }
    }

    #[test]
    fn test_parent_primitive_consistency() {
        for primitive in LexPrimitiva::all() {
            for atom in BedrockAtom::for_primitive(primitive) {
                assert_eq!(atom.parent_primitive(), primitive);
            }
        }
    }

    #[test]
    fn test_void_atoms_ground_to_zero() {
        for atom in BedrockAtom::for_primitive(LexPrimitiva::Void) {
            assert_eq!(atom.primary_constant().symbol, "0");
        }
    }
}
