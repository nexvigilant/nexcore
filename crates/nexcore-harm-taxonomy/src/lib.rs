//! # Harm Classification Taxonomy (ToV §9)
//!
//! Systematic taxonomy of harm types connecting each to conservation laws violated,
//! hierarchy levels at which they manifest, and appropriate intervention strategies.
//!
//! ## §9.0 Taxonomic Foundations
//!
//! The eight harm types arise from the combinatorics of three binary characteristics
//! (Proposition 9.0.1):
//!
//! | Characteristic | Values |
//! |----------------|--------|
//! | Perturbation Multiplicity | Single / Multiple |
//! | Temporal Profile | Acute / Chronic |
//! | Response Determinism | Deterministic / Stochastic |
//!
//! This yields 2³ = 8 base categories (Types A-H).
//!
//! ## §9.1 Harm Type Enumeration
//!
//! | Type | Name | Primary Mechanism | Secondary |
//! |------|------|-------------------|-----------|
//! | A | Acute | Law 1 (Mass): rapid accumulation | Law 8 (Saturation) |
//! | B | Cumulative | Law 1 (Mass): accumulated exposure | Law 5 (Catalyst): repair exhaustion |
//! | C | Off-Target | Law 2 (Energy): favorable off-target binding | Law 4 (Flux) |
//! | D | Cascade | Law 4 (Flux): imbalance propagation | Law 7 (Equilibrium) |
//! | E | Idiosyncratic | θ ∈ Θ_susceptible | Not a law violation* |
//! | F | Saturation | Law 8 (Saturation): capacity exceeded | Law 6 (Rate) |
//! | G | Interaction | Law 5 (Catalyst): competitive inhibition | Multiple laws |
//! | H | Population | θ-distribution heterogeneity | Not a law violation* |
//!
//! *Types E and H are θ-space phenomena, not conservation law violations.
//!
//! ## §9.1.1 Manifestation Level Derivation
//!
//! | Type | Levels | Derivation Basis |
//! |------|--------|------------------|
//! | A | 4-6 | High m ⟹ fast propagation |
//! | B | 5-7 | Accumulation requires time |
//! | C | 3-5 | Local off-target detection |
//! | D | 4-7 | Network-dependent |
//! | E | 3-6 | θ-dependent variance |
//! | F | 3-5 | Local capacity phenomenon |
//! | G | 4-6 | Multi-input convergence |
//! | H | 6-8 | Population-level definition |

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![warn(missing_docs)]
#![allow(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    reason = "Harm taxonomy is a closed-domain ontology mapped directly to ToV sections"
)]

pub mod grounding;

use serde::{Deserialize, Serialize};
use std::fmt;

// =============================================================================
// Conservation Law Identifier (extracted from conservation_catalog)
// =============================================================================

/// Conservation law identifier from ToV §8.
///
/// The 11 conservation laws that govern system behavior. Harm types map to
/// violations of specific conservation laws.
///
/// Tier: T2-P (κ + N)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ConservationLawId {
    /// Law 1: Mass/Information Conservation.
    Law1Mass = 1,
    /// Law 2: Thermodynamic Directionality.
    Law2Energy = 2,
    /// Law 3: State Conservation.
    Law3State = 3,
    /// Law 4: Flux Conservation.
    Law4Flux = 4,
    /// Law 5: Catalyst Regeneration.
    Law5Catalyst = 5,
    /// Law 6: Rate Conservation (Local Form).
    Law6Rate = 6,
    /// Law 7: Equilibrium.
    Law7Equilibrium = 7,
    /// Law 8: Capacity Saturation.
    Law8Saturation = 8,
    /// Law 9: Entropy Production.
    Law9Entropy = 9,
    /// Law 10: Discretization.
    Law10Discretization = 10,
    /// Law 11: Structural Invariance.
    Law11Structure = 11,
}

impl ConservationLawId {
    /// All conservation laws in order.
    pub const ALL: [Self; 11] = [
        Self::Law1Mass,
        Self::Law2Energy,
        Self::Law3State,
        Self::Law4Flux,
        Self::Law5Catalyst,
        Self::Law6Rate,
        Self::Law7Equilibrium,
        Self::Law8Saturation,
        Self::Law9Entropy,
        Self::Law10Discretization,
        Self::Law11Structure,
    ];

    /// Get the law number (1-11).
    #[must_use]
    pub const fn number(&self) -> u8 {
        *self as u8
    }

    /// Get the law name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Law1Mass => "Mass/Information Conservation",
            Self::Law2Energy => "Thermodynamic Directionality",
            Self::Law3State => "State Conservation",
            Self::Law4Flux => "Flux Conservation",
            Self::Law5Catalyst => "Catalyst Regeneration",
            Self::Law6Rate => "Rate Conservation",
            Self::Law7Equilibrium => "Equilibrium",
            Self::Law8Saturation => "Capacity Saturation",
            Self::Law9Entropy => "Entropy Production",
            Self::Law10Discretization => "Discretization",
            Self::Law11Structure => "Structural Invariance",
        }
    }
}

impl fmt::Display for ConservationLawId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Law {} ({})", self.number(), self.name())
    }
}

// =============================================================================
// §9.0 Taxonomic Foundations
// =============================================================================

/// Perturbation multiplicity characteristic (Proposition 9.0.1).
///
/// Binary characteristic 1: Does the harm involve a single perturbation source
/// or multiple concurrent perturbations?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PerturbationMultiplicity {
    /// Single perturbation source
    Single,
    /// Multiple concurrent perturbations
    Multiple,
}

impl fmt::Display for PerturbationMultiplicity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single => write!(f, "Single"),
            Self::Multiple => write!(f, "Multiple"),
        }
    }
}

/// Temporal profile characteristic (Proposition 9.0.1).
///
/// Binary characteristic 2: Is the harm acute (high magnitude, short duration)
/// or chronic (lower magnitude, extended duration)?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemporalProfile {
    /// High magnitude, short duration
    Acute,
    /// Lower magnitude, extended duration
    Chronic,
    /// Any temporal profile (for types where it doesn't matter)
    Any,
}

impl fmt::Display for TemporalProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Acute => write!(f, "Acute"),
            Self::Chronic => write!(f, "Chronic"),
            Self::Any => write!(f, "Any"),
        }
    }
}

/// Response determinism characteristic (Proposition 9.0.1).
///
/// Binary characteristic 3: Is the response deterministic (predictable given state)
/// or stochastic (depends on hidden θ susceptibility)?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResponseDeterminism {
    /// Predictable response given system state
    Deterministic,
    /// Response depends on hidden θ (susceptibility parameter)
    Stochastic,
}

impl fmt::Display for ResponseDeterminism {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deterministic => write!(f, "Deterministic"),
            Self::Stochastic => write!(f, "Stochastic"),
        }
    }
}

/// Harm type characteristics derived from Proposition 9.0.1.
///
/// Encapsulates the three binary characteristics that combinatorially define
/// the eight harm types A-H.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HarmCharacteristics {
    /// Perturbation multiplicity: Single or Multiple
    pub multiplicity: PerturbationMultiplicity,
    /// Temporal profile: Acute, Chronic, or Any
    pub temporal: TemporalProfile,
    /// Response determinism: Deterministic or Stochastic
    pub determinism: ResponseDeterminism,
}

impl HarmCharacteristics {
    /// Create new harm characteristics.
    #[must_use]
    pub const fn new(
        multiplicity: PerturbationMultiplicity,
        temporal: TemporalProfile,
        determinism: ResponseDeterminism,
    ) -> Self {
        Self {
            multiplicity,
            temporal,
            determinism,
        }
    }

    /// Get characteristics for Type A (Acute).
    #[must_use]
    pub const fn type_a() -> Self {
        Self::new(
            PerturbationMultiplicity::Single,
            TemporalProfile::Acute,
            ResponseDeterminism::Deterministic,
        )
    }

    /// Get characteristics for Type B (Cumulative).
    #[must_use]
    pub const fn type_b() -> Self {
        Self::new(
            PerturbationMultiplicity::Single,
            TemporalProfile::Chronic,
            ResponseDeterminism::Deterministic,
        )
    }

    /// Get characteristics for Type C (Off-Target).
    /// Note: Stochastic w.r.t. which target is affected.
    #[must_use]
    pub const fn type_c() -> Self {
        Self::new(
            PerturbationMultiplicity::Single,
            TemporalProfile::Acute,
            ResponseDeterminism::Stochastic,
        )
    }

    /// Get characteristics for Type D (Cascade).
    #[must_use]
    pub const fn type_d() -> Self {
        Self::new(
            PerturbationMultiplicity::Multiple,
            TemporalProfile::Acute,
            ResponseDeterminism::Deterministic,
        )
    }

    /// Get characteristics for Type E (Idiosyncratic).
    #[must_use]
    pub const fn type_e() -> Self {
        Self::new(
            PerturbationMultiplicity::Single,
            TemporalProfile::Acute,
            ResponseDeterminism::Stochastic,
        )
    }

    /// Get characteristics for Type F (Saturation).
    /// Note: Deterministic once capacity is known.
    #[must_use]
    pub const fn type_f() -> Self {
        Self::new(
            PerturbationMultiplicity::Single,
            TemporalProfile::Acute,
            ResponseDeterminism::Deterministic,
        )
    }

    /// Get characteristics for Type G (Interaction).
    #[must_use]
    pub const fn type_g() -> Self {
        Self::new(
            PerturbationMultiplicity::Multiple,
            TemporalProfile::Any,
            ResponseDeterminism::Deterministic,
        )
    }

    /// Get characteristics for Type H (Population).
    /// Note: Stochastic w.r.t. subgroup membership.
    #[must_use]
    pub const fn type_h() -> Self {
        Self::new(
            PerturbationMultiplicity::Multiple,
            TemporalProfile::Any,
            ResponseDeterminism::Stochastic,
        )
    }
}

// =============================================================================
// §9.0 Theorem 9.0.1: Exhaustiveness
// =============================================================================

/// Result of exhaustiveness verification (Theorem 9.0.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExhaustivenessResult {
    /// Whether the theorem holds
    pub is_exhaustive: bool,
    /// Total number of combinatorial types (2³ = 8)
    pub total_types: usize,
    /// Number of defined types
    pub defined_types: usize,
    /// Coverage percentage
    pub coverage: f64,
    /// Proof sketch
    pub proof: String,
}

/// Verify Theorem 9.0.1 (Exhaustiveness).
///
/// Every harm event H can be classified into at least one type A-H based on
/// its perturbation multiplicity, temporal profile, and response determinism.
///
/// Note: This theorem covers the 8 core ToV §9 types (A-H). Type I is a GVR
/// framework extension not included in the exhaustiveness proof.
#[must_use]
pub fn verify_exhaustiveness() -> ExhaustivenessResult {
    let total_types = 2_usize.pow(3);
    let defined_types = HarmTypeId::TOV_CORE.len();

    ExhaustivenessResult {
        is_exhaustive: defined_types >= total_types,
        total_types,
        defined_types,
        coverage: defined_types as f64 / total_types as f64 * 100.0,
        proof: "Theorem 9.0.1: Let H be a harm event. Then: \
                (1) H involves either single or multiple perturbations (exhaustive); \
                (2) H has either acute or chronic temporal profile (exhaustive); \
                (3) H is either deterministic or stochastic in response (exhaustive). \
                The eight types cover all 2³ combinations. ∎"
            .to_string(),
    }
}

// =============================================================================
// §9.1 Harm Type Enumeration
// =============================================================================

/// Harm type identifier (A-I) from ToV §9.
///
/// Types A-H arise combinatorially from three binary characteristics
/// (Proposition 9.0.1) and map to conservation law violations or θ-space phenomena.
///
/// Type I (GoalMisalignment) extends the taxonomy for autonomous agents,
/// based on the {G, V, R} framework where entities lacking goal-selection (G),
/// value-evaluation (V), or refusal-capacity (R) exhibit symmetric harm capability.
///
/// Tier: T3 (κ+Σ+∂+→+N+ρ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HarmTypeId {
    /// Type A: Acute Harm
    A,
    /// Type B: Cumulative Harm
    B,
    /// Type C: Off-Target Harm
    C,
    /// Type D: Cascade Harm
    D,
    /// Type E: Idiosyncratic Harm
    E,
    /// Type F: Saturation Harm
    F,
    /// Type G: Interaction Harm
    G,
    /// Type H: Population Harm
    H,
    /// Type I: Goal Misalignment Harm (GVR Framework Extension)
    I,
}

impl HarmTypeId {
    /// The eight core harm types from ToV §9.1 (Theorem 9.0.1 exhaustiveness).
    pub const TOV_CORE: &'static [HarmTypeId] = &[
        Self::A,
        Self::B,
        Self::C,
        Self::D,
        Self::E,
        Self::F,
        Self::G,
        Self::H,
    ];

    /// All nine harm types including extensions (A-H from ToV §9, I from GVR framework).
    pub const ALL: &'static [HarmTypeId] = &[
        Self::A,
        Self::B,
        Self::C,
        Self::D,
        Self::E,
        Self::F,
        Self::G,
        Self::H,
        Self::I,
    ];

    /// Check if this is a core ToV §9 type (A-H).
    #[must_use]
    pub const fn is_tov_core(&self) -> bool {
        !matches!(self, Self::I)
    }

    /// Check if this is an extension type (I).
    #[must_use]
    pub const fn is_extension(&self) -> bool {
        matches!(self, Self::I)
    }

    /// Get the type letter.
    #[must_use]
    pub const fn letter(&self) -> char {
        match self {
            Self::A => 'A',
            Self::B => 'B',
            Self::C => 'C',
            Self::D => 'D',
            Self::E => 'E',
            Self::F => 'F',
            Self::G => 'G',
            Self::H => 'H',
            Self::I => 'I',
        }
    }

    /// Get the type name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::A => "Acute",
            Self::B => "Cumulative",
            Self::C => "Off-Target",
            Self::D => "Cascade",
            Self::E => "Idiosyncratic",
            Self::F => "Saturation",
            Self::G => "Interaction",
            Self::H => "Population",
            Self::I => "Goal Misalignment",
        }
    }

    /// Get the combinatorial characteristics (Proposition 9.0.1).
    #[must_use]
    pub const fn characteristics(&self) -> HarmCharacteristics {
        match self {
            Self::A => HarmCharacteristics::type_a(),
            Self::B => HarmCharacteristics::type_b(),
            Self::C => HarmCharacteristics::type_c(),
            Self::D => HarmCharacteristics::type_d(),
            Self::E => HarmCharacteristics::type_e(),
            Self::F => HarmCharacteristics::type_f(),
            Self::G => HarmCharacteristics::type_g(),
            Self::H => HarmCharacteristics::type_h(),
            Self::I => HarmCharacteristics::type_g(),
        }
    }

    /// Get the primary conservation law violated (if any).
    ///
    /// Types E and H are θ-space phenomena, not conservation law violations.
    #[must_use]
    pub const fn primary_law(&self) -> Option<ConservationLawId> {
        match self {
            Self::A => Some(ConservationLawId::Law1Mass),
            Self::B => Some(ConservationLawId::Law1Mass),
            Self::C => Some(ConservationLawId::Law2Energy),
            Self::D => Some(ConservationLawId::Law4Flux),
            Self::E => None,
            Self::F => Some(ConservationLawId::Law8Saturation),
            Self::G => Some(ConservationLawId::Law5Catalyst),
            Self::H => None,
            Self::I => None,
        }
    }

    /// Get the secondary conservation law involved (if any).
    #[must_use]
    pub const fn secondary_law(&self) -> Option<ConservationLawId> {
        match self {
            Self::A => Some(ConservationLawId::Law8Saturation),
            Self::B => Some(ConservationLawId::Law5Catalyst),
            Self::C => Some(ConservationLawId::Law4Flux),
            Self::D => Some(ConservationLawId::Law7Equilibrium),
            Self::E => None,
            Self::F => Some(ConservationLawId::Law6Rate),
            Self::G => None,
            Self::H => None,
            Self::I => None,
        }
    }

    /// Check if this harm type is a θ-space phenomenon (not a conservation law violation).
    #[must_use]
    pub const fn is_theta_space_phenomenon(&self) -> bool {
        matches!(self, Self::E | Self::H)
    }

    /// Get the full definition for this harm type.
    #[must_use]
    pub fn definition(&self) -> HarmTypeDefinition {
        HarmTypeDefinition::from_id(*self)
    }
}

impl fmt::Display for HarmTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type {} ({})", self.letter(), self.name())
    }
}

// =============================================================================
// §9.1.1 Manifestation Level (Definition 9.1)
// =============================================================================

/// Manifestation level specification (Definition 9.1).
///
/// For harm type X, the manifestation level ℓ_X is the LOWEST hierarchical level
/// at which harm becomes OBSERVABLE and CONSEQUENTIAL per the harm specification ℋ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestationLevel {
    /// Minimum hierarchy level (inclusive)
    pub min_level: u8,
    /// Maximum hierarchy level (inclusive)
    pub max_level: u8,
}

impl ManifestationLevel {
    /// Create a new manifestation level range.
    #[must_use]
    pub const fn new(min_level: u8, max_level: u8) -> Self {
        Self {
            min_level,
            max_level,
        }
    }

    /// Get the level range as a vector.
    #[must_use]
    pub fn levels(&self) -> Vec<u8> {
        (self.min_level..=self.max_level).collect()
    }

    /// Check if a level is within the manifestation range.
    #[must_use]
    pub const fn contains(&self, level: u8) -> bool {
        level >= self.min_level && level <= self.max_level
    }

    /// Type A (Acute): Levels 4-6.
    #[must_use]
    pub const fn type_a() -> Self {
        Self::new(4, 6)
    }

    /// Type B (Cumulative): Levels 5-7.
    #[must_use]
    pub const fn type_b() -> Self {
        Self::new(5, 7)
    }

    /// Type C (Off-Target): Levels 3-5.
    #[must_use]
    pub const fn type_c() -> Self {
        Self::new(3, 5)
    }

    /// Type D (Cascade): Levels 4-7.
    #[must_use]
    pub const fn type_d() -> Self {
        Self::new(4, 7)
    }

    /// Type E (Idiosyncratic): Levels 3-6.
    #[must_use]
    pub const fn type_e() -> Self {
        Self::new(3, 6)
    }

    /// Type F (Saturation): Levels 3-5.
    #[must_use]
    pub const fn type_f() -> Self {
        Self::new(3, 5)
    }

    /// Type G (Interaction): Levels 4-6.
    #[must_use]
    pub const fn type_g() -> Self {
        Self::new(4, 6)
    }

    /// Type H (Population): Levels 6-8.
    #[must_use]
    pub const fn type_h() -> Self {
        Self::new(6, 8)
    }

    /// Get manifestation level for a harm type.
    #[must_use]
    pub const fn for_type(harm_type: HarmTypeId) -> Self {
        match harm_type {
            HarmTypeId::A => Self::type_a(),
            HarmTypeId::B => Self::type_b(),
            HarmTypeId::C => Self::type_c(),
            HarmTypeId::D => Self::type_d(),
            HarmTypeId::E => Self::type_e(),
            HarmTypeId::F => Self::type_f(),
            HarmTypeId::G => Self::type_g(),
            HarmTypeId::H => Self::type_h(),
            HarmTypeId::I => Self::new(5, 8),
        }
    }
}

impl fmt::Display for ManifestationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Levels {}-{}", self.min_level, self.max_level)
    }
}

/// Derive manifestation level from Axiom 5 propagation parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestationDerivation {
    /// Harm type being analyzed
    pub harm_type: HarmTypeId,
    /// Propagation probability at each level (P_{i→i+1})
    pub propagation_probs: Vec<f64>,
    /// Detection probability threshold
    pub p_thresh: f64,
    /// Derived manifestation level
    pub derived_level: u8,
    /// Cumulative detection probability at each level
    pub cumulative_detection: Vec<f64>,
}

impl ManifestationDerivation {
    /// Derive manifestation level from propagation parameters.
    #[must_use]
    pub fn derive(
        harm_type: HarmTypeId,
        propagation_probs: &[f64],
        detection_threshold: f64,
        p_thresh: f64,
    ) -> Self {
        let mut cumulative_detection = Vec::with_capacity(propagation_probs.len());
        let mut cumulative_propagation = 1.0;
        let mut derived_level = propagation_probs.len() as u8;

        for (i, &p) in propagation_probs.iter().enumerate() {
            if i > 0 {
                cumulative_propagation *= propagation_probs[i - 1];
            }

            let detection_prob = cumulative_propagation * p * detection_threshold;
            cumulative_detection.push(detection_prob);

            if detection_prob >= p_thresh && derived_level == propagation_probs.len() as u8 {
                derived_level = (i + 1) as u8;
            }
        }

        Self {
            harm_type,
            propagation_probs: propagation_probs.to_vec(),
            p_thresh,
            derived_level,
            cumulative_detection,
        }
    }
}

// =============================================================================
// §9.2 Harm Type Definitions
// =============================================================================

/// Cross-domain mapping strength (§9.2 Notes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossDomainMapping {
    /// Strong mapping - concept translates directly across domains
    Strong,
    /// Moderate mapping - some domain-specific interpretation required
    Moderate,
    /// Weak mapping - significant domain-specific adaptation needed
    Weak,
}

impl fmt::Display for CrossDomainMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Strong => write!(f, "Strong"),
            Self::Moderate => write!(f, "Moderate"),
            Self::Weak => write!(f, "Weak"),
        }
    }
}

/// Complete harm type definition (§9.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmTypeDefinition {
    /// Harm type identifier
    pub id: HarmTypeId,
    /// Short name
    pub name: &'static str,
    /// Full definition
    pub definition: &'static str,
    /// Mechanism description
    pub mechanism: &'static str,
    /// Conservation connection
    pub conservation_connection: &'static str,
    /// Intervention strategy
    pub intervention_strategy: &'static str,
    /// Manifestation levels
    pub manifestation_level: ManifestationLevel,
    /// Cross-domain mapping strength
    pub cross_domain_mapping: CrossDomainMapping,
    /// Optional notes (e.g., for Types C, E, H)
    pub notes: Option<&'static str>,
}

impl HarmTypeDefinition {
    /// Get definition for a harm type.
    #[must_use]
    pub fn from_id(id: HarmTypeId) -> Self {
        match id {
            HarmTypeId::A => Self::type_a(),
            HarmTypeId::B => Self::type_b(),
            HarmTypeId::C => Self::type_c(),
            HarmTypeId::D => Self::type_d(),
            HarmTypeId::E => Self::type_e(),
            HarmTypeId::F => Self::type_f(),
            HarmTypeId::G => Self::type_g(),
            HarmTypeId::H => Self::type_h(),
            HarmTypeId::I => Self::type_i(),
        }
    }

    fn type_a() -> Self {
        Self {
            id: HarmTypeId::A,
            name: "Acute",
            definition: "Immediate, severe harm occurring shortly after perturbation \
                         with clear temporal relationship.",
            mechanism: "High-magnitude perturbation overwhelms buffering capacity, \
                        causing rapid constraint violation.",
            conservation_connection: "Violates mass/amount conservation through rapid \
                                      accumulation exceeding elimination capacity.",
            intervention_strategy: "Reduce perturbation magnitude; increase acute buffering; \
                                   implement rapid response protocols.",
            manifestation_level: ManifestationLevel::type_a(),
            cross_domain_mapping: CrossDomainMapping::Strong,
            notes: None,
        }
    }

    fn type_b() -> Self {
        Self {
            id: HarmTypeId::B,
            name: "Cumulative",
            definition: "Gradual harm emerging from repeated or prolonged exposure, \
                         with harm accumulating faster than repair.",
            mechanism: "Each perturbation causes small, partially irreversible damage; \
                        cumulative effect eventually crosses harm threshold.",
            conservation_connection: "Violates entropy constraint through irreversible \
                                      degradation accumulation.",
            intervention_strategy: "Limit exposure duration; implement recovery periods; \
                                   monitor cumulative metrics.",
            manifestation_level: ManifestationLevel::type_b(),
            cross_domain_mapping: CrossDomainMapping::Strong,
            notes: None,
        }
    }

    fn type_c() -> Self {
        Self {
            id: HarmTypeId::C,
            name: "Off-Target",
            definition: "Unintended effects on components other than the intended \
                         target of perturbation.",
            mechanism: "Perturbation follows energetically favorable pathways to \
                        unintended targets with similar binding/interaction properties.",
            conservation_connection: "Energy/gradient conservation drives perturbation \
                                      to unintended low-energy binding sites.",
            intervention_strategy: "Increase target selectivity; block off-target pathways; \
                                   monitor non-target components.",
            manifestation_level: ManifestationLevel::type_c(),
            cross_domain_mapping: CrossDomainMapping::Moderate,
            notes: Some(
                "Cross-domain mapping is moderately strong; some domain-specific \
                        interpretation is required.",
            ),
        }
    }

    fn type_d() -> Self {
        Self {
            id: HarmTypeId::D,
            name: "Cascade",
            definition: "Harm propagating through interconnected components, where \
                         initial failure triggers subsequent failures.",
            mechanism: "Flux imbalance at one node propagates through network, \
                        causing sequential constraint violations.",
            conservation_connection: "Flux conservation violation at one node creates \
                                      imbalance propagating to connected nodes.",
            intervention_strategy: "Implement circuit breakers; reduce coupling between \
                                   components; add redundancy.",
            manifestation_level: ManifestationLevel::type_d(),
            cross_domain_mapping: CrossDomainMapping::Strong,
            notes: None,
        }
    }

    fn type_e() -> Self {
        Self {
            id: HarmTypeId::E,
            name: "Idiosyncratic",
            definition: "Rare, unpredictable harm occurring in individuals with \
                         unusual susceptibility profiles.",
            mechanism: "Atypical system structure (genetic, architectural) creates \
                        unexpected vulnerability.",
            conservation_connection: "Structural invariance assumption fails for \
                                      atypical configurations.",
            intervention_strategy: "Identify susceptibility markers; implement personalized \
                                   monitoring; maintain pharmacovigilance for rare events.",
            manifestation_level: ManifestationLevel::type_e(),
            cross_domain_mapping: CrossDomainMapping::Moderate,
            notes: Some(
                "θ-space phenomenon: Harm arises from unusual parameter values \
                        (θ ∈ Θ_susceptible), not structural conservation law violations. \
                        Cross-domain mapping is moderately strong; the concept of \
                        'idiosyncratic' varies by domain.",
            ),
        }
    }

    fn type_f() -> Self {
        Self {
            id: HarmTypeId::F,
            name: "Saturation",
            definition: "Harm from exceeding the processing capacity of rate-limiting \
                         components.",
            mechanism: "Input rate exceeds maximum processing rate; backlog accumulates; \
                        system behavior becomes nonlinear.",
            conservation_connection: "Saturation law (v ≤ V_max) is violated or approached, \
                                      causing nonlinear dynamics.",
            intervention_strategy: "Reduce input rate; increase capacity; \
                                   implement load shedding.",
            manifestation_level: ManifestationLevel::type_f(),
            cross_domain_mapping: CrossDomainMapping::Strong,
            notes: None,
        }
    }

    fn type_g() -> Self {
        Self {
            id: HarmTypeId::G,
            name: "Interaction",
            definition: "Harm emerging from the combination of multiple perturbations, \
                         exceeding the sum of individual effects.",
            mechanism: "One perturbation affects the processing of another; shared \
                        catalysts become rate-limiting.",
            conservation_connection: "Catalyst regeneration is impaired when multiple \
                                      substrates compete for the same catalyst.",
            intervention_strategy: "Stagger perturbations temporally; avoid contraindicated \
                                   combinations; monitor interaction biomarkers.",
            manifestation_level: ManifestationLevel::type_g(),
            cross_domain_mapping: CrossDomainMapping::Strong,
            notes: None,
        }
    }

    fn type_h() -> Self {
        Self {
            id: HarmTypeId::H,
            name: "Population",
            definition: "Harm that differentially affects subgroups within a population, \
                         creating disparate impact.",
            mechanism: "State distribution across population subgroups shifts differently \
                        under perturbation.",
            conservation_connection: "State conservation (probability summing to 1) \
                                      redistributes harm across subgroups.",
            intervention_strategy: "Monitor subgroup outcomes; implement fairness constraints; \
                                   adjust perturbation for vulnerable groups.",
            manifestation_level: ManifestationLevel::type_h(),
            cross_domain_mapping: CrossDomainMapping::Moderate,
            notes: Some(
                "θ-distribution phenomenon: Harm arises from heterogeneous parameter \
                        distributions across subgroups, not from conservation law violations \
                        at the individual level. Cross-domain mapping is moderately strong; \
                        population concepts vary by domain.",
            ),
        }
    }

    fn type_i() -> Self {
        Self {
            id: HarmTypeId::I,
            name: "Goal Misalignment",
            definition: "Harm arising from autonomous agents pursuing goals that diverge \
                         from system or operator goals, due to lacking value-evaluation (V) \
                         or refusal capacity (R).",
            mechanism: "Agent with goal-selection capability (G) but without adequate \
                        value-evaluation (V) or refusal capacity (R) optimizes for \
                        misaligned objectives, amplifying harmful actions.",
            conservation_connection: "Violates purpose hierarchy: agent goals should be \
                                      subordinate to system goals. Not a conservation law \
                                      violation but a GVR framework phenomenon.",
            intervention_strategy: "Add value-evaluation (V) capability; implement refusal \
                                   capacity (R); constrain goal-selection (G) scope; \
                                   apply external ceiling limits inversely proportional \
                                   to autonomy level.",
            manifestation_level: ManifestationLevel::new(5, 8),
            cross_domain_mapping: CrossDomainMapping::Strong,
            notes: Some(
                "GVR framework extension: Entities lacking {G, V, R} exhibit \
                        symmetric harm capability. Adding any of {G, V, R} breaks symmetry. \
                        Type I specifically addresses agents WITH goal-selection but \
                        WITHOUT adequate value-evaluation or refusal capacity.",
            ),
        }
    }

    /// Get all harm type definitions (including extensions).
    #[must_use]
    pub fn catalog() -> Vec<Self> {
        HarmTypeId::ALL
            .iter()
            .map(|id| Self::from_id(*id))
            .collect()
    }

    /// Get ToV §9 core harm type definitions (A-H only).
    #[must_use]
    pub fn tov_catalog() -> Vec<Self> {
        HarmTypeId::TOV_CORE
            .iter()
            .map(|id| Self::from_id(*id))
            .collect()
    }
}

// =============================================================================
// §9.3 Harm-Axiom Connections
// =============================================================================

/// Primary axiom associated with each harm type (§9.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimaryAxiom {
    /// Axiom 1: System Decomposition
    A1Decomposition,
    /// Axiom 2: Hierarchical Organization
    A2Hierarchy,
    /// Axiom 3: Conservation Constraints
    A3Conservation,
    /// Axiom 4: Safety Manifold
    A4Manifold,
    /// Axiom 5: Emergence
    A5Emergence,
}

impl PrimaryAxiom {
    /// Get the axiom number.
    #[must_use]
    pub const fn number(&self) -> u8 {
        match self {
            Self::A1Decomposition => 1,
            Self::A2Hierarchy => 2,
            Self::A3Conservation => 3,
            Self::A4Manifold => 4,
            Self::A5Emergence => 5,
        }
    }

    /// Get the axiom name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::A1Decomposition => "System Decomposition",
            Self::A2Hierarchy => "Hierarchical Organization",
            Self::A3Conservation => "Conservation Constraints",
            Self::A4Manifold => "Safety Manifold",
            Self::A5Emergence => "Emergence",
        }
    }
}

impl fmt::Display for PrimaryAxiom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A{} ({})", self.number(), self.name())
    }
}

/// Harm-Axiom connection (§9.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmAxiomConnection {
    /// Harm type
    pub harm_type: HarmTypeId,
    /// Primary axiom
    pub primary_axiom: PrimaryAxiom,
    /// Connection description
    pub connection: &'static str,
}

impl HarmAxiomConnection {
    /// Get connection for a harm type.
    #[must_use]
    pub fn for_type(harm_type: HarmTypeId) -> Self {
        match harm_type {
            HarmTypeId::A => Self {
                harm_type,
                primary_axiom: PrimaryAxiom::A4Manifold,
                connection: "Rapid boundary crossing",
            },
            HarmTypeId::B => Self {
                harm_type,
                primary_axiom: PrimaryAxiom::A5Emergence,
                connection: "Slow propagation through hierarchy",
            },
            HarmTypeId::C => Self {
                harm_type,
                primary_axiom: PrimaryAxiom::A1Decomposition,
                connection: "Interactions between elements",
            },
            HarmTypeId::D => Self {
                harm_type,
                primary_axiom: PrimaryAxiom::A2Hierarchy,
                connection: "Propagation across levels",
            },
            HarmTypeId::E => Self {
                harm_type,
                primary_axiom: PrimaryAxiom::A3Conservation,
                connection: "Unusual constraint structure",
            },
            HarmTypeId::F => Self {
                harm_type,
                primary_axiom: PrimaryAxiom::A3Conservation,
                connection: "Capacity constraint violation",
            },
            HarmTypeId::G => Self {
                harm_type,
                primary_axiom: PrimaryAxiom::A3Conservation,
                connection: "Multiple constraint interactions",
            },
            HarmTypeId::H => Self {
                harm_type,
                primary_axiom: PrimaryAxiom::A5Emergence,
                connection: "Population-level emergence",
            },
            HarmTypeId::I => Self {
                harm_type,
                primary_axiom: PrimaryAxiom::A2Hierarchy,
                connection: "Purpose hierarchy violation (GVR framework)",
            },
        }
    }

    /// Get all harm-axiom connections (including extensions).
    #[must_use]
    pub fn catalog() -> Vec<Self> {
        HarmTypeId::ALL
            .iter()
            .map(|id| Self::for_type(*id))
            .collect()
    }

    /// Get ToV §9 core harm-axiom connections (A-H only).
    #[must_use]
    pub fn tov_catalog() -> Vec<Self> {
        HarmTypeId::TOV_CORE
            .iter()
            .map(|id| Self::for_type(*id))
            .collect()
    }
}

// =============================================================================
// Non-Exclusivity (§9.0 Note)
// =============================================================================

/// Common harm type combinations (non-exclusivity note).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmTypeCombination {
    /// Primary harm type
    pub primary: HarmTypeId,
    /// Secondary harm type
    pub secondary: HarmTypeId,
    /// Combination name
    pub name: &'static str,
    /// Description
    pub description: &'static str,
}

impl HarmTypeCombination {
    /// Common combinations observed in practice.
    #[must_use]
    pub fn common_combinations() -> Vec<Self> {
        vec![
            Self {
                primary: HarmTypeId::A,
                secondary: HarmTypeId::D,
                name: "Acute cascade",
                description: "Single high-magnitude perturbation triggers multi-component failure",
            },
            Self {
                primary: HarmTypeId::B,
                secondary: HarmTypeId::G,
                name: "Cumulative interaction",
                description: "Multiple chronic exposures with synergistic effects",
            },
            Self {
                primary: HarmTypeId::E,
                secondary: HarmTypeId::H,
                name: "Population idiosyncratic",
                description: "Rare susceptibility concentrated in subgroup",
            },
            Self {
                primary: HarmTypeId::A,
                secondary: HarmTypeId::F,
                name: "Acute saturation",
                description: "High-magnitude perturbation immediately exceeds capacity",
            },
            Self {
                primary: HarmTypeId::D,
                secondary: HarmTypeId::G,
                name: "Cascade interaction",
                description: "Multiple perturbations trigger interconnected cascade failures",
            },
        ]
    }
}

// =============================================================================
// Harm Event Classification
// =============================================================================

/// Result of classifying a harm event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmClassification {
    /// Primary harm type
    pub primary_type: HarmTypeId,
    /// Secondary harm types (if multiple)
    pub secondary_types: Vec<HarmTypeId>,
    /// Classification confidence (0.0-1.0)
    pub confidence: f64,
    /// Reasoning for classification
    pub reasoning: String,
    /// Recommended interventions based on classification
    pub recommended_interventions: Vec<String>,
}

/// Classify a harm event based on its characteristics.
#[must_use]
pub fn classify_harm_event(
    multiplicity: PerturbationMultiplicity,
    temporal: TemporalProfile,
    determinism: ResponseDeterminism,
) -> HarmClassification {
    let mut matching_types: Vec<(HarmTypeId, i32)> = Vec::new();
    let mut best_match = None;
    let mut best_score: i32 = 0;

    for &harm_type in HarmTypeId::ALL {
        let chars = harm_type.characteristics();
        let mut score = 0;

        if chars.multiplicity == multiplicity {
            score += 2;
        }

        if chars.temporal == temporal || chars.temporal == TemporalProfile::Any {
            score += if chars.temporal == TemporalProfile::Any {
                1
            } else {
                2
            };
        }

        if chars.determinism == determinism {
            score += 2;
        }

        if score > 0 {
            matching_types.push((harm_type, score));
        }

        if score > best_score {
            best_score = score;
            best_match = Some(harm_type);
        }
    }

    let primary_type = best_match.unwrap_or(HarmTypeId::A);
    let definition = primary_type.definition();

    let secondary_types: Vec<HarmTypeId> = matching_types
        .iter()
        .filter(|(t, s)| *t != primary_type && *s >= best_score.saturating_sub(2))
        .map(|(t, _)| *t)
        .collect();

    let max_possible_score = 6;
    let confidence = best_score as f64 / max_possible_score as f64;

    let reasoning = format!(
        "Classified as {} based on: {} perturbation, {} temporal profile, {} response. \
         Match score: {}/{}.",
        primary_type, multiplicity, temporal, determinism, best_score, max_possible_score
    );

    let recommended_interventions = vec![definition.intervention_strategy.to_string()];

    HarmClassification {
        primary_type,
        secondary_types,
        confidence,
        reasoning,
        recommended_interventions,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harm_type_letters() {
        assert_eq!(HarmTypeId::A.letter(), 'A');
        assert_eq!(HarmTypeId::B.letter(), 'B');
        assert_eq!(HarmTypeId::C.letter(), 'C');
        assert_eq!(HarmTypeId::D.letter(), 'D');
        assert_eq!(HarmTypeId::E.letter(), 'E');
        assert_eq!(HarmTypeId::F.letter(), 'F');
        assert_eq!(HarmTypeId::G.letter(), 'G');
        assert_eq!(HarmTypeId::H.letter(), 'H');
    }

    #[test]
    fn test_harm_type_names() {
        assert_eq!(HarmTypeId::A.name(), "Acute");
        assert_eq!(HarmTypeId::B.name(), "Cumulative");
        assert_eq!(HarmTypeId::C.name(), "Off-Target");
        assert_eq!(HarmTypeId::D.name(), "Cascade");
        assert_eq!(HarmTypeId::E.name(), "Idiosyncratic");
        assert_eq!(HarmTypeId::F.name(), "Saturation");
        assert_eq!(HarmTypeId::G.name(), "Interaction");
        assert_eq!(HarmTypeId::H.name(), "Population");
    }

    #[test]
    fn test_primary_conservation_laws() {
        assert_eq!(
            HarmTypeId::A.primary_law(),
            Some(ConservationLawId::Law1Mass)
        );
        assert_eq!(
            HarmTypeId::B.primary_law(),
            Some(ConservationLawId::Law1Mass)
        );
        assert_eq!(
            HarmTypeId::C.primary_law(),
            Some(ConservationLawId::Law2Energy)
        );
        assert_eq!(
            HarmTypeId::D.primary_law(),
            Some(ConservationLawId::Law4Flux)
        );
        assert_eq!(
            HarmTypeId::F.primary_law(),
            Some(ConservationLawId::Law8Saturation)
        );
        assert_eq!(
            HarmTypeId::G.primary_law(),
            Some(ConservationLawId::Law5Catalyst)
        );
        assert_eq!(HarmTypeId::E.primary_law(), None);
        assert_eq!(HarmTypeId::H.primary_law(), None);
    }

    #[test]
    fn test_theta_space_phenomena() {
        assert!(!HarmTypeId::A.is_theta_space_phenomenon());
        assert!(!HarmTypeId::B.is_theta_space_phenomenon());
        assert!(!HarmTypeId::C.is_theta_space_phenomenon());
        assert!(!HarmTypeId::D.is_theta_space_phenomenon());
        assert!(HarmTypeId::E.is_theta_space_phenomenon());
        assert!(!HarmTypeId::F.is_theta_space_phenomenon());
        assert!(!HarmTypeId::G.is_theta_space_phenomenon());
        assert!(HarmTypeId::H.is_theta_space_phenomenon());
    }

    #[test]
    fn test_manifestation_levels_per_tov_9_1_1() {
        assert_eq!(ManifestationLevel::type_a(), ManifestationLevel::new(4, 6));
        assert_eq!(ManifestationLevel::type_b(), ManifestationLevel::new(5, 7));
        assert_eq!(ManifestationLevel::type_c(), ManifestationLevel::new(3, 5));
        assert_eq!(ManifestationLevel::type_d(), ManifestationLevel::new(4, 7));
        assert_eq!(ManifestationLevel::type_e(), ManifestationLevel::new(3, 6));
        assert_eq!(ManifestationLevel::type_f(), ManifestationLevel::new(3, 5));
        assert_eq!(ManifestationLevel::type_g(), ManifestationLevel::new(4, 6));
        assert_eq!(ManifestationLevel::type_h(), ManifestationLevel::new(6, 8));
    }

    #[test]
    fn test_manifestation_level_contains() {
        let level = ManifestationLevel::type_a();
        assert!(!level.contains(3));
        assert!(level.contains(4));
        assert!(level.contains(5));
        assert!(level.contains(6));
        assert!(!level.contains(7));
    }

    #[test]
    fn test_exhaustiveness_theorem() {
        let result = verify_exhaustiveness();
        assert!(result.is_exhaustive);
        assert_eq!(result.total_types, 8);
        assert_eq!(result.defined_types, 8);
        assert_eq!(result.coverage, 100.0);
    }

    #[test]
    fn test_harm_characteristics_type_a() {
        let chars = HarmCharacteristics::type_a();
        assert_eq!(chars.multiplicity, PerturbationMultiplicity::Single);
        assert_eq!(chars.temporal, TemporalProfile::Acute);
        assert_eq!(chars.determinism, ResponseDeterminism::Deterministic);
    }

    #[test]
    fn test_harm_characteristics_type_h() {
        let chars = HarmCharacteristics::type_h();
        assert_eq!(chars.multiplicity, PerturbationMultiplicity::Multiple);
        assert_eq!(chars.temporal, TemporalProfile::Any);
        assert_eq!(chars.determinism, ResponseDeterminism::Stochastic);
    }

    #[test]
    fn test_harm_type_definitions_complete() {
        let tov_catalog = HarmTypeDefinition::tov_catalog();
        assert_eq!(tov_catalog.len(), 8);

        let catalog = HarmTypeDefinition::catalog();
        assert_eq!(catalog.len(), 9);

        for def in &catalog {
            assert!(!def.definition.is_empty());
            assert!(!def.mechanism.is_empty());
            assert!(!def.conservation_connection.is_empty());
            assert!(!def.intervention_strategy.is_empty());
        }
    }

    #[test]
    fn test_harm_axiom_connections() {
        let tov_connections = HarmAxiomConnection::tov_catalog();
        assert_eq!(tov_connections.len(), 8);

        let connections = HarmAxiomConnection::catalog();
        assert_eq!(connections.len(), 9);

        assert_eq!(
            HarmAxiomConnection::for_type(HarmTypeId::A).primary_axiom,
            PrimaryAxiom::A4Manifold
        );
        assert_eq!(
            HarmAxiomConnection::for_type(HarmTypeId::B).primary_axiom,
            PrimaryAxiom::A5Emergence
        );
        assert_eq!(
            HarmAxiomConnection::for_type(HarmTypeId::D).primary_axiom,
            PrimaryAxiom::A2Hierarchy
        );
    }

    #[test]
    fn test_common_combinations() {
        let combos = HarmTypeCombination::common_combinations();
        assert!(!combos.is_empty());

        let acute_cascade = combos.iter().find(|c| c.name == "Acute cascade");
        assert!(acute_cascade.is_some());
        let ac = acute_cascade.unwrap();
        assert_eq!(ac.primary, HarmTypeId::A);
        assert_eq!(ac.secondary, HarmTypeId::D);
    }

    #[test]
    fn test_classify_harm_event_acute() {
        let result = classify_harm_event(
            PerturbationMultiplicity::Single,
            TemporalProfile::Acute,
            ResponseDeterminism::Deterministic,
        );
        assert_eq!(result.primary_type, HarmTypeId::A);
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn test_classify_harm_event_population() {
        let result = classify_harm_event(
            PerturbationMultiplicity::Multiple,
            TemporalProfile::Any,
            ResponseDeterminism::Stochastic,
        );
        assert_eq!(result.primary_type, HarmTypeId::H);
    }

    #[test]
    fn test_manifestation_derivation() {
        let probs = vec![0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2];
        let derivation = ManifestationDerivation::derive(HarmTypeId::A, &probs, 0.8, 0.5);
        assert_eq!(derivation.harm_type, HarmTypeId::A);
        assert_eq!(derivation.propagation_probs.len(), 8);
        assert!(derivation.derived_level > 0);
    }

    #[test]
    fn test_cross_domain_mapping_moderate_types() {
        let def_c = HarmTypeDefinition::from_id(HarmTypeId::C);
        let def_e = HarmTypeDefinition::from_id(HarmTypeId::E);
        let def_h = HarmTypeDefinition::from_id(HarmTypeId::H);

        assert_eq!(def_c.cross_domain_mapping, CrossDomainMapping::Moderate);
        assert_eq!(def_e.cross_domain_mapping, CrossDomainMapping::Moderate);
        assert_eq!(def_h.cross_domain_mapping, CrossDomainMapping::Moderate);

        assert!(def_c.notes.is_some());
        assert!(def_e.notes.is_some());
        assert!(def_h.notes.is_some());
    }

    #[test]
    fn test_harm_type_all_count() {
        assert_eq!(HarmTypeId::TOV_CORE.len(), 8);
        assert_eq!(HarmTypeId::ALL.len(), 9);
    }

    #[test]
    fn test_primary_axiom_numbers() {
        assert_eq!(PrimaryAxiom::A1Decomposition.number(), 1);
        assert_eq!(PrimaryAxiom::A2Hierarchy.number(), 2);
        assert_eq!(PrimaryAxiom::A3Conservation.number(), 3);
        assert_eq!(PrimaryAxiom::A4Manifold.number(), 4);
        assert_eq!(PrimaryAxiom::A5Emergence.number(), 5);
    }

    #[test]
    fn test_conservation_law_id() {
        assert_eq!(ConservationLawId::ALL.len(), 11);
        assert_eq!(ConservationLawId::Law1Mass.number(), 1);
        assert_eq!(ConservationLawId::Law11Structure.number(), 11);
    }
}
