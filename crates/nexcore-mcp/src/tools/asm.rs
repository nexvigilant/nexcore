//! Autonomous State Machine MCP tools.
//!
//! Exposes nexcore-asm's runtime state machines through MCP,
//! enabling autonomous state transitions via Claude Code.

use crate::params::asm::{
    AsmGuardDef, AsmHistoryParams, AsmListParams, AsmRegisterParams, AsmStateParams, AsmTickParams,
    AsmTransitionParams,
};
use nexcore_asm::prelude::*;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;
use std::sync::OnceLock;

/// Global tick engine (lazily initialized).
static ENGINE: OnceLock<parking_lot::Mutex<TickEngine>> = OnceLock::new();

fn engine() -> &'static parking_lot::Mutex<TickEngine> {
    ENGINE.get_or_init(|| parking_lot::Mutex::new(TickEngine::new()))
}

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_json(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

/// Convert MCP guard definition to nexcore-asm Guard.
fn convert_guard(def: &AsmGuardDef) -> Guard {
    match def {
        AsmGuardDef::Always => Guard::Always,
        AsmGuardDef::Never => Guard::Never,
        AsmGuardDef::Threshold {
            metric,
            op,
            threshold,
        } => {
            let comparison = match op.as_str() {
                ">" => ComparisonOp::GreaterThan,
                ">=" => ComparisonOp::GreaterOrEqual,
                "<" => ComparisonOp::LessThan,
                "<=" => ComparisonOp::LessOrEqual,
                "==" => ComparisonOp::Equal,
                _ => ComparisonOp::GreaterThan,
            };
            Guard::threshold(metric, comparison, *threshold)
        }
        AsmGuardDef::Flag { flag, expected } => Guard::flag(flag, *expected),
        AsmGuardDef::All { guards } => Guard::all(guards.iter().map(convert_guard).collect()),
        AsmGuardDef::Any { guards } => Guard::any(guards.iter().map(convert_guard).collect()),
    }
}

/// Convert state kind string to StateKind enum.
fn parse_state_kind(s: &str) -> StateKind {
    match s.to_lowercase().as_str() {
        "initial" => StateKind::Initial,
        "terminal" => StateKind::Terminal,
        "error" => StateKind::Error,
        _ => StateKind::Normal,
    }
}

/// Register a new autonomous state machine.
pub fn register(p: AsmRegisterParams) -> Result<CallToolResult, McpError> {
    let mut machine = Machine::new(&p.id, &p.name);

    for (name, kind_str) in &p.states {
        machine.add_state(name, parse_state_kind(kind_str));
    }

    for t in &p.transitions {
        let guard = convert_guard(&t.guard);
        machine.add_transition_with_priority(&t.name, &t.from, &t.to, guard, t.priority);
    }

    if let Err(e) = machine.validate() {
        return err_json(&format!("validation failed: {e}"));
    }

    let summary = machine.summary();
    let mut eng = engine().lock();
    if let Err(e) = eng.register(machine) {
        return err_json(&format!("registration failed: {e}"));
    }

    ok_json(json!({
        "status": "registered",
        "machine_id": summary.id,
        "name": summary.name,
        "state_count": summary.state_count,
        "transition_count": summary.transition_def_count
    }))
}

/// Tick all machines or a single machine.
pub fn tick(p: AsmTickParams) -> Result<CallToolResult, McpError> {
    let mut ctx = GuardContext::new();
    for (k, v) in &p.metrics {
        ctx.metrics.insert(k.clone(), *v);
    }
    for (k, v) in &p.flags {
        ctx.flags.insert(k.clone(), *v);
    }

    let mut eng = engine().lock();

    if let Some(ref id) = p.machine_id {
        // Tick one
        match eng.tick_one(id, &ctx) {
            Ok(record) => ok_json(json!({
                "status": "transitioned",
                "machine_id": id,
                "from": record.from.name(),
                "to": record.to.name(),
                "transition": record.transition_name,
                "guard": record.guard_description,
                "timestamp": record.timestamp.to_rfc3339()
            })),
            Err(MachineError::NoValidTransition) => ok_json(json!({
                "status": "no_transition",
                "machine_id": id
            })),
            Err(e) => err_json(&format!("tick failed for {id}: {e}")),
        }
    } else {
        // Tick all
        let result = eng.tick(&ctx);
        let transitions: Vec<serde_json::Value> = result
            .transitions
            .iter()
            .map(|t| {
                json!({
                    "machine_id": t.machine_id,
                    "from": t.record.from.name(),
                    "to": t.record.to.name(),
                    "transition": t.record.transition_name,
                    "guard": t.record.guard_description
                })
            })
            .collect();

        ok_json(json!({
            "status": "tick_complete",
            "tick_number": result.tick_number,
            "transitions_fired": transitions.len(),
            "transitions": transitions,
            "no_transition": result.no_transition,
            "terminal": result.terminal,
            "errors": result.errors.iter().map(|e| json!({
                "machine_id": e.machine_id,
                "error": e.error
            })).collect::<Vec<_>>(),
            "timestamp": result.timestamp.to_rfc3339()
        }))
    }
}

/// Query the current state of a machine.
pub fn state(p: AsmStateParams) -> Result<CallToolResult, McpError> {
    let eng = engine().lock();
    let Some(machine) = eng.get(&p.machine_id) else {
        return err_json(&format!("machine '{}' not found", p.machine_id));
    };

    let summary = machine.summary();
    ok_json(json!({
        "machine_id": summary.id,
        "name": summary.name,
        "current_state": summary.current_state,
        "is_terminal": summary.is_terminal,
        "transition_count": summary.transition_count,
        "state_count": summary.state_count,
        "transition_def_count": summary.transition_def_count,
        "last_transition_at": summary.last_transition_at.map(|t| t.to_rfc3339()),
        "created_at": summary.created_at.to_rfc3339()
    }))
}

/// Force a transition to a specific state (bypass guards).
pub fn force_transition(p: AsmTransitionParams) -> Result<CallToolResult, McpError> {
    let mut eng = engine().lock();
    let Some(machine) = eng.get_mut(&p.machine_id) else {
        return err_json(&format!("machine '{}' not found", p.machine_id));
    };

    match machine.force_transition(&p.target_state) {
        Ok(record) => ok_json(json!({
            "status": "forced",
            "machine_id": p.machine_id,
            "from": record.from.name(),
            "to": record.to.name(),
            "transition": record.transition_name,
            "timestamp": record.timestamp.to_rfc3339()
        })),
        Err(e) => err_json(&format!("force transition failed: {e}")),
    }
}

/// Get transition history for a machine.
pub fn history(p: AsmHistoryParams) -> Result<CallToolResult, McpError> {
    let eng = engine().lock();
    let Some(machine) = eng.get(&p.machine_id) else {
        return err_json(&format!("machine '{}' not found", p.machine_id));
    };

    let hist = machine.history();
    let limit = p.limit.min(hist.len());
    let recent: Vec<serde_json::Value> = hist[hist.len().saturating_sub(limit)..]
        .iter()
        .map(|r| {
            json!({
                "timestamp": r.timestamp.to_rfc3339(),
                "transition": r.transition_name,
                "from": r.from.name(),
                "to": r.to.name(),
                "guard": r.guard_description
            })
        })
        .collect();

    ok_json(json!({
        "machine_id": p.machine_id,
        "total_transitions": machine.summary().transition_count,
        "showing": recent.len(),
        "history": recent
    }))
}

/// List all registered machines.
pub fn list(_p: AsmListParams) -> Result<CallToolResult, McpError> {
    let eng = engine().lock();
    let summaries = eng.summaries();

    let machines: Vec<serde_json::Value> = summaries
        .iter()
        .map(|s| {
            json!({
                "id": s.id,
                "name": s.name,
                "current_state": s.current_state,
                "is_terminal": s.is_terminal,
                "transition_count": s.transition_count
            })
        })
        .collect();

    ok_json(json!({
        "machine_count": machines.len(),
        "tick_count": eng.tick_count(),
        "machines": machines
    }))
}
