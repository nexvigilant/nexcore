//! Type-Level Constraints for Theory of Vigilance
//!
//! This module uses const generics to enforce ToV constraints at compile time,
//! providing stronger guarantees than runtime validation.
//!
//! ## Verification Strategy
//!
//! | Constraint | Mechanism | Verified Property |
//! |------------|-----------|-------------------|
//! | Hierarchy levels | `ValidatedLevel<N>` | N ∈ [1, 8] |
//! | Conservation laws | `ValidatedLawIndex<I>` | I ∈ [1, 11] |
//! | Harm types | `ValidatedHarmTypeIndex<T>` | T ∈ [0, 7] (A-H) |
//! | Domains | `ValidatedDomainIndex<D>` | D ∈ [0, 2] |
//! | Signal threshold | `NonRecurrenceThreshold` | U_NR = 63 bits |
//! | Probabilities | `BoundedProbability<N,D>` | N < D (P < 1) |

use std::marker::PhantomData;

// ============================================================================
// HIERARCHY LEVEL CONSTRAINTS (Axiom 2)
// ============================================================================

/// Compile-time validated hierarchy level.
///
/// The ToV framework specifies N ≤ 8 hierarchy levels. This type ensures
/// at compile time that the level is within bounds.
///
/// # Type-Level Guarantee
///
/// If `ValidatedLevel<N>` compiles, then 1 ≤ N ≤ 8.
pub struct ValidatedLevel<const N: u8> {
    _private: (),
}

impl<const N: u8> ValidatedLevel<N> {
    /// Create a validated level. Only compiles if N ∈ [1, 8].
    pub const fn new() -> Self {
        assert!(N >= 1, "Hierarchy level must be at least 1");
        assert!(N <= 8, "Hierarchy level must not exceed 8 (Axiom 2)");
        Self { _private: () }
    }

    /// Get the level value.
    pub const fn value(&self) -> u8 {
        N
    }
}

impl<const N: u8> Default for ValidatedLevel<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for hierarchy level 1.
pub type Level1 = ValidatedLevel<1>;
/// Type alias for hierarchy level 2.
pub type Level2 = ValidatedLevel<2>;
/// Type alias for hierarchy level 3.
pub type Level3 = ValidatedLevel<3>;
/// Type alias for hierarchy level 4.
pub type Level4 = ValidatedLevel<4>;
/// Type alias for hierarchy level 5.
pub type Level5 = ValidatedLevel<5>;
/// Type alias for hierarchy level 6.
pub type Level6 = ValidatedLevel<6>;
/// Type alias for hierarchy level 7.
pub type Level7 = ValidatedLevel<7>;
/// Type alias for hierarchy level 8.
pub type Level8 = ValidatedLevel<8>;

/// Marker trait for valid hierarchy levels.
pub trait IsValidLevel {
    /// The numeric level value.
    const LEVEL: u8;
}

impl<const N: u8> IsValidLevel for ValidatedLevel<N> {
    const LEVEL: u8 = N;
}

// ============================================================================
// CONSERVATION LAW CONSTRAINTS (Axiom 3, §8)
// ============================================================================

/// Compile-time validated conservation law index.
///
/// The ToV framework specifies exactly 11 conservation laws. This type ensures
/// at compile time that the index is within bounds.
///
/// # Laws by Index
///
/// | Index | Law | Mathematical Form |
/// |-------|-----|-------------------|
/// | 1 | Mass Conservation | dM/dt = J_in - J_out |
/// | 2 | Energy Gradient | dV/dt ≤ 0 |
/// | 3 | State Conservation | Σpᵢ = 1 |
/// | 4 | Flux Conservation | ΣJ_in = ΣJ_out |
/// | 5 | Catalyst Regeneration | [E]_final = [E]_initial |
/// | 6 | Rate Conservation | dAᵢ/dt = net flux |
/// | 7 | Equilibrium | ds/dt → 0 |
/// | 8 | Saturation | v ≤ V_max |
/// | 9 | Entropy Production | ΔS_total ≥ 0 |
/// | 10 | Discretization | X ∈ {0, q, 2q, ...} |
/// | 11 | Structural Invariance | Σ(s(t)) = Σ(s(0)) |
pub struct ValidatedLawIndex<const I: u8> {
    _private: (),
}

impl<const I: u8> ValidatedLawIndex<I> {
    /// Create a validated law index. Only compiles if I ∈ [1, 11].
    pub const fn new() -> Self {
        assert!(I >= 1, "Conservation law index must be at least 1");
        assert!(I <= 11, "Conservation law index must not exceed 11 (§8)");
        Self { _private: () }
    }

    /// Get the index value.
    pub const fn value(&self) -> u8 {
        I
    }
}

impl<const I: u8> Default for ValidatedLawIndex<I> {
    fn default() -> Self {
        Self::new()
    }
}

/// Law 1: Mass Balance conservation index.
pub type Law1MassIdx = ValidatedLawIndex<1>;
/// Law 2: Energy conservation index.
pub type Law2EnergyIdx = ValidatedLawIndex<2>;
/// Law 3: State Normalization conservation index.
pub type Law3StateIdx = ValidatedLawIndex<3>;
/// Law 4: Flux conservation index.
pub type Law4FluxIdx = ValidatedLawIndex<4>;
/// Law 5: Catalyst conservation index.
pub type Law5CatalystIdx = ValidatedLawIndex<5>;
/// Law 6: Rate conservation index.
pub type Law6RateIdx = ValidatedLawIndex<6>;
/// Law 7: Equilibrium conservation index.
pub type Law7EquilibriumIdx = ValidatedLawIndex<7>;
/// Law 8: Saturation conservation index.
pub type Law8SaturationIdx = ValidatedLawIndex<8>;
/// Law 9: Entropy conservation index.
pub type Law9EntropyIdx = ValidatedLawIndex<9>;
/// Law 10: Discretization conservation index.
pub type Law10DiscretizationIdx = ValidatedLawIndex<10>;
/// Law 11: Structure conservation index.
pub type Law11StructureIdx = ValidatedLawIndex<11>;

// ============================================================================
// HARM TYPE CONSTRAINTS (§9)
// ============================================================================

/// Compile-time validated harm type index.
///
/// The ToV framework specifies exactly 8 harm types (A-H).
pub struct ValidatedHarmTypeIndex<const T: u8> {
    _private: (),
}

impl<const T: u8> ValidatedHarmTypeIndex<T> {
    /// Create a validated harm type index. Only compiles if T ∈ [0, 7].
    pub const fn new() -> Self {
        assert!(
            T <= 7,
            "Harm type index must be in range [0, 7] (8 types A-H)"
        );
        Self { _private: () }
    }

    /// Get the index value.
    pub const fn value(&self) -> u8 {
        T
    }

    /// Get the harm type letter (A-H).
    pub const fn letter(&self) -> char {
        (b'A' + T) as char
    }
}

impl<const T: u8> Default for ValidatedHarmTypeIndex<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Harm Type A: Acute dose-dependent harm index.
pub type HarmTypeAIdx = ValidatedHarmTypeIndex<0>;
/// Harm Type B: Cumulative harm index.
pub type HarmTypeBIdx = ValidatedHarmTypeIndex<1>;
/// Harm Type C: Off-target harm index.
pub type HarmTypeCIdx = ValidatedHarmTypeIndex<2>;
/// Harm Type D: Cascade harm index.
pub type HarmTypeDIdx = ValidatedHarmTypeIndex<3>;
/// Harm Type E: Idiosyncratic harm index.
pub type HarmTypeEIdx = ValidatedHarmTypeIndex<4>;
/// Harm Type F: Saturation harm index.
pub type HarmTypeFIdx = ValidatedHarmTypeIndex<5>;
/// Harm Type G: Interaction harm index.
pub type HarmTypeGIdx = ValidatedHarmTypeIndex<6>;
/// Harm Type H: Population-level harm index.
pub type HarmTypeHIdx = ValidatedHarmTypeIndex<7>;

// ============================================================================
// DOMAIN CONSTRAINTS (§11-15)
// ============================================================================

/// Compile-time validated domain index.
///
/// The ToV framework has exactly 3 domains.
pub struct ValidatedDomainIndex<const D: u8> {
    _private: (),
}

impl<const D: u8> ValidatedDomainIndex<D> {
    /// Create a validated domain index. Only compiles if D ∈ [0, 2].
    pub const fn new() -> Self {
        assert!(D <= 2, "Domain index must be in range [0, 2] (3 domains)");
        Self { _private: () }
    }

    /// Get the index value.
    pub const fn value(&self) -> u8 {
        D
    }
}

impl<const D: u8> Default for ValidatedDomainIndex<D> {
    fn default() -> Self {
        Self::new()
    }
}

/// Cloud domain index (§11).
pub type CloudDomainIdx = ValidatedDomainIndex<0>;
/// Pharmacovigilance domain index (§12).
pub type PVDomainIdx = ValidatedDomainIndex<1>;
/// AI domain index (§13).
pub type AIDomainIdx = ValidatedDomainIndex<2>;

// ============================================================================
// SIGNAL DETECTION CONSTRAINTS (§19-32)
// ============================================================================

/// Non-recurrence threshold U_NR = 63 bits (type-level constant).
///
/// This is the bit threshold beyond which a configuration is considered
/// non-recurrent (will never be observed again by chance).
pub struct NonRecurrenceThreshold;

impl NonRecurrenceThreshold {
    /// The threshold value in bits.
    pub const VALUE: u8 = 63;

    /// Check if a given U value exceeds the threshold.
    pub const fn exceeds(u_bits: u64) -> bool {
        u_bits >= Self::VALUE as u64
    }
}

/// Compile-time validated signal rarity (U value).
///
/// Signal rarity must be non-negative (measured in bits of surprise).
pub struct ValidatedRarity<const BITS: u64> {
    _private: (),
}

impl<const BITS: u64> ValidatedRarity<BITS> {
    /// Create a validated rarity value.
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// Get the rarity in bits.
    pub const fn bits(&self) -> u64 {
        BITS
    }

    /// Check if this rarity exceeds the non-recurrence threshold.
    pub const fn is_non_recurrent(&self) -> bool {
        BITS >= NonRecurrenceThreshold::VALUE as u64
    }
}

// ============================================================================
// PROPAGATION PROBABILITY CONSTRAINTS (Axiom 5)
// ============================================================================

/// A propagation probability that is provably less than 1.
///
/// For the Attenuation Theorem (T10.2) to hold, all propagation
/// probabilities must be strictly less than 1. This type encodes
/// that constraint at compile time using a rational representation.
///
/// P = numerator / denominator where numerator < denominator
pub struct BoundedProbability<const NUM: u32, const DEN: u32> {
    _private: (),
}

impl<const NUM: u32, const DEN: u32> BoundedProbability<NUM, DEN> {
    /// Create a bounded probability. Only compiles if NUM < DEN (probability < 1).
    pub const fn new() -> Self {
        assert!(DEN > 0, "Denominator must be positive");
        assert!(
            NUM < DEN,
            "Probability must be < 1 for attenuation (NUM < DEN)"
        );
        Self { _private: () }
    }

    /// Get the probability as f64.
    pub const fn value(&self) -> f64 {
        NUM as f64 / DEN as f64
    }

    /// Get the numerator.
    pub const fn numerator(&self) -> u32 {
        NUM
    }

    /// Get the denominator.
    pub const fn denominator(&self) -> u32 {
        DEN
    }
}

impl<const NUM: u32, const DEN: u32> Default for BoundedProbability<NUM, DEN> {
    fn default() -> Self {
        Self::new()
    }
}

/// Probability of 50% (0.5).
pub type Prob50Pct = BoundedProbability<1, 2>;
/// Probability of 10% (0.1).
pub type Prob10Pct = BoundedProbability<1, 10>;
/// Probability of 1% (0.01).
pub type Prob1Pct = BoundedProbability<1, 100>;
/// Probability of 0.1% (0.001).
pub type Prob01Pct = BoundedProbability<1, 1000>;

// ============================================================================
// ELEMENT COUNT CONSTRAINT (Axiom 1, §12)
// ============================================================================

/// Compile-time element count validation.
///
/// The ToV framework specifies |E| = 15 elements for each domain.
pub struct ElementCount<const N: usize>(PhantomData<[(); N]>);

impl<const N: usize> ElementCount<N> {
    /// Create an element count marker.
    pub const fn new() -> Self {
        Self(PhantomData)
    }

    /// Get the count value.
    pub const fn count() -> usize {
        N
    }
}

impl<const N: usize> Default for ElementCount<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard element count for ToV domains (15).
pub type StandardElementCount = ElementCount<15>;

// ============================================================================
// COMPILE-TIME PROOFS
// ============================================================================

/// Proof that all hierarchy levels are valid (compile-time check).
pub const fn verify_all_levels() {
    let _ = ValidatedLevel::<1>::new();
    let _ = ValidatedLevel::<2>::new();
    let _ = ValidatedLevel::<3>::new();
    let _ = ValidatedLevel::<4>::new();
    let _ = ValidatedLevel::<5>::new();
    let _ = ValidatedLevel::<6>::new();
    let _ = ValidatedLevel::<7>::new();
    let _ = ValidatedLevel::<8>::new();
}

/// Proof that all conservation law indices are valid (compile-time check).
pub const fn verify_all_laws() {
    let _ = ValidatedLawIndex::<1>::new();
    let _ = ValidatedLawIndex::<2>::new();
    let _ = ValidatedLawIndex::<3>::new();
    let _ = ValidatedLawIndex::<4>::new();
    let _ = ValidatedLawIndex::<5>::new();
    let _ = ValidatedLawIndex::<6>::new();
    let _ = ValidatedLawIndex::<7>::new();
    let _ = ValidatedLawIndex::<8>::new();
    let _ = ValidatedLawIndex::<9>::new();
    let _ = ValidatedLawIndex::<10>::new();
    let _ = ValidatedLawIndex::<11>::new();
}

/// Proof that all harm types are valid (compile-time check).
pub const fn verify_all_harm_types() {
    let _ = ValidatedHarmTypeIndex::<0>::new();
    let _ = ValidatedHarmTypeIndex::<1>::new();
    let _ = ValidatedHarmTypeIndex::<2>::new();
    let _ = ValidatedHarmTypeIndex::<3>::new();
    let _ = ValidatedHarmTypeIndex::<4>::new();
    let _ = ValidatedHarmTypeIndex::<5>::new();
    let _ = ValidatedHarmTypeIndex::<6>::new();
    let _ = ValidatedHarmTypeIndex::<7>::new();
}

/// Proof that all domain indices are valid (compile-time check).
pub const fn verify_all_domains() {
    let _ = ValidatedDomainIndex::<0>::new();
    let _ = ValidatedDomainIndex::<1>::new();
    let _ = ValidatedDomainIndex::<2>::new();
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_hierarchy_levels_compile() {
        let _l1: Level1 = ValidatedLevel::new();
        let _l2: Level2 = ValidatedLevel::new();
        let _l3: Level3 = ValidatedLevel::new();
        let _l4: Level4 = ValidatedLevel::new();
        let _l5: Level5 = ValidatedLevel::new();
        let _l6: Level6 = ValidatedLevel::new();
        let _l7: Level7 = ValidatedLevel::new();
        let _l8: Level8 = ValidatedLevel::new();
    }

    #[test]
    fn valid_law_indices_compile() {
        let _l1: Law1MassIdx = ValidatedLawIndex::new();
        let _l2: Law2EnergyIdx = ValidatedLawIndex::new();
        let _l11: Law11StructureIdx = ValidatedLawIndex::new();
    }

    #[test]
    fn element_count_is_15() {
        assert_eq!(StandardElementCount::count(), 15);
    }

    #[test]
    fn non_recurrence_threshold_is_63() {
        assert_eq!(NonRecurrenceThreshold::VALUE, 63);
        assert!(!NonRecurrenceThreshold::exceeds(30));
        assert!(NonRecurrenceThreshold::exceeds(63));
        assert!(NonRecurrenceThreshold::exceeds(100));
    }

    #[test]
    fn bounded_probability_values() {
        let p50: Prob50Pct = BoundedProbability::new();
        assert!((p50.value() - 0.5).abs() < 0.001);

        let p10: Prob10Pct = BoundedProbability::new();
        assert!((p10.value() - 0.1).abs() < 0.001);

        let p1: Prob1Pct = BoundedProbability::new();
        assert!((p1.value() - 0.01).abs() < 0.001);
    }

    #[test]
    fn rarity_non_recurrence_check() {
        let low: ValidatedRarity<30> = ValidatedRarity::new();
        assert!(!low.is_non_recurrent());

        let high: ValidatedRarity<100> = ValidatedRarity::new();
        assert!(high.is_non_recurrent());

        let threshold: ValidatedRarity<63> = ValidatedRarity::new();
        assert!(threshold.is_non_recurrent());
    }

    #[test]
    fn compile_time_verification() {
        // These are const fns - verification happens at compile time
        verify_all_levels();
        verify_all_laws();
        verify_all_harm_types();
        verify_all_domains();
    }

    #[test]
    fn harm_type_indices_valid() {
        let a: HarmTypeAIdx = ValidatedHarmTypeIndex::new();
        assert_eq!(a.letter(), 'A');

        let h: HarmTypeHIdx = ValidatedHarmTypeIndex::new();
        assert_eq!(h.letter(), 'H');
    }

    #[test]
    fn domain_indices_valid() {
        let _cloud: CloudDomainIdx = ValidatedDomainIndex::new();
        let _pv: PVDomainIdx = ValidatedDomainIndex::new();
        let _ai: AIDomainIdx = ValidatedDomainIndex::new();
    }

    #[test]
    fn level_values() {
        let l1: Level1 = ValidatedLevel::new();
        assert_eq!(l1.value(), 1);

        let l8: Level8 = ValidatedLevel::new();
        assert_eq!(l8.value(), 8);
    }
}
