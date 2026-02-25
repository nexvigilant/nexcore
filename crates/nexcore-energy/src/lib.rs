//! # NexVigilant Core — Energy
//!
//! Token-as-energy system modeled on ATP/ADP biochemistry for the NexVigilant platform.
//!
//! ## Core Insight (Atkinson, 1968)
//!
//! Cells maintain an **Energy Charge** (EC) between 0 and 1:
//!
//! ```text
//! EC = (ATP + 0.5 * ADP) / (ATP + ADP + AMP)
//! ```
//!
//! This single scalar drives all metabolic decisions. We apply the same
//! principle to AI token budgets:
//!
//! | Pool | Biology | Token Analog |
//! |------|---------|-------------|
//! | **tATP** | ATP (ready energy) | Tokens remaining in budget |
//! | **tADP** | ADP (spent, recyclable) | Tokens spent on productive work |
//! | **tAMP** | AMP (degraded) | Tokens wasted (retries, failures, hallucinations) |
//!
//! ## Metabolic Regimes
//!
//! | EC Range | Regime | Strategy |
//! |----------|--------|----------|
//! | > 0.85 | Anabolic | Invest freely: Opus, deep exploration |
//! | 0.70-0.85 | Homeostatic | Balanced: right-sized models |
//! | 0.50-0.70 | Catabolic | Conserve: Haiku, compress, cache-first |
//! | < 0.50 | Crisis | Checkpoint and stop |
//!
//! ## Transfer Confidence
//!
//! ATP energy metabolism -> token budget management: **0.92**
//! (T2-P cross-domain primitive: both are resource allocation under scarcity)

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![warn(missing_docs)]

pub mod adaptive_thresholds;
pub mod composites;
pub mod grounding;
pub mod prelude;
pub mod primitives;
pub mod pv_gate;
pub mod transfer;

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Constants — Biological thresholds mapped to token management
// ============================================================================

/// Anabolic threshold: EC above this = invest freely.
/// Biology: cells shift to growth/storage above EC ~0.85.
pub const EC_ANABOLIC: f64 = 0.85;

/// Homeostatic floor: EC above this = normal operation.
/// Biology: cells maintain steady state in [0.70, 0.85].
pub const EC_HOMEOSTATIC: f64 = 0.70;

/// Catabolic floor: EC above this = conserve energy.
/// Biology: cells break down reserves below 0.70.
pub const EC_CATABOLIC: f64 = 0.50;

/// ADP weighting in energy charge formula.
/// Biology: ADP is "half-charged" — it produced value but needs recycling.
pub const ADP_WEIGHT: f64 = 0.5;

/// Minimum coupling ratio to justify Opus in homeostatic regime.
/// If value/cost < this, downgrade to cheaper model.
pub const MIN_OPUS_COUPLING: f64 = 2.0;

/// Minimum coupling ratio to justify Sonnet in homeostatic regime.
pub const MIN_SONNET_COUPLING: f64 = 1.5;

// ============================================================================
// Regime — Metabolic state classification
// ============================================================================

/// Metabolic regime derived from Energy Charge.
///
/// Tier: T2-P (kappa -- Comparison/Classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Regime {
    /// EC > 0.85: Growth mode. Invest in complex operations.
    Anabolic,
    /// 0.70 <= EC <= 0.85: Steady state. Balanced operation.
    Homeostatic,
    /// 0.50 <= EC < 0.70: Conservation mode. Minimize waste.
    Catabolic,
    /// EC < 0.50: Emergency. Checkpoint and halt.
    Crisis,
}

impl Regime {
    /// Classify EC into a regime.
    #[must_use]
    pub fn from_ec(ec: f64) -> Self {
        if ec > EC_ANABOLIC {
            Self::Anabolic
        } else if ec >= EC_HOMEOSTATIC {
            Self::Homeostatic
        } else if ec >= EC_CATABOLIC {
            Self::Catabolic
        } else {
            Self::Crisis
        }
    }

    /// Human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Anabolic => "Anabolic (invest)",
            Self::Homeostatic => "Homeostatic (balanced)",
            Self::Catabolic => "Catabolic (conserve)",
            Self::Crisis => "Crisis (checkpoint)",
        }
    }

    /// Whether this regime allows expensive operations.
    #[must_use]
    pub const fn allows_expensive(&self) -> bool {
        matches!(self, Self::Anabolic | Self::Homeostatic)
    }
}

impl fmt::Display for Regime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

// ============================================================================
// Strategy — Model/action selection
// ============================================================================

/// Recommended execution strategy based on energy state.
///
/// Tier: T2-P (kappa -- Comparison/Classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Strategy {
    /// Use the most capable model. Deep exploration permitted.
    Opus,
    /// Use a balanced model. Standard operations.
    Sonnet,
    /// Use the cheapest model. Fast, shallow operations.
    Haiku,
    /// Check cache before any model call.
    HaikuCacheFirst,
    /// Save state and stop. No more API calls.
    Checkpoint,
}

impl Strategy {
    /// Human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Opus => "Opus (full power)",
            Self::Sonnet => "Sonnet (balanced)",
            Self::Haiku => "Haiku (efficient)",
            Self::HaikuCacheFirst => "Haiku+Cache (conserve)",
            Self::Checkpoint => "Checkpoint (halt)",
        }
    }

    /// Relative token cost multiplier (Haiku = 1.0 baseline).
    #[must_use]
    pub const fn cost_multiplier(&self) -> f64 {
        match self {
            Self::Opus => 15.0,
            Self::Sonnet => 5.0,
            Self::Haiku => 1.0,
            Self::HaikuCacheFirst => 0.5, // cache hits are free
            Self::Checkpoint => 0.0,
        }
    }
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

// ============================================================================
// EnergySystem — The three metabolic pathways
// ============================================================================

/// The three energy systems (from exercise physiology).
///
/// Tier: T2-P (kappa -- Classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnergySystem {
    /// Phosphocreatine: instant energy from cache.
    /// Biology: PCr + ADP -> Cr + ATP (< 10 seconds).
    Phosphocreatine,
    /// Glycolytic: fast energy from cheap models.
    /// Biology: Glucose -> 2 ATP + Lactate (10s - 2min).
    Glycolytic,
    /// Oxidative: sustained energy from expensive models.
    /// Biology: Glucose -> 36 ATP via Krebs + ETC (> 2min).
    Oxidative,
}

impl EnergySystem {
    /// Map a strategy to its energy system.
    #[must_use]
    pub const fn for_strategy(strategy: Strategy) -> Self {
        match strategy {
            Strategy::HaikuCacheFirst | Strategy::Checkpoint => Self::Phosphocreatine, // no energy spent
            Strategy::Haiku | Strategy::Sonnet => Self::Glycolytic,
            Strategy::Opus => Self::Oxidative,
        }
    }

    /// ATP yield per glucose equivalent (efficiency ranking).
    #[must_use]
    pub const fn yield_per_unit(&self) -> f64 {
        match self {
            Self::Phosphocreatine => 1.0, // instant but limited
            Self::Glycolytic => 2.0,      // fast, low yield
            Self::Oxidative => 36.0,      // slow, high yield
        }
    }
}

impl fmt::Display for EnergySystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Phosphocreatine => f.write_str("Phosphocreatine (cache)"),
            Self::Glycolytic => f.write_str("Glycolytic (fast)"),
            Self::Oxidative => f.write_str("Oxidative (sustained)"),
        }
    }
}

// ============================================================================
// WasteClass — tAMP classification
// ============================================================================

/// Classification of wasted tokens (tAMP sources).
///
/// Tier: T2-C (kappa + varsigma -- Classification + State)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WasteClass {
    /// Futile cycling: read -> rejected edit -> re-read -> re-edit.
    FutileCycling,
    /// Uncoupled: long exploration producing no artifacts.
    Uncoupled,
    /// Heat loss: verbose output nobody reads.
    HeatLoss,
    /// Substrate cycling: same work done by parent and child agent.
    SubstrateCycling,
    /// Retries: failed tool calls requiring do-over.
    Retry,
}

impl WasteClass {
    /// Human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::FutileCycling => "Futile cycling",
            Self::Uncoupled => "Uncoupled respiration",
            Self::HeatLoss => "Heat loss (verbose output)",
            Self::SubstrateCycling => "Substrate cycling (duplicate work)",
            Self::Retry => "Retry overhead",
        }
    }

    /// Prevention strategy.
    #[must_use]
    pub const fn prevention(&self) -> &'static str {
        match self {
            Self::FutileCycling => "Check permissions before editing",
            Self::Uncoupled => "Set max_turns, require deliverables",
            Self::HeatLoss => "Compendious scoring, density targets",
            Self::SubstrateCycling => "Delegate OR do, never both",
            Self::Retry => "Validate inputs before tool calls",
        }
    }
}

impl fmt::Display for WasteClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

// ============================================================================
// TokenPool — The three-pool energy model
// ============================================================================

/// The three token pools: ATP (available), ADP (productive spend), AMP (waste).
///
/// Tier: T2-C (N + varsigma + ∝ -- Quantity + State + Irreversibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPool {
    /// Tokens remaining (available for work).
    pub t_atp: u64,
    /// Tokens spent on productive work (generated value).
    pub t_adp: u64,
    /// Tokens wasted (retries, failures, verbose output).
    pub t_amp: u64,
}

impl TokenPool {
    /// Create a pool with a given budget. All tokens start as tATP.
    #[must_use]
    pub fn new(budget: u64) -> Self {
        Self {
            t_atp: budget,
            t_adp: 0,
            t_amp: 0,
        }
    }

    /// Total tokens across all pools (conservation law).
    #[must_use]
    pub fn total(&self) -> u64 {
        self.t_atp + self.t_adp + self.t_amp
    }

    /// Spend tokens productively: tATP -> tADP.
    ///
    /// Returns actual amount spent (capped at available tATP).
    pub fn spend_productive(&mut self, amount: u64) -> u64 {
        let actual = amount.min(self.t_atp);
        self.t_atp -= actual;
        self.t_adp += actual;
        actual
    }

    /// Waste tokens: tATP -> tAMP.
    ///
    /// Returns actual amount wasted (capped at available tATP).
    pub fn spend_waste(&mut self, amount: u64) -> u64 {
        let actual = amount.min(self.t_atp);
        self.t_atp -= actual;
        self.t_amp += actual;
        actual
    }

    /// Degrade productive spend to waste: tADP -> tAMP.
    ///
    /// Used when previously "productive" work turns out useless
    /// (e.g., code that fails tests, artifacts that get discarded).
    pub fn degrade(&mut self, amount: u64) -> u64 {
        let actual = amount.min(self.t_adp);
        self.t_adp -= actual;
        self.t_amp += actual;
        actual
    }

    /// Recycle spent tokens: tADP -> tATP.
    ///
    /// ATP Synthase analog: context compression, cache population,
    /// and pattern learning effectively recover spent tokens.
    pub fn recycle(&mut self, amount: u64) -> u64 {
        let actual = amount.min(self.t_adp);
        self.t_adp -= actual;
        self.t_atp += actual;
        actual
    }

    /// Atkinson Energy Charge.
    ///
    /// EC = (tATP + 0.5 * tADP) / (tATP + tADP + tAMP)
    ///
    /// Returns 1.0 if total is zero (fully charged empty pool).
    #[must_use]
    pub fn energy_charge(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            return 1.0;
        }
        #[allow(clippy::cast_precision_loss)]
        // Token counts may exceed f64 mantissa; acceptable for ratio computation
        {
            ADP_WEIGHT.mul_add(self.t_adp as f64, self.t_atp as f64) / total as f64
        }
    }

    /// Current metabolic regime.
    #[must_use]
    pub fn regime(&self) -> Regime {
        Regime::from_ec(self.energy_charge())
    }

    /// Coupling efficiency: value per productive token.
    ///
    /// `total_value` is an external measure of work accomplished.
    #[must_use]
    pub fn coupling_efficiency(&self, total_value: f64) -> f64 {
        if self.t_adp == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)] // Token count ratio; exact precision not critical
        {
            total_value / self.t_adp as f64
        }
    }

    /// Waste ratio: fraction of total spend that was wasted.
    #[must_use]
    pub fn waste_ratio(&self) -> f64 {
        let spent = self.t_adp + self.t_amp;
        if spent == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)] // Token count ratio; exact precision not critical
        {
            self.t_amp as f64 / spent as f64
        }
    }

    /// Fraction of budget consumed.
    #[must_use]
    pub fn burn_rate(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            return 1.0;
        }
        #[allow(clippy::cast_precision_loss)] // Token count ratio; exact precision not critical
        {
            1.0 - (self.t_atp as f64 / total as f64)
        }
    }

    /// Estimate remaining operations at current metabolic rate.
    ///
    /// `avg_cost_per_op` is the rolling average tokens per operation.
    #[must_use]
    pub fn estimated_remaining_ops(&self, avg_cost_per_op: u64) -> u64 {
        if avg_cost_per_op == 0 {
            return u64::MAX;
        }
        self.t_atp / avg_cost_per_op
    }
}

impl fmt::Display for TokenPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ec = self.energy_charge();
        write!(
            f,
            "EC={:.2} [{}] tATP={} tADP={} tAMP={} waste={:.0}%",
            ec,
            self.regime(),
            self.t_atp,
            self.t_adp,
            self.t_amp,
            self.waste_ratio() * 100.0,
        )
    }
}

// ============================================================================
// Operation — Estimated cost/value for decision-making
// ============================================================================

/// An operation being considered, with estimated cost and value.
///
/// Tier: T2-C (N + ∝ -- Quantity + Irreversibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Human-readable label.
    pub label: String,
    /// Estimated token cost.
    pub estimated_cost: u64,
    /// Estimated value (arbitrary units, consistent within session).
    pub estimated_value: f64,
    /// Whether a cached result might exist.
    pub cache_possible: bool,
}

impl Operation {
    /// Create a new operation builder with a label.
    #[must_use]
    pub fn builder(label: impl Into<String>) -> OperationBuilder {
        OperationBuilder {
            label: label.into(),
            estimated_cost: 0,
            estimated_value: 0.0,
            cache_possible: false,
        }
    }

    /// Coupling ratio: expected value per token.
    #[must_use]
    pub fn coupling_ratio(&self) -> f64 {
        if self.estimated_cost == 0 {
            return f64::MAX;
        }
        #[allow(clippy::cast_precision_loss)]
        // Token cost is u64; precision loss acceptable for ratio
        {
            self.estimated_value / self.estimated_cost as f64
        }
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (cost={}, value={:.1}, CR={:.2}{})",
            self.label,
            self.estimated_cost,
            self.estimated_value,
            self.coupling_ratio(),
            if self.cache_possible {
                ", cacheable"
            } else {
                ""
            },
        )
    }
}

/// Fluent builder for [`Operation`].
///
/// ```
/// use nexcore_energy::Operation;
///
/// let op = Operation::builder("deep analysis")
///     .cost(5_000)
///     .value(20_000.0)
///     .cacheable()
///     .build();
/// assert_eq!(op.estimated_cost, 5_000);
/// ```
#[derive(Debug, Clone)]
pub struct OperationBuilder {
    label: String,
    estimated_cost: u64,
    estimated_value: f64,
    cache_possible: bool,
}

impl OperationBuilder {
    /// Set estimated token cost.
    #[must_use]
    pub const fn cost(mut self, cost: u64) -> Self {
        self.estimated_cost = cost;
        self
    }

    /// Set estimated value.
    #[must_use]
    pub fn value(mut self, value: f64) -> Self {
        self.estimated_value = value;
        self
    }

    /// Mark as cacheable.
    #[must_use]
    pub const fn cacheable(mut self) -> Self {
        self.cache_possible = true;
        self
    }

    /// Build the operation.
    #[must_use]
    pub fn build(self) -> Operation {
        Operation {
            label: self.label,
            estimated_cost: self.estimated_cost,
            estimated_value: self.estimated_value,
            cache_possible: self.cache_possible,
        }
    }
}

// ============================================================================
// RecyclingRate — ATP Synthase efficiency
// ============================================================================

/// Token recycling metrics (ATP Synthase analog).
///
/// R = compression_ratio x cache_hit_rate x pattern_reuse_rate
///
/// Tier: T2-C (∝ + ∝ + ∝ -- Irreversibility x3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecyclingRate {
    /// How much context compression reduces future token cost [0, 1].
    pub compression_ratio: f64,
    /// Fraction of lookups served from cache [0, 1].
    pub cache_hit_rate: f64,
    /// Fraction of patterns reused vs. re-derived [0, 1].
    pub pattern_reuse_rate: f64,
}

impl RecyclingRate {
    /// Create with all rates.
    #[must_use]
    pub fn new(compression: f64, cache: f64, reuse: f64) -> Self {
        Self {
            compression_ratio: compression.clamp(0.0, 1.0),
            cache_hit_rate: cache.clamp(0.0, 1.0),
            pattern_reuse_rate: reuse.clamp(0.0, 1.0),
        }
    }

    /// Combined recycling rate.
    #[must_use]
    pub fn combined(&self) -> f64 {
        self.compression_ratio * self.cache_hit_rate * self.pattern_reuse_rate
    }

    /// Estimate tokens recoverable from tADP pool.
    #[must_use]
    pub fn recoverable(&self, t_adp: u64) -> u64 {
        #[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
        // Rates are [0,1] so result is non-negative; precision loss acceptable
        {
            (t_adp as f64 * self.combined()) as u64
        }
    }
}

// ============================================================================
// EnergyState — Real-time dashboard
// ============================================================================

/// Complete energy state for monitoring and decision-making.
///
/// Tier: T3 (Full domain state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyState {
    /// The three token pools.
    pub pool: TokenPool,
    /// Current metabolic regime.
    pub regime: Regime,
    /// Energy charge [0, 1].
    pub energy_charge: f64,
    /// Coupling efficiency (value/token).
    pub coupling_efficiency: f64,
    /// Waste ratio [0, 1].
    pub waste_ratio: f64,
    /// Burn rate [0, 1].
    pub burn_rate: f64,
    /// Recommended strategy.
    pub recommended_strategy: Strategy,
    /// Active energy system.
    pub energy_system: EnergySystem,
}

impl fmt::Display for EnergyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} | {} via {} | burn={:.0}%",
            self.pool,
            self.recommended_strategy,
            self.energy_system,
            self.burn_rate * 100.0,
        )
    }
}

// ============================================================================
// decide — The core decision algorithm
// ============================================================================

/// Decide the optimal strategy for an operation given current energy state.
///
/// This is the **central regulation enzyme** — like phosphofructokinase
/// in glycolysis, it's the rate-limiting step that controls everything.
///
/// Tier: T2-C (kappa + ∝ + partial -- Comparison + Irreversibility + Boundary)
#[must_use]
pub fn decide(pool: &TokenPool, operation: &Operation) -> Strategy {
    let ec = pool.energy_charge();
    let cr = operation.coupling_ratio();

    // Crisis: always checkpoint
    if ec < EC_CATABOLIC {
        return Strategy::Checkpoint;
    }

    // Catabolic: cache-first or bare-minimum Haiku
    if ec < EC_HOMEOSTATIC {
        return if operation.cache_possible {
            Strategy::HaikuCacheFirst
        } else {
            Strategy::Haiku
        };
    }

    // Homeostatic: model selection based on coupling ratio
    if ec < EC_ANABOLIC {
        return if cr >= MIN_OPUS_COUPLING {
            Strategy::Sonnet // invest moderately for high-yield ops
        } else if operation.cache_possible {
            Strategy::HaikuCacheFirst
        } else {
            Strategy::Haiku
        };
    }

    // Anabolic: full power for high-yield, efficient for low-yield
    if cr >= MIN_OPUS_COUPLING {
        Strategy::Opus
    } else if cr >= MIN_SONNET_COUPLING {
        Strategy::Sonnet
    } else {
        Strategy::Haiku // don't waste even when rich
    }
}

/// Compute the full energy state snapshot.
///
/// Tier: T3 (Domain-specific dashboard)
#[must_use]
pub fn snapshot(pool: &TokenPool, total_value: f64) -> EnergyState {
    let ec = pool.energy_charge();
    let regime = Regime::from_ec(ec);

    // Default operation for strategy recommendation
    let default_op = Operation {
        label: "default".to_string(),
        estimated_cost: 1000,
        estimated_value: 1.0,
        cache_possible: false,
    };
    let strategy = decide(pool, &default_op);
    let system = EnergySystem::for_strategy(strategy);

    EnergyState {
        pool: pool.clone(),
        regime,
        energy_charge: ec,
        coupling_efficiency: pool.coupling_efficiency(total_value),
        waste_ratio: pool.waste_ratio(),
        burn_rate: pool.burn_rate(),
        recommended_strategy: strategy,
        energy_system: system,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =============================
    // TokenPool Tests
    // =============================

    #[test]
    fn test_new_pool_full_charge() {
        let pool = TokenPool::new(100_000);
        assert_eq!(pool.t_atp, 100_000);
        assert_eq!(pool.t_adp, 0);
        assert_eq!(pool.t_amp, 0);
        assert!((pool.energy_charge() - 1.0).abs() < f64::EPSILON);
        assert_eq!(pool.regime(), Regime::Anabolic);
    }

    #[test]
    fn test_empty_pool_full_charge() {
        let pool = TokenPool::new(0);
        assert!((pool.energy_charge() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_conservation_law() {
        let mut pool = TokenPool::new(100_000);
        let total_before = pool.total();
        pool.spend_productive(30_000);
        pool.spend_waste(10_000);
        pool.recycle(5_000);
        pool.degrade(2_000);
        assert_eq!(pool.total(), total_before);
    }

    #[test]
    fn test_spend_productive() {
        let mut pool = TokenPool::new(10_000);
        let spent = pool.spend_productive(3_000);
        assert_eq!(spent, 3_000);
        assert_eq!(pool.t_atp, 7_000);
        assert_eq!(pool.t_adp, 3_000);
    }

    #[test]
    fn test_spend_capped_at_available() {
        let mut pool = TokenPool::new(100);
        let spent = pool.spend_productive(500);
        assert_eq!(spent, 100);
        assert_eq!(pool.t_atp, 0);
        assert_eq!(pool.t_adp, 100);
    }

    #[test]
    fn test_spend_waste() {
        let mut pool = TokenPool::new(10_000);
        pool.spend_waste(2_000);
        assert_eq!(pool.t_amp, 2_000);
        assert_eq!(pool.t_atp, 8_000);
    }

    #[test]
    fn test_degrade() {
        let mut pool = TokenPool::new(10_000);
        pool.spend_productive(5_000);
        pool.degrade(2_000);
        assert_eq!(pool.t_adp, 3_000);
        assert_eq!(pool.t_amp, 2_000);
    }

    #[test]
    fn test_recycle() {
        let mut pool = TokenPool::new(10_000);
        pool.spend_productive(5_000);
        pool.recycle(2_000);
        assert_eq!(pool.t_atp, 7_000);
        assert_eq!(pool.t_adp, 3_000);
    }

    // =============================
    // Energy Charge Tests
    // =============================

    #[test]
    fn test_energy_charge_formula() {
        // EC = (ATP + 0.5*ADP) / (ATP + ADP + AMP)
        let mut pool = TokenPool::new(10_000);
        pool.spend_productive(4_000); // ATP=6000, ADP=4000, AMP=0
        // EC = (6000 + 0.5*4000) / 10000 = 8000/10000 = 0.80
        let ec = pool.energy_charge();
        assert!((ec - 0.80).abs() < f64::EPSILON);
        assert_eq!(pool.regime(), Regime::Homeostatic);
    }

    #[test]
    fn test_energy_charge_with_waste() {
        // ATP=5000, ADP=3000, AMP=2000
        let mut pool = TokenPool::new(10_000);
        pool.spend_productive(3_000);
        pool.spend_waste(2_000);
        // EC = (5000 + 0.5*3000) / 10000 = 6500/10000 = 0.65
        let ec = pool.energy_charge();
        assert!((ec - 0.65).abs() < f64::EPSILON);
        assert_eq!(pool.regime(), Regime::Catabolic);
    }

    #[test]
    fn test_all_wasted_is_crisis() {
        // ATP=0, ADP=0, AMP=10000
        let mut pool = TokenPool::new(10_000);
        pool.spend_waste(10_000);
        assert!((pool.energy_charge() - 0.0).abs() < f64::EPSILON);
        assert_eq!(pool.regime(), Regime::Crisis);
    }

    #[test]
    fn test_half_productive_is_homeostatic() {
        // ATP=5000, ADP=5000, AMP=0
        // EC = (5000 + 2500) / 10000 = 0.75
        let mut pool = TokenPool::new(10_000);
        pool.spend_productive(5_000);
        assert!((pool.energy_charge() - 0.75).abs() < f64::EPSILON);
        assert_eq!(pool.regime(), Regime::Homeostatic);
    }

    // =============================
    // Regime Tests
    // =============================

    #[test]
    fn test_regime_boundaries() {
        assert_eq!(Regime::from_ec(0.90), Regime::Anabolic);
        assert_eq!(Regime::from_ec(0.85), Regime::Homeostatic); // boundary: > 0.85
        assert_eq!(Regime::from_ec(0.86), Regime::Anabolic);
        assert_eq!(Regime::from_ec(0.70), Regime::Homeostatic);
        assert_eq!(Regime::from_ec(0.69), Regime::Catabolic);
        assert_eq!(Regime::from_ec(0.50), Regime::Catabolic);
        assert_eq!(Regime::from_ec(0.49), Regime::Crisis);
        assert_eq!(Regime::from_ec(0.0), Regime::Crisis);
    }

    #[test]
    fn test_regime_allows_expensive() {
        assert!(Regime::Anabolic.allows_expensive());
        assert!(Regime::Homeostatic.allows_expensive());
        assert!(!Regime::Catabolic.allows_expensive());
        assert!(!Regime::Crisis.allows_expensive());
    }

    // =============================
    // Strategy Decision Tests
    // =============================

    #[test]
    fn test_decide_anabolic_high_yield() {
        let pool = TokenPool::new(100_000); // EC = 1.0
        let op = Operation {
            label: "deep analysis".to_string(),
            estimated_cost: 5_000,
            estimated_value: 20_000.0, // CR = 4.0
            cache_possible: false,
        };
        assert_eq!(decide(&pool, &op), Strategy::Opus);
    }

    #[test]
    fn test_decide_anabolic_low_yield() {
        let pool = TokenPool::new(100_000);
        let op = Operation {
            label: "simple lookup".to_string(),
            estimated_cost: 5_000,
            estimated_value: 1_000.0, // CR = 0.2
            cache_possible: false,
        };
        // Even when rich, don't waste Opus on low-yield ops
        assert_eq!(decide(&pool, &op), Strategy::Haiku);
    }

    #[test]
    fn test_decide_homeostatic_high_yield() {
        let mut pool = TokenPool::new(100_000);
        pool.spend_productive(30_000);
        pool.spend_waste(5_000);
        // ATP=65000, ADP=30000, AMP=5000
        // EC = (65000 + 15000) / 100000 = 0.80 -> Homeostatic
        let op = Operation {
            label: "code gen".to_string(),
            estimated_cost: 2_000,
            estimated_value: 10_000.0, // CR = 5.0
            cache_possible: false,
        };
        assert_eq!(pool.regime(), Regime::Homeostatic);
        assert_eq!(decide(&pool, &op), Strategy::Sonnet);
    }

    #[test]
    fn test_decide_catabolic_cache_first() {
        let mut pool = TokenPool::new(100_000);
        pool.spend_productive(30_000);
        pool.spend_waste(20_000); // EC = (50000 + 15000)/100000 = 0.65
        let op = Operation {
            label: "file search".to_string(),
            estimated_cost: 500,
            estimated_value: 100.0,
            cache_possible: true,
        };
        assert_eq!(decide(&pool, &op), Strategy::HaikuCacheFirst);
    }

    #[test]
    fn test_decide_crisis_always_checkpoint() {
        let mut pool = TokenPool::new(100_000);
        pool.spend_waste(80_000); // EC = (20000 + 0)/100000 = 0.20
        let op = Operation {
            label: "anything".to_string(),
            estimated_cost: 100,
            estimated_value: 100_000.0,
            cache_possible: true,
        };
        assert_eq!(decide(&pool, &op), Strategy::Checkpoint);
    }

    // =============================
    // Recycling Tests
    // =============================

    #[test]
    fn test_recycling_rate() {
        let rate = RecyclingRate::new(0.5, 0.8, 0.6);
        let combined = rate.combined();
        assert!((combined - 0.24).abs() < f64::EPSILON); // 0.5 * 0.8 * 0.6
    }

    #[test]
    fn test_recycling_recoverable() {
        let rate = RecyclingRate::new(0.5, 1.0, 1.0);
        assert_eq!(rate.recoverable(10_000), 5_000);
    }

    #[test]
    fn test_recycling_clamped() {
        let rate = RecyclingRate::new(1.5, -0.5, 2.0);
        assert!((rate.compression_ratio - 1.0).abs() < f64::EPSILON);
        assert!((rate.cache_hit_rate - 0.0).abs() < f64::EPSILON);
        assert!((rate.pattern_reuse_rate - 1.0).abs() < f64::EPSILON);
    }

    // =============================
    // Energy System Tests
    // =============================

    #[test]
    fn test_energy_system_mapping() {
        assert_eq!(
            EnergySystem::for_strategy(Strategy::HaikuCacheFirst),
            EnergySystem::Phosphocreatine
        );
        assert_eq!(
            EnergySystem::for_strategy(Strategy::Haiku),
            EnergySystem::Glycolytic
        );
        assert_eq!(
            EnergySystem::for_strategy(Strategy::Opus),
            EnergySystem::Oxidative
        );
    }

    // =============================
    // Waste Classification Tests
    // =============================

    #[test]
    fn test_waste_ratio() {
        let mut pool = TokenPool::new(10_000);
        pool.spend_productive(6_000);
        pool.spend_waste(4_000);
        // waste_ratio = 4000 / (6000+4000) = 0.4
        assert!((pool.waste_ratio() - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn test_waste_ratio_zero_spend() {
        let pool = TokenPool::new(10_000);
        assert!((pool.waste_ratio() - 0.0).abs() < f64::EPSILON);
    }

    // =============================
    // Snapshot Tests
    // =============================

    #[test]
    fn test_snapshot_fresh_pool() {
        let pool = TokenPool::new(100_000);
        let state = snapshot(&pool, 0.0);
        assert_eq!(state.regime, Regime::Anabolic);
        assert!((state.energy_charge - 1.0).abs() < f64::EPSILON);
        assert!((state.waste_ratio - 0.0).abs() < f64::EPSILON);
        assert!((state.burn_rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_snapshot_depleted_pool() {
        let mut pool = TokenPool::new(100_000);
        pool.spend_waste(90_000);
        let state = snapshot(&pool, 0.0);
        assert_eq!(state.regime, Regime::Crisis);
        assert!(state.energy_charge < EC_CATABOLIC);
        assert_eq!(state.recommended_strategy, Strategy::Checkpoint);
    }

    // =============================
    // Coupling Efficiency Tests
    // =============================

    #[test]
    fn test_coupling_efficiency() {
        let mut pool = TokenPool::new(100_000);
        pool.spend_productive(10_000);
        // If we produced 50 units of value from 10k tokens:
        let eta = pool.coupling_efficiency(50.0);
        assert!((eta - 0.005).abs() < f64::EPSILON); // 50/10000
    }

    #[test]
    fn test_coupling_efficiency_zero_spend() {
        let pool = TokenPool::new(100_000);
        assert!((pool.coupling_efficiency(100.0) - 0.0).abs() < f64::EPSILON);
    }

    // =============================
    // Estimated Remaining Ops Tests
    // =============================

    #[test]
    fn test_estimated_remaining_ops() {
        let pool = TokenPool::new(100_000);
        assert_eq!(pool.estimated_remaining_ops(1_000), 100);
        assert_eq!(pool.estimated_remaining_ops(0), u64::MAX);
    }

    // =============================
    // Integration: Full Lifecycle
    // =============================

    #[test]
    fn test_full_lifecycle() {
        let mut pool = TokenPool::new(100_000);

        // Phase 1: Anabolic — invest freely
        assert_eq!(pool.regime(), Regime::Anabolic);
        pool.spend_productive(20_000); // research
        pool.spend_productive(15_000); // code generation
        pool.spend_waste(5_000); // a failed tool call
        // ATP=60000, ADP=35000, AMP=5000
        // EC = (60000 + 17500) / 100000 = 0.775
        assert_eq!(pool.regime(), Regime::Homeostatic);

        // Phase 2: Homeostatic — balanced ops
        pool.spend_productive(10_000);
        pool.spend_waste(5_000);
        // ATP=45000, ADP=45000, AMP=10000
        // EC = (45000 + 22500) / 100000 = 0.675
        assert_eq!(pool.regime(), Regime::Catabolic);

        // Phase 3: Recycle via context compression
        pool.recycle(10_000); // compression recovered 10k effective tokens
        // ATP=55000, ADP=35000, AMP=10000
        // EC = (55000 + 17500) / 100000 = 0.725
        assert_eq!(pool.regime(), Regime::Homeostatic);

        // Conservation law holds
        assert_eq!(pool.total(), 100_000);
    }

    #[test]
    fn test_operation_coupling_ratio() {
        let op = Operation {
            label: "test".to_string(),
            estimated_cost: 1_000,
            estimated_value: 5_000.0,
            cache_possible: false,
        };
        assert!((op.coupling_ratio() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_operation_zero_cost() {
        let op = Operation {
            label: "free".to_string(),
            estimated_cost: 0,
            estimated_value: 100.0,
            cache_possible: false,
        };
        assert_eq!(op.coupling_ratio(), f64::MAX);
    }

    // =============================
    // Display Tests
    // =============================

    #[test]
    fn test_display_regime() {
        assert_eq!(format!("{}", Regime::Anabolic), "Anabolic (invest)");
        assert_eq!(format!("{}", Regime::Crisis), "Crisis (checkpoint)");
    }

    #[test]
    fn test_display_strategy() {
        assert_eq!(format!("{}", Strategy::Opus), "Opus (full power)");
        assert_eq!(format!("{}", Strategy::Checkpoint), "Checkpoint (halt)");
    }

    #[test]
    fn test_display_energy_system() {
        assert_eq!(
            format!("{}", EnergySystem::Phosphocreatine),
            "Phosphocreatine (cache)"
        );
        assert_eq!(
            format!("{}", EnergySystem::Oxidative),
            "Oxidative (sustained)"
        );
    }

    #[test]
    fn test_display_waste_class() {
        assert_eq!(format!("{}", WasteClass::FutileCycling), "Futile cycling");
        assert_eq!(
            format!("{}", WasteClass::SubstrateCycling),
            "Substrate cycling (duplicate work)"
        );
    }

    #[test]
    fn test_display_token_pool() {
        let pool = TokenPool::new(100_000);
        let display = format!("{pool}");
        assert!(display.contains("EC=1.00"));
        assert!(display.contains("Anabolic"));
        assert!(display.contains("tATP=100000"));
    }

    #[test]
    fn test_display_operation() {
        let op = Operation::builder("deep analysis")
            .cost(5_000)
            .value(20_000.0)
            .cacheable()
            .build();
        let display = format!("{op}");
        assert!(display.contains("deep analysis"));
        assert!(display.contains("cost=5000"));
        assert!(display.contains("cacheable"));
    }

    #[test]
    fn test_display_energy_state() {
        let pool = TokenPool::new(100_000);
        let state = snapshot(&pool, 0.0);
        let display = format!("{state}");
        assert!(display.contains("EC=1.00"));
        assert!(display.contains("burn=0%"));
    }

    // =============================
    // Builder Tests
    // =============================

    #[test]
    fn test_operation_builder_basic() {
        let op = Operation::builder("test op")
            .cost(1_000)
            .value(5_000.0)
            .build();
        assert_eq!(op.label, "test op");
        assert_eq!(op.estimated_cost, 1_000);
        assert!((op.estimated_value - 5_000.0).abs() < f64::EPSILON);
        assert!(!op.cache_possible);
    }

    #[test]
    fn test_operation_builder_cacheable() {
        let op = Operation::builder("cached lookup")
            .cost(500)
            .value(100.0)
            .cacheable()
            .build();
        assert!(op.cache_possible);
    }

    #[test]
    fn test_operation_builder_defaults() {
        let op = Operation::builder("minimal").build();
        assert_eq!(op.estimated_cost, 0);
        assert!((op.estimated_value - 0.0).abs() < f64::EPSILON);
        assert!(!op.cache_possible);
    }

    #[test]
    fn test_builder_integrates_with_decide() {
        let pool = TokenPool::new(100_000);
        let op = Operation::builder("deep analysis")
            .cost(5_000)
            .value(20_000.0)
            .build();
        assert_eq!(decide(&pool, &op), Strategy::Opus);
    }
}
