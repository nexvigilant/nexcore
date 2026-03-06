//! Refinery theory applied to the flywheel.
//!
//! A refinery is a sequenced set of unit operations converting crude feedstock
//! into differentiated products of increasing value. The flywheel IS a refinery:
//! raw signals enter, get fractionated by type, processed through unit nodes,
//! blended into actionable intelligence, and recycled if unconverted.
//!
//! ## Three Refinery Laws → Flywheel Invariants
//!
//! 1. **Mass Balance** (N): signals_in = products + recycle + loss
//! 2. **Energy Balance** (→): value_produced / tokens_consumed
//! 3. **Irreversibility** (∝): classified signals cannot un-classify
//!
//! ## T1 Primitive Grounding: σ(Sequence) + μ(Mapping) + ∂(Boundary) + N(Quantity) + ∝(Irreversibility)

use serde::{Deserialize, Serialize};

// ============================================================================
// Signal Fractions — what the crude separates into
// ============================================================================

/// Signal fraction after fractionation (distillation by type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalFraction {
    /// Health signals → homeostasis node
    Health,
    /// Threat signals → immunity node
    Threat,
    /// Learning signals → trust/insight nodes
    Learning,
    /// Noise — stripped during desalting (pre-treatment)
    Noise,
}

impl std::fmt::Display for SignalFraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Health => write!(f, "health"),
            Self::Threat => write!(f, "threat"),
            Self::Learning => write!(f, "learning"),
            Self::Noise => write!(f, "noise"),
        }
    }
}

// ============================================================================
// Product Types — what the refinery outputs
// ============================================================================

/// Refined product after unit processing and blending.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductType {
    /// Immediate response (gasoline — high octane, fast burn)
    Action,
    /// Persisted artifact (diesel — steady, durable)
    Report,
    /// Compounded knowledge (kerosene — fuels future flights)
    Learning,
    /// Recycled signal — returned as next cycle's feedstock
    Recycle,
    /// Loss — signal that neither produced nor recycled
    Loss,
}

impl std::fmt::Display for ProductType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Action => write!(f, "action"),
            Self::Report => write!(f, "report"),
            Self::Learning => write!(f, "learning"),
            Self::Recycle => write!(f, "recycle"),
            Self::Loss => write!(f, "loss"),
        }
    }
}

// ============================================================================
// Refinery Metrics — the 6 measures from refinery theory
// ============================================================================

/// Complete refinery metrics for one flywheel cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineryMetrics {
    /// Total signals entering the system this cycle.
    pub crude_in: u64,
    /// Signals that became actionable products.
    pub products_out: u64,
    /// Signals returned for reprocessing.
    pub recycled: u64,
    /// Signals that vanished (neither product nor recycle).
    pub loss: u64,

    /// Per-product-type breakdown.
    pub product_breakdown: ProductBreakdown,
}

/// Breakdown of products by type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductBreakdown {
    pub actions: u64,
    pub reports: u64,
    pub learnings: u64,
}

impl RefineryMetrics {
    /// Yield: actionable_outputs / total_inputs.
    /// What % of signals became products.
    pub fn yield_pct(&self) -> f64 {
        if self.crude_in == 0 {
            return 0.0;
        }
        self.products_out as f64 / self.crude_in as f64
    }

    /// Conversion: processed_signals / raw_signals.
    /// What % of crude entered unit processes (vs stripped as noise).
    pub fn conversion(&self) -> f64 {
        if self.crude_in == 0 {
            return 0.0;
        }
        let processed = self.products_out + self.recycled;
        processed as f64 / self.crude_in as f64
    }

    /// Selectivity: desired_product / all_products.
    /// What % of output is the RIGHT product (actions, not noise).
    pub fn selectivity(&self) -> f64 {
        if self.products_out == 0 {
            return 0.0;
        }
        self.product_breakdown.actions as f64 / self.products_out as f64
    }

    /// Recycle ratio: recycled / (recycled + products).
    /// Lower is better — means less reprocessing needed.
    pub fn recycle_ratio(&self) -> f64 {
        let total = self.recycled + self.products_out;
        if total == 0 {
            return 0.0;
        }
        self.recycled as f64 / total as f64
    }

    /// Loss ratio: loss / crude_in.
    /// Signals that disappeared — a leak in the system.
    pub fn loss_ratio(&self) -> f64 {
        if self.crude_in == 0 {
            return 0.0;
        }
        self.loss as f64 / self.crude_in as f64
    }

    /// Octane rating: confidence × coverage (product quality).
    /// Takes external confidence and coverage as inputs.
    pub fn octane_rating(confidence: f64, coverage: f64) -> f64 {
        (confidence * coverage).clamp(0.0, 1.0)
    }

    /// Verify mass balance: crude_in == products_out + recycled + loss.
    /// Returns the imbalance (should be 0).
    pub fn mass_balance_check(&self) -> i64 {
        self.crude_in as i64 - (self.products_out as i64 + self.recycled as i64 + self.loss as i64)
    }
}

// ============================================================================
// Fractionation Result
// ============================================================================

/// Result of fractionating crude signals into typed streams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FractionationResult {
    pub health_count: u64,
    pub threat_count: u64,
    pub learning_count: u64,
    pub noise_stripped: u64,
    pub total_input: u64,
}

impl FractionationResult {
    /// Noise ratio — what % was stripped during desalting.
    pub fn noise_ratio(&self) -> f64 {
        if self.total_input == 0 {
            return 0.0;
        }
        self.noise_stripped as f64 / self.total_input as f64
    }

    /// Fractionation efficiency — what % reached a processing stream.
    pub fn efficiency(&self) -> f64 {
        if self.total_input == 0 {
            return 0.0;
        }
        let useful = self.health_count + self.threat_count + self.learning_count;
        useful as f64 / self.total_input as f64
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metrics() -> RefineryMetrics {
        RefineryMetrics {
            crude_in: 100,
            products_out: 70,
            recycled: 20,
            loss: 10,
            product_breakdown: ProductBreakdown {
                actions: 40,
                reports: 20,
                learnings: 10,
            },
        }
    }

    #[test]
    fn yield_pct() {
        let m = sample_metrics();
        assert!((m.yield_pct() - 0.70).abs() < f64::EPSILON);
    }

    #[test]
    fn conversion() {
        let m = sample_metrics();
        // (70 + 20) / 100 = 0.90
        assert!((m.conversion() - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn selectivity() {
        let m = sample_metrics();
        // 40 / 70 = 0.5714...
        assert!((m.selectivity() - 40.0 / 70.0).abs() < 1e-10);
    }

    #[test]
    fn recycle_ratio() {
        let m = sample_metrics();
        // 20 / (20 + 70) = 0.2222...
        assert!((m.recycle_ratio() - 20.0 / 90.0).abs() < 1e-10);
    }

    #[test]
    fn loss_ratio() {
        let m = sample_metrics();
        assert!((m.loss_ratio() - 0.10).abs() < f64::EPSILON);
    }

    #[test]
    fn mass_balance_holds() {
        let m = sample_metrics();
        assert_eq!(m.mass_balance_check(), 0);
    }

    #[test]
    fn mass_balance_detects_leak() {
        let m = RefineryMetrics {
            crude_in: 100,
            products_out: 50,
            recycled: 20,
            loss: 10,
            product_breakdown: ProductBreakdown::default(),
        };
        assert_eq!(m.mass_balance_check(), 20); // 20 unaccounted
    }

    #[test]
    fn octane_rating_bounded() {
        assert!((RefineryMetrics::octane_rating(0.9, 0.95) - 0.855).abs() < 1e-10);
        assert!((RefineryMetrics::octane_rating(1.5, 1.0) - 1.0).abs() < f64::EPSILON);
        assert!((RefineryMetrics::octane_rating(0.0, 0.5) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn zero_crude_safe() {
        let m = RefineryMetrics {
            crude_in: 0,
            products_out: 0,
            recycled: 0,
            loss: 0,
            product_breakdown: ProductBreakdown::default(),
        };
        assert!((m.yield_pct() - 0.0).abs() < f64::EPSILON);
        assert!((m.conversion() - 0.0).abs() < f64::EPSILON);
        assert!((m.selectivity() - 0.0).abs() < f64::EPSILON);
        assert!((m.recycle_ratio() - 0.0).abs() < f64::EPSILON);
        assert!((m.loss_ratio() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fractionation_efficiency() {
        let f = FractionationResult {
            health_count: 30,
            threat_count: 20,
            learning_count: 40,
            noise_stripped: 10,
            total_input: 100,
        };
        assert!((f.efficiency() - 0.90).abs() < f64::EPSILON);
        assert!((f.noise_ratio() - 0.10).abs() < f64::EPSILON);
    }

    #[test]
    fn serialization_roundtrip() {
        let m = sample_metrics();
        let json = serde_json::to_string(&m).unwrap();
        let parsed: RefineryMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.crude_in, 100);
        assert_eq!(parsed.products_out, 70);
    }
}
