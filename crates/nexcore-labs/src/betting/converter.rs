//! Intelligence Converter
//!
//! Maps Pharmacovigilance signals to Prediction Market conditions.
//!
//! # Codex Compliance
//! - **Tier**: T3 (Domain-Specific Translator)
//! - **Commandments**: I (Quantify), IV (From), V (Wrap)

use super::exchange::{LimitPrice, MarketId, MarketOrder, OrderQuantity, OrderSide};
use super::thresholds::QuantityMultiplier;
use nexcore_signal_types::{SignalMethod, SignalResult};
use serde::{Deserialize, Serialize};

// =============================================================================
// CONSTANTS - T1 Primitives
// =============================================================================

/// Base scale factor for price translation.
const PRICE_SCALE_FACTOR: f64 = 10.0;
/// Minimum allowable limit price.
const MIN_LIMIT_PRICE: u8 = 10;
/// Maximum allowable limit price.
const MAX_LIMIT_PRICE: u8 = 95;

/// Mapping between a clinical signal and a market event.
///
/// # Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMapping {
    pub signal_method: SignalMethod,
    pub market_id: MarketId,
    /// Threshold of signal strength required to trigger an exchange.
    pub activation_threshold: f64,
}

/// Converter for transforming intelligence into resources.
///
/// # Tier: T3
pub struct SignalConverter {
    pub mappings: Vec<SignalMapping>,
}

impl SignalConverter {
    /// Create a new converter with standard mappings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }

    /// Add a mapping to the converter.
    pub fn add_mapping(&mut self, mapping: SignalMapping) {
        self.mappings.push(mapping);
    }

    /// Attempt to convert a clinical signal into a market order.
    ///
    /// # Logic
    /// 1. Finds a mapping for the signal's method and target.
    /// 2. Verifies the signal exceeds the activation threshold.
    /// 3. Computes order quantity based on signal confidence (lower_ci).
    pub fn convert(
        &self,
        signal: &SignalResult,
        market_id: &MarketId,
        base_quantity: u64,
    ) -> Option<MarketOrder> {
        let mapping = self
            .mappings
            .iter()
            .find(|m| m.signal_method == signal.method && m.market_id == *market_id)?;

        if signal.point_estimate < mapping.activation_threshold || !signal.is_signal {
            return None;
        }

        // Higher confidence (lower_ci) -> Higher quantity multiplier
        let confidence_multiplier = (signal.lower_ci / mapping.activation_threshold)
            .max(1.0)
            .min(5.0);
        let quantity = OrderQuantity((base_quantity as f64 * confidence_multiplier) as u64);

        // Price we are willing to pay scales with point estimate
        let price_val = (signal.point_estimate * PRICE_SCALE_FACTOR)
            .clamp(MIN_LIMIT_PRICE as f64, MAX_LIMIT_PRICE as f64) as u8;

        Some(MarketOrder {
            market_id: market_id.clone(),
            side: OrderSide::Buy,
            limit_price: LimitPrice(price_val),
            quantity,
        })
    }

    /// Convert from refined ECS (Edge Confidence Score) directly.
    ///
    /// # Tier: T3
    /// Implementation of the Intelligence -> Resource transformation contract.
    pub fn convert_from_ecs(
        &self,
        market_id: MarketId,
        ecs: &crate::betting::EcsResult,
        base_quantity: u64,
    ) -> Option<MarketOrder> {
        if !ecs.is_actionable {
            return None;
        }

        // Operationalize: SignalStrength -> QuantityMultiplier
        let multiplier: QuantityMultiplier = ecs.signal_strength.into();

        // If multiplier is 0 (Weak/Avoid), do not execute
        if multiplier.0 == 0 {
            return None;
        }

        let quantity = OrderQuantity(base_quantity * multiplier.0);

        // Base price 50c + bonus for edge. Scale 10 (Elite) to ~90c.
        let price_val = (50.0 + (ecs.ecs * 4.0)).clamp(10.0, 99.0) as u8;

        Some(MarketOrder {
            market_id,
            side: OrderSide::Buy,
            limit_price: LimitPrice(price_val),
            quantity,
        })
    }
}

impl Default for SignalConverter {
    fn default() -> Self {
        Self::new()
    }
}
