//! # Capability 18: Treasury Act (Revenue/Finance Domain)
//!
//! Implementation of the Treasury Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "Asymmetry Conversion" and "Resource Liquidity" of the Union.
//!
//! Matches 1:1 to the US Department of the Treasury mandate to
//! maintain a strong economy and create economic and job opportunities
//! by promoting the conditions that enable economic growth and stability.
//!
//! ## Treasury Agency Mappings
//! - **IRS (Collection):** Converts signal asymmetry into usable prediction market resources.
//! - **OCC (Banking):** Manages the liquidity and stability of the Union's treasury.
//! - **Mint (Issuance):** Controls the issuance and distribution of Union tokens/credits.
//! - **FinCEN (Intelligence):** Monitors for financial "Heresy" and illicit signal arbitrage.

use crate::primitives::governance::{Odds, Token, Treasury};
use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: TreasuryAct - Capability 18 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryAct {
    /// The unique capability identifier.
    pub id: String,
    /// Whether the revenue/arbitrage engine is currently active.
    pub revenue_engine_active: bool,
}

/// T2-A: AsymmetryValue - The quantified value of a signal's informational edge.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AsymmetryValue(pub f64);

/// T2-L: LiquidityEvent - A conversion of asymmetry into resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityEvent {
    /// The identifier of the signal that provided the asymmetry.
    pub signal_id: String,
    /// The total value captured from the informational edge.
    pub value_captured: f64,
    /// The token reward issued to the Union treasury.
    pub token_reward: Token,
    /// The confidence in the conversion accuracy.
    pub conversion_confidence: Confidence,
}

impl TreasuryAct {
    /// Creates a new instance of the TreasuryAct.
    pub fn new() -> Self {
        Self {
            id: "CAP-018".into(),
            revenue_engine_active: true,
        }
    }

    /// Convert signal asymmetry into resources (Polymarket-style integration).
    pub fn convert_asymmetry(
        &self,
        signal_id: &str,
        asymmetry: AsymmetryValue,
        _market_odds: Odds,
    ) -> Measured<LiquidityEvent> {
        // Simulation of information arbitrage
        // value = asymmetry * (1.0 / market_odds_implied_prob)
        let value = asymmetry.0 * 2.0; // Placeholder multiplier

        let event = LiquidityEvent {
            signal_id: signal_id.to_string(),
            value_captured: value,
            token_reward: Token((value * 100.0) as u64),
            conversion_confidence: Confidence::new(0.85),
        };

        Measured::uncertain(event, Confidence::new(0.92))
    }

    /// Audit the Union's treasury status.
    pub fn audit_treasury(&self, treasury: &Treasury) -> bool {
        treasury.compute_quota > 0 && treasury.memory_quota > 0
    }
}
