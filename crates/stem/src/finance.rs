//! # stem-finance: Finance Primitives as Rust Traits
//!
//! Implements cross-domain T2-P primitives derived from finance.
//!
//! ## The FINANCE Composite (T2-C)
//!
//! ```text
//! A - APPRAISE  : Asset → Monetary value             (T1: QUANTITY N)
//! F - FLOW      : Value moves between accounts        (T1: SEQUENCE σ)
//! D - DISCOUNT  : Future value → Present value        (T1: CAUSALITY →)
//! C - COMPOUND  : Growth applied to growth            (T1: RECURSION ρ)
//! H - HEDGE     : Position → Bounded risk             (T1: BOUNDARY ∂)
//! A - ARBITRAGE : Two prices → Exploit gap            (T1: COMPARISON κ)
//! M - MATURE    : Instrument → Terminal event         (T1: IRREVERSIBILITY ∝)
//! L - LEVERAGE  : Equity × Multiplier → Exposure      (T1: PRODUCT ×)
//! D - DIVERSIFY : Positions → Aggregated reduced risk (T1: SUM Σ)
//! ```
//!
//! ## Cross-Domain Transfer
//!
//! | Finance | PV Signals | Physics | Software |
//! |---------|------------|---------|----------|
//! | Discount (FV→PV) | Signal decay | Time dilation | Data depreciation |
//! | Compound | Signal accumulation | Wave amplification | Tech debt growth |
//! | Hedge | Risk mitigation | Shielding | Circuit breaker |
//! | Arbitrage | Cross-source detection | Calibration | A/B testing |
//! | Leverage | Amplified reporting | Mechanical advantage | Cache multiplier |
//!
//! ## 77 Measurable Variables (10 Categories)
//!
//! Price(11) · Return(8) · Risk(9) · Rate(10) · Flow(8) ·
//! Position/Greeks(8) · Time(5) · Ratio(8) · Liquidity(5) · Credit(5)
//!
//! ## T1 Coverage: 9 of 16 (5 NEW for STEM)
//!
//! Existing STEM T1s: μ, σ, ρ, ς, ∂, Σ, π
//! **New T1s from Finance**: N (Quantity), → (Causality), κ (Comparison),
//! ∝ (Irreversibility), × (Product)
//!
//! ## Three Unfixable Limits
//!
//! 1. **Heisenberg**: Observing a market changes it (observer effect on prices)
//! 2. **Gödel**: No financial model can predict its own market impact
//! 3. **Shannon**: Market information has irreducible noise (EMH weak form)

use crate::core::Confidence;
use serde::{Deserialize, Serialize};

// ============================================================================
// Core Types (T2-P)
// ============================================================================

/// Monetary value (T2-P)
///
/// Grounded in T1 Quantity (N): discrete monetary amount.
///
/// Covers: spot, forward, strike, bid, ask, mid, VWAP, NAV, book value,
/// market cap, enterprise value (11 price variables).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Price(f64);

impl Price {
    /// Create new price (must be non-negative)
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.max(0.0))
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Zero price
    pub const ZERO: Self = Self(0.0);

    /// Compute mid price from bid and ask
    #[must_use]
    pub fn mid(bid: Self, ask: Self) -> Self {
        Self((bid.0 + ask.0) / 2.0)
    }
}

impl Default for Price {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Add for Price {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Price {
    type Output = f64; // Can be negative (P&L)
    fn sub(self, rhs: Self) -> f64 {
        self.0 - rhs.0
    }
}

/// Proportional return (T2-P)
///
/// Grounded in T1 Mapping (μ): price change → proportion.
///
/// Covers: simple return, log return, total return, excess return,
/// alpha, beta, risk-adjusted (Sharpe), CAPM expected (8 return variables).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Return(f64);

impl Return {
    /// Create new return (can be negative, must be finite)
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        if value.is_finite() {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Simple return: (P₁ - P₀) / P₀
    #[must_use]
    pub fn simple(p0: Price, p1: Price) -> Option<Self> {
        if p0.value() > 0.0 {
            Self::new((p1.value() - p0.value()) / p0.value())
        } else {
            None
        }
    }

    /// Log return: ln(P₁ / P₀)
    #[must_use]
    pub fn log(p0: Price, p1: Price) -> Option<Self> {
        if p0.value() > 0.0 && p1.value() > 0.0 {
            Self::new((p1.value() / p0.value()).ln())
        } else {
            None
        }
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Check if return is positive
    #[must_use]
    pub fn is_positive(&self) -> bool {
        self.0 > 0.0
    }

    /// Zero return
    pub const ZERO: Self = Self(0.0);
}

/// Periodic interest/discount rate (T2-P)
///
/// Grounded in T1 Frequency (ν): change per time period.
///
/// Named `InterestRate` to disambiguate from `chem::Rate` (reaction rate).
/// Chemistry rates measure speed of transformation; finance rates measure
/// the price of money over time.
///
/// Covers: interest rate, discount rate, yield, YTM, coupon rate,
/// inflation rate, risk-free rate, forward rate, swap rate,
/// dividend yield (10 rate variables).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct InterestRate(f64);

impl InterestRate {
    /// Create new rate (must be finite)
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        if value.is_finite() {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Get raw value (as decimal, e.g. 0.05 = 5%)
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Convert to percentage
    #[must_use]
    pub fn as_percent(&self) -> f64 {
        self.0 * 100.0
    }

    /// Zero rate
    pub const ZERO: Self = Self(0.0);
}

/// Price gap between two levels (T2-P)
///
/// Grounded in T1 Boundary (∂): non-negative distance between prices.
///
/// Covers: bid-ask spread, credit spread, risk premium,
/// option spread (4+ spread variables).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Spread(f64);

impl Spread {
    /// Create new spread (non-negative)
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.max(0.0))
    }

    /// Compute from bid-ask
    #[must_use]
    pub fn from_bid_ask(bid: Price, ask: Price) -> Self {
        Self((ask.value() - bid.value()).max(0.0))
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Spread as percentage of mid price
    #[must_use]
    pub fn as_percent_of_mid(&self, bid: Price, ask: Price) -> f64 {
        let mid = Price::mid(bid, ask).value();
        if mid > 0.0 { self.0 / mid * 100.0 } else { 0.0 }
    }

    /// Zero spread (perfect liquidity)
    pub const ZERO: Self = Self(0.0);
}

/// Time to terminal event (T2-P)
///
/// Grounded in T1 Irreversibility (∝): countdown to maturity.
///
/// Covers: maturity, duration, convexity, tenor, settlement period
/// (5 time variables).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Maturity(f64);

impl Maturity {
    /// Create new maturity in years (must be non-negative)
    #[must_use]
    pub fn new(years: f64) -> Self {
        Self(years.max(0.0))
    }

    /// Get years to maturity
    #[must_use]
    pub fn years(&self) -> f64 {
        self.0
    }

    /// Check if expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.0 <= 0.0
    }

    /// Expired maturity
    pub const EXPIRED: Self = Self(0.0);
}

impl Default for Maturity {
    fn default() -> Self {
        Self::EXPIRED
    }
}

/// Aggregate risk exposure (T2-P)
///
/// Grounded in T1 Sum (Σ): total position value.
///
/// Covers: notional, net exposure, gross exposure, leverage-adjusted
/// exposure, delta-equivalent exposure (5+ position variables).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Exposure(f64);

impl Exposure {
    /// Create new exposure (can be negative for short positions)
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Absolute exposure
    #[must_use]
    pub fn abs(&self) -> f64 {
        self.0.abs()
    }

    /// Zero exposure
    pub const ZERO: Self = Self(0.0);

    /// Check if net long
    #[must_use]
    pub fn is_long(&self) -> bool {
        self.0 > 0.0
    }

    /// Check if net short
    #[must_use]
    pub fn is_short(&self) -> bool {
        self.0 < 0.0
    }
}

impl Default for Exposure {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Add for Exposure {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Neg for Exposure {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

// ============================================================================
// FINANCE Traits (T2-P)
// ============================================================================

/// T2-P: Assign monetary value to an asset
///
/// Grounded in T1 Quantity (N): asset → monetary value
///
/// # Cross-Domain Transfer
/// - PV: Cost of illness, QALY valuation
/// - Physics: Energy pricing
/// - Software: Resource cost estimation
pub trait Appraise {
    /// The asset being valued
    type Asset;

    /// Assign a price to the asset
    fn appraise(&self, asset: &Self::Asset) -> Price;
}

/// T2-P: Directed movement of value between accounts
///
/// Grounded in T1 Sequence (σ): source → destination
///
/// # Cross-Domain Transfer
/// - PV: Case report flow through pipeline
/// - Physics: Energy flow (current)
/// - Software: Data pipeline, message passing
pub trait Flow {
    /// Source of value
    type Source;
    /// Destination of value
    type Destination;
    /// Amount type
    type Amount;

    /// Move amount from source to destination.
    /// Returns true if the flow succeeded.
    fn flow(
        &mut self,
        source: &Self::Source,
        dest: &Self::Destination,
        amount: Self::Amount,
    ) -> bool;

    /// Net flow (inflows - outflows)
    fn net_flow(&self) -> Exposure;
}

/// T2-P: Map future value to present value via time preference
///
/// Grounded in T1 Causality (→): future → present via rate
///
/// This is the time value of money — the foundational finance primitive.
/// All valuation ultimately reduces to discounting.
///
/// # Cross-Domain Transfer
/// - PV: Signal urgency decay over time
/// - Physics: Time dilation, wave attenuation
/// - Software: Cache staleness, data depreciation
pub trait Discount {
    /// Compute present value of future amount
    fn present_value(&self, future_value: Price, rate: InterestRate, periods: f64) -> Price;

    /// Compute future value of present amount
    fn future_value(&self, present_value: Price, rate: InterestRate, periods: f64) -> Price;
}

/// T2-P: Recursive growth — interest on interest
///
/// Grounded in T1 Recursion (ρ): apply growth function to its own output
///
/// # Cross-Domain Transfer
/// - PV: Signal accumulation over time
/// - Physics: Chain reaction, resonance amplification
/// - Software: Technical debt compounding
pub trait Compound {
    /// Compute compounded value after n discrete periods
    fn compound(&self, principal: Price, rate: InterestRate, periods: u32) -> Price;

    /// Compute continuous compounding: P × e^(rt)
    fn compound_continuous(&self, principal: Price, rate: InterestRate, time: f64) -> Price;

    /// Effective annual rate from nominal rate compounded m times per year
    fn effective_rate(&self, nominal: InterestRate, periods_per_year: u32) -> InterestRate;
}

/// T2-P: Contain risk within boundaries
///
/// Grounded in T1 Boundary (∂): limit downside exposure
///
/// # Cross-Domain Transfer
/// - PV: Safety signal threshold enforcement
/// - Physics: Radiation shielding, containment
/// - Software: Circuit breaker, rate limiter
pub trait Hedge {
    /// The position being hedged
    type Position;
    /// The hedging instrument
    type Instrument;

    /// Compute residual exposure after hedging
    fn hedge(&self, position: &Self::Position, instrument: &Self::Instrument) -> Exposure;

    /// Hedge effectiveness ratio [0, 1]
    fn effectiveness(&self, position: &Self::Position, instrument: &Self::Instrument) -> f64;
}

/// T2-P: Detect price discrepancy across equivalent assets
///
/// Grounded in T1 Comparison (κ): price_A vs price_B for same value
///
/// # Cross-Domain Transfer
/// - PV: Cross-source signal discrepancy detection
/// - Physics: Instrument calibration offset
/// - Software: A/B test result divergence
pub trait Arbitrage {
    /// The comparable asset type
    type Asset;

    /// Compute the spread between two equivalent assets
    fn spread(&self, a: &Self::Asset, b: &Self::Asset) -> Spread;

    /// Check if spread exceeds transaction costs (arbitrage exists)
    fn is_exploitable(&self, a: &Self::Asset, b: &Self::Asset, cost: Spread) -> bool {
        self.spread(a, b).value() > cost.value()
    }
}

/// T2-P: Time progression toward terminal event
///
/// Grounded in T1 Irreversibility (∝): maturity is one-way
///
/// # Cross-Domain Transfer
/// - PV: Regulatory deadline progression (ICH E2A timelines)
/// - Physics: Radioactive decay, entropy increase
/// - Software: Token/session expiry, lease timeout
pub trait Mature {
    /// The instrument type
    type Instrument;

    /// Time remaining until maturity
    fn time_to_maturity(&self, instrument: &Self::Instrument) -> Maturity;

    /// Check if instrument has expired
    fn is_expired(&self, instrument: &Self::Instrument) -> bool {
        self.time_to_maturity(instrument).is_expired()
    }
}

/// T2-P: Multiplicative exposure amplification
///
/// Grounded in T1 Product (×): equity × multiplier = total exposure
///
/// # Cross-Domain Transfer
/// - PV: Signal amplification factor
/// - Physics: Mechanical advantage (lever, pulley)
/// - Software: Cache hit multiplier, CDN amplification
pub trait Leverage {
    /// Compute leveraged exposure from equity
    fn leverage(&self, equity: Price, multiplier: f64) -> Exposure;

    /// Compute leverage ratio (total / equity)
    fn leverage_ratio(&self, total: Exposure, equity: Price) -> f64;

    /// Maximum safe leverage before margin call
    fn max_leverage(&self) -> f64;
}

/// T2-P: Risk reduction through aggregation of uncorrelated positions
///
/// Grounded in T1 Sum (Σ): portfolio risk < sum of individual risks
///
/// # Cross-Domain Transfer
/// - PV: Multi-source signal confirmation
/// - Physics: Wave interference (destructive cancellation)
/// - Software: Redundancy, multi-region deployment
pub trait Diversify {
    /// Individual position type
    type Position;

    /// Aggregate positions into portfolio exposure
    fn aggregate(&self, positions: &[Self::Position]) -> Exposure;

    /// Diversification benefit (1 - portfolio_risk / sum_of_risks)
    fn diversification_benefit(&self, positions: &[Self::Position]) -> f64;
}

// ============================================================================
// Finance Composite Trait (T2-C)
// ============================================================================

/// T2-C: The complete finance methodology as composite trait.
///
/// Combines all nine T2-P primitives into a coherent system covering:
/// valuation (Appraise), flow (Flow), time-value (Discount), growth
/// (Compound), risk management (Hedge), comparison (Arbitrage), expiry
/// (Mature), amplification (Leverage), and diversification (Diversify).
///
/// # Gödel Acknowledgment
///
/// A financial model predicting its own market impact is incomplete.
/// This is the reflexivity problem (Soros): observation changes the system.
///
/// # T1 Coverage: 9 of 16
///
/// N (Appraise) · σ (Flow) · → (Discount) · ρ (Compound) ·
/// ∂ (Hedge) · κ (Arbitrage) · ∝ (Mature) · × (Leverage) · Σ (Diversify)
pub trait Finance: Appraise + Discount + Compound + Mature + Leverage {
    /// Execute one valuation cycle (mark-to-market)
    fn mark_to_market(&mut self)
    where
        Self: Sized;
}

// ============================================================================
// Standard Implementations
// ============================================================================

/// Standard time-value-of-money calculator
///
/// Implements `Discount` and `Compound` using classical finance formulas.
/// This is the foundational building block: PV = FV / (1+r)^n.
#[derive(Debug, Clone, Copy, Default)]
pub struct TimeValueOfMoney;

impl Discount for TimeValueOfMoney {
    fn present_value(&self, future_value: Price, rate: InterestRate, periods: f64) -> Price {
        if rate.value() <= -1.0 {
            return Price::ZERO;
        }
        Price::new(future_value.value() / (1.0 + rate.value()).powf(periods))
    }

    fn future_value(&self, present_value: Price, rate: InterestRate, periods: f64) -> Price {
        Price::new(present_value.value() * (1.0 + rate.value()).powf(periods))
    }
}

impl Compound for TimeValueOfMoney {
    fn compound(&self, principal: Price, rate: InterestRate, periods: u32) -> Price {
        let mut value = principal.value();
        for _ in 0..periods {
            value *= 1.0 + rate.value();
        }
        Price::new(value)
    }

    fn compound_continuous(&self, principal: Price, rate: InterestRate, time: f64) -> Price {
        Price::new(principal.value() * (rate.value() * time).exp())
    }

    fn effective_rate(&self, nominal: InterestRate, periods_per_year: u32) -> InterestRate {
        if periods_per_year == 0 {
            return InterestRate::ZERO;
        }
        let r = nominal.value() / periods_per_year as f64;
        InterestRate::new((1.0 + r).powi(periods_per_year as i32) - 1.0)
            .unwrap_or(InterestRate::ZERO)
    }
}

// ============================================================================
// Measured Finance Types
// ============================================================================

/// A price with confidence (Codex IX: MEASURE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasuredPrice {
    /// The measured price
    pub value: Price,
    /// Confidence in the price
    pub confidence: Confidence,
}

impl MeasuredPrice {
    /// Create new measured price
    pub fn new(value: Price, confidence: Confidence) -> Self {
        Self { value, confidence }
    }
}

/// A return with confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasuredReturn {
    /// The measured return
    pub value: Return,
    /// Confidence in the return
    pub confidence: Confidence,
}

impl MeasuredReturn {
    /// Create new measured return
    pub fn new(value: Return, confidence: Confidence) -> Self {
        Self { value, confidence }
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors in financial operations
#[derive(Debug, nexcore_error::Error)]
pub enum FinanceError {
    /// Invalid price (negative)
    #[error("invalid price: {0}")]
    InvalidPrice(f64),

    /// Division by zero in ratio computation
    #[error("division by zero in financial ratio")]
    DivisionByZero,

    /// Rate out of bounds
    #[error("rate out of bounds: {0}")]
    RateOutOfBounds(f64),

    /// Insufficient liquidity
    #[error("insufficient liquidity: need {needed}, have {available}")]
    InsufficientLiquidity { needed: f64, available: f64 },

    /// Leverage exceeded
    #[error("leverage {actual} exceeds maximum {max}")]
    LeverageExceeded { actual: f64, max: f64 },

    /// Expired instrument
    #[error("instrument has expired (maturity <= 0)")]
    Expired,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Price tests ==========

    #[test]
    fn price_clamps_negative() {
        let p = Price::new(-10.0);
        assert!((p.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn price_arithmetic() {
        let a = Price::new(100.0);
        let b = Price::new(60.0);
        assert!(((a + b).value() - 160.0).abs() < f64::EPSILON);
        assert!(((a - b) - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn price_mid() {
        let bid = Price::new(99.0);
        let ask = Price::new(101.0);
        assert!((Price::mid(bid, ask).value() - 100.0).abs() < f64::EPSILON);
    }

    // ========== Return tests ==========

    #[test]
    fn return_simple() {
        let p0 = Price::new(100.0);
        let p1 = Price::new(110.0);
        let r = Return::simple(p0, p1);
        assert!(r.is_some());
        let r = r.unwrap(); // INVARIANT: p0 > 0, guaranteed by test setup
        assert!((r.value() - 0.10).abs() < 1e-10);
        assert!(r.is_positive());
    }

    #[test]
    fn return_log() {
        let p0 = Price::new(100.0);
        let p1 = Price::new(110.0);
        let r = Return::log(p0, p1);
        assert!(r.is_some());
        let r = r.unwrap(); // INVARIANT: both prices > 0
        assert!((r.value() - (1.1_f64).ln()).abs() < 1e-10);
    }

    #[test]
    fn return_division_by_zero() {
        assert!(Return::simple(Price::ZERO, Price::new(10.0)).is_none());
    }

    #[test]
    fn return_negative() {
        let p0 = Price::new(100.0);
        let p1 = Price::new(90.0);
        let r = Return::simple(p0, p1);
        assert!(r.is_some());
        let r = r.unwrap(); // INVARIANT: p0 > 0
        assert!(!r.is_positive());
        assert!((r.value() - (-0.10)).abs() < 1e-10);
    }

    // ========== Rate tests ==========

    #[test]
    fn rate_as_percent() {
        let r = InterestRate::new(0.05);
        assert!(r.is_some());
        assert!((r.unwrap().as_percent() - 5.0).abs() < f64::EPSILON); // INVARIANT: 0.05 is finite
    }

    #[test]
    fn rate_rejects_nan() {
        assert!(InterestRate::new(f64::NAN).is_none());
    }

    #[test]
    fn rate_rejects_infinity() {
        assert!(InterestRate::new(f64::INFINITY).is_none());
    }

    // ========== Spread tests ==========

    #[test]
    fn spread_from_bid_ask() {
        let bid = Price::new(99.50);
        let ask = Price::new(100.50);
        let s = Spread::from_bid_ask(bid, ask);
        assert!((s.value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn spread_as_percent() {
        let bid = Price::new(99.0);
        let ask = Price::new(101.0);
        let s = Spread::from_bid_ask(bid, ask);
        // 2.0 / 100.0 * 100 = 2.0%
        assert!((s.as_percent_of_mid(bid, ask) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn spread_clamps_negative() {
        // Ask < Bid (inverted market) → spread = 0
        let bid = Price::new(101.0);
        let ask = Price::new(99.0);
        let s = Spread::from_bid_ask(bid, ask);
        assert!((s.value() - 0.0).abs() < f64::EPSILON);
    }

    // ========== Maturity tests ==========

    #[test]
    fn maturity_expiry() {
        let m = Maturity::new(0.0);
        assert!(m.is_expired());

        let m2 = Maturity::new(2.5);
        assert!(!m2.is_expired());
        assert!((m2.years() - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn maturity_clamps_negative() {
        let m = Maturity::new(-1.0);
        assert!((m.years() - 0.0).abs() < f64::EPSILON);
        assert!(m.is_expired());
    }

    // ========== Exposure tests ==========

    #[test]
    fn exposure_long_short() {
        let long = Exposure::new(100.0);
        let short = Exposure::new(-50.0);

        assert!(long.is_long());
        assert!(!long.is_short());
        assert!(short.is_short());
        assert!(!short.is_long());
        assert!((long.abs() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn exposure_netting() {
        let a = Exposure::new(100.0);
        let b = Exposure::new(-30.0);
        let net = a + b;
        assert!((net.value() - 70.0).abs() < f64::EPSILON);
    }

    #[test]
    fn exposure_negation() {
        let e = Exposure::new(50.0);
        let neg = -e;
        assert!((neg.value() - (-50.0)).abs() < f64::EPSILON);
    }

    // ========== TimeValueOfMoney tests ==========

    #[test]
    fn tvm_present_value() {
        let tvm = TimeValueOfMoney;
        let fv = Price::new(110.0);
        let rate = InterestRate::new(0.10).unwrap(); // INVARIANT: 0.10 is finite
        let pv = tvm.present_value(fv, rate, 1.0);
        assert!((pv.value() - 100.0).abs() < 0.01);
    }

    #[test]
    fn tvm_future_value() {
        let tvm = TimeValueOfMoney;
        let pv = Price::new(100.0);
        let rate = InterestRate::new(0.10).unwrap(); // INVARIANT: 0.10 is finite
        let fv = tvm.future_value(pv, rate, 1.0);
        assert!((fv.value() - 110.0).abs() < 0.01);
    }

    #[test]
    fn tvm_pv_fv_roundtrip() {
        let tvm = TimeValueOfMoney;
        let original = Price::new(1000.0);
        let rate = InterestRate::new(0.08).unwrap(); // INVARIANT: finite
        let periods = 5.0;

        let fv = tvm.future_value(original, rate, periods);
        let pv = tvm.present_value(fv, rate, periods);
        assert!((pv.value() - original.value()).abs() < 0.01);
    }

    #[test]
    fn tvm_compound_discrete() {
        let tvm = TimeValueOfMoney;
        let principal = Price::new(1000.0);
        let rate = InterestRate::new(0.10).unwrap(); // INVARIANT: finite
        let result = tvm.compound(principal, rate, 3);
        // 1000 × 1.1³ = 1331.0
        assert!((result.value() - 1331.0).abs() < 0.01);
    }

    #[test]
    fn tvm_compound_continuous() {
        let tvm = TimeValueOfMoney;
        let principal = Price::new(1000.0);
        let rate = InterestRate::new(0.10).unwrap(); // INVARIANT: finite
        let result = tvm.compound_continuous(principal, rate, 3.0);
        // 1000 × e^(0.3) ≈ 1349.86
        assert!((result.value() - 1349.86).abs() < 0.01);
    }

    #[test]
    fn tvm_effective_rate() {
        let tvm = TimeValueOfMoney;
        let nominal = InterestRate::new(0.12).unwrap(); // INVARIANT: finite
        // Monthly compounding: (1 + 0.01)¹² - 1 ≈ 0.12683
        let eff = tvm.effective_rate(nominal, 12);
        assert!((eff.value() - 0.12683).abs() < 0.001);
    }

    #[test]
    fn tvm_effective_rate_zero_periods() {
        let tvm = TimeValueOfMoney;
        let nominal = InterestRate::new(0.12).unwrap(); // INVARIANT: finite
        let eff = tvm.effective_rate(nominal, 0);
        assert!((eff.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn tvm_present_value_extreme_negative_rate() {
        let tvm = TimeValueOfMoney;
        let fv = Price::new(100.0);
        let rate = InterestRate::new(-1.5).unwrap(); // INVARIANT: finite
        // Rate ≤ -1 should return ZERO (can't discount with ≤ -100% rate)
        let pv = tvm.present_value(fv, rate, 1.0);
        assert!((pv.value() - 0.0).abs() < f64::EPSILON);
    }

    // ========== Measured types tests ==========

    #[test]
    fn measured_price_confidence() {
        let mp = MeasuredPrice::new(Price::new(150.0), Confidence::new(0.90));
        assert!((mp.value.value() - 150.0).abs() < f64::EPSILON);
        assert!((mp.confidence.value() - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn measured_return_confidence() {
        let mr = MeasuredReturn::new(Return::new(0.05).unwrap(), Confidence::new(0.75));
        assert!((mr.value.value() - 0.05).abs() < f64::EPSILON);
        assert!((mr.confidence.value() - 0.75).abs() < f64::EPSILON);
    }

    // ========== Cross-domain transfer tests ==========

    #[test]
    fn discount_models_signal_decay() {
        // Finance: FV $100 discounted at 10% for 5 years
        // PV transfer: signal strength decays over time at same rate
        let tvm = TimeValueOfMoney;
        let signal_strength = Price::new(100.0);
        let decay_rate = InterestRate::new(0.10).unwrap(); // INVARIANT: finite
        let years_old = 5.0;

        let current_relevance = tvm.present_value(signal_strength, decay_rate, years_old);

        // Signal loses ~39% of relevance over 5 years at 10% decay
        assert!(current_relevance.value() < signal_strength.value());
        assert!((current_relevance.value() - 62.09).abs() < 0.1);
    }

    #[test]
    fn compound_models_tech_debt() {
        // Finance: $1000 at 20% annual compound for 5 years
        // Software: small debt grows recursively
        let tvm = TimeValueOfMoney;
        let initial_debt = Price::new(1000.0);
        let growth_rate = InterestRate::new(0.20).unwrap(); // INVARIANT: finite
        let accumulated = tvm.compound(initial_debt, growth_rate, 5);

        // 1000 × 1.2⁵ = 2488.32
        assert!((accumulated.value() - 2488.32).abs() < 0.1);
        // Debt more than doubled in 5 periods
        assert!(accumulated.value() > 2.0 * initial_debt.value());
    }

    #[test]
    fn spread_measures_detection_gap() {
        // Finance: bid-ask spread = transaction cost
        // PV transfer: spread between two data sources = disagreement
        let source_a = Price::new(2.5); // PRR from FAERS
        let source_b = Price::new(3.1); // PRR from EudraVigilance
        let gap = Spread::from_bid_ask(source_a, source_b);

        // 0.6 spread = 21.4% disagreement
        assert!((gap.value() - 0.6).abs() < f64::EPSILON);
        assert!((gap.as_percent_of_mid(source_a, source_b) - 21.43).abs() < 0.1);
    }
}
