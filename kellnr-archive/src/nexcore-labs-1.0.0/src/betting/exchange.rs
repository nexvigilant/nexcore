//! Polymarket Exchange Primitives
//!
//! Implements the Omega (Ω) Exchange primitive, converting intelligence (S)
//! into prediction market resources (Ω).
//!
//! # Codex Compliance
//! - **Tier**: T2-C / T3
//! - **Commandments**: I (Quantify), II (Classify), IV (From), V (Wrap)

use serde::{Deserialize, Serialize};
use std::fmt;

use super::ecs::EcsResult;
use super::thresholds::SignalStrength;

/// Quantification of order side.
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    /// Buying the "Yes" outcome or Long position.
    Buy = 1,
    /// Selling the position or buying the "No" outcome.
    Sell = 2,
}

impl fmt::Display for OrderSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Buy => write!(f, "BUY"),
            Self::Sell => write!(f, "SELL"),
        }
    }
}

/// Identifier for a Polymarket condition/market.
///
/// # Tier: T2-P
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MarketId(pub String);

/// Quantity of shares to exchange.
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderQuantity(pub u64);

/// Limit price in cents (0-100).
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LimitPrice(pub u8);

impl From<u8> for LimitPrice {
    fn from(val: u8) -> Self {
        Self(val.clamp(0, 100))
    }
}

/// A structured order for the Polymarket exchange.
///
/// # Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOrder {
    /// The target market.
    pub market_id: MarketId,
    /// Direction of exchange.
    pub side: OrderSide,
    /// Price ceiling/floor.
    pub limit_price: LimitPrice,
    /// Magnitude of exchange.
    pub quantity: OrderQuantity,
}

/// Conversion from Intelligence (ECS) to Resource (Order).
///
/// This implements the operationalization contract of the framework.
///
/// # Logic
/// - Elite Signals (ECS > 5) -> High conviction, larger quantity.
/// - Strong Signals (ECS 3-5) -> Medium conviction.
/// - Moderate Signals (ECS 2-3) -> Minimum viable quantity.
impl MarketOrder {
    /// Construct an order from a detected signal and market context.
    pub fn from_signal(market_id: MarketId, ecs: &EcsResult, base_quantity: u64) -> Option<Self> {
        if !ecs.is_actionable {
            return None;
        }

        let multiplier = match ecs.signal_strength {
            SignalStrength::Elite => 5,
            SignalStrength::Strong => 2,
            SignalStrength::Moderate => 1,
            _ => return None,
        };

        let quantity = OrderQuantity(base_quantity * multiplier);

        // Use lower_credibility as a proxy for the price floor we are willing to pay
        // Scale ecs (0-10) to price range (e.g., 50-90 cents)
        let price_val = (50.0 + (ecs.ecs * 4.0)).clamp(1.0, 99.0) as u8;

        Some(Self {
            market_id,
            side: OrderSide::Buy,
            limit_price: LimitPrice(price_val),
            quantity,
        })
    }
}

/// Result of an exchange attempt.
///
/// # Tier: T3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeResult {
    pub order_id: String,
    pub status: ExchangeStatus,
    pub filled_quantity: OrderQuantity,
    pub avg_price: LimitPrice,
}

/// Status of the exchange event.
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExchangeStatus {
    Pending = 0,
    Filled = 1,
    PartiallyFilled = 2,
    Cancelled = 3,
    Rejected = 4,
}
