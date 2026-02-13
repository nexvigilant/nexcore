//! # GroundsTo implementations for nexcore-energy types
//!
//! Connects the token-as-ATP/ADP biochemistry model to the Lex Primitiva type system.
//!
//! ## Dominant Primitive Distribution
//!
//! - `Regime`, `Strategy`, `EnergySystem`, `WasteClass` -- Classification enums ground to
//!   Comparison (kappa) because they partition a continuous domain into discrete categories.
//! - `TokenPool`, `RecyclingRate` -- Quantity (N) dominates because these types are
//!   fundamentally numeric pools tracking resource levels.
//! - `Operation`, `OperationBuilder` -- Mapping (mu) dominates because they map
//!   estimated inputs to strategic decisions.
//! - `EnergyState` -- Full T3 domain dashboard composing all sub-types.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::{
    EnergyState, EnergySystem, Operation, OperationBuilder, RecyclingRate, Regime, Strategy,
    TokenPool, WasteClass,
};

// ---------------------------------------------------------------------------
// Classification enums -- kappa (Comparison) dominant
// ---------------------------------------------------------------------------

/// Regime: T2-P (kappa + partial), dominant kappa
///
/// Metabolic regime classification derived from Energy Charge thresholds.
/// Comparison-dominant: the entire purpose is to compare EC against thresholds
/// (0.85, 0.70, 0.50) and classify into one of four discrete categories.
/// Boundary is secondary because the thresholds define partition boundaries.
impl GroundsTo for Regime {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- EC threshold comparison (>, >=, <)
            LexPrimitiva::Boundary,   // partial -- threshold boundaries between regimes
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// Strategy: T2-P (kappa + mu), dominant kappa
///
/// Recommended execution strategy (Opus/Sonnet/Haiku/Cache/Checkpoint).
/// Comparison-dominant: strategy selection is a multi-way comparison of
/// coupling ratio against MIN_OPUS_COUPLING and MIN_SONNET_COUPLING.
/// Mapping is secondary because the enum maps regime+CR to a model choice.
impl GroundsTo for Strategy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- CR threshold comparison
            LexPrimitiva::Mapping,    // mu -- maps energy state to model selection
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// EnergySystem: T2-P (kappa + mu), dominant kappa
///
/// Three metabolic pathways: Phosphocreatine, Glycolytic, Oxidative.
/// Comparison-dominant: the classification partitions strategies by
/// energy yield and latency characteristics (instant vs. fast vs. sustained).
/// Mapping is secondary: maps Strategy variants to pathway categories.
impl GroundsTo for EnergySystem {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- pathway classification
            LexPrimitiva::Mapping,    // mu -- Strategy -> EnergySystem mapping
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// WasteClass: T2-P (kappa + varsigma + partial), dominant kappa
///
/// Classification of wasted tokens (tAMP sources): futile cycling,
/// uncoupled respiration, heat loss, substrate cycling, retries.
/// Comparison-dominant: fundamentally a discriminant that classifies
/// waste events by cause. State is secondary (waste is a state transition).
/// Boundary is tertiary (each class has a prevention boundary).
impl GroundsTo for WasteClass {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- waste cause classification
            LexPrimitiva::State,      // varsigma -- waste as state transition (ATP->AMP)
            LexPrimitiva::Boundary,   // partial -- prevention thresholds
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

// ---------------------------------------------------------------------------
// Numeric pool types -- N (Quantity) dominant
// ---------------------------------------------------------------------------

/// TokenPool: T2-C (N + varsigma + partial + proportional), dominant N
///
/// The three-pool energy model: tATP (available), tADP (productive spend),
/// tAMP (waste). Conservation law: tATP + tADP + tAMP = constant.
/// Quantity-dominant: the pools are fundamentally three u64 counters.
/// State is secondary (pool mutates via spend/recycle/degrade).
/// Boundary is tertiary (capped-at-available constraints).
/// Irreversibility is quaternary (degrade is a one-way ADP->AMP transition).
impl GroundsTo for TokenPool {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,        // N -- three numeric pools (tATP, tADP, tAMP)
            LexPrimitiva::State,           // varsigma -- mutable pool state
            LexPrimitiva::Boundary,        // partial -- spend capped at available
            LexPrimitiva::Irreversibility, // proportional -- degrade is one-way ADP->AMP
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// RecyclingRate: T2-P (N + proportional + mu), dominant N
///
/// Token recycling metrics: compression_ratio, cache_hit_rate, pattern_reuse_rate.
/// R = compression x cache x reuse. ATP Synthase analog.
/// Quantity-dominant: three f64 ratios clamped to [0, 1] and multiplied.
/// Irreversibility is secondary (recycling reverses the spend direction).
/// Mapping is tertiary (maps rates to recoverable token count).
impl GroundsTo for RecyclingRate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,        // N -- three f64 rates, combined product
            LexPrimitiva::Irreversibility, // proportional -- recovery direction
            LexPrimitiva::Mapping,         // mu -- rates -> recoverable tokens
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Operation types -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// Operation: T2-C (mu + N + kappa + exists), dominant mu
///
/// An operation being considered, with estimated cost and value.
/// Mapping-dominant: the type maps estimated inputs (cost, value) to a
/// coupling ratio that drives strategy selection via the decide() function.
/// Quantity is secondary (cost and value are numeric).
/// Comparison is tertiary (coupling_ratio enables threshold comparison).
/// Existence is quaternary (cache_possible tests whether a cache entry exists).
impl GroundsTo for Operation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- maps cost+value to coupling ratio
            LexPrimitiva::Quantity,   // N -- estimated_cost, estimated_value
            LexPrimitiva::Comparison, // kappa -- coupling_ratio comparison
            LexPrimitiva::Existence,  // exists -- cache_possible flag
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// OperationBuilder: T2-C (mu + varsigma + sigma + N), dominant mu
///
/// Fluent builder for Operation. Accumulates parameters via method chaining.
/// Mapping-dominant: the builder maps partial configuration into a complete
/// Operation via the build() method (A -> B transformation).
/// State is secondary (mutable accumulation of settings).
/// Sequence is tertiary (method chain ordering).
/// Quantity is quaternary (cost/value are numeric fields).
impl GroundsTo for OperationBuilder {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- builder pattern: partial config -> Operation
            LexPrimitiva::State,    // varsigma -- mutable accumulation
            LexPrimitiva::Sequence, // sigma -- method chain ordering
            LexPrimitiva::Quantity, // N -- numeric fields (cost, value)
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ---------------------------------------------------------------------------
// Domain state -- T3 composite
// ---------------------------------------------------------------------------

/// EnergyState: T3 (varsigma + N + kappa + mu + partial + proportional), dominant varsigma
///
/// Complete energy state snapshot for monitoring and decision-making.
/// Composes TokenPool, Regime, Strategy, EnergySystem, and derived metrics.
/// State-dominant: this is the real-time dashboard -- a snapshot of the entire
/// energy system at a point in time. All other primitives serve to describe
/// what that state contains (quantities, comparisons, mappings, boundaries).
impl GroundsTo for EnergyState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,           // varsigma -- system-wide state snapshot
            LexPrimitiva::Quantity,        // N -- EC, waste_ratio, burn_rate, coupling
            LexPrimitiva::Comparison,      // kappa -- regime classification
            LexPrimitiva::Mapping,         // mu -- strategy recommendation
            LexPrimitiva::Boundary,        // partial -- regime thresholds
            LexPrimitiva::Irreversibility, // proportional -- waste is irreversible
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // =============================
    // Tier Classification Tests
    // =============================

    #[test]
    fn regime_is_t2p() {
        // 2 primitives (Comparison, Boundary) = T2-P
        assert_eq!(Regime::tier(), Tier::T2Primitive);
    }

    #[test]
    fn strategy_is_t2p() {
        // 2 primitives (Comparison, Mapping) = T2-P
        assert_eq!(Strategy::tier(), Tier::T2Primitive);
    }

    #[test]
    fn energy_system_is_t2p() {
        // 2 primitives (Comparison, Mapping) = T2-P
        assert_eq!(EnergySystem::tier(), Tier::T2Primitive);
    }

    #[test]
    fn waste_class_is_t2p() {
        // 3 primitives (Comparison, State, Boundary) = T2-P
        assert_eq!(WasteClass::tier(), Tier::T2Primitive);
    }

    #[test]
    fn token_pool_is_t2c() {
        // 4 primitives (Quantity, State, Boundary, Irreversibility) = T2-C
        assert_eq!(TokenPool::tier(), Tier::T2Composite);
    }

    #[test]
    fn recycling_rate_is_t2p() {
        // 3 primitives (Quantity, Irreversibility, Mapping) = T2-P
        assert_eq!(RecyclingRate::tier(), Tier::T2Primitive);
    }

    #[test]
    fn operation_is_t2c() {
        // 4 primitives (Mapping, Quantity, Comparison, Existence) = T2-C
        assert_eq!(Operation::tier(), Tier::T2Composite);
    }

    #[test]
    fn operation_builder_is_t2c() {
        // 4 primitives (Mapping, State, Sequence, Quantity) = T2-C
        assert_eq!(OperationBuilder::tier(), Tier::T2Composite);
    }

    #[test]
    fn energy_state_is_t3() {
        // 6 primitives (State, Quantity, Comparison, Mapping, Boundary, Irreversibility) = T3
        assert_eq!(EnergyState::tier(), Tier::T3DomainSpecific);
    }

    // =============================
    // Dominant Primitive Tests
    // =============================

    #[test]
    fn regime_dominant_is_comparison() {
        let comp = Regime::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn strategy_dominant_is_comparison() {
        let comp = Strategy::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn energy_system_dominant_is_comparison() {
        let comp = EnergySystem::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn waste_class_dominant_is_comparison() {
        let comp = WasteClass::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn token_pool_dominant_is_quantity() {
        let comp = TokenPool::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn recycling_rate_dominant_is_quantity() {
        let comp = RecyclingRate::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn operation_dominant_is_mapping() {
        let comp = Operation::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn operation_builder_dominant_is_mapping() {
        let comp = OperationBuilder::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn energy_state_dominant_is_state() {
        let comp = EnergyState::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    // =============================
    // Composition Content Tests
    // =============================

    #[test]
    fn token_pool_contains_state_and_boundary() {
        let comp = TokenPool::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::State));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
    }

    #[test]
    fn operation_contains_existence_for_cache() {
        let comp = Operation::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
    }

    #[test]
    fn energy_state_contains_all_six_primitives() {
        let comp = EnergyState::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::State));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Irreversibility));
    }

    #[test]
    fn waste_class_contains_state_transition() {
        let comp = WasteClass::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::State));
    }

    // =============================
    // Confidence Tests
    // =============================

    #[test]
    fn all_confidences_in_valid_range() {
        let types_and_confidences = [
            Regime::primitive_composition().confidence,
            Strategy::primitive_composition().confidence,
            EnergySystem::primitive_composition().confidence,
            WasteClass::primitive_composition().confidence,
            TokenPool::primitive_composition().confidence,
            RecyclingRate::primitive_composition().confidence,
            Operation::primitive_composition().confidence,
            OperationBuilder::primitive_composition().confidence,
            EnergyState::primitive_composition().confidence,
        ];
        for conf in types_and_confidences {
            assert!(conf >= 0.80, "Confidence {conf} below 0.80");
            assert!(conf <= 0.95, "Confidence {conf} above 0.95");
        }
    }

    // =============================
    // Cross-checks: tier matches primitive count
    // =============================

    #[test]
    fn tier_matches_unique_primitive_count() {
        // T2-P: 2-3 unique primitives
        assert!(Regime::primitive_composition().unique().len() <= 3);
        assert!(Strategy::primitive_composition().unique().len() <= 3);
        assert!(EnergySystem::primitive_composition().unique().len() <= 3);
        assert!(WasteClass::primitive_composition().unique().len() <= 3);
        assert!(RecyclingRate::primitive_composition().unique().len() <= 3);

        // T2-C: 4-5 unique primitives
        let tp_len = TokenPool::primitive_composition().unique().len();
        assert!(tp_len >= 4 && tp_len <= 5);

        let op_len = Operation::primitive_composition().unique().len();
        assert!(op_len >= 4 && op_len <= 5);

        let ob_len = OperationBuilder::primitive_composition().unique().len();
        assert!(ob_len >= 4 && ob_len <= 5);

        // T3: 6+ unique primitives
        assert!(EnergyState::primitive_composition().unique().len() >= 6);
    }
}
