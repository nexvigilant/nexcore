//! # Grounding Implementations
//!
//! `GroundsTo` trait implementations for nexcore-vigilance types.
//! This connects the symbolic foundation to concrete domain types.

use super::{GroundsTo, LexPrimitiva, PrimitiveComposition};
use nexcore_constants::{Confidence, Measured};
use nexcore_lex_primitiva::state_mode::StateMode;

// ============================================================================
// Chemistry Primitives Grounding
// ============================================================================

use nexcore_primitives::chemistry::{
    AggregationPipeline, BufferSystem, DecayKinetics, EquilibriumSystem, Favorability,
    FeasibilityAssessment, RateLaw, SaturationKinetics, SignalDetector, ThresholdGate,
};

impl GroundsTo for ThresholdGate {
    fn primitive_composition() -> PrimitiveComposition {
        // Arrhenius: threshold gating = Comparison + Boundary + Quantity
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // threshold check
            LexPrimitiva::Boundary,   // activation barrier
            LexPrimitiva::Quantity,   // numeric threshold value
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.92)
    }
}

impl GroundsTo for SaturationKinetics {
    fn primitive_composition() -> PrimitiveComposition {
        // Michaelis-Menten: saturation = Boundary + Quantity + Mapping + State
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // capacity limit (Vmax)
            LexPrimitiva::Quantity, // substrate concentration
            LexPrimitiva::Mapping,  // input → output rate
            LexPrimitiva::State,    // enzyme-substrate complex
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.88)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for FeasibilityAssessment {
    fn primitive_composition() -> PrimitiveComposition {
        // Gibbs: feasibility = Comparison + State + Quantity
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // favorable vs unfavorable
            LexPrimitiva::State,      // energy state
            LexPrimitiva::Quantity,   // ΔG value
            LexPrimitiva::Causality,  // spontaneous reaction
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

impl GroundsTo for Favorability {
    fn primitive_composition() -> PrimitiveComposition {
        // Favorability classification = Comparison + Sum
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // threshold comparison
            LexPrimitiva::Sum,        // enum of states
        ])
        .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

impl GroundsTo for RateLaw {
    fn primitive_composition() -> PrimitiveComposition {
        // Rate Law: dependency = Causality + Frequency + Quantity + Mapping
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // cause → effect
            LexPrimitiva::Frequency, // rate
            LexPrimitiva::Quantity,  // order, coefficients
            LexPrimitiva::Mapping,   // concentration → rate
        ])
        .with_dominant(LexPrimitiva::Causality, 0.82)
    }
}

impl GroundsTo for BufferSystem {
    fn primitive_composition() -> PrimitiveComposition {
        // Henderson-Hasselbalch: buffer = State + Boundary + Quantity
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // pH state
            LexPrimitiva::Boundary, // resistance to change
            LexPrimitiva::Quantity, // pKa, ratio
        ])
        .with_dominant(LexPrimitiva::State, 0.78)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for SignalDetector {
    fn primitive_composition() -> PrimitiveComposition {
        // Beer-Lambert: signal_intensity = Quantity + Mapping + Comparison
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // absorbance, concentration
            LexPrimitiva::Mapping,    // linear relationship
            LexPrimitiva::Comparison, // detection limit
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.75)
    }
}

impl GroundsTo for DecayKinetics {
    fn primitive_composition() -> PrimitiveComposition {
        // Half-Life: decay = Frequency + Quantity + Irreversibility + Sequence
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,       // decay rate
            LexPrimitiva::Quantity,        // amount remaining
            LexPrimitiva::Irreversibility, // one-way decay
            LexPrimitiva::Sequence,        // temporal progression
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.90)
    }
}

impl GroundsTo for EquilibriumSystem {
    fn primitive_composition() -> PrimitiveComposition {
        // Equilibrium: steady state = State + Boundary + Frequency
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // equilibrium state
            LexPrimitiva::Boundary,  // steady-state limits
            LexPrimitiva::Frequency, // forward/reverse rates
        ])
        .with_dominant(LexPrimitiva::State, 0.72)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for AggregationPipeline {
    fn primitive_composition() -> PrimitiveComposition {
        // Multi-stage causal pipeline: Beer-Lambert → Hill → Arrhenius
        // Σ Sum: weighted feature summation (Beer-Lambert absorptivity)
        // ρ Recursion: cooperative amplification (Hill binding feedback)
        // ∂ Boundary: threshold gate (Arrhenius activation energy)
        // → Causality: directed pipeline — features flow through stages causally
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Beer-Lambert weighted aggregation
            LexPrimitiva::Recursion, // Hill cooperative binding loops
            LexPrimitiva::Boundary,  // Arrhenius activation threshold
            LexPrimitiva::Causality, // directed pipeline flow
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

// ============================================================================
// PV Signal Detection Grounding
// ============================================================================

use crate::pv::signals::core::types::{ContingencyTable, SignalResult};

impl GroundsTo for ContingencyTable {
    fn primitive_composition() -> PrimitiveComposition {
        // 2×2 cross-tabulation: (exposure, event) → count
        // Structural grounding: μ (core mapping) + ∂ (category boundaries) +
        // κ (observed vs expected) + N (cell counts) + Σ (marginal totals)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // (drug±, event±) → count
            LexPrimitiva::Boundary,   // binary category partitions
            LexPrimitiva::Comparison, // observed vs expected comparison
            LexPrimitiva::Quantity,   // cell counts a, b, c, d
            LexPrimitiva::Sum,        // marginal totals, grand total N
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

impl GroundsTo for SignalResult {
    fn primitive_composition() -> PrimitiveComposition {
        // Signal detection result: "Does a disproportionate association EXIST?"
        // Core semantic: Existence determination via Comparison against Boundary
        // - ∃ (Existence): detected/not detected — the fundamental output
        // - κ (Comparison): observed vs expected (PRR, ROR, IC ratios)
        // - N (Quantity): numeric metric values
        // - ∂ (Boundary): confidence intervals and thresholds
        // - Σ (Sum): method selection coproduct (PRR|ROR|IC|EBGM|χ²)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // signal exists or not — core semantic
            LexPrimitiva::Comparison, // threshold comparison (O/E)
            LexPrimitiva::Quantity,   // PRR, ROR, IC numeric values
            LexPrimitiva::Boundary,   // confidence intervals
            LexPrimitiva::Sum,        // method selection enum
        ])
        .with_dominant(LexPrimitiva::Existence, 0.85)
    }
}

// ============================================================================
// Domain Discovery Grounding
// ============================================================================

use crate::domain_discovery::primitives::{Primitive, PrimitiveTier};

impl GroundsTo for Primitive {
    fn primitive_composition() -> PrimitiveComposition {
        // Domain primitive = State + Existence + Recursion + Location
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // definition, tier
            LexPrimitiva::Existence, // id, name
            LexPrimitiva::Recursion, // depends_on (self-referential)
            LexPrimitiva::Location,  // domain_coverage
        ])
        .with_dominant(LexPrimitiva::Existence, 0.90)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

impl GroundsTo for PrimitiveTier {
    fn primitive_composition() -> PrimitiveComposition {
        // Tier classification = Sum + Comparison + Quantity
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // T1/T2/T3 enum
            LexPrimitiva::Comparison, // coverage threshold
            LexPrimitiva::Quantity,   // min_coverage
        ])
        .with_dominant(LexPrimitiva::Sum, 0.95)
    }
}

// ============================================================================
// ToV Grounding
// ============================================================================

use crate::tov::{HarmType, SafetyMargin};

impl GroundsTo for HarmType {
    fn primitive_composition() -> PrimitiveComposition {
        // Harm type = Sum + Causality + Comparison
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // 8 harm types (A-H)
            LexPrimitiva::Causality,  // cause of harm
            LexPrimitiva::Comparison, // severity classification
        ])
        .with_dominant(LexPrimitiva::Sum, 0.95)
    }
}

impl GroundsTo for SafetyMargin {
    fn primitive_composition() -> PrimitiveComposition {
        // Safety margin = Quantity + Boundary + Comparison
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // d(s) distance
            LexPrimitiva::Boundary,   // safety boundary
            LexPrimitiva::Comparison, // safe vs unsafe
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.88)
    }
}

// ============================================================================
// Frontier T2-P Grounding (completing T1 surface coverage)
// ============================================================================

use crate::primitives::frontier::{
    AbsenceMarker, AuditTrail, ConsumptionMark, EntityStatus, Pipeline, RecordStructure,
    RecursionBound, ResourcePath,
};

impl GroundsTo for AuditTrail {
    fn primitive_composition() -> PrimitiveComposition {
        // π (Persistence): durable record = Persistence + Sequence + Existence
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // continuity across time
            LexPrimitiva::Sequence,    // ordered entries
            LexPrimitiva::Existence,   // record creation
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.92)
    }
}

impl GroundsTo for AbsenceMarker {
    fn primitive_composition() -> PrimitiveComposition {
        // ∅ (Void): meaningful absence = Void + Comparison + Boundary
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,       // explicit nothing
            LexPrimitiva::Comparison, // detected vs not detected
            LexPrimitiva::Boundary,   // detection threshold
        ])
        .with_dominant(LexPrimitiva::Void, 0.95)
    }
}

impl GroundsTo for Pipeline {
    fn primitive_composition() -> PrimitiveComposition {
        // σ (Sequence): ordered phases = Sequence + Quantity + Boundary
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // ordered succession
            LexPrimitiva::Quantity, // phase count/index
            LexPrimitiva::Boundary, // phase boundaries
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.92)
    }
}

impl GroundsTo for ConsumptionMark {
    fn primitive_composition() -> PrimitiveComposition {
        // ∝ (Irreversibility): one-way consumption = Irreversibility + Quantity + Causality
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility, // cannot unconsume
            LexPrimitiva::Quantity,        // consumed amount
            LexPrimitiva::Causality,       // consumption → depletion
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.90)
    }
}

impl GroundsTo for EntityStatus {
    fn primitive_composition() -> PrimitiveComposition {
        // ∃ (Existence): lifecycle state = Existence + Sum + Comparison
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // being/not-being
            LexPrimitiva::Sum,        // enum of states
            LexPrimitiva::Comparison, // status classification
        ])
        .with_dominant(LexPrimitiva::Existence, 0.92)
    }
}

impl GroundsTo for ResourcePath {
    fn primitive_composition() -> PrimitiveComposition {
        // λ (Location): positional addressing = Location + Sequence + Existence
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,  // positional context
            LexPrimitiva::Sequence,  // path segments in order
            LexPrimitiva::Existence, // resource exists at path
        ])
        .with_dominant(LexPrimitiva::Location, 0.90)
    }
}

impl GroundsTo for RecursionBound {
    fn primitive_composition() -> PrimitiveComposition {
        // ρ (Recursion): depth limit = Recursion + Quantity + Boundary
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // self-referential traversal
            LexPrimitiva::Quantity,  // depth count
            LexPrimitiva::Boundary,  // max depth limit
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.88)
    }
}

impl GroundsTo for RecordStructure {
    fn primitive_composition() -> PrimitiveComposition {
        // × (Product): conjunctive field combination = Product + Existence + Quantity
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,   // conjunctive combination (AND)
            LexPrimitiva::Existence, // fields must exist
            LexPrimitiva::Quantity,  // field count / arity
        ])
        .with_dominant(LexPrimitiva::Product, 0.90)
    }
}

// ============================================================================
// Emoji Domain Grounding (20 types, each concept used once)
// ============================================================================

use crate::primitives::emoji::{
    Ambiguity, Category, Codepoint, CulturalInterpretation, EmojiComposition, EmojiEvolution,
    EmojiVersion, Encoding, FallbackDisplay, Glyph, InputMethod, ProposalProcess, Reaction,
    Rendering, Sentiment, Shortcode, SkinToneModifier, UnicodeStandard, UsageFrequency,
    ZwjSequence,
};

// 1. Codepoint: ∃ Existence + N Quantity
impl GroundsTo for Codepoint {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // a codepoint IS an existence proof
            LexPrimitiva::Quantity,  // expressed as a number (U+XXXX)
        ])
        .with_dominant(LexPrimitiva::Existence, 0.95)
    }
}

// 2. Glyph: μ Mapping + ∃ Existence
impl GroundsTo for Glyph {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // codepoint → visual representation
            LexPrimitiva::Existence, // the visual form exists
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.93)
    }
}

// 3. UnicodeStandard: ∂ Boundary + κ Comparison + σ Sequence
impl GroundsTo for UnicodeStandard {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // defines valid/invalid codepoints
            LexPrimitiva::Comparison, // character validation rules
            LexPrimitiva::Sequence,   // versioned releases
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.92)
    }
}

// 4. SkinToneModifier: ς State + × Product
impl GroundsTo for SkinToneModifier {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,   // variant selector (Fitzpatrick level)
            LexPrimitiva::Product, // composed with base emoji
        ])
        .with_dominant(LexPrimitiva::State, 0.91)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

// 5. Category: κ Comparison + Σ Sum
impl GroundsTo for Category {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // classification by group membership
            LexPrimitiva::Sum,        // mutually exclusive enum
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

// 6. ZwjSequence: σ Sequence + × Product + ∃ Existence
impl GroundsTo for ZwjSequence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // ordered codepoint chain
            LexPrimitiva::Product,   // components fuse into new meaning
            LexPrimitiva::Existence, // compound may or may not be recognized
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.88)
    }
}

// 7. Shortcode: μ Mapping + ∃ Existence
impl GroundsTo for Shortcode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // text alias → codepoint
            LexPrimitiva::Existence, // target codepoint must exist
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.87)
    }
}

// 8. Encoding: μ Mapping + N Quantity + σ Sequence
impl GroundsTo for Encoding {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // codepoint → byte sequence
            LexPrimitiva::Quantity, // byte width
            LexPrimitiva::Sequence, // ordered bytes
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.86)
    }
}

// 9. Rendering: μ Mapping + ∂ Boundary + λ Location
impl GroundsTo for Rendering {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // codepoint → platform-specific pixels
            LexPrimitiva::Boundary, // platform rendering boundary
            LexPrimitiva::Location, // platform position in ecosystem
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.84)
    }
}

// 10. Reaction: → Causality + ν Frequency
impl GroundsTo for Reaction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // causal response to content
            LexPrimitiva::Frequency, // accumulated count
        ])
        .with_dominant(LexPrimitiva::Causality, 0.82)
    }
}

// 11. Sentiment: μ Mapping + κ Comparison
impl GroundsTo for Sentiment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // visual → emotional classification
            LexPrimitiva::Comparison, // valence comparison
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// 12. InputMethod: σ Sequence + ∂ Boundary
impl GroundsTo for InputMethod {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sequence of user actions
            LexPrimitiva::Boundary, // constrains accessible emoji
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.78)
    }
}

// 13. EmojiVersion: σ Sequence + π Persistence + N Quantity
impl GroundsTo for EmojiVersion {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,    // ordered version succession
            LexPrimitiva::Persistence, // immutable once published
            LexPrimitiva::Quantity,    // new emoji count
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.75)
    }
}

// 14. CulturalInterpretation: μ Mapping + λ Location + κ Comparison
impl GroundsTo for CulturalInterpretation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // glyph → region-specific meaning
            LexPrimitiva::Location,   // region-dependent
            LexPrimitiva::Comparison, // divergence from default
        ])
        .with_dominant(LexPrimitiva::Location, 0.62)
    }
}

// 15. UsageFrequency: ν Frequency + N Quantity
impl GroundsTo for UsageFrequency {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // rate of occurrence
            LexPrimitiva::Quantity,  // rank and measured rate
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.72)
    }
}

// 16. FallbackDisplay: ∅ Void + μ Mapping
impl GroundsTo for FallbackDisplay {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,    // handling of absence
            LexPrimitiva::Mapping, // absence maps to visible fallback
        ])
        .with_dominant(LexPrimitiva::Void, 0.85)
    }
}

// 17. Ambiguity: κ Comparison + ∅ Void + μ Mapping
impl GroundsTo for Ambiguity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // comparing interpretations
            LexPrimitiva::Void,       // void of consensus
            LexPrimitiva::Mapping,    // multiple meaning-mappings
        ])
        .with_dominant(LexPrimitiva::Void, 0.60)
    }
}

// 18. EmojiComposition: × Product + σ Sequence + ρ Recursion
impl GroundsTo for EmojiComposition {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,   // conjunctive combination of parts
            LexPrimitiva::Sequence,  // ordered modifier application
            LexPrimitiva::Recursion, // compositions can nest
        ])
        .with_dominant(LexPrimitiva::Product, 0.83)
    }
}

// 19. ProposalProcess: σ Sequence + ∂ Boundary + → Causality
impl GroundsTo for ProposalProcess {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // ordered governance phases
            LexPrimitiva::Boundary,  // gate-keeping criteria
            LexPrimitiva::Causality, // phase outcome → next transition
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.70)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

// 20. EmojiEvolution: ∝ Irreversibility + σ Sequence + Σ Sum
impl GroundsTo for EmojiEvolution {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility, // cannot remove assigned codepoints
            LexPrimitiva::Sequence,        // version timeline
            LexPrimitiva::Sum,             // accumulated total across versions
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.68)
    }
}

// ============================================================================
// Bathroom Lock Grounding (ς State T2-P)
// ============================================================================

use nexcore_constants::bathroom_lock::Occupancy;

impl GroundsTo for Occupancy {
    fn primitive_composition() -> PrimitiveComposition {
        // ς (State): binary state machine = State + Boundary + Existence
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // Vacant ↔ Occupied FSM
            LexPrimitiva::Boundary,  // mutually-exclusive access
            LexPrimitiva::Existence, // lock file existence = signal
        ])
        .with_dominant(LexPrimitiva::State, 0.92)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

// ============================================================================
// Measured<T> and Confidence Grounding
// ============================================================================

impl GroundsTo for Confidence {
    fn primitive_composition() -> PrimitiveComposition {
        // Confidence = Quantity + Boundary
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // 0.0-1.0 value
            LexPrimitiva::Boundary, // clamped range
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.95)
    }
}

impl<T> GroundsTo for Measured<T>
where
    T: GroundsTo,
{
    fn primitive_composition() -> PrimitiveComposition {
        // Measured<T> = T's composition + State (for uncertainty)
        let mut inner = T::primitive_composition();
        if !inner.primitives.contains(&LexPrimitiva::State) {
            inner.primitives.push(LexPrimitiva::State);
        }
        inner.confidence *= 0.95; // slight reduction for wrapper
        inner
    }
}

// ============================================================================
// Betting Domain Grounding
// ============================================================================

use nexcore_labs::betting::bdi::Bdi;
use nexcore_labs::betting::ecs::EcsScore;

impl GroundsTo for Bdi {
    fn primitive_composition() -> PrimitiveComposition {
        // BDI (Bayesian Disproportionality Index) = Quantity + Comparison + Boundary
        // Cross-domain transfer from PRR (Proportional Reporting Ratio)
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // numeric score value
            LexPrimitiva::Comparison, // threshold comparison (≥2.0)
            LexPrimitiva::Boundary,   // signal/no-signal boundary
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.92)
    }
}

impl GroundsTo for EcsScore {
    fn primitive_composition() -> PrimitiveComposition {
        // ECS (Expected Confidence Score) = U × R × T
        // Campion Signal Theory: Unexpectedness × Reliability × Temporal
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // composite score
            LexPrimitiva::Mapping,   // U×R×T transformation
            LexPrimitiva::Causality, // signal → action
            LexPrimitiva::Frequency, // temporal decay component
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.88)
    }
}

// ============================================================================
// Patient/Safety Primitives Grounding
// ============================================================================

use crate::primitives::recipient::{Recipient, Tracked, Vulnerable};
use crate::primitives::safety::{BoundaryBreach, Harm, Monitoring, SafetyBoundary};

impl GroundsTo for Recipient {
    fn primitive_composition() -> PrimitiveComposition {
        // → (Causality): arrow target = Causality + Existence
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // receives causal action
            LexPrimitiva::Existence, // entity that exists
        ])
        .with_dominant(LexPrimitiva::Causality, 0.94)
    }
}

impl GroundsTo for SafetyBoundary<f64> {
    fn primitive_composition() -> PrimitiveComposition {
        // κ (Comparison): boundary gate = Comparison + Boundary + Quantity
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // contains/violated check
            LexPrimitiva::Boundary,   // range limits
            LexPrimitiva::Quantity,   // numeric bounds
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.91)
    }
}

impl GroundsTo for Harm {
    fn primitive_composition() -> PrimitiveComposition {
        // ∂+∅ (Boundary + Void): harm state = Boundary + Void + Sum
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // safety boundary breach
            LexPrimitiva::Void,     // None variant (absence of harm)
            LexPrimitiva::Sum,      // enum variants
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.89)
    }
}

impl GroundsTo for Vulnerable {
    fn primitive_composition() -> PrimitiveComposition {
        // ∂+∃ (Boundary + Existence): susceptibility = Boundary + Existence + Quantity + Causality
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // weakened defenses
            LexPrimitiva::Existence, // existing recipient
            LexPrimitiva::Quantity,  // susceptibility score
            LexPrimitiva::Causality, // receives harm
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.88)
    }
}

impl GroundsTo for BoundaryBreach {
    fn primitive_composition() -> PrimitiveComposition {
        // ∂+κ+∅ (Boundary + Comparison + Void): breach = Boundary + Comparison + Void + Quantity
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // defined limits
            LexPrimitiva::Comparison, // actual vs boundary
            LexPrimitiva::Void,       // gap between expected and actual
            LexPrimitiva::Quantity,   // margin measurement
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.87)
    }
}

impl GroundsTo for Monitoring {
    fn primitive_composition() -> PrimitiveComposition {
        // σ+ρ (Sequence + Recursion): recurring checks = Sequence + Recursion + Quantity
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // ordered check history
            LexPrimitiva::Recursion, // recursive context building
            LexPrimitiva::Quantity,  // check/violation counts
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.90)
    }
}

impl GroundsTo for Tracked {
    fn primitive_composition() -> PrimitiveComposition {
        // Σ+σ (Sum + Sequence): longitudinal tracking = Sum + Sequence + Existence + Causality
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // active/closed state
            LexPrimitiva::Sequence,  // observation series
            LexPrimitiva::Existence, // tracked recipient
            LexPrimitiva::Causality, // observation events
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.88)
    }
}

// ============================================================================
// Cross-Domain Transfer Primitives Grounding (T2-P)
// Extracted from 2026-02-05 primitive validation audit
// ============================================================================

use nexcore_primitives::transfer::{
    Accumulator, Atomicity, Checkpoint, CircuitBreaker, CompareAndSwap, DecayFunction,
    EventClassifier, ExploreExploit, FeedbackLoop, Homeostasis, Idempotency, MessageBus,
    NegativeEvidence, PatternMatcher, RateLimiter, ResourceRatio, SchemaContract,
    SerializationGuard, SpecializedWorker, StagedValidation, ThreatSignature, ToctouWindow,
    TopologicalAddress,
};

impl GroundsTo for ThreatSignature {
    fn primitive_composition() -> PrimitiveComposition {
        // ∂ (Boundary) + μ (Mapping): pattern → boundary violation
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // threat crosses defense perimeter
            LexPrimitiva::Mapping,  // pattern → classification
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

impl GroundsTo for ResourceRatio {
    fn primitive_composition() -> PrimitiveComposition {
        // κ (Comparison) + N (Quantity): ratio of two quantities
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // available vs capacity
            LexPrimitiva::Quantity,   // numeric values
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.93)
    }
}

impl GroundsTo for PatternMatcher {
    fn primitive_composition() -> PrimitiveComposition {
        // μ (Mapping) + κ (Comparison): input → match decision
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // input → output
            LexPrimitiva::Comparison, // match vs no-match
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.92)
    }
}

impl GroundsTo for ExploreExploit {
    fn primitive_composition() -> PrimitiveComposition {
        // ρ (Recursion) + κ (Comparison): iterative choice
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,  // iterative refinement
            LexPrimitiva::Comparison, // explore vs exploit decision
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.88)
    }
}

impl GroundsTo for EventClassifier {
    fn primitive_composition() -> PrimitiveComposition {
        // μ (Mapping) + κ (Comparison) + ∂ (Boundary): value → class via thresholds
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // value → class
            LexPrimitiva::Comparison, // threshold comparisons
            LexPrimitiva::Boundary,   // class boundaries
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.91)
    }
}

impl GroundsTo for FeedbackLoop {
    fn primitive_composition() -> PrimitiveComposition {
        // ρ (Recursion) + κ (Comparison) + → (Causality): error → correction cycle
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,  // iterative correction
            LexPrimitiva::Comparison, // setpoint vs current
            LexPrimitiva::Causality,  // error causes correction
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.94)
    }
}

impl GroundsTo for SchemaContract {
    fn primitive_composition() -> PrimitiveComposition {
        // μ (Mapping) + π (Persistence) + ∂ (Boundary): versioned structure agreement
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,     // field definitions
            LexPrimitiva::Persistence, // versioned and stored
            LexPrimitiva::Boundary,    // required fields constraint
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

impl GroundsTo for MessageBus {
    fn primitive_composition() -> PrimitiveComposition {
        // σ (Sequence) + μ (Mapping) + → (Causality): ordered message dispatch
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // message ordering
            LexPrimitiva::Mapping,   // topic → subscribers
            LexPrimitiva::Causality, // publish causes delivery
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.91)
    }
}

impl GroundsTo for SpecializedWorker {
    fn primitive_composition() -> PrimitiveComposition {
        // μ (Mapping) + σ (Sequence): task specialization
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // input → output transform
            LexPrimitiva::Sequence, // task queue processing
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.89)
    }
}

impl GroundsTo for DecayFunction {
    fn primitive_composition() -> PrimitiveComposition {
        // → (Causality) + N (Quantity) + ∝ (Irreversibility): time-value decay
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,       // time causes value loss
            LexPrimitiva::Quantity,        // numeric decay curve
            LexPrimitiva::Irreversibility, // cannot un-decay
        ])
        .with_dominant(LexPrimitiva::Causality, 0.90)
    }
}

impl GroundsTo for Homeostasis {
    fn primitive_composition() -> PrimitiveComposition {
        // ρ (Recursion) + κ (Comparison) + ∂ (Boundary): self-correcting feedback with deadband
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,  // iterative correction cycles
            LexPrimitiva::Comparison, // setpoint vs current
            LexPrimitiva::Boundary,   // tolerance band
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.92)
    }
}

impl GroundsTo for StagedValidation {
    fn primitive_composition() -> PrimitiveComposition {
        // σ (Sequence) + ∂ (Boundary) + π (Persistence): multi-stage gating
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,    // stage ordering
            LexPrimitiva::Boundary,    // stage gates
            LexPrimitiva::Persistence, // evidence accumulation
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.91)
    }
}

impl GroundsTo for Atomicity {
    fn primitive_composition() -> PrimitiveComposition {
        // Σ (Sum) + ∝ (Irreversibility): all-or-nothing binary outcome
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,             // commit | rollback
            LexPrimitiva::Irreversibility, // cannot undo commit
        ])
        .with_dominant(LexPrimitiva::Sum, 0.93)
    }
}

impl GroundsTo for CompareAndSwap {
    fn primitive_composition() -> PrimitiveComposition {
        // κ (Comparison) + ∝ (Irreversibility): atomic conditional update
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,      // expected vs current
            LexPrimitiva::Irreversibility, // swap is one-shot
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.94)
    }
}

impl GroundsTo for ToctouWindow {
    fn primitive_composition() -> PrimitiveComposition {
        // σ (Sequence) + ∂ (Boundary) + ν (Frequency): staleness detection
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // check → use ordering
            LexPrimitiva::Boundary,  // max_gap threshold
            LexPrimitiva::Frequency, // time-based gap measurement
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.90)
    }
}

impl GroundsTo for SerializationGuard {
    fn primitive_composition() -> PrimitiveComposition {
        // σ (Sequence) + ∝ (Irreversibility) + π (Persistence): total ordering
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,        // monotonic order
            LexPrimitiva::Irreversibility, // committed ops cannot reorder
            LexPrimitiva::Persistence,     // durable sequence numbers
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.92)
    }
}

impl GroundsTo for RateLimiter {
    fn primitive_composition() -> PrimitiveComposition {
        // ν (Frequency) + ∂ (Boundary) + N (Quantity): throughput capping
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // events per time window
            LexPrimitiva::Boundary,  // max_events threshold
            LexPrimitiva::Quantity,  // count tracking
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.93)
    }
}

impl GroundsTo for CircuitBreaker {
    fn primitive_composition() -> PrimitiveComposition {
        // ς (State) + ∂ (Boundary) + κ (Comparison): 3-state resilience FSM
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // Closed/Open/HalfOpen
            LexPrimitiva::Boundary,   // failure threshold
            LexPrimitiva::Comparison, // count vs threshold
        ])
        .with_dominant(LexPrimitiva::State, 0.94)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for Idempotency {
    fn primitive_composition() -> PrimitiveComposition {
        // ∃ (Existence) + κ (Comparison): already-applied check
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // has operation been applied?
            LexPrimitiva::Comparison, // key match / dedup check
        ])
        .with_dominant(LexPrimitiva::Existence, 0.92)
    }
}

impl GroundsTo for NegativeEvidence {
    fn primitive_composition() -> PrimitiveComposition {
        // ∅ (Void) + κ (Comparison): absence as signal
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,       // absence / null observation
            LexPrimitiva::Comparison, // below significance threshold
        ])
        .with_dominant(LexPrimitiva::Void, 0.93)
    }
}

impl GroundsTo for TopologicalAddress {
    fn primitive_composition() -> PrimitiveComposition {
        // λ (Location) + σ (Sequence): hierarchical spatial addressing
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // position in topology
            LexPrimitiva::Sequence, // ordered path segments
        ])
        .with_dominant(LexPrimitiva::Location, 0.91)
    }
}

impl GroundsTo for Accumulator {
    fn primitive_composition() -> PrimitiveComposition {
        // N (Quantity) + σ (Sequence) + ∝ (Irreversibility): monotonic running total
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,        // numeric total
            LexPrimitiva::Sequence,        // ordered additions
            LexPrimitiva::Irreversibility, // monotonically increasing
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.92)
    }
}

impl GroundsTo for Checkpoint {
    fn primitive_composition() -> PrimitiveComposition {
        // π (Persistence) + ς (State) + ∝ (Irreversibility): durable snapshot
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,     // durably saved
            LexPrimitiva::State,           // state snapshot
            LexPrimitiva::Irreversibility, // confirmed checkpoints are permanent
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.94)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

// ============================================================================
// Quantum Primitives Grounding (10 T2-P, 3 T2-C)
// ============================================================================

use nexcore_primitives::quantum::{
    Amplitude, Decoherence, Eigenstate, Entanglement, Hermiticity, Interference, Measurement,
    Observable, Phase, Qubit, Superposition, Uncertainty, Unitarity,
};

impl GroundsTo for Amplitude {
    fn primitive_composition() -> PrimitiveComposition {
        // N (Quantity) + ν (Frequency): magnitude at a frequency
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // magnitude value
            LexPrimitiva::Frequency, // associated frequency
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.92)
    }
}

impl GroundsTo for Phase {
    fn primitive_composition() -> PrimitiveComposition {
        // ν (Frequency) + κ (Comparison): angular offset relative to reference
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,  // periodic position
            LexPrimitiva::Comparison, // relative to reference
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.90)
    }
}

impl GroundsTo for Superposition {
    fn primitive_composition() -> PrimitiveComposition {
        // ∃ (Existence) + Σ (Sum) + ∂ (Boundary): coexisting weighted states
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // multiple states exist simultaneously
            LexPrimitiva::Sum,       // weighted combination
            LexPrimitiva::Boundary,  // normalization constraint
        ])
        .with_dominant(LexPrimitiva::Sum, 0.88)
    }
}

impl GroundsTo for Measurement {
    fn primitive_composition() -> PrimitiveComposition {
        // ∂ (Boundary) + → (Causality) + ∝ (Irreversibility): irreversible collapse
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,        // observation boundary
            LexPrimitiva::Causality,       // measurement causes collapse
            LexPrimitiva::Irreversibility, // cannot uncollapse
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.87)
    }
}

impl GroundsTo for Interference {
    fn primitive_composition() -> PrimitiveComposition {
        // Σ (Sum) + ν (Frequency) + N (Quantity): wave summation
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // phasor addition
            LexPrimitiva::Frequency, // phase components
            LexPrimitiva::Quantity,  // amplitude values
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

impl GroundsTo for Uncertainty {
    fn primitive_composition() -> PrimitiveComposition {
        // ∂ (Boundary) + κ (Comparison) + ν (Frequency): conjugate tradeoff
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // minimum product bound
            LexPrimitiva::Comparison, // precision tradeoff
            LexPrimitiva::Frequency,  // conjugate variable periodicity
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

impl GroundsTo for Unitarity {
    fn primitive_composition() -> PrimitiveComposition {
        // π (Persistence) + ρ (Recursion) + N (Quantity): information-preserving transform
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // information preserved
            LexPrimitiva::Recursion,   // reversible cycle
            LexPrimitiva::Quantity,    // probability conserved
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.80)
    }
}

impl GroundsTo for Eigenstate {
    fn primitive_composition() -> PrimitiveComposition {
        // π (Persistence) + κ (Comparison) + μ (Mapping): stable fixed point
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // unchanged by operation
            LexPrimitiva::Comparison,  // eigenvalue scaling
            LexPrimitiva::Mapping,     // operator → eigenvalue
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.80)
    }
}

impl GroundsTo for Observable {
    fn primitive_composition() -> PrimitiveComposition {
        // μ (Mapping) + N (Quantity) + κ (Comparison): measurable property
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // state → outcome
            LexPrimitiva::Quantity,   // eigenvalue spectrum
            LexPrimitiva::Comparison, // outcome selection
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.82)
    }
}

impl GroundsTo for Hermiticity {
    fn primitive_composition() -> PrimitiveComposition {
        // κ (Comparison) + μ (Mapping) + π (Persistence): self-adjoint symmetry
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,  // forward = reverse check
            LexPrimitiva::Mapping,     // operator action
            LexPrimitiva::Persistence, // eigenvalues are real (stable)
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.78)
    }
}

impl GroundsTo for Entanglement {
    fn primitive_composition() -> PrimitiveComposition {
        // → (Causality) + λ (Location) + κ (Comparison) + ρ (Recursion): non-local correlation
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // correlated outcomes
            LexPrimitiva::Location,   // spatially separated subsystems
            LexPrimitiva::Comparison, // Bell inequality violation
            LexPrimitiva::Recursion,  // mutual dependency
        ])
        .with_dominant(LexPrimitiva::Causality, 0.82)
    }
}

impl GroundsTo for Decoherence {
    fn primitive_composition() -> PrimitiveComposition {
        // ∝ (Irreversibility) + ∂ (Boundary) + ν (Frequency) + → (Causality): coherence loss
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility, // irreversible information loss
            LexPrimitiva::Boundary,        // system-environment boundary
            LexPrimitiva::Frequency,       // decay rate
            LexPrimitiva::Causality,       // environment causes decoherence
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.78)
    }
}

impl GroundsTo for Qubit {
    fn primitive_composition() -> PrimitiveComposition {
        // ς (State) + ∂ (Boundary) + Σ (Sum) + N (Quantity): two-level quantum system
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // |0>/|1> basis states
            LexPrimitiva::Boundary, // normalization constraint
            LexPrimitiva::Sum,      // superposition of basis states
            LexPrimitiva::Quantity, // amplitude values
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

// ============================================================================
// Biological Pipeline Grounding (transcriptase → ribosome → phenotype → hormones)
// ============================================================================

// --- Transcriptase: JSON → Schema inference ---

use nexcore_transcriptase::{DiagnosticLevel, Fidelity, Schema, SchemaKind};

impl GroundsTo for Schema {
    fn primitive_composition() -> PrimitiveComposition {
        // Schema: inferred structure = Comparison + Sequence + Mapping + Boundary
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // type classification
            LexPrimitiva::Sequence,   // observation accumulation
            LexPrimitiva::Mapping,    // field name → type mapping
            LexPrimitiva::Boundary,   // range bounds (min/max)
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.88)
    }
}

impl GroundsTo for SchemaKind {
    fn primitive_composition() -> PrimitiveComposition {
        // SchemaKind: type variants = Sum + Mapping
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // 8 type variants (Null/Bool/Int/Float/Str/Array/Record/Mixed)
            LexPrimitiva::Mapping, // each variant maps to structural properties
        ])
        .with_dominant(LexPrimitiva::Sum, 0.92)
    }
}

impl GroundsTo for DiagnosticLevel {
    fn primitive_composition() -> PrimitiveComposition {
        // DiagnosticLevel: violation severity = Sum + Comparison
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Critical/Warning/Info enum
            LexPrimitiva::Comparison, // severity ordering
        ])
        .with_dominant(LexPrimitiva::Sum, 0.93)
    }
}

impl GroundsTo for Fidelity {
    fn primitive_composition() -> PrimitiveComposition {
        // Fidelity: round-trip quality = Sum + Comparison
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Exact/Approximate/Failed enum
            LexPrimitiva::Comparison, // fidelity level comparison
        ])
        .with_dominant(LexPrimitiva::Sum, 0.93)
    }
}

// --- Ribosome: Schema → Contract + Drift detection ---

use nexcore_ribosome::{Contract, DriftResult, DriftSeverity, DriftSignal, DriftType};

impl GroundsTo for Contract {
    fn primitive_composition() -> PrimitiveComposition {
        // Contract: versioned schema agreement = Mapping + Persistence + Boundary + Sequence
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,     // schema field definitions
            LexPrimitiva::Persistence, // versioned and stored
            LexPrimitiva::Boundary,    // required field constraints
            LexPrimitiva::Sequence,    // version history
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.87)
    }
}

impl GroundsTo for DriftType {
    fn primitive_composition() -> PrimitiveComposition {
        // DriftType: 7 drift variants = Sum + Boundary
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // 7 variants (TypeMismatch, FieldAdded, FieldRemoved, etc.)
            LexPrimitiva::Boundary, // schema boundary violation classification
        ])
        .with_dominant(LexPrimitiva::Sum, 0.91)
    }
}

impl GroundsTo for DriftSeverity {
    fn primitive_composition() -> PrimitiveComposition {
        // DriftSeverity: 3 levels = Sum + Comparison
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Breaking/Warning/Info enum
            LexPrimitiva::Comparison, // severity ordering
        ])
        .with_dominant(LexPrimitiva::Sum, 0.93)
    }
}

impl GroundsTo for DriftResult {
    fn primitive_composition() -> PrimitiveComposition {
        // DriftResult: drift analysis output = Comparison + Sequence + Sum + Mapping
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // old vs new schema comparison
            LexPrimitiva::Sequence,   // ordered list of drifts
            LexPrimitiva::Sum,        // compatible/incompatible summary
            LexPrimitiva::Mapping,    // field → drift mapping
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

impl GroundsTo for DriftSignal {
    fn primitive_composition() -> PrimitiveComposition {
        // DriftSignal: Guardian-compatible signal = Causality + Frequency + Quantity
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // drift causes alert
            LexPrimitiva::Frequency, // signal emission rate
            LexPrimitiva::Quantity,  // severity score
        ])
        .with_dominant(LexPrimitiva::Causality, 0.89)
    }
}

// --- Phenotype: adversarial mutation testing ---

use nexcore_phenotype::{Mutation, Phenotype};

impl GroundsTo for Mutation {
    fn primitive_composition() -> PrimitiveComposition {
        // Mutation: 7 adversarial variants = Boundary + Mapping + Sum + Causality
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // boundary-crossing transformations
            LexPrimitiva::Mapping,   // input → mutated output
            LexPrimitiva::Sum,       // 7 mutation variants
            LexPrimitiva::Causality, // mutation causes drift
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.86)
    }
}

impl GroundsTo for Phenotype {
    fn primitive_composition() -> PrimitiveComposition {
        // Phenotype: observable mutated data = Sequence + Mapping + Boundary + Causality
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // mutations_applied sequence
            LexPrimitiva::Mapping,   // data → mutated data
            LexPrimitiva::Boundary,  // expected_drifts boundaries
            LexPrimitiva::Causality, // mutations cause observable drifts
        ])
        .with_dominant(LexPrimitiva::Causality, 0.84)
    }
}

// --- Hormones: persistent state modulation ---

use nexcore_hormones::{BehavioralModifiers, EndocrineState, HormoneLevel, HormoneType, Stimulus};

impl GroundsTo for HormoneLevel {
    fn primitive_composition() -> PrimitiveComposition {
        // HormoneLevel: clamped 0-1 newtype = Quantity + Boundary
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // f64 value
            LexPrimitiva::Boundary, // clamped [0.0, 1.0]
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.95)
    }
}

impl GroundsTo for HormoneType {
    fn primitive_composition() -> PrimitiveComposition {
        // HormoneType: 6 hormone variants = Sum + Causality
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Cortisol/Dopamine/Serotonin/Adrenaline/Oxytocin/Melatonin
            LexPrimitiva::Causality, // each hormone causes behavioral effects
        ])
        .with_dominant(LexPrimitiva::Sum, 0.92)
    }
}

impl GroundsTo for Stimulus {
    fn primitive_composition() -> PrimitiveComposition {
        // Stimulus: 18 triggering events = Causality + Sum + Quantity
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // stimulus causes hormone change
            LexPrimitiva::Sum,       // 18 stimulus variants
            LexPrimitiva::Quantity,  // intensity magnitude
        ])
        .with_dominant(LexPrimitiva::Causality, 0.90)
    }
}

impl GroundsTo for EndocrineState {
    fn primitive_composition() -> PrimitiveComposition {
        // EndocrineState: full hormone snapshot = State + Mapping + Quantity + Frequency
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // current system state
            LexPrimitiva::Mapping,   // hormone_type → level mapping
            LexPrimitiva::Quantity,  // 6 hormone levels
            LexPrimitiva::Frequency, // temporal state changes
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for BehavioralModifiers {
    fn primitive_composition() -> PrimitiveComposition {
        // BehavioralModifiers: hormone-derived behavior weights = Mapping + Quantity + Causality
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // hormone → behavior effect
            LexPrimitiva::Quantity,  // modifier magnitudes
            LexPrimitiva::Causality, // hormones cause behavior changes
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.87)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex_primitiva::GroundingTier;

    #[test]
    fn test_threshold_gate_grounding() {
        let comp = ThresholdGate::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!((comp.confidence - 0.92).abs() < 0.01);
    }

    #[test]
    fn test_saturation_kinetics_grounding() {
        let comp = SaturationKinetics::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
    }

    #[test]
    fn test_decay_kinetics_grounding() {
        let comp = DecayKinetics::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Irreversibility));
    }

    #[test]
    fn test_harm_type_grounding() {
        let comp = HarmType::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
    }

    #[test]
    fn test_primitive_tier_grounding() {
        let comp = PrimitiveTier::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        // PrimitiveTier is a simple enum - should be T2-P
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_measured_wrapping() {
        // Measured<f64> should have Quantity + State
        let comp = <Measured<f64>>::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::State));
    }

    // Frontier T2-P tests

    #[test]
    fn test_audit_trail_grounding() {
        let comp = AuditTrail::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_absence_marker_grounding() {
        let comp = AbsenceMarker::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Void));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Void));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_pipeline_grounding() {
        let comp = Pipeline::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_consumption_mark_grounding() {
        let comp = ConsumptionMark::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Irreversibility));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_entity_status_grounding() {
        let comp = EntityStatus::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Existence));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_resource_path_grounding() {
        let comp = ResourcePath::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Location));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_recursion_bound_grounding() {
        let comp = RecursionBound::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_record_structure_grounding() {
        let comp = RecordStructure::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Product));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Product));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_all_16_t1_have_t2p_representation() {
        // After frontier types, every T1 should have at least one T2-P grounding
        let all_compositions: Vec<PrimitiveComposition> = vec![
            // Existing T2-P
            ThresholdGate::primitive_composition(),
            SafetyMargin::primitive_composition(),
            Bdi::primitive_composition(),
            BufferSystem::primitive_composition(),
            SignalDetector::primitive_composition(),
            EquilibriumSystem::primitive_composition(),
            HarmType::primitive_composition(),
            PrimitiveTier::primitive_composition(),
            // Frontier T2-P
            AuditTrail::primitive_composition(),
            AbsenceMarker::primitive_composition(),
            Pipeline::primitive_composition(),
            ConsumptionMark::primitive_composition(),
            EntityStatus::primitive_composition(),
            ResourcePath::primitive_composition(),
            RecursionBound::primitive_composition(),
            RecordStructure::primitive_composition(),
        ];

        // Collect all T1 primitives that appear in any T2-P composition
        let mut covered: std::collections::HashSet<LexPrimitiva> = std::collections::HashSet::new();
        for comp in &all_compositions {
            if GroundingTier::classify(comp) == GroundingTier::T2Primitive {
                for p in &comp.primitives {
                    covered.insert(*p);
                }
            }
        }

        // All 16 T1 primitives should be covered
        for p in LexPrimitiva::all() {
            assert!(
                covered.contains(&p),
                "T1 primitive {:?} ({}) has no T2-P representation",
                p,
                p.symbol()
            );
        }
    }

    #[test]
    fn test_occupancy_grounding() {
        let comp = Occupancy::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::State));
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_chemistry_tier_distribution() {
        // Most chemistry types should be T2-P or T2-C
        let types: Vec<PrimitiveComposition> = vec![
            ThresholdGate::primitive_composition(),
            SaturationKinetics::primitive_composition(),
            FeasibilityAssessment::primitive_composition(),
            DecayKinetics::primitive_composition(),
        ];

        for comp in types {
            let tier = GroundingTier::classify(&comp);
            assert!(
                tier == GroundingTier::T2Primitive || tier == GroundingTier::T2Composite,
                "Chemistry primitive should be T2"
            );
        }
    }

    // Patient/Safety T2-P/T2-C grounding tests

    #[test]
    fn test_recipient_grounding() {
        let comp = Recipient::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!((comp.confidence - 0.94).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_safety_boundary_grounding() {
        let comp = SafetyBoundary::<f64>::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!((comp.confidence - 0.91).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_harm_grounding() {
        let comp = Harm::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Void));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!((comp.confidence - 0.89).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_vulnerable_grounding() {
        let comp = Vulnerable::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!((comp.confidence - 0.88).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
    }

    #[test]
    fn test_boundary_breach_grounding() {
        let comp = BoundaryBreach::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Void));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!((comp.confidence - 0.87).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
    }

    #[test]
    fn test_monitoring_grounding() {
        let comp = Monitoring::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!((comp.confidence - 0.90).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_tracked_grounding() {
        let comp = Tracked::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!((comp.confidence - 0.88).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
    }

    // ====================================================================
    // Transfer Primitives Grounding Tests (10 T2-P types)
    // ====================================================================

    #[test]
    fn test_threat_signature_grounding() {
        let comp = ThreatSignature::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_resource_ratio_grounding() {
        let comp = ResourceRatio::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_pattern_matcher_grounding() {
        let comp = PatternMatcher::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_explore_exploit_grounding() {
        let comp = ExploreExploit::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_event_classifier_grounding() {
        let comp = EventClassifier::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_feedback_loop_grounding() {
        let comp = FeedbackLoop::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_schema_contract_grounding() {
        let comp = SchemaContract::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_message_bus_grounding() {
        let comp = MessageBus::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_specialized_worker_grounding() {
        let comp = SpecializedWorker::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_decay_function_grounding() {
        let comp = DecayFunction::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_contingency_table_grounding() {
        let comp = ContingencyTable::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!((comp.confidence - 0.85).abs() < 0.01);
        // 5 unique primitives → T2-C (matches doc comment on type)
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
    }

    #[test]
    fn test_homeostasis_grounding() {
        let comp = Homeostasis::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
        assert!((comp.confidence - 0.92).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_staged_validation_grounding() {
        let comp = StagedValidation::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!((comp.confidence - 0.91).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_atomicity_grounding() {
        let comp = Atomicity::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!((comp.confidence - 0.93).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_compare_and_swap_grounding() {
        let comp = CompareAndSwap::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!((comp.confidence - 0.94).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_toctou_window_grounding() {
        let comp = ToctouWindow::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!((comp.confidence - 0.90).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_serialization_guard_grounding() {
        let comp = SerializationGuard::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!((comp.confidence - 0.92).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_rate_limiter_grounding() {
        let comp = RateLimiter::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Frequency));
        assert!((comp.confidence - 0.93).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_circuit_breaker_grounding() {
        let comp = CircuitBreaker::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::State));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!((comp.confidence - 0.94).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_idempotency_grounding() {
        let comp = Idempotency::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Existence));
        assert!((comp.confidence - 0.92).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_negative_evidence_grounding() {
        let comp = NegativeEvidence::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Void));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Void));
        assert!((comp.confidence - 0.93).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_topological_address_grounding() {
        let comp = TopologicalAddress::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Location));
        assert!((comp.confidence - 0.91).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_accumulator_grounding() {
        let comp = Accumulator::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!((comp.confidence - 0.92).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_checkpoint_grounding() {
        let comp = Checkpoint::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert!(comp.primitives.contains(&LexPrimitiva::State));
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert!((comp.confidence - 0.94).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_all_transfer_primitives_are_t2p() {
        // All 23 transfer primitives should classify as T2-P
        let all: Vec<(&str, PrimitiveComposition)> = vec![
            ("ThreatSignature", ThreatSignature::primitive_composition()),
            ("ResourceRatio", ResourceRatio::primitive_composition()),
            ("PatternMatcher", PatternMatcher::primitive_composition()),
            ("ExploreExploit", ExploreExploit::primitive_composition()),
            ("EventClassifier", EventClassifier::primitive_composition()),
            ("FeedbackLoop", FeedbackLoop::primitive_composition()),
            ("SchemaContract", SchemaContract::primitive_composition()),
            ("MessageBus", MessageBus::primitive_composition()),
            (
                "SpecializedWorker",
                SpecializedWorker::primitive_composition(),
            ),
            ("DecayFunction", DecayFunction::primitive_composition()),
            ("Homeostasis", Homeostasis::primitive_composition()),
            (
                "StagedValidation",
                StagedValidation::primitive_composition(),
            ),
            ("Atomicity", Atomicity::primitive_composition()),
            ("CompareAndSwap", CompareAndSwap::primitive_composition()),
            ("ToctouWindow", ToctouWindow::primitive_composition()),
            (
                "SerializationGuard",
                SerializationGuard::primitive_composition(),
            ),
            ("RateLimiter", RateLimiter::primitive_composition()),
            ("CircuitBreaker", CircuitBreaker::primitive_composition()),
            ("Idempotency", Idempotency::primitive_composition()),
            (
                "NegativeEvidence",
                NegativeEvidence::primitive_composition(),
            ),
            (
                "TopologicalAddress",
                TopologicalAddress::primitive_composition(),
            ),
            ("Accumulator", Accumulator::primitive_composition()),
            ("Checkpoint", Checkpoint::primitive_composition()),
        ];

        for (name, comp) in &all {
            assert_eq!(
                GroundingTier::classify(comp),
                GroundingTier::T2Primitive,
                "{} should be T2-P but has {} unique primitives",
                name,
                comp.primitives.len()
            );
        }
    }

    // ====================================================================
    // Quantum Primitives Grounding Tests (10 T2-P, 3 T2-C)
    // ====================================================================

    #[test]
    fn test_amplitude_grounding() {
        let comp = Amplitude::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!((comp.confidence - 0.92).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_phase_grounding() {
        let comp = Phase::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Frequency));
        assert!((comp.confidence - 0.90).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_superposition_grounding() {
        let comp = Superposition::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!((comp.confidence - 0.88).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_measurement_grounding() {
        let comp = Measurement::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Irreversibility));
        assert!((comp.confidence - 0.87).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_interference_grounding() {
        let comp = Interference::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!((comp.confidence - 0.85).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_uncertainty_grounding() {
        let comp = Uncertainty::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!((comp.confidence - 0.90).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_unitarity_grounding() {
        let comp = Unitarity::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert!((comp.confidence - 0.80).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_eigenstate_grounding() {
        let comp = Eigenstate::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert!((comp.confidence - 0.80).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_observable_grounding() {
        let comp = Observable::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!((comp.confidence - 0.82).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_hermiticity_grounding() {
        let comp = Hermiticity::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!((comp.confidence - 0.78).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_entanglement_grounding() {
        let comp = Entanglement::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!((comp.confidence - 0.82).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
    }

    #[test]
    fn test_decoherence_grounding() {
        let comp = Decoherence::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Irreversibility));
        assert!((comp.confidence - 0.78).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
    }

    #[test]
    fn test_qubit_grounding() {
        let comp = Qubit::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::State));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!((comp.confidence - 0.85).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
    }

    #[test]
    fn test_quantum_t2p_types() {
        // All 10 T2-P quantum types should classify correctly
        let t2p: Vec<(&str, PrimitiveComposition)> = vec![
            ("Amplitude", Amplitude::primitive_composition()),
            ("Phase", Phase::primitive_composition()),
            ("Superposition", Superposition::primitive_composition()),
            ("Measurement", Measurement::primitive_composition()),
            ("Interference", Interference::primitive_composition()),
            ("Uncertainty", Uncertainty::primitive_composition()),
            ("Unitarity", Unitarity::primitive_composition()),
            ("Eigenstate", Eigenstate::primitive_composition()),
            ("Observable", Observable::primitive_composition()),
            ("Hermiticity", Hermiticity::primitive_composition()),
        ];

        for (name, comp) in &t2p {
            assert_eq!(
                GroundingTier::classify(comp),
                GroundingTier::T2Primitive,
                "{} should be T2-P but has {} unique primitives",
                name,
                comp.primitives.len()
            );
        }
    }

    #[test]
    fn test_quantum_t2c_types() {
        // All 3 T2-C quantum types should classify correctly
        let t2c: Vec<(&str, PrimitiveComposition)> = vec![
            ("Entanglement", Entanglement::primitive_composition()),
            ("Decoherence", Decoherence::primitive_composition()),
            ("Qubit", Qubit::primitive_composition()),
        ];

        for (name, comp) in &t2c {
            assert_eq!(
                GroundingTier::classify(comp),
                GroundingTier::T2Composite,
                "{} should be T2-C but has {} unique primitives",
                name,
                comp.primitives.len()
            );
        }
    }

    // ====================================================================
    // Biological Pipeline Grounding Tests (16 types across 4 crates)
    // ====================================================================

    // --- Transcriptase ---

    #[test]
    fn test_schema_grounding() {
        let comp = Schema::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!((comp.confidence - 0.88).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
    }

    #[test]
    fn test_schema_kind_grounding() {
        let comp = SchemaKind::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!((comp.confidence - 0.92).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_transcriptase_severity_grounding() {
        let comp = DiagnosticLevel::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!((comp.confidence - 0.93).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_fidelity_grounding() {
        let comp = Fidelity::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!((comp.confidence - 0.93).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    // --- Ribosome ---

    #[test]
    fn test_contract_grounding() {
        let comp = Contract::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!((comp.confidence - 0.87).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
    }

    #[test]
    fn test_drift_type_grounding() {
        let comp = DriftType::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!((comp.confidence - 0.91).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_drift_severity_grounding() {
        let comp = DriftSeverity::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!((comp.confidence - 0.93).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_drift_result_grounding() {
        let comp = DriftResult::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!((comp.confidence - 0.85).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
    }

    #[test]
    fn test_drift_signal_grounding() {
        let comp = DriftSignal::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!((comp.confidence - 0.89).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    // --- Phenotype ---

    #[test]
    fn test_mutation_grounding() {
        let comp = Mutation::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!((comp.confidence - 0.86).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
    }

    #[test]
    fn test_phenotype_grounding() {
        let comp = Phenotype::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!((comp.confidence - 0.84).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
    }

    // --- Hormones ---

    #[test]
    fn test_hormone_level_grounding() {
        let comp = HormoneLevel::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!((comp.confidence - 0.95).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_hormone_type_grounding() {
        let comp = HormoneType::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!((comp.confidence - 0.92).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_stimulus_grounding() {
        let comp = Stimulus::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!((comp.confidence - 0.90).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_endocrine_state_grounding() {
        let comp = EndocrineState::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::State));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!((comp.confidence - 0.85).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
    }

    #[test]
    fn test_behavioral_modifiers_grounding() {
        let comp = BehavioralModifiers::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!((comp.confidence - 0.87).abs() < 0.01);
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Primitive);
    }

    #[test]
    fn test_bio_pipeline_tier_distribution() {
        // Transcriptase T2-P types
        let t2p_bio: Vec<(&str, PrimitiveComposition)> = vec![
            ("SchemaKind", SchemaKind::primitive_composition()),
            ("DiagnosticLevel", DiagnosticLevel::primitive_composition()),
            ("Fidelity", Fidelity::primitive_composition()),
            ("DriftType", DriftType::primitive_composition()),
            ("DriftSeverity", DriftSeverity::primitive_composition()),
            ("DriftSignal", DriftSignal::primitive_composition()),
            ("HormoneLevel", HormoneLevel::primitive_composition()),
            ("HormoneType", HormoneType::primitive_composition()),
            ("Stimulus", Stimulus::primitive_composition()),
            (
                "BehavioralModifiers",
                BehavioralModifiers::primitive_composition(),
            ),
        ];

        for (name, comp) in &t2p_bio {
            assert_eq!(
                GroundingTier::classify(comp),
                GroundingTier::T2Primitive,
                "{} should be T2-P but has {} unique primitives",
                name,
                comp.primitives.len()
            );
        }

        // Transcriptase/Ribosome/Phenotype/Hormones T2-C types
        let t2c_bio: Vec<(&str, PrimitiveComposition)> = vec![
            ("Schema", Schema::primitive_composition()),
            ("Contract", Contract::primitive_composition()),
            ("DriftResult", DriftResult::primitive_composition()),
            ("Mutation", Mutation::primitive_composition()),
            ("Phenotype", Phenotype::primitive_composition()),
            ("EndocrineState", EndocrineState::primitive_composition()),
        ];

        for (name, comp) in &t2c_bio {
            assert_eq!(
                GroundingTier::classify(comp),
                GroundingTier::T2Composite,
                "{} should be T2-C but has {} unique primitives",
                name,
                comp.primitives.len()
            );
        }
    }
}
