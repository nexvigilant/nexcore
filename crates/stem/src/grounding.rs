//! # GroundsTo implementations for stem (facade) types
//!
//! Connects the finance module types (defined inline in the stem facade) to
//! the Lex Primitiva type system.
//!
//! ## Scope
//!
//! The `stem` facade re-exports types from `stem-core`, `stem-bio`, `stem-chem`,
//! `stem-phys`, and `stem-math` -- those crates have their own `grounding.rs`.
//! This module grounds the finance types, which are defined inline in the
//! facade (not a separate `stem-finance` crate).
//!
//! ## Finance Primitive Profile
//!
//! The FINANCE composite uses 9 of 16 T1 primitives:
//! N (Appraise), sigma (Flow), arrow (Discount), rho (Compound),
//! partial (Hedge), kappa (Arbitrage), irreversibility (Mature),
//! times (Leverage), Sigma (Diversify).
//!
//! - **Price**: T1 (Quantity) -- monetary value
//! - **Return**: T2-P (Mapping + Quantity) -- proportional change
//! - **InterestRate**: T2-P (Frequency + Quantity) -- periodic rate
//! - **Spread**: T2-P (Boundary + Quantity) -- non-negative price gap
//! - **Maturity**: T2-P (Irreversibility + Quantity) -- countdown to terminal
//! - **Exposure**: T2-P (Sum + Quantity) -- aggregate risk position
//! - **TimeValueOfMoney**: T2-C (Causality + Recursion + Quantity) -- TVM calc
//! - **MeasuredPrice**: T2-P (Quantity + Mapping) -- price with confidence
//! - **MeasuredReturn**: T2-P (Quantity + Mapping) -- return with confidence
//! - **FinanceError**: T1 (Boundary) -- operation failure

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::finance::{
    Exposure, FinanceError, InterestRate, Maturity, MeasuredPrice, MeasuredReturn, Price, Return,
    Spread, TimeValueOfMoney,
};

// ===========================================================================
// Core value types
// ===========================================================================

/// Price: T1 (Quantity), dominant Quantity
///
/// Monetary value clamped to non-negative.
/// Pure quantity: it IS a numeric value representing monetary amount.
impl GroundsTo for Price {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- monetary amount
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.95)
    }
}

/// Return: T2-P (Mapping + Quantity), dominant Mapping
///
/// Proportional change between two prices.
/// Mapping-dominant: it IS a transformation from price pair to proportion.
impl GroundsTo for Return {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- (P0, P1) -> proportion
            LexPrimitiva::Quantity, // N -- numeric return value
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// `InterestRate`: T2-P (Frequency + Quantity), dominant Frequency
///
/// Periodic rate of change -- the price of money over time.
/// Frequency-dominant: it IS a rate per time period (change per period).
impl GroundsTo for InterestRate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // nu -- change per time period
            LexPrimitiva::Quantity,  // N -- numeric rate value
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.85)
    }
}

/// Spread: T2-P (Boundary + Quantity), dominant Boundary
///
/// Non-negative price gap (bid-ask, credit, risk premium).
/// Boundary-dominant: it IS a distance between two price boundaries.
impl GroundsTo for Spread {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- gap between price levels
            LexPrimitiva::Quantity, // N -- numeric spread value
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// Maturity: T2-P (Irreversibility + Quantity), dominant Irreversibility
///
/// Time countdown to terminal event.
/// Irreversibility-dominant: maturity IS one-way progression toward
/// expiry -- once expired, it cannot un-expire.
impl GroundsTo for Maturity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Irreversibility, // proportional -- one-way expiry
            LexPrimitiva::Quantity,        // N -- years to maturity
        ])
        .with_dominant(LexPrimitiva::Irreversibility, 0.85)
    }
}

/// Exposure: T2-P (Sum + Quantity), dominant Sum
///
/// Aggregate risk position (can be long or short).
/// Sum-dominant: exposure IS the aggregation of multiple positions
/// into a net value. Add/Neg ops support portfolio netting.
impl GroundsTo for Exposure {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- aggregate of positions
            LexPrimitiva::Quantity, // N -- numeric exposure value
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// `TimeValueOfMoney`: T2-C (Causality + Recursion + Quantity), dominant Causality
///
/// Standard TVM calculator implementing Discount and Compound traits.
/// Causality-dominant: the core operation IS discounting -- mapping
/// future value to present value via causal time-preference.
impl GroundsTo for TimeValueOfMoney {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // arrow -- future -> present causal map
            LexPrimitiva::Recursion, // rho -- compound interest recursion
            LexPrimitiva::Quantity,  // N -- numeric PV/FV values
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

// ===========================================================================
// Measured types
// ===========================================================================

/// `MeasuredPrice`: T2-P (Quantity + Mapping), dominant Quantity
///
/// A price paired with confidence. Quantity-dominant: two numeric
/// values (price + confidence) are the primary content.
impl GroundsTo for MeasuredPrice {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- price value + confidence
            LexPrimitiva::Mapping,  // mu -- asset -> measured price
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// `MeasuredReturn`: T2-P (Quantity + Mapping), dominant Quantity
///
/// A return paired with confidence. Quantity-dominant: two numeric
/// values (return + confidence) are the primary content.
impl GroundsTo for MeasuredReturn {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- return value + confidence
            LexPrimitiva::Mapping,  // mu -- price pair -> measured return
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ===========================================================================
// Error type
// ===========================================================================

/// `FinanceError`: T1 (Boundary), dominant Boundary
///
/// Error enum for financial operations. Pure boundary: each variant
/// represents a failed financial constraint.
impl GroundsTo for FinanceError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- financial constraint failure
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

// ===========================================================================
// Chemistry domain types (ported from stem-chem grounding.rs)
// ===========================================================================

use crate::chem::{
    Affinity, Balance, ChemistryError, Fraction, MeasuredRate, MeasuredRatio, Rate, Ratio,
};

/// Ratio: T2-P (Quantity + Mapping), dominant Quantity
impl GroundsTo for Ratio {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric ratio value
            LexPrimitiva::Mapping,  // mu -- substance -> ratio transformation
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// Fraction: T2-P (Quantity + Boundary), dominant Quantity
impl GroundsTo for Fraction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric proportion value
            LexPrimitiva::Boundary, // partial -- [0,1] clamping
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// Rate: T1 (Mapping), dominant Mapping
impl GroundsTo for Rate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping, // mu -- time -> change rate
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.95)
    }
}

/// Affinity: T2-P (Mapping + Boundary), dominant Mapping
impl GroundsTo for Affinity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- interaction -> strength
            LexPrimitiva::Boundary, // partial -- [0,1] clamping
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// Balance: T2-C (State + Quantity + Comparison), dominant State
impl GroundsTo for Balance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- equilibrium state
            LexPrimitiva::Quantity,   // N -- forward rate, reverse rate, constant K
            LexPrimitiva::Comparison, // kappa -- rate comparison for equilibrium check
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// MeasuredRatio: T2-P (Quantity + Mapping), dominant Quantity
impl GroundsTo for MeasuredRatio {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- ratio value + confidence value
            LexPrimitiva::Mapping,  // mu -- substance -> measured ratio
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// MeasuredRate: T2-P (Quantity + Mapping), dominant Quantity
impl GroundsTo for MeasuredRate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- rate value + confidence value
            LexPrimitiva::Mapping,  // mu -- time -> measured rate
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// ChemistryError: T1 (Boundary), dominant Boundary
impl GroundsTo for ChemistryError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- operation failure boundary
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn price_is_t1_quantity_dominant() {
        assert_eq!(Price::tier(), Tier::T1Universal);
        assert_eq!(
            Price::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
        assert!(Price::is_pure_primitive());
    }

    #[test]
    fn return_is_t2p_mapping_dominant() {
        assert_eq!(Return::tier(), Tier::T2Primitive);
        assert_eq!(
            Return::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn interest_rate_is_t2p_frequency_dominant() {
        assert_eq!(InterestRate::tier(), Tier::T2Primitive);
        assert_eq!(
            InterestRate::primitive_composition().dominant,
            Some(LexPrimitiva::Frequency)
        );
    }

    #[test]
    fn spread_is_t2p_boundary_dominant() {
        assert_eq!(Spread::tier(), Tier::T2Primitive);
        assert_eq!(
            Spread::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn maturity_is_t2p_irreversibility_dominant() {
        assert_eq!(Maturity::tier(), Tier::T2Primitive);
        assert_eq!(
            Maturity::primitive_composition().dominant,
            Some(LexPrimitiva::Irreversibility)
        );
    }

    #[test]
    fn exposure_is_t2p_sum_dominant() {
        assert_eq!(Exposure::tier(), Tier::T2Primitive);
        assert_eq!(
            Exposure::primitive_composition().dominant,
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn time_value_of_money_is_t2p_causality_dominant() {
        // 3 primitives = T2-P
        assert_eq!(TimeValueOfMoney::tier(), Tier::T2Primitive);
        assert_eq!(
            TimeValueOfMoney::primitive_composition().dominant,
            Some(LexPrimitiva::Causality)
        );
        let comp = TimeValueOfMoney::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
    }

    #[test]
    fn measured_price_is_t2p_quantity_dominant() {
        assert_eq!(MeasuredPrice::tier(), Tier::T2Primitive);
        assert_eq!(
            MeasuredPrice::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn measured_return_is_t2p_quantity_dominant() {
        assert_eq!(MeasuredReturn::tier(), Tier::T2Primitive);
        assert_eq!(
            MeasuredReturn::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn finance_error_is_t1_boundary_dominant() {
        assert_eq!(FinanceError::tier(), Tier::T1Universal);
        assert_eq!(
            FinanceError::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
        assert!(FinanceError::is_pure_primitive());
    }

    #[test]
    fn tier_distribution_is_reasonable() {
        // T1: Price, FinanceError = 2
        let t1_count = [Price::tier(), FinanceError::tier()]
            .iter()
            .filter(|t| **t == Tier::T1Universal)
            .count();

        // T2-P (2-3 primitives): Return, InterestRate, Spread, Maturity,
        //   Exposure, MeasuredPrice, MeasuredReturn, TimeValueOfMoney = 8
        let t2p_count = [
            Return::tier(),
            InterestRate::tier(),
            Spread::tier(),
            Maturity::tier(),
            Exposure::tier(),
            MeasuredPrice::tier(),
            MeasuredReturn::tier(),
            TimeValueOfMoney::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T2Primitive)
        .count();

        assert_eq!(t1_count, 2, "expected 2 T1 types");
        assert_eq!(t2p_count, 8, "expected 8 T2-P types");
    }

    // Chemistry grounding tests

    #[test]
    fn chem_ratio_is_t2p_quantity_dominant() {
        assert_eq!(Ratio::tier(), Tier::T2Primitive);
        assert_eq!(Ratio::primitive_composition().dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn chem_rate_is_t1_mapping_dominant() {
        assert_eq!(Rate::tier(), Tier::T1Universal);
        assert!(Rate::is_pure_primitive());
    }

    #[test]
    fn chem_balance_is_t2p_state_dominant() {
        assert_eq!(Balance::tier(), Tier::T2Primitive);
        assert_eq!(Balance::primitive_composition().dominant, Some(LexPrimitiva::State));
    }

    #[test]
    fn chem_error_is_t1_boundary_dominant() {
        assert_eq!(ChemistryError::tier(), Tier::T1Universal);
        assert!(ChemistryError::is_pure_primitive());
    }

    #[test]
    fn finance_covers_five_new_t1_primitives() {
        // Finance introduces 5 T1 primitives not covered by other STEM domains:
        // N (Quantity), arrow (Causality), kappa (Comparison via Arbitrage trait),
        // irreversibility (Maturity), times (Product via Leverage trait)
        //
        // We verify the type-level groundings cover at least 3 of these.
        let dominants = [
            Price::dominant_primitive(),            // N
            Maturity::dominant_primitive(),         // irreversibility
            TimeValueOfMoney::dominant_primitive(), // arrow
        ];
        assert_eq!(dominants[0], Some(LexPrimitiva::Quantity));
        assert_eq!(dominants[1], Some(LexPrimitiva::Irreversibility));
        assert_eq!(dominants[2], Some(LexPrimitiva::Causality));
    }
}
