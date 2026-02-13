//! # MCP Efficacy Monitor Hook (CTVP Phase 4 Surveillance)
//!
//! **Event:** SessionStart
//!
//! Monitors MCP tool adoption metrics and alerts on drift. This provides
//! Phase 4 (Surveillance) continuous validation for the MCP suggestion system.
//!
//! ## Purpose
//!
//! Implements CTVP Phase 4 surveillance for MCP tool suggestions:
//! - Tracks Cumulative Adoption Rate (CAR) over sessions
//! - Detects drift when adoption falls below thresholds
//! - Alerts user when intervention may be needed
//!
//! ## Metrics
//!
//! | Metric | Formula | Target |
//! |--------|---------|--------|
//! | CAR | followup_count / suggestion_count | ≥ 30% |
//! | Trend | Direction over rolling window | Stable or Improving |
//!
//! ## Thresholds (from config)
//!
//! - `car_target`: Target CAR (default 0.30)
//! - `car_alert_threshold`: Alert below this (default 0.15)
//! - `min_sessions`: Minimum data before monitoring (default 10)
//!
//! ## Exit Codes
//!
//! - 0 (skip): Insufficient data or on-target
//! - 0 (context): Below target, adds session context

use nexcore_hooks::ctvp::{DriftDetector, TrendDirection};
use nexcore_hooks::mcp_config::McpEfficacyConfig;
use nexcore_hooks::mcp_efficacy::McpEfficacyRegistry;
use nexcore_hooks::{exit_skip_session, exit_with_session_context};

fn main() {
    let config = McpEfficacyConfig::load();

    // Skip if not enabled
    if !config.feature_flags.enabled {
        exit_skip_session();
    }

    let registry = McpEfficacyRegistry::load();

    // Compute current metrics
    let total = registry.suggestions.len() as u32;
    let followup = registry
        .usages
        .iter()
        .filter(|u| u.followed_suggestion)
        .count() as u32;

    // Need minimum data
    if total < config.thresholds.min_sessions {
        exit_skip_session();
    }

    // Calculate CAR
    let car = if total == 0 {
        0.0
    } else {
        followup as f64 / total as f64
    };

    // Create drift detector
    let mut detector = DriftDetector::new(
        config.thresholds.car_target,
        config.thresholds.car_alert_threshold,
    );

    // Record current value
    detector.record(car);

    // Check for drift
    let trend = detector.trend();
    let is_drifting = detector.is_drifting(car);
    let meets_target = detector.meets_target(car);

    // Generate status message
    let status = if meets_target {
        format!("✅ CAR {:.1}% - On target", car * 100.0)
    } else if is_drifting {
        format!(
            "🚨 CAR {:.1}% - DRIFTING (below {:.0}%)",
            car * 100.0,
            config.thresholds.car_alert_threshold * 100.0
        )
    } else {
        format!(
            "⚠️ CAR {:.1}% - Below target {:.0}%",
            car * 100.0,
            config.thresholds.car_target * 100.0
        )
    };

    let trend_emoji = match trend {
        TrendDirection::Improving => "📈",
        TrendDirection::Stable => "➡️",
        TrendDirection::Degrading => "📉",
        TrendDirection::Unknown => "❓",
    };

    // Only show context if there's something noteworthy
    if is_drifting || !meets_target {
        let context = format!(
            "\n📊 **MCP EFFICACY MONITOR** ─────────────────────────────\n\
             {}\n\
             Trend: {} {:?}\n\
             Sessions: {} suggestions, {} followup\n\
             Run `mcp_efficacy_report` for details\n\
             ───────────────────────────────────────────────────────\n",
            status, trend_emoji, trend, total, followup
        );
        exit_with_session_context(&context);
    }

    exit_skip_session();
}
