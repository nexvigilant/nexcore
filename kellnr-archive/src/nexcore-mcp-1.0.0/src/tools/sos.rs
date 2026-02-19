//! State Operating System (SOS) tools
//!
//! Exposes the 15-layer StateKernel via MCP tools for state machine
//! orchestration, transition execution, and inspection.
//!
//! ## Tools
//!
//! - `sos_create`: Create a machine from specification
//! - `sos_transition`: Execute a transition by event name
//! - `sos_state`: Get current state and available transitions
//! - `sos_history`: Get transition history
//! - `sos_validate`: Validate specification without creating
//! - `sos_list`: List active machines
//!
//! ## Tier: T3 (ς + → + μ + Σ + π)
//!
//! Dominant primitive: ς (State)

use crate::params::{
    SosAuditParams, SosCreateParams, SosCyclesParams, SosHistoryParams, SosListParams,
    SosRouteParams, SosScheduleParams, SosStateParams, SosStateSpec, SosTransitionParams,
    SosTransitionSpec, SosValidateParams,
};
use nexcore_state_os::stos::state_registry::StateKind;
use nexcore_state_os::{MachineSpec, StateKernel};
use parking_lot::Mutex;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::HashMap;
use std::sync::OnceLock;

// ============================================================================
// Global Kernel State
// ============================================================================

/// Global SOS kernel instance (lazily initialized).
/// Uses OnceLock<Mutex<T>> pattern from guardian.rs.
static SOS_KERNEL: OnceLock<Mutex<SosKernelState>> = OnceLock::new();

/// Extended kernel state with machine name tracking.
struct SosKernelState {
    kernel: StateKernel,
    /// Machine ID → name mapping for display.
    names: HashMap<u64, String>,
    /// Machine ID → spec mapping for event lookup.
    specs: HashMap<u64, MachineSpec>,
}

impl SosKernelState {
    fn new() -> Self {
        Self {
            kernel: StateKernel::new(),
            names: HashMap::new(),
            specs: HashMap::new(),
        }
    }
}

fn get_kernel() -> &'static Mutex<SosKernelState> {
    SOS_KERNEL.get_or_init(|| Mutex::new(SosKernelState::new()))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse state kind from string.
fn parse_state_kind(s: &str) -> Result<StateKind, McpError> {
    match s.to_lowercase().as_str() {
        "initial" => Ok(StateKind::Initial),
        "normal" => Ok(StateKind::Normal),
        "terminal" => Ok(StateKind::Terminal),
        "error" => Ok(StateKind::Error),
        _ => Err(McpError::invalid_params(
            format!(
                "Invalid state kind: '{}'. Expected: initial, normal, terminal, error",
                s
            ),
            None,
        )),
    }
}

/// Build a MachineSpec from param specs.
fn build_spec(
    name: &str,
    states: &[SosStateSpec],
    transitions: &[SosTransitionSpec],
) -> Result<MachineSpec, McpError> {
    let mut builder = MachineSpec::builder(name);

    for state in states {
        let kind = parse_state_kind(&state.kind)?;
        builder = builder.state(&state.name, kind);
    }

    for trans in transitions {
        builder = builder.transition(&trans.from, &trans.to, &trans.event);
    }

    Ok(builder.build())
}

/// Validate a machine spec for correctness.
fn validate_spec(spec: &MachineSpec) -> Vec<String> {
    let mut errors = Vec::new();

    // Check for initial state
    if spec.initial_state().is_none() {
        errors.push("No initial state defined".to_string());
    }

    // Check for multiple initial states
    let initial_count = spec
        .states
        .iter()
        .filter(|s| s.kind == StateKind::Initial)
        .count();
    if initial_count > 1 {
        errors.push(format!(
            "Multiple initial states defined: {}",
            initial_count
        ));
    }

    // Check transitions reference valid states
    for trans in &spec.transitions {
        if spec.state_id(&trans.from).is_none() {
            errors.push(format!(
                "Transition '{}' references unknown source state '{}'",
                trans.event, trans.from
            ));
        }
        if spec.state_id(&trans.to).is_none() {
            errors.push(format!(
                "Transition '{}' references unknown target state '{}'",
                trans.event, trans.to
            ));
        }
    }

    errors
}

// ============================================================================
// MCP Tool Functions
// ============================================================================

/// Create a new state machine from specification.
///
/// Returns the machine ID on success.
pub fn sos_create(params: SosCreateParams) -> Result<CallToolResult, McpError> {
    let spec = build_spec(&params.name, &params.states, &params.transitions)?;

    // Validate before creating
    let errors = validate_spec(&spec);
    if !errors.is_empty() {
        return Err(McpError::invalid_params(
            format!("Invalid specification: {}", errors.join("; ")),
            None,
        ));
    }

    let mut state = get_kernel().lock();

    let machine_id = state.kernel.load_machine(&spec).map_err(|e| {
        McpError::invalid_params(format!("Failed to create machine: {:?}", e), None)
    })?;

    // Store name and spec for later lookup
    state.names.insert(machine_id, params.name.clone());
    state.specs.insert(machine_id, spec);

    let json = json!({
        "machine_id": machine_id,
        "name": params.name,
        "states": params.states.len(),
        "transitions": params.transitions.len(),
        "status": "created"
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Execute a transition by event name.
pub fn sos_transition(params: SosTransitionParams) -> Result<CallToolResult, McpError> {
    let mut state = get_kernel().lock();

    // Check machine exists
    if !state.specs.contains_key(&params.machine_id) {
        return Err(McpError::invalid_params(
            format!("Machine not found: {}", params.machine_id),
            None,
        ));
    }

    // Get current state name
    let current_state_id = state
        .kernel
        .current_state(params.machine_id)
        .map_err(|e| McpError::invalid_params(format!("Failed to get state: {:?}", e), None))?;

    let spec = state.specs.get(&params.machine_id).cloned();
    let spec = spec.ok_or_else(|| {
        McpError::invalid_params(
            format!("Spec not found for machine: {}", params.machine_id),
            None,
        )
    })?;

    let current_state_name = spec
        .state(current_state_id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| format!("state_{}", current_state_id));

    // Find matching transition
    let transition = spec
        .transitions
        .iter()
        .find(|t| t.from == current_state_name && t.event == params.event);

    let transition = match transition {
        Some(t) => t.clone(),
        None => {
            // List available events
            let available: Vec<_> = spec
                .transitions_from(&current_state_name)
                .iter()
                .map(|t| t.event.as_str())
                .collect();
            return Err(McpError::invalid_params(
                format!(
                    "No transition '{}' from state '{}'. Available events: {:?}",
                    params.event, current_state_name, available
                ),
                None,
            ));
        }
    };

    // Execute the transition via handle_event (Layer 15: μ)
    // We need to register the event mapping first, then handle
    // For now, directly execute via transition ID lookup

    // Find the kernel-registered transition ID
    let from_id = spec.state_id(&transition.from).ok_or_else(|| {
        McpError::invalid_params(format!("State not found: {}", transition.from), None)
    })?;
    let to_id = spec.state_id(&transition.to).ok_or_else(|| {
        McpError::invalid_params(format!("State not found: {}", transition.to), None)
    })?;

    // The transition ID in kernel is the index in registration order
    let transition_id = spec
        .transitions
        .iter()
        .position(|t| t.id == transition.id)
        .map(|i| i as u32)
        .ok_or_else(|| {
            McpError::invalid_params(format!("Transition ID not found: {}", transition.id), None)
        })?;

    let result = state
        .kernel
        .transition(params.machine_id, transition_id)
        .map_err(|e| McpError::invalid_params(format!("Transition failed: {:?}", e), None))?;

    let new_state_id = state
        .kernel
        .current_state(params.machine_id)
        .map_err(|e| McpError::invalid_params(format!("Failed to get new state: {:?}", e), None))?;

    let new_state_name = spec
        .state(new_state_id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| format!("state_{}", new_state_id));

    let is_terminal = state.kernel.is_terminal(params.machine_id).unwrap_or(false);

    let json = json!({
        "machine_id": params.machine_id,
        "event": params.event,
        "from_state": current_state_name,
        "to_state": new_state_name,
        "is_terminal": is_terminal,
        "transition_id": result.transition_id,
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Get current state and available transitions.
pub fn sos_state(params: SosStateParams) -> Result<CallToolResult, McpError> {
    let state = get_kernel().lock();

    // Check machine exists
    let name = state
        .names
        .get(&params.machine_id)
        .cloned()
        .unwrap_or_else(|| format!("machine_{}", params.machine_id));

    let spec = state.specs.get(&params.machine_id).ok_or_else(|| {
        McpError::invalid_params(format!("Machine not found: {}", params.machine_id), None)
    })?;

    let current_state_id = state
        .kernel
        .current_state(params.machine_id)
        .map_err(|e| McpError::invalid_params(format!("Failed to get state: {:?}", e), None))?;

    let current_state = spec.state(current_state_id);
    let current_state_name = current_state
        .map(|s| s.name.clone())
        .unwrap_or_else(|| format!("state_{}", current_state_id));
    let current_state_kind = current_state
        .map(|s| format!("{:?}", s.kind).to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());

    let is_terminal = state.kernel.is_terminal(params.machine_id).unwrap_or(false);

    // Get available transitions from current state
    let available_transitions: Vec<_> = spec
        .transitions_from(&current_state_name)
        .iter()
        .map(|t| {
            json!({
                "event": t.event,
                "to_state": t.to,
            })
        })
        .collect();

    let json = json!({
        "machine_id": params.machine_id,
        "name": name,
        "current_state": current_state_name,
        "state_kind": current_state_kind,
        "is_terminal": is_terminal,
        "available_transitions": available_transitions,
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Get transition history for a machine.
pub fn sos_history(params: SosHistoryParams) -> Result<CallToolResult, McpError> {
    let state = get_kernel().lock();

    let spec = state.specs.get(&params.machine_id).ok_or_else(|| {
        McpError::invalid_params(format!("Machine not found: {}", params.machine_id), None)
    })?;

    // Get boundary crossings as history (Layer 3: ∂)
    let crossings = state
        .kernel
        .boundary_crossings(params.machine_id)
        .map_err(|e| McpError::invalid_params(format!("Failed to get history: {:?}", e), None))?;

    let history: Vec<_> = crossings
        .iter()
        .take(params.limit)
        .map(|c| {
            let state_name = spec
                .state(c.state)
                .map(|s| s.name.clone())
                .unwrap_or_else(|| format!("state_{}", c.state));
            json!({
                "state": state_name,
                "is_entry": c.entering,
                "timestamp": c.timestamp,
            })
        })
        .collect();

    // Get metrics (Layer 5: N)
    let metrics = state.kernel.metrics(params.machine_id).ok();

    let json = json!({
        "machine_id": params.machine_id,
        "history": history,
        "total_crossings": crossings.len(),
        "metrics": metrics.map(|m| json!({
            "state_visits": m.state_visits.values().sum::<u64>(),
            "total_executions": m.executions,
        })),
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Validate a machine specification without creating it.
pub fn sos_validate(params: SosValidateParams) -> Result<CallToolResult, McpError> {
    let spec = build_spec(&params.name, &params.states, &params.transitions)?;
    let errors = validate_spec(&spec);

    let valid = errors.is_empty();

    let json = json!({
        "valid": valid,
        "name": params.name,
        "states": params.states.len(),
        "transitions": params.transitions.len(),
        "initial_state": spec.initial_state().and_then(|id| spec.state(id)).map(|s| &s.name),
        "terminal_states": spec.terminal_states().len(),
        "errors": errors,
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// List all active machines.
pub fn sos_list(params: SosListParams) -> Result<CallToolResult, McpError> {
    let state = get_kernel().lock();

    let filter = params.filter.as_deref().unwrap_or("");

    let machines: Vec<_> = state
        .kernel
        .machine_ids()
        .into_iter()
        .filter_map(|id| {
            let name = state.names.get(&id)?;
            if !filter.is_empty() && !name.contains(filter) {
                return None;
            }

            let current = state.kernel.current_state(id).ok()?;
            let spec = state.specs.get(&id)?;
            let state_name = spec
                .state(current)
                .map(|s| s.name.clone())
                .unwrap_or_else(|| format!("state_{}", current));
            let is_terminal = state.kernel.is_terminal(id).unwrap_or(false);

            Some(json!({
                "machine_id": id,
                "name": name,
                "current_state": state_name,
                "is_terminal": is_terminal,
            }))
        })
        .collect();

    let aggregate = state.kernel.aggregate_stats();

    let json = json!({
        "total": machines.len(),
        "machines": machines,
        "aggregate": {
            "total_machines": aggregate.total_machines,
            "active_machines": aggregate.active_count,
            "terminated_machines": aggregate.terminated_count,
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Detect cycles in machine transition graph (Layer 7: ρ Recursion).
pub fn sos_cycles(params: SosCyclesParams) -> Result<CallToolResult, McpError> {
    let mut state = get_kernel().lock();

    let spec = state
        .specs
        .get(&params.machine_id)
        .cloned()
        .ok_or_else(|| {
            McpError::invalid_params(format!("Machine not found: {}", params.machine_id), None)
        })?;

    let cycles = state
        .kernel
        .detect_cycles(params.machine_id)
        .map_err(|e| McpError::invalid_params(format!("Failed to detect cycles: {:?}", e), None))?;

    // Filter self-loops if requested
    let cycles: Vec<_> = if params.include_self_loops {
        cycles
    } else {
        cycles.into_iter().filter(|c| c.states.len() > 1).collect()
    };

    // Map state IDs to names with cycle metadata
    let cycle_details: Vec<_> = cycles
        .iter()
        .map(|cycle| {
            let state_names: Vec<String> = cycle
                .states
                .iter()
                .map(|&sid| {
                    spec.state(sid)
                        .map(|s| s.name.clone())
                        .unwrap_or_else(|| format!("state_{}", sid))
                })
                .collect();
            json!({
                "states": state_names,
                "intentional": cycle.intentional,
                "detected_at": cycle.detected_at,
            })
        })
        .collect();

    let json = json!({
        "machine_id": params.machine_id,
        "cycle_count": cycle_details.len(),
        "cycles": cycle_details,
        "has_cycles": !cycle_details.is_empty(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Get irreversibility audit trail (Layer 14: ∝ Irreversibility).
pub fn sos_audit(params: SosAuditParams) -> Result<CallToolResult, McpError> {
    let state = get_kernel().lock();

    let spec = state.specs.get(&params.machine_id).ok_or_else(|| {
        McpError::invalid_params(format!("Machine not found: {}", params.machine_id), None)
    })?;

    let trail_valid = state
        .kernel
        .verify_audit_trail(params.machine_id)
        .map_err(|e| McpError::invalid_params(format!("Failed to verify trail: {:?}", e), None))?;

    let trail_len = state
        .kernel
        .audit_trail_len(params.machine_id)
        .map_err(|e| {
            McpError::invalid_params(format!("Failed to get trail length: {:?}", e), None)
        })?;

    // Get boundary crossings as audit events
    let crossings = state
        .kernel
        .boundary_crossings(params.machine_id)
        .map_err(|e| McpError::invalid_params(format!("Failed to get crossings: {:?}", e), None))?;

    let audit_entries: Vec<_> = crossings
        .iter()
        .take(params.limit)
        .map(|c| {
            let state_name = spec
                .state(c.state)
                .map(|s| s.name.clone())
                .unwrap_or_else(|| format!("state_{}", c.state));
            json!({
                "state": state_name,
                "action": if c.entering { "enter" } else { "exit" },
                "timestamp": c.timestamp,
                "boundary_kind": format!("{:?}", c.kind),
            })
        })
        .collect();

    let json = json!({
        "machine_id": params.machine_id,
        "trail_valid": trail_valid,
        "trail_length": trail_len,
        "audit_entries": audit_entries,
        "total_entries": crossings.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Schedule a delayed transition (Layer 12: ν Frequency).
pub fn sos_schedule(params: SosScheduleParams) -> Result<CallToolResult, McpError> {
    let mut state = get_kernel().lock();

    let spec = state
        .specs
        .get(&params.machine_id)
        .cloned()
        .ok_or_else(|| {
            McpError::invalid_params(format!("Machine not found: {}", params.machine_id), None)
        })?;

    // Get current state to find valid transition
    let current_state_id = state
        .kernel
        .current_state(params.machine_id)
        .map_err(|e| McpError::invalid_params(format!("Failed to get state: {:?}", e), None))?;

    let current_state_name = spec
        .state(current_state_id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| format!("state_{}", current_state_id));

    // Find matching transition
    let transition = spec
        .transitions
        .iter()
        .find(|t| t.from == current_state_name && t.event == params.event);

    let transition = match transition {
        Some(t) => t.clone(),
        None => {
            return Err(McpError::invalid_params(
                format!(
                    "No transition '{}' from state '{}'",
                    params.event, current_state_name
                ),
                None,
            ));
        }
    };

    // Find the kernel-registered transition ID
    let transition_id = spec
        .transitions
        .iter()
        .position(|t| t.id == transition.id)
        .map(|i| i as u32)
        .ok_or_else(|| {
            McpError::invalid_params(format!("Transition ID not found: {}", transition.id), None)
        })?;

    // Enqueue the transition with delay
    state
        .kernel
        .enqueue_transition(params.machine_id, transition_id)
        .map_err(|e| McpError::invalid_params(format!("Failed to enqueue: {:?}", e), None))?;

    let json = json!({
        "machine_id": params.machine_id,
        "event": params.event,
        "transition_id": transition_id,
        "delay_ticks": params.delay_ticks,
        "from_state": current_state_name,
        "to_state": transition.to,
        "status": "scheduled",
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Route machine to location (Layer 13: λ Location).
pub fn sos_route(params: SosRouteParams) -> Result<CallToolResult, McpError> {
    let mut state = get_kernel().lock();

    // Check machine exists
    if !state.specs.contains_key(&params.machine_id) {
        return Err(McpError::invalid_params(
            format!("Machine not found: {}", params.machine_id),
            None,
        ));
    }

    let name = state
        .names
        .get(&params.machine_id)
        .cloned()
        .unwrap_or_else(|| format!("machine_{}", params.machine_id));

    // Either use specified location or auto-route
    let result = match params.location_id {
        Some(loc_id) => {
            // Assign to specific location
            state
                .kernel
                .assign_location(params.machine_id, loc_id)
                .map_err(|e| {
                    McpError::invalid_params(format!("Failed to assign location: {:?}", e), None)
                })?;
            json!({
                "machine_id": params.machine_id,
                "name": name,
                "location_id": loc_id,
                "routing": "manual",
            })
        }
        None => {
            // Auto-route using least-loaded strategy
            let loc_id = state.kernel.route_machine(params.machine_id).map_err(|e| {
                McpError::invalid_params(format!("Failed to route machine: {:?}", e), None)
            })?;
            json!({
                "machine_id": params.machine_id,
                "name": name,
                "location_id": loc_id,
                "routing": "auto",
            })
        }
    };

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_state_kind() {
        assert!(matches!(
            parse_state_kind("initial"),
            Ok(StateKind::Initial)
        ));
        assert!(matches!(
            parse_state_kind("TERMINAL"),
            Ok(StateKind::Terminal)
        ));
        assert!(parse_state_kind("invalid").is_err());
    }

    #[test]
    fn test_build_spec() {
        let states = vec![
            SosStateSpec {
                name: "pending".to_string(),
                kind: "initial".to_string(),
            },
            SosStateSpec {
                name: "done".to_string(),
                kind: "terminal".to_string(),
            },
        ];
        let transitions = vec![SosTransitionSpec {
            from: "pending".to_string(),
            to: "done".to_string(),
            event: "complete".to_string(),
        }];

        let spec = build_spec("test", &states, &transitions);
        assert!(spec.is_ok());

        let spec = spec.ok();
        assert!(spec.is_some());

        let spec = spec.as_ref();
        assert!(spec.is_some());

        let s = spec.unwrap();
        assert_eq!(s.states.len(), 2);
        assert_eq!(s.transitions.len(), 1);
    }

    #[test]
    fn test_validate_spec() {
        // Missing initial state
        let spec = MachineSpec::builder("test")
            .state("a", StateKind::Normal)
            .build();
        let errors = validate_spec(&spec);
        assert!(!errors.is_empty());

        // Valid spec
        let spec = MachineSpec::builder("test")
            .state("start", StateKind::Initial)
            .state("end", StateKind::Terminal)
            .transition("start", "end", "go")
            .build();
        let errors = validate_spec(&spec);
        assert!(errors.is_empty());
    }
}
