//! # Capability 27: Federal Reserve Act (Token Stability)
//!
//! Implementation of the Federal Reserve Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "Monetary Policy" and "Token Stability" of the Union.
//!
//! Matches 1:1 to the US Federal Reserve mandate to promote the
//! effective operation of the U.S. economy and, more generally,
//! the public interest.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │            FEDERAL RESERVE ACT (CAP-027)                    │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                              │
//! │  TOKEN ECONOMICS                                             │
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐                      │
//! │  │ Budget  │  │  Rate   │  │  Cost   │                      │
//! │  │Tracking │  │Limiting │  │Metrics  │                      │
//! │  └────┬────┘  └────┬────┘  └────┬────┘                      │
//! │       │            │            │                            │
//! │       ▼            ▼            ▼                            │
//! │  ┌─────────────────────────────────────────────┐            │
//! │  │          MONETARY POLICY ENGINE             │            │
//! │  │  • Token budget allocation                  │            │
//! │  │  • Inflation detection (overspending)       │            │
//! │  │  • Model cost optimization                  │            │
//! │  └────────────────────┬────────────────────────┘            │
//! │                       │                                      │
//! │                       ▼                                      │
//! │  ┌─────────────────────────────────────────────┐            │
//! │  │          STABILITY CONTROLS                 │            │
//! │  │  • Rate limiting (requests/window)          │            │
//! │  │  • Budget caps per session                  │            │
//! │  │  • Model delegation for cost reduction      │            │
//! │  └─────────────────────────────────────────────┘            │
//! │                                                              │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// T1 PRIMITIVES (Universal)
// ============================================================================

/// T1: ModelTier - LLM model cost tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelTier {
    /// Cheapest, fastest (e.g., Haiku).
    Economy,
    /// Balanced cost/capability (e.g., Sonnet).
    Standard,
    /// Most capable, expensive (e.g., Opus).
    Premium,
}

impl ModelTier {
    /// Get cost multiplier relative to Economy.
    pub fn cost_multiplier(&self) -> f64 {
        match self {
            Self::Economy => 1.0,
            Self::Standard => 3.0,
            Self::Premium => 15.0,
        }
    }

    /// Get tokens per dollar estimate (input).
    pub fn tokens_per_dollar_input(&self) -> u64 {
        match self {
            Self::Economy => 4_000_000,  // $0.25/MTok
            Self::Standard => 1_000_000, // $1/MTok
            Self::Premium => 200_000,    // $5/MTok
        }
    }

    /// Get tokens per dollar estimate (output).
    pub fn tokens_per_dollar_output(&self) -> u64 {
        match self {
            Self::Economy => 800_000,  // $1.25/MTok
            Self::Standard => 200_000, // $5/MTok
            Self::Premium => 40_000,   // $25/MTok
        }
    }
}

/// T1: StabilityLevel - Current economic health.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StabilityLevel {
    /// Under budget, healthy spending.
    Stable,
    /// Approaching budget limits.
    Cautious,
    /// Budget exceeded, restrict spending.
    Restricted,
    /// Critical overspend, emergency measures.
    Emergency,
}

// ============================================================================
// T2-P PRIMITIVES (Cross-Domain)
// ============================================================================

/// T2-P: InflationRate - The quantified change in resource costs.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct InflationRate(pub f64);

impl InflationRate {
    /// Create new rate (can be negative for deflation).
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    /// Is inflation above target (2%)?
    pub fn is_high(&self) -> bool {
        self.0 > 0.02
    }

    /// Is there deflation?
    pub fn is_deflating(&self) -> bool {
        self.0 < 0.0
    }

    /// Inner value.
    pub fn value(&self) -> f64 {
        self.0
    }
}

/// T2-P: TokenCount - Atomic token unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TokenCount(pub u64);

impl TokenCount {
    /// Create new count.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Add tokens.
    pub fn add(&self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Convert to kilotoken display.
    pub fn as_ktok(&self) -> f64 {
        self.0 as f64 / 1000.0
    }

    /// Convert to megatoken display.
    pub fn as_mtok(&self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }

    /// Inner value.
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// T2-P: CostEstimate - Estimated USD cost.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CostEstimate(pub f64);

impl CostEstimate {
    /// Create new estimate.
    pub fn new(value: f64) -> Self {
        Self(value.max(0.0))
    }

    /// Compute from token usage.
    pub fn from_tokens(input: TokenCount, output: TokenCount, tier: ModelTier) -> Self {
        let input_cost = input.0 as f64 / tier.tokens_per_dollar_input() as f64;
        let output_cost = output.0 as f64 / tier.tokens_per_dollar_output() as f64;
        Self::new(input_cost + output_cost)
    }

    /// Inner value.
    pub fn value(&self) -> f64 {
        self.0
    }
}

// ============================================================================
// T2-C COMPOSITES (Cross-Domain)
// ============================================================================

/// T2-C: TokenUsage - Usage metrics for a period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Input tokens consumed.
    pub input_tokens: TokenCount,
    /// Output tokens generated.
    pub output_tokens: TokenCount,
    /// Model tier used.
    pub model_tier: ModelTier,
    /// Timestamp (Unix).
    pub timestamp: i64,
}

impl TokenUsage {
    /// Total tokens.
    pub fn total(&self) -> TokenCount {
        self.input_tokens.add(self.output_tokens)
    }

    /// Estimated cost.
    pub fn cost(&self) -> CostEstimate {
        CostEstimate::from_tokens(self.input_tokens, self.output_tokens, self.model_tier)
    }
}

/// T2-C: MonetaryPolicy - The current economic constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonetaryPolicy {
    /// Daily token budget.
    pub daily_budget: TokenCount,
    /// Session token budget.
    pub session_budget: TokenCount,
    /// Target inflation rate.
    pub target_inflation: InflationRate,
    /// Current interest rate (cost multiplier for exceeding budget).
    pub interest_rate: f64,
    /// Rate limit (requests per minute).
    pub rate_limit_rpm: u32,
    /// Preferred model tier.
    pub preferred_tier: ModelTier,
}

impl Default for MonetaryPolicy {
    fn default() -> Self {
        Self {
            daily_budget: TokenCount(10_000_000),  // 10M tokens/day
            session_budget: TokenCount(1_000_000), // 1M tokens/session
            target_inflation: InflationRate(0.02), // 2% target
            interest_rate: 1.0,                    // No penalty
            rate_limit_rpm: 60,                    // 60 req/min
            preferred_tier: ModelTier::Standard,
        }
    }
}

/// T2-C: BudgetReport - Current spending status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetReport {
    /// Tokens used today.
    pub daily_used: TokenCount,
    /// Tokens used this session.
    pub session_used: TokenCount,
    /// Daily budget remaining.
    pub daily_remaining: TokenCount,
    /// Session budget remaining.
    pub session_remaining: TokenCount,
    /// Current stability level.
    pub stability: StabilityLevel,
    /// Estimated cost so far.
    pub estimated_cost: CostEstimate,
    /// Inflation rate (spending vs budget).
    pub inflation: InflationRate,
}

/// T2-C: RateLimitStatus - Current rate limiting state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    /// Requests in current window.
    pub current_requests: u32,
    /// Window limit.
    pub window_limit: u32,
    /// Seconds until window reset.
    pub reset_in_seconds: u32,
    /// Is currently throttled?
    pub is_throttled: bool,
}

// ============================================================================
// T3 DOMAIN-SPECIFIC (FederalReserveAct)
// ============================================================================

/// T3: FederalReserveAct - Capability 27 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederalReserveAct {
    /// The unique capability identifier.
    pub id: String,
    /// Whether the monetary policy engine is active.
    pub stability_active: bool,
    /// Current monetary policy.
    policy: MonetaryPolicy,
    /// Usage tracking by session.
    usage_by_session: HashMap<String, Vec<TokenUsage>>,
    /// Daily usage total.
    daily_usage: TokenCount,
    /// Current session ID.
    current_session: Option<String>,
    /// Request count in current window.
    request_count: u32,
    /// Window start timestamp.
    window_start: i64,
}

impl Default for FederalReserveAct {
    fn default() -> Self {
        Self::new()
    }
}

impl FederalReserveAct {
    /// Creates a new instance of the FederalReserveAct.
    pub fn new() -> Self {
        Self {
            id: "CAP-027".into(),
            stability_active: true,
            policy: MonetaryPolicy::default(),
            usage_by_session: HashMap::new(),
            daily_usage: TokenCount(0),
            current_session: None,
            request_count: 0,
            window_start: chrono::Utc::now().timestamp(),
        }
    }

    /// Set current session.
    pub fn set_session(&mut self, session_id: &str) {
        self.current_session = Some(session_id.to_string());
    }

    /// Set monetary policy.
    pub fn set_policy(&mut self, policy: MonetaryPolicy) -> Measured<MonetaryPolicy> {
        self.policy = policy.clone();
        Measured::uncertain(policy, Confidence::new(0.98))
    }

    /// Record token usage.
    pub fn record_usage(&mut self, usage: TokenUsage) -> Measured<BudgetReport> {
        let total = usage.total();

        // Update daily total
        self.daily_usage = self.daily_usage.add(total);

        // Update session tracking
        if let Some(session) = &self.current_session {
            self.usage_by_session
                .entry(session.clone())
                .or_default()
                .push(usage);
        }

        // Increment request count
        self.request_count += 1;

        self.get_budget_report()
    }

    /// Get current budget report.
    pub fn get_budget_report(&self) -> Measured<BudgetReport> {
        let session_used = self.get_session_usage();
        let daily_remaining = TokenCount(
            self.policy
                .daily_budget
                .0
                .saturating_sub(self.daily_usage.0),
        );
        let session_remaining =
            TokenCount(self.policy.session_budget.0.saturating_sub(session_used.0));

        // Calculate stability level
        let daily_pct = self.daily_usage.0 as f64 / self.policy.daily_budget.0 as f64;
        let stability = match daily_pct {
            p if p < 0.5 => StabilityLevel::Stable,
            p if p < 0.8 => StabilityLevel::Cautious,
            p if p < 1.0 => StabilityLevel::Restricted,
            _ => StabilityLevel::Emergency,
        };

        // Calculate inflation (spending rate vs target)
        let inflation = InflationRate::new(daily_pct - 1.0);

        let report = BudgetReport {
            daily_used: self.daily_usage,
            session_used,
            daily_remaining,
            session_remaining,
            stability,
            estimated_cost: self.estimate_daily_cost(),
            inflation,
        };

        let confidence = if stability == StabilityLevel::Stable {
            0.95
        } else {
            0.8
        };
        Measured::uncertain(report, Confidence::new(confidence))
    }

    /// Get session token usage.
    fn get_session_usage(&self) -> TokenCount {
        self.current_session
            .as_ref()
            .and_then(|s| self.usage_by_session.get(s))
            .map(|usages| {
                usages
                    .iter()
                    .fold(TokenCount(0), |acc, u| acc.add(u.total()))
            })
            .unwrap_or(TokenCount(0))
    }

    /// Estimate daily cost.
    fn estimate_daily_cost(&self) -> CostEstimate {
        let mut total = 0.0;
        for usages in self.usage_by_session.values() {
            for u in usages {
                total += u.cost().0;
            }
        }
        CostEstimate::new(total)
    }

    /// Check rate limit status.
    pub fn check_rate_limit(&mut self) -> RateLimitStatus {
        let now = chrono::Utc::now().timestamp();
        let window_duration = 60; // 1 minute window

        // Reset window if expired
        if now - self.window_start >= window_duration {
            self.request_count = 0;
            self.window_start = now;
        }

        let reset_in = (self.window_start + window_duration - now).max(0) as u32;
        let is_throttled = self.request_count >= self.policy.rate_limit_rpm;

        RateLimitStatus {
            current_requests: self.request_count,
            window_limit: self.policy.rate_limit_rpm,
            reset_in_seconds: reset_in,
            is_throttled,
        }
    }

    /// Recommend model tier based on budget status.
    pub fn recommend_model_tier(&self) -> ModelTier {
        let report = self.get_budget_report();

        match report.value.stability {
            StabilityLevel::Stable => self.policy.preferred_tier,
            StabilityLevel::Cautious => {
                // Downgrade one tier if possible
                match self.policy.preferred_tier {
                    ModelTier::Premium => ModelTier::Standard,
                    _ => ModelTier::Economy,
                }
            }
            StabilityLevel::Restricted | StabilityLevel::Emergency => ModelTier::Economy,
        }
    }

    /// Reset daily counters.
    pub fn reset_daily(&mut self) {
        self.daily_usage = TokenCount(0);
        self.usage_by_session.clear();
    }

    /// Get current policy.
    pub fn current_policy(&self) -> &MonetaryPolicy {
        &self.policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_cost_calculation() {
        let input = TokenCount(1_000_000);
        let output = TokenCount(100_000);

        let economy_cost = CostEstimate::from_tokens(input, output, ModelTier::Economy);
        let standard_cost = CostEstimate::from_tokens(input, output, ModelTier::Standard);

        // Economy should be cheaper
        assert!(economy_cost.0 < standard_cost.0);
    }

    #[test]
    fn test_budget_tracking() {
        let mut fed = FederalReserveAct::new();
        fed.set_session("test-session");

        let usage = TokenUsage {
            input_tokens: TokenCount(10_000),
            output_tokens: TokenCount(5_000),
            model_tier: ModelTier::Standard,
            timestamp: chrono::Utc::now().timestamp(),
        };

        let report = fed.record_usage(usage);
        assert_eq!(report.value.session_used.0, 15_000);
        assert_eq!(report.value.daily_used.0, 15_000);
    }

    #[test]
    fn test_stability_levels() {
        let mut fed = FederalReserveAct::new();

        // Under 50% = Stable
        fed.daily_usage = TokenCount(4_000_000);
        let report = fed.get_budget_report();
        assert_eq!(report.value.stability, StabilityLevel::Stable);

        // 50-80% = Cautious
        fed.daily_usage = TokenCount(7_000_000);
        let report = fed.get_budget_report();
        assert_eq!(report.value.stability, StabilityLevel::Cautious);

        // 80-100% = Restricted
        fed.daily_usage = TokenCount(9_000_000);
        let report = fed.get_budget_report();
        assert_eq!(report.value.stability, StabilityLevel::Restricted);

        // Over 100% = Emergency
        fed.daily_usage = TokenCount(11_000_000);
        let report = fed.get_budget_report();
        assert_eq!(report.value.stability, StabilityLevel::Emergency);
    }

    #[test]
    fn test_model_tier_recommendation() {
        let mut fed = FederalReserveAct::new();
        fed.policy.preferred_tier = ModelTier::Premium;

        // Stable = use preferred
        fed.daily_usage = TokenCount(2_000_000);
        assert_eq!(fed.recommend_model_tier(), ModelTier::Premium);

        // Cautious = downgrade one
        fed.daily_usage = TokenCount(7_000_000);
        assert_eq!(fed.recommend_model_tier(), ModelTier::Standard);

        // Emergency = economy only
        fed.daily_usage = TokenCount(12_000_000);
        assert_eq!(fed.recommend_model_tier(), ModelTier::Economy);
    }

    #[test]
    fn test_inflation_rate() {
        let high = InflationRate::new(0.05);
        assert!(high.is_high());

        let normal = InflationRate::new(0.01);
        assert!(!normal.is_high());

        let deflation = InflationRate::new(-0.02);
        assert!(deflation.is_deflating());
    }
}
