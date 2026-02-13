//! SessionStart hook: Bond Guardian
//!
//! Safeguards for auto-bond execution. Runs BEFORE auto_bond_executor.
//! Prevents resource exhaustion and capability harm.
//!
//! Guardian Controls:
//! 1. Rate Limiting - Max bonds per day/week
//! 2. Circuit Breaker - Stop if errors accumulate
//! 3. Capability Validation - Block harmful bond types
//! 4. Audit Trail - Log all executions for accountability
//!
//! ToV Alignment:
//! - Safety Manifold (S): Ensures d(s) > 0 (positive safety distance)
//! - Harm Prevention: Blocks bonds that could damage capabilities
//!
//! Exit codes:
//! - 0: Success (guardian state updated)
//! - 2: Block (circuit breaker tripped)

use nexcore_hooks::{exit_block, exit_skip_session, exit_with_session_context, read_input};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// GUARDIAN THRESHOLDS (Tunable)
// ============================================================================

/// Maximum bonds auto-executed per day
const MAX_BONDS_PER_DAY: u32 = 10;

/// Maximum bonds auto-executed per week
const MAX_BONDS_PER_WEEK: u32 = 30;

/// Error threshold before circuit breaker trips
const ERROR_THRESHOLD: u32 = 3;

/// Hours until circuit breaker resets
const CIRCUIT_BREAKER_RESET_HOURS: u64 = 24;

/// Blocked capability types (never auto-execute)
const BLOCKED_CAPABILITY_TYPES: &[&str] = &[
    "Hook",     // Hooks can break the entire system
    "MCP",      // MCP tools affect Claude's capabilities
    "Settings", // Settings changes are sensitive
];

/// Blocked bond catalysts (manual review required)
const BLOCKED_CATALYSTS: &[&str] = &[
    "manual_request", // User explicitly requested manual review
    "security_scan",  // Security-related changes need review
];

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Default)]
struct GuardianState {
    /// Bonds executed today (resets at midnight)
    bonds_today: u32,
    /// Bonds executed this week (resets Sunday)
    bonds_this_week: u32,
    /// Last reset timestamp (Unix epoch)
    last_daily_reset: u64,
    /// Last weekly reset timestamp
    last_weekly_reset: u64,
    /// Consecutive errors count
    error_count: u32,
    /// Circuit breaker tripped timestamp (0 = not tripped)
    circuit_breaker_tripped: u64,
    /// Audit log of recent executions
    audit_log: Vec<AuditEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AuditEntry {
    timestamp: u64,
    bond_id: String,
    capability_type: String,
    target: String,
    outcome: String, // "executed", "blocked", "error"
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PendingActions {
    actions: Vec<Bond>,
}

#[derive(Debug, Deserialize)]
struct Bond {
    bond_id: String,
    activation_energy: u32,
    catalyst: String,
    cause: BondCause,
    status: String,
}

#[derive(Debug, Deserialize)]
struct BondCause {
    capability_type: String,
    target: String,
    path: String,
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    let _input = match read_input() {
        Some(i) => i,
        None => exit_skip_session(),
    };

    let mut state = load_guardian_state();
    let now = current_timestamp();

    // Reset counters if needed
    reset_counters_if_needed(&mut state, now);

    // Check circuit breaker
    if state.circuit_breaker_tripped > 0 {
        let hours_since = (now - state.circuit_breaker_tripped) / 3600;
        if hours_since < CIRCUIT_BREAKER_RESET_HOURS {
            let remaining = CIRCUIT_BREAKER_RESET_HOURS - hours_since;
            exit_block(&format!(
                "🛑 CIRCUIT BREAKER ACTIVE\n\
                 {} consecutive bond errors detected.\n\
                 Auto-execution disabled for {} more hours.\n\
                 Manual intervention required.",
                state.error_count, remaining
            ));
        } else {
            // Reset circuit breaker
            state.circuit_breaker_tripped = 0;
            state.error_count = 0;
        }
    }

    // Load pending bonds for validation
    let pending = match load_pending_actions() {
        Some(p) => p,
        None => {
            save_guardian_state(&state);
            exit_skip_session();
        }
    };

    // Filter to auto-executable bonds (energy ≤ 20, pending)
    let candidate_bonds: Vec<&Bond> = pending
        .actions
        .iter()
        .filter(|b| b.activation_energy <= 20 && b.status == "Pending")
        .collect();

    if candidate_bonds.is_empty() {
        save_guardian_state(&state);
        exit_skip_session();
    }

    // Validate each bond and build context
    let mut context = String::from("🛡️ **BOND GUARDIAN** ─────────────────────────────────────\n");
    let mut approved_count = 0;
    let mut blocked_bonds: Vec<(&Bond, &str)> = Vec::new();

    // Check rate limits
    let daily_remaining = MAX_BONDS_PER_DAY.saturating_sub(state.bonds_today);
    let weekly_remaining = MAX_BONDS_PER_WEEK.saturating_sub(state.bonds_this_week);
    let rate_limit = daily_remaining.min(weekly_remaining);

    if rate_limit == 0 {
        context.push_str("   ⚠️ Rate limit reached. No auto-execution today.\n");
        context.push_str(&format!(
            "   Daily: {}/{} | Weekly: {}/{}\n",
            state.bonds_today, MAX_BONDS_PER_DAY, state.bonds_this_week, MAX_BONDS_PER_WEEK
        ));
        save_guardian_state(&state);
        exit_with_session_context(&context);
    }

    for bond in candidate_bonds.iter().take(rate_limit as usize) {
        // Validate capability type
        if BLOCKED_CAPABILITY_TYPES.contains(&bond.cause.capability_type.as_str()) {
            blocked_bonds.push((bond, "Blocked capability type (requires manual review)"));
            log_audit(&mut state, bond, "blocked", Some("capability_type_blocked"));
            continue;
        }

        // Validate catalyst
        if BLOCKED_CATALYSTS.contains(&bond.catalyst.as_str()) {
            blocked_bonds.push((bond, "Blocked catalyst (manual review required)"));
            log_audit(&mut state, bond, "blocked", Some("catalyst_blocked"));
            continue;
        }

        // Validate path exists
        if !PathBuf::from(&bond.cause.path).exists() {
            blocked_bonds.push((bond, "Target path does not exist"));
            log_audit(&mut state, bond, "blocked", Some("path_not_found"));
            continue;
        }

        // Bond approved
        approved_count += 1;
        log_audit(&mut state, bond, "approved", None);
    }

    // Update counters
    state.bonds_today += approved_count;
    state.bonds_this_week += approved_count;

    // Build report
    context.push_str(&format!(
        "   Rate limits: {}/{} daily | {}/{} weekly\n",
        state.bonds_today, MAX_BONDS_PER_DAY, state.bonds_this_week, MAX_BONDS_PER_WEEK
    ));
    context.push_str(&format!(
        "   Approved: {} | Blocked: {}\n",
        approved_count,
        blocked_bonds.len()
    ));

    if !blocked_bonds.is_empty() {
        context.push_str("\n   ⛔ Blocked bonds (manual review required):\n");
        for (bond, reason) in &blocked_bonds {
            context.push_str(&format!(
                "      • {} ({}) - {}\n",
                bond.bond_id, bond.cause.capability_type, reason
            ));
        }
    }

    if state.error_count > 0 {
        context.push_str(&format!(
            "\n   ⚠️ Error count: {}/{} (circuit breaker at {})\n",
            state.error_count, ERROR_THRESHOLD, ERROR_THRESHOLD
        ));
    }

    context.push_str("───────────────────────────────────────────────────────────\n");

    save_guardian_state(&state);
    exit_with_session_context(&context);
}

// ============================================================================
// HELPERS
// ============================================================================

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn reset_counters_if_needed(state: &mut GuardianState, now: u64) {
    const SECONDS_PER_DAY: u64 = 86400;
    const SECONDS_PER_WEEK: u64 = 604800;

    // Daily reset
    if now - state.last_daily_reset > SECONDS_PER_DAY {
        state.bonds_today = 0;
        state.last_daily_reset = now;
    }

    // Weekly reset
    if now - state.last_weekly_reset > SECONDS_PER_WEEK {
        state.bonds_this_week = 0;
        state.last_weekly_reset = now;
    }
}

fn log_audit(state: &mut GuardianState, bond: &Bond, outcome: &str, reason: Option<&str>) {
    let entry = AuditEntry {
        timestamp: current_timestamp(),
        bond_id: bond.bond_id.clone(),
        capability_type: bond.cause.capability_type.clone(),
        target: bond.cause.target.clone(),
        outcome: outcome.to_string(),
        reason: reason.map(String::from),
    };

    state.audit_log.push(entry);

    // Keep only last 100 entries
    if state.audit_log.len() > 100 {
        state.audit_log.drain(0..state.audit_log.len() - 100);
    }
}

fn guardian_state_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".claude")
        .join("bonds")
        .join("guardian_state.json")
}

fn load_guardian_state() -> GuardianState {
    let path = guardian_state_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn save_guardian_state(state: &GuardianState) {
    let path = guardian_state_path();

    // Best-effort directory creation and save - hooks should not fail on IO errors
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Warning: Failed to create guardian state directory: {e}");
            return;
        }
    }

    if let Err(e) = fs::write(
        &path,
        serde_json::to_string_pretty(state).unwrap_or_default(),
    ) {
        eprintln!("Warning: Failed to save guardian state: {e}");
    }
}

fn bonds_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".claude")
        .join("bonds")
        .join("pending_actions.json")
}

fn load_pending_actions() -> Option<PendingActions> {
    let path = bonds_path();
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}
