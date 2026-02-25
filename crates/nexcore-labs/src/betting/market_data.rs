//! Live Market Data Ingestion
//!
//! Provides real-time odds and market depth from Polymarket and sports providers.
//!
//! # Codex Compliance
//! - **Tier**: T3 (System Service)
//! - **Commandments**: I (Quantify), II (Classify), IX (Measure)

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::exchange::MarketId;

/// Quantification of odds.
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Odds(pub f64);

/// Side of the order book.
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketSide {
    Bid = 0,
    Ask = 1,
}

/// A single price level in the order book.
///
/// # Tier: T2-P
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub size: f64,
}

/// Full order book for a market outcome.
///
/// # Tier: T2-C
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrderBook {
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

/// Snapshot of a market's current state.
///
/// # Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub market_id: MarketId,
    pub price_yes: Odds,
    pub price_no: Odds,
    pub order_book: OrderBook,
    pub liquidity: f64,
    pub last_trade_at: DateTime,
}

/// Service for ingesting and caching live market data.
///
/// # Tier: T3
pub struct MarketDataService {
    /// Active snapshots indexed by market ID.
    pub cache: HashMap<MarketId, MarketSnapshot>,
    pub last_sync: DateTime,
}

impl MarketDataService {
    /// Create a new empty service.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            last_sync: DateTime::now(),
        }
    }

    /// Update a market snapshot.
    pub fn update_market(&mut self, snapshot: MarketSnapshot) {
        self.cache.insert(snapshot.market_id.clone(), snapshot);
        self.last_sync = DateTime::now();
    }

    /// Get current odds for a market.
    pub fn get_odds(&self, market_id: &MarketId) -> Option<&MarketSnapshot> {
        self.cache.get(market_id)
    }

    /// Calculate the mid-price of a market.
    pub fn get_mid_price(&self, market_id: &MarketId) -> Option<f64> {
        let snapshot = self.cache.get(market_id)?;
        let best_bid = snapshot.order_book.bids.first()?.price;
        let best_ask = snapshot.order_book.asks.first()?.price;
        Some((best_bid + best_ask) / 2.0)
    }

    /// Check if market data is stale (older than N seconds).
    pub fn is_stale(&self, market_id: &MarketId, max_age_secs: i64) -> bool {
        if let Some(snapshot) = self.cache.get(market_id) {
            let age = DateTime::now() - snapshot.last_trade_at;
            return age.num_seconds() > max_age_secs;
        }
        true
    }
}

impl Default for MarketDataService {
    fn default() -> Self {
        Self::new()
    }
}
