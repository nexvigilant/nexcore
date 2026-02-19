//! # Universal Primitives
//!
//! Domain-agnostic computational patterns extracted from scientific equations.
//! Each primitive decomposes to T1 universals (cause, effect, quantity, threshold,
//! frequency, duration, state, ratio) enabling cross-domain transfer.
//!
//! ## Modules
//!
//! - **chemistry**: Chemistry equations as computational patterns
//!   - Arrhenius (threshold gating)
//!   - Michaelis-Menten (saturation)
//!   - Gibbs Free Energy (feasibility)
//!   - Rate Laws (dependency)
//!   - Henderson-Hasselbalch (buffer stability)
//!   - Beer-Lambert (signal intensity)
//!   - Half-Life (decay kinetics)
//!   - Equilibrium Constant (steady state)
//!
//! - **delegation**: Model routing and task classification
//!   - Model (capabilities, strengths, error tolerance)
//!   - ClassificationTree (predicate→action mapping)
//!   - ConfidenceScore (multi-dimensional scoring)
//!   - ReviewProtocol (5-stage validation)
//!   - DelegationRouter (task→model routing)

pub mod access;
pub use nexcore_constants::bathroom_lock;
pub use nexcore_primitives::chemistry;
pub mod delegation;
pub mod emoji;
pub mod frontier;
pub mod governance;
pub use nexcore_primitives::dynamics;
pub use nexcore_primitives::measurement;
pub mod ksb;
pub mod pharmacovigilance;
pub mod recipient;
pub mod safety;
pub mod signal;
pub mod temporal;

// Re-export measurement primitives
pub use measurement::{Confidence, Measured};

// Re-export dynamics types
pub use dynamics::{EnvironmentalCoupling, Interaction, Observer, Phasor};

// Re-export chemistry primitives at top level for convenience
pub use chemistry::{
    BufferError,
    BufferSystem,
    DecayError,
    DecayKinetics,
    DependencyError,
    EquilibriumError,
    EquilibriumSystem,
    Favorability,
    FeasibilityAssessment,
    FeasibilityError,
    PvMapping,
    RateLaw,
    SaturationError,
    SaturationKinetics,
    SignalDetector,
    SignalError,
    ThresholdError,
    ThresholdGate,
    // Threshold gating
    activation_probability,
    arrhenius_rate,
    // Signal intensity
    beer_lambert_absorbance,
    // Buffer stability
    buffer_capacity,
    // Dependency
    calculate_rate_law,
    // Feasibility
    classify_favorability,
    // Decay kinetics
    decay_constant_from_half_life,
    detection_limit,
    // Equilibrium
    equilibrium_constant,
    first_order_decay,
    gibbs_free_energy,
    half_life_from_decay_constant,
    infer_concentration,
    ionization_ratio,
    is_favorable,
    // Saturation
    michaelis_menten_rate,
    // PV mappings
    pv_mappings,
    rate_limiting_factor,
    remaining_after_time,
    saturation_fraction,
    steady_state_fractions,
    threshold_exceeded,
    time_to_equilibrium,
    time_to_fraction,
    utilization_at_load,
};

// Re-export delegation primitives
pub use delegation::{
    // Classification
    ClassificationBuilder,
    ClassificationTree,
    // Confidence
    ConfidenceScore,
    DelegationConfidence,
    // Routing
    DelegationRouter,
    ErrorCost,
    // Model
    Model,
    ModelCapability,
    ModelStrength,
    PredicateResult,
    // Review
    ReviewPhase,
    ReviewProtocol,
    ReviewResult,
    RoutingDecision,
    ScoreDimension,
    TaskCharacteristics,
};

// Re-export governance primitives (Confidence is re-exported from measurement above)
pub use governance::{Action, Resolution, Rule, Term, Treasury, Verdict, VoteWeight};

// Re-export temporal primitives
pub use temporal::{ExpiryRecord, SlidingWindow, ThresholdCounter, Ttl};

// Re-export access primitives
pub use access::AllowList;

// Re-export frontier T2-P primitives (complete T1 surface coverage)
pub use frontier::{
    AbsenceMarker, AuditTrail, ConsumptionMark, EntityStatus, Pipeline, RecursionBound,
    ResourcePath,
};

// Re-export emoji domain primitives (20 types, each concept used once)
pub use emoji::{
    Ambiguity, Category, Codepoint, CulturalInterpretation, EmojiComposition, EmojiEvolution,
    EmojiVersion, Encoding, FallbackDisplay, Glyph, InputMethod, ProposalProcess, Reaction,
    Rendering, Sentiment, Shortcode, SkinToneModifier, UnicodeStandard, UsageFrequency,
    ZwjSequence,
};

// Re-export bathroom lock (ς State + ∂ Boundary + ∃ Existence)
pub use bathroom_lock::{BathroomLock, LockError, Occupancy, OccupiedGuard};

// Re-export recipient T2-P/T2-C primitives (patient/safety domain)
pub use recipient::{Recipient, Tracked, Vulnerable};

// Re-export safety T2-P/T2-C primitives (boundary/harm domain)
pub use safety::{BoundaryBreach, Harm, Monitoring, SafetyBoundary};

// Re-export KSB framework types (PV Knowledge, Skills, Behaviors)
pub use ksb::{
    AiIntegrationPoint, AiTechnique, BloomLevel, CriticalBehavior, Epa, EpaTier, EssentialSkill,
    KnowledgeComponent, KsbFramework, KsbType, ProficiencyLevel, PvDomain,
};

// Re-export cross-domain transfer T2-P primitives
pub use nexcore_primitives::transfer;

// Re-export quantum domain primitives (10 T2-P, 3 T2-C)
pub use nexcore_primitives::quantum;
pub use quantum::{
    Amplitude, Decoherence, Eigenstate, Entanglement, Hermiticity, Interference, Measurement,
    Observable, Phase, Qubit, Superposition, Uncertainty, Unitarity,
};
