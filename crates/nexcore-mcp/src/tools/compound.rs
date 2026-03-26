//! Compound Growth MCP tool — primitive basis velocity tracking.
//!
//! Reports compound growth metrics by constructing a CompoundTracker with the
//! current state of our primitive basis. Tracks velocity gains from investments
//! in different tiers.
//!
//! Current basis snapshot (as of 2026-02-05):
//! - T1: 15 (the Lex Primitiva)
//! - T2-P: 2 (Stratification, Reference Density)
//! - T2-C: 13 (7 HOF + 6 synthesis primitives)
//! - T3: 0
//! - Reused: 28 (all currently cataloged)
//! - Total needed: 30

use crate::params::CompoundGrowthParams;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Current primitive basis snapshot (hard-coded as of 2026-02-05).
const T1_COUNT: u32 = 15;
const T2_P_COUNT: u32 = 2;
const T2_C_COUNT: u32 = 13;
const T3_COUNT: u32 = 0;
const REUSED_COUNT: u32 = 28;
const TOTAL_NEEDED: u32 = 30;

/// Decay constant for reuse probability.
const DELTA: f64 = 0.1;

/// Transfer efficiency multipliers by tier.
const TRANSFER_T1: f64 = 1.0;
const TRANSFER_T2_P: f64 = 0.9;
const TRANSFER_T2_C: f64 = 0.7;
const TRANSFER_T3: f64 = 0.4;

/// Compound growth tracker (inlined from lex-primitiva).
#[derive(Debug, Clone)]
struct CompoundTracker {
    t1: u32,
    t2_p: u32,
    t2_c: u32,
    t3: u32,
    reused: u32,
    total_needed: u32,
}

impl CompoundTracker {
    /// Create tracker with current basis snapshot.
    fn new() -> Self {
        Self {
            t1: T1_COUNT,
            t2_p: T2_P_COUNT,
            t2_c: T2_C_COUNT,
            t3: T3_COUNT,
            reused: REUSED_COUNT,
            total_needed: TOTAL_NEEDED,
        }
    }

    /// Calculate transfer efficiency (weighted average of tier multipliers).
    fn transfer_efficiency(&self) -> f64 {
        let total = self.t1 + self.t2_p + self.t2_c + self.t3;
        if total == 0 {
            return 0.0;
        }

        let weighted_sum = (self.t1 as f64 * TRANSFER_T1)
            + (self.t2_p as f64 * TRANSFER_T2_P)
            + (self.t2_c as f64 * TRANSFER_T2_C)
            + (self.t3 as f64 * TRANSFER_T3);

        weighted_sum / total as f64
    }

    /// Calculate reuse rate with exponential decay.
    fn reuse_rate(&self) -> f64 {
        let ratio = if self.total_needed == 0 {
            0.0
        } else {
            self.reused as f64 / self.total_needed as f64
        };

        ratio * (-DELTA * ratio).exp()
    }

    /// Calculate velocity: basis_size × transfer_efficiency × reuse_rate.
    fn velocity(&self) -> f64 {
        let basis_size = (self.t1 + self.t2_p + self.t2_c + self.t3) as f64;
        basis_size * self.transfer_efficiency() * self.reuse_rate()
    }

    /// Get basis size.
    fn basis_size(&self) -> u32 {
        self.t1 + self.t2_p + self.t2_c + self.t3
    }

    /// Create projection by adding primitives to a tier.
    fn project(&self, tier: &str, count: u32) -> Result<Self, nexcore_error::NexError> {
        let mut projected = self.clone();

        match tier {
            "T1" => projected.t1 += count,
            "T2-P" => projected.t2_p += count,
            "T2-C" => projected.t2_c += count,
            "T3" => projected.t3 += count,
            _ => return Err(nexcore_error::nexerror!("Invalid tier: {}", tier)),
        }

        Ok(projected)
    }

    /// Calculate optimal investment recommendation.
    fn optimal_investment(&self) -> String {
        // Simple heuristic: recommend tier with highest transfer efficiency that's not empty
        if self.t1 > 0 {
            "T1 (Lex Primitiva) — highest transfer efficiency (1.0)".to_string()
        } else if self.t2_p > 0 {
            "T2-P (Primitive) — strong transfer efficiency (0.9)".to_string()
        } else if self.t2_c > 0 {
            "T2-C (Composite) — moderate transfer efficiency (0.7)".to_string()
        } else {
            "T3 (Domain) — foundational investment needed (0.4)".to_string()
        }
    }
}

/// Report compound growth metrics.
///
/// If no params: return current velocity, basis size, transfer efficiency, and optimal investment.
/// If params given: return projection result showing velocity gain from that investment.
pub fn compound_growth(params: CompoundGrowthParams) -> Result<CallToolResult, McpError> {
    let tracker = CompoundTracker::new();

    // If no parameters, return current state
    if params.add_tier.is_none() && params.add_count.is_none() {
        let result = json!({
            "current_state": {
                "velocity": format!("{:.4}", tracker.velocity()),
                "basis_size": tracker.basis_size(),
                "transfer_efficiency": format!("{:.4}", tracker.transfer_efficiency()),
                "reuse_rate": format!("{:.4}", tracker.reuse_rate()),
                "breakdown": {
                    "T1": tracker.t1,
                    "T2-P": tracker.t2_p,
                    "T2-C": tracker.t2_c,
                    "T3": tracker.t3,
                },
                "reused": tracker.reused,
                "total_needed": tracker.total_needed,
            },
            "optimal_investment": tracker.optimal_investment(),
            "formula": "velocity = basis_size × transfer_efficiency × reuse_rate",
            "transfer_multipliers": {
                "T1": TRANSFER_T1,
                "T2-P": TRANSFER_T2_P,
                "T2-C": TRANSFER_T2_C,
                "T3": TRANSFER_T3,
            },
        });

        return Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]));
    }

    // Projection mode: both parameters must be provided
    let tier = params.add_tier.ok_or_else(|| {
        McpError::invalid_params("add_tier required when add_count is specified", None)
    })?;

    let count = params.add_count.ok_or_else(|| {
        McpError::invalid_params("add_count required when add_tier is specified", None)
    })?;

    // Validate tier
    if !["T1", "T2-P", "T2-C", "T3"].contains(&tier.as_str()) {
        return Err(McpError::invalid_params(
            format!("Invalid tier: {}. Must be one of: T1, T2-P, T2-C, T3", tier),
            None,
        ));
    }

    // Calculate projection
    let projected = tracker
        .project(&tier, count)
        .map_err(|e| McpError::invalid_params(format!("Projection failed: {}", e), None))?;

    let velocity_before = tracker.velocity();
    let velocity_after = projected.velocity();
    let velocity_gain = velocity_after - velocity_before;
    let velocity_gain_percent = if velocity_before > 0.0 {
        (velocity_gain / velocity_before) * 100.0
    } else {
        0.0
    };

    let result = json!({
        "projection": {
            "investment": format!("Add {} primitives to {}", count, tier),
            "velocity_before": format!("{:.4}", velocity_before),
            "velocity_after": format!("{:.4}", velocity_after),
            "velocity_gain": format!("{:.4}", velocity_gain),
            "velocity_gain_percent": format!("{:.2}%", velocity_gain_percent),
        },
        "before": {
            "basis_size": tracker.basis_size(),
            "transfer_efficiency": format!("{:.4}", tracker.transfer_efficiency()),
            "reuse_rate": format!("{:.4}", tracker.reuse_rate()),
        },
        "after": {
            "basis_size": projected.basis_size(),
            "transfer_efficiency": format!("{:.4}", projected.transfer_efficiency()),
            "reuse_rate": format!("{:.4}", projected.reuse_rate()),
            "breakdown": {
                "T1": projected.t1,
                "T2-P": projected.t2_p,
                "T2-C": projected.t2_c,
                "T3": projected.t3,
            },
        },
        "recommendation": if velocity_gain > 0.0 {
            format!("Investment yields {:.2}% velocity gain", velocity_gain_percent)
        } else {
            "Investment does not increase velocity — consider alternative tier".to_string()
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
