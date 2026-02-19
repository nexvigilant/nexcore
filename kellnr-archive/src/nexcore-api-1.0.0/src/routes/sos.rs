//! State Operating System (SOS) REST API endpoints
//!
//! Exposes the 15-layer state machine runtime via HTTP.
//! Mirrors MCP tools: sos_create, sos_transition, sos_state, etc.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use nexcore_state_os::stos::state_registry::StateKind;
use nexcore_state_os::{MachineSpec, StateKernel};

use super::common::{ApiError, ApiResult};

// ============================================================================
// Global Kernel State
// ============================================================================

struct SosKernelState {
    kernel: StateKernel,
    names: HashMap<u64, String>,
    specs: HashMap<u64, MachineSpec>,
}

static SOS_KERNEL: OnceLock<Mutex<SosKernelState>> = OnceLock::new();

fn get_kernel() -> &'static Mutex<SosKernelState> {
    SOS_KERNEL.get_or_init(|| {
        Mutex::new(SosKernelState {
            kernel: StateKernel::new(),
            names: HashMap::new(),
            specs: HashMap::new(),
        })
    })
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// State specification for creating a machine
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct StateSpecRequest {
    /// State name
    pub name: String,
    /// State kind: "initial", "normal", "terminal", or "error"
    pub kind: String,
}

/// Transition specification for creating a machine
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct TransitionSpecRequest {
    /// Source state name
    pub from: String,
    /// Target state name
    pub to: String,
    /// Event name that triggers this transition
    pub event: String,
}

/// Request to create a new state machine
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMachineRequest {
    /// Machine name
    pub name: String,
    /// List of states
    pub states: Vec<StateSpecRequest>,
    /// List of transitions
    pub transitions: Vec<TransitionSpecRequest>,
}

/// Response after creating a machine
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateMachineResponse {
    /// Unique machine ID
    pub machine_id: u64,
    /// Machine name
    pub name: String,
    /// Number of states
    pub states: usize,
    /// Number of transitions
    pub transitions: usize,
    /// Status
    pub status: String,
}

/// Request to execute a transition
#[derive(Debug, Deserialize, ToSchema)]
pub struct TransitionRequest {
    /// Machine ID
    pub machine_id: u64,
    /// Event name to trigger
    pub event: String,
}

/// Response after executing a transition
#[derive(Debug, Serialize, ToSchema)]
pub struct TransitionResponse {
    /// Machine ID
    pub machine_id: u64,
    /// Event that was fired
    pub event: String,
    /// Previous state
    pub from_state: String,
    /// New state
    pub to_state: String,
    /// Whether machine is now in terminal state
    pub is_terminal: bool,
    /// Transition ID
    pub transition_id: u32,
}

/// Request to query machine state
#[derive(Debug, Deserialize, ToSchema)]
pub struct StateQueryRequest {
    /// Machine ID
    pub machine_id: u64,
}

/// Available transition info
#[derive(Debug, Serialize, ToSchema)]
pub struct AvailableTransition {
    /// Event name
    pub event: String,
    /// Target state
    pub to_state: String,
}

/// Response with current machine state
#[derive(Debug, Serialize, ToSchema)]
pub struct StateResponse {
    /// Machine ID
    pub machine_id: u64,
    /// Machine name
    pub name: String,
    /// Current state name
    pub current_state: String,
    /// State kind
    pub state_kind: String,
    /// Whether in terminal state
    pub is_terminal: bool,
    /// Available transitions from current state
    pub available_transitions: Vec<AvailableTransition>,
}

/// Request for transition history
#[derive(Debug, Deserialize, ToSchema)]
pub struct HistoryRequest {
    /// Machine ID
    pub machine_id: u64,
    /// Maximum entries to return (default: 50)
    #[serde(default = "default_history_limit")]
    pub limit: usize,
}

fn default_history_limit() -> usize {
    50
}

/// History entry
#[derive(Debug, Serialize, ToSchema)]
pub struct HistoryEntry {
    /// State name
    pub state: String,
    /// Whether this was an entry (true) or exit (false)
    pub is_entry: bool,
    /// Logical timestamp
    pub timestamp: u64,
}

/// History response
#[derive(Debug, Serialize, ToSchema)]
pub struct HistoryResponse {
    /// Machine ID
    pub machine_id: u64,
    /// History entries
    pub history: Vec<HistoryEntry>,
    /// Total boundary crossings
    pub total_crossings: usize,
    /// Metrics
    pub metrics: Option<HistoryMetrics>,
}

/// History metrics
#[derive(Debug, Serialize, ToSchema)]
pub struct HistoryMetrics {
    /// Number of state visits
    pub state_visits: u64,
    /// Total transitions executed
    pub total_executions: u64,
}

/// Machine summary for list endpoint
#[derive(Debug, Serialize, ToSchema)]
pub struct MachineSummary {
    /// Machine ID
    pub machine_id: u64,
    /// Machine name
    pub name: String,
    /// Current state
    pub current_state: String,
    /// Whether in terminal state
    pub is_terminal: bool,
}

/// List machines response
#[derive(Debug, Serialize, ToSchema)]
pub struct ListMachinesResponse {
    /// Total machines
    pub total: usize,
    /// Machine summaries
    pub machines: Vec<MachineSummary>,
    /// Aggregate stats
    pub aggregate: AggregateStats,
}

/// Aggregate statistics
#[derive(Debug, Serialize, ToSchema)]
pub struct AggregateStats {
    /// Total machines
    pub total_machines: usize,
    /// Active (non-terminal) machines
    pub active_machines: usize,
    /// Terminated machines
    pub terminated_machines: usize,
}

/// Validation response
#[derive(Debug, Serialize, ToSchema)]
pub struct ValidateResponse {
    /// Whether specification is valid
    pub valid: bool,
    /// Machine name
    pub name: String,
    /// Number of states
    pub states: usize,
    /// Number of transitions
    pub transitions: usize,
    /// Initial state name
    pub initial_state: Option<String>,
    /// Number of terminal states
    pub terminal_states: usize,
    /// Validation errors
    pub errors: Vec<String>,
}

// ============================================================================
// Helpers
// ============================================================================

fn parse_state_kind(s: &str) -> Result<StateKind, ApiError> {
    match s.to_lowercase().as_str() {
        "initial" => Ok(StateKind::Initial),
        "normal" => Ok(StateKind::Normal),
        "terminal" => Ok(StateKind::Terminal),
        "error" => Ok(StateKind::Error),
        _ => Err(ApiError::new(
            "VALIDATION_ERROR",
            format!(
                "Invalid state kind: {}. Must be initial, normal, terminal, or error",
                s
            ),
        )),
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/v1/sos/create - Create a new state machine
#[utoipa::path(
    post,
    path = "/api/v1/sos/create",
    request_body = CreateMachineRequest,
    responses(
        (status = 200, description = "Machine created", body = CreateMachineResponse),
        (status = 400, description = "Validation error", body = ApiError),
    ),
    tag = "SOS"
)]
pub async fn create_machine(
    Json(req): Json<CreateMachineRequest>,
) -> ApiResult<CreateMachineResponse> {
    let mut kernel_state = get_kernel()
        .lock()
        .map_err(|e| ApiError::new("INTERNAL_ERROR", format!("Failed to lock kernel: {}", e)))?;

    // Build spec
    let mut builder = MachineSpec::builder(&req.name);

    for s in &req.states {
        let kind = parse_state_kind(&s.kind)?;
        builder = builder.state(&s.name, kind);
    }

    for t in &req.transitions {
        builder = builder.transition(&t.from, &t.to, &t.event);
    }

    let spec = builder.build();

    // Create machine
    let machine_id = kernel_state
        .kernel
        .load_machine(&spec)
        .map_err(|e| ApiError::new("VALIDATION_ERROR", format!("{:?}", e)))?;

    // Store metadata
    kernel_state.names.insert(machine_id, req.name.clone());
    kernel_state.specs.insert(machine_id, spec);

    Ok(Json(CreateMachineResponse {
        machine_id,
        name: req.name,
        states: req.states.len(),
        transitions: req.transitions.len(),
        status: "created".to_string(),
    }))
}

/// POST /api/v1/sos/transition - Execute a transition
#[utoipa::path(
    post,
    path = "/api/v1/sos/transition",
    request_body = TransitionRequest,
    responses(
        (status = 200, description = "Transition executed", body = TransitionResponse),
        (status = 400, description = "Invalid transition", body = ApiError),
        (status = 404, description = "Machine not found", body = ApiError),
    ),
    tag = "SOS"
)]
pub async fn execute_transition(
    Json(req): Json<TransitionRequest>,
) -> ApiResult<TransitionResponse> {
    let mut kernel_state = get_kernel()
        .lock()
        .map_err(|e| ApiError::new("INTERNAL_ERROR", format!("Failed to lock kernel: {}", e)))?;

    // Get spec (also validates machine exists)
    let spec = kernel_state
        .specs
        .get(&req.machine_id)
        .cloned()
        .ok_or_else(|| {
            ApiError::new(
                "NOT_FOUND",
                format!("Machine not found: {}", req.machine_id),
            )
        })?;

    // Get current state
    let current_state_id = kernel_state
        .kernel
        .current_state(req.machine_id)
        .map_err(|e| ApiError::new("NOT_FOUND", format!("Machine error: {:?}", e)))?;

    let from_state = spec
        .state(current_state_id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| format!("state_{}", current_state_id));

    // Find transition by event name
    let transition = spec
        .transitions
        .iter()
        .find(|t| t.from == from_state && t.event == req.event)
        .ok_or_else(|| {
            let available: Vec<_> = spec
                .transitions_from(&from_state)
                .iter()
                .map(|t| t.event.as_str())
                .collect();
            ApiError::new(
                "VALIDATION_ERROR",
                format!(
                    "No transition for event '{}' from state '{}'. Available: {:?}",
                    req.event, from_state, available
                ),
            )
        })?;

    // Find transition ID
    let transition_id = spec
        .transitions
        .iter()
        .position(|t| t.id == transition.id)
        .map(|i| i as u32)
        .ok_or_else(|| ApiError::new("INTERNAL_ERROR", "Transition ID not found"))?;

    // Execute transition
    let result = kernel_state
        .kernel
        .transition(req.machine_id, transition_id)
        .map_err(|e| ApiError::new("VALIDATION_ERROR", format!("{:?}", e)))?;

    // Get new state
    let new_state_id = kernel_state
        .kernel
        .current_state(req.machine_id)
        .map_err(|e| ApiError::new("INTERNAL_ERROR", format!("{:?}", e)))?;

    let to_state = spec
        .state(new_state_id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| format!("state_{}", new_state_id));

    let is_terminal = kernel_state
        .kernel
        .is_terminal(req.machine_id)
        .unwrap_or(false);

    Ok(Json(TransitionResponse {
        machine_id: req.machine_id,
        event: req.event,
        from_state,
        to_state,
        is_terminal,
        transition_id: result.transition_id,
    }))
}

/// POST /api/v1/sos/state - Get current machine state
#[utoipa::path(
    post,
    path = "/api/v1/sos/state",
    request_body = StateQueryRequest,
    responses(
        (status = 200, description = "Current state", body = StateResponse),
        (status = 404, description = "Machine not found", body = ApiError),
    ),
    tag = "SOS"
)]
pub async fn get_state(Json(req): Json<StateQueryRequest>) -> ApiResult<StateResponse> {
    let kernel_state = get_kernel()
        .lock()
        .map_err(|e| ApiError::new("INTERNAL_ERROR", format!("Failed to lock kernel: {}", e)))?;

    let spec = kernel_state.specs.get(&req.machine_id).ok_or_else(|| {
        ApiError::new(
            "NOT_FOUND",
            format!("Machine not found: {}", req.machine_id),
        )
    })?;

    let name = kernel_state
        .names
        .get(&req.machine_id)
        .cloned()
        .unwrap_or_default();

    let current_state_id = kernel_state
        .kernel
        .current_state(req.machine_id)
        .map_err(|e| ApiError::new("NOT_FOUND", format!("Machine error: {:?}", e)))?;

    let current_state_info = spec.state(current_state_id);
    let current_state = current_state_info
        .map(|s| s.name.clone())
        .unwrap_or_else(|| format!("state_{}", current_state_id));
    let state_kind = current_state_info
        .map(|s| format!("{:?}", s.kind).to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());

    let is_terminal = kernel_state
        .kernel
        .is_terminal(req.machine_id)
        .unwrap_or(false);

    // Get available transitions
    let available_transitions = spec
        .transitions_from(&current_state)
        .iter()
        .map(|t| AvailableTransition {
            event: t.event.clone(),
            to_state: t.to.clone(),
        })
        .collect();

    Ok(Json(StateResponse {
        machine_id: req.machine_id,
        name,
        current_state,
        state_kind,
        is_terminal,
        available_transitions,
    }))
}

/// POST /api/v1/sos/history - Get transition history
#[utoipa::path(
    post,
    path = "/api/v1/sos/history",
    request_body = HistoryRequest,
    responses(
        (status = 200, description = "Transition history", body = HistoryResponse),
        (status = 404, description = "Machine not found", body = ApiError),
    ),
    tag = "SOS"
)]
pub async fn get_history(Json(req): Json<HistoryRequest>) -> ApiResult<HistoryResponse> {
    let kernel_state = get_kernel()
        .lock()
        .map_err(|e| ApiError::new("INTERNAL_ERROR", format!("Failed to lock kernel: {}", e)))?;

    let spec = kernel_state.specs.get(&req.machine_id).ok_or_else(|| {
        ApiError::new(
            "NOT_FOUND",
            format!("Machine not found: {}", req.machine_id),
        )
    })?;

    // Get boundary crossings
    let crossings = kernel_state
        .kernel
        .boundary_crossings(req.machine_id)
        .map_err(|e| ApiError::new("NOT_FOUND", format!("Machine error: {:?}", e)))?;

    let history: Vec<HistoryEntry> = crossings
        .iter()
        .take(req.limit)
        .map(|c| {
            let state_name = spec
                .state(c.state)
                .map(|s| s.name.clone())
                .unwrap_or_else(|| format!("state_{}", c.state));
            HistoryEntry {
                state: state_name,
                is_entry: c.entering,
                timestamp: c.timestamp,
            }
        })
        .collect();

    let total_crossings = crossings.len();

    // Get metrics
    let metrics = kernel_state
        .kernel
        .metrics(req.machine_id)
        .ok()
        .map(|m| HistoryMetrics {
            state_visits: m.state_visits.values().sum(),
            total_executions: m.executions,
        });

    Ok(Json(HistoryResponse {
        machine_id: req.machine_id,
        history,
        total_crossings,
        metrics,
    }))
}

/// POST /api/v1/sos/validate - Validate specification without creating
#[utoipa::path(
    post,
    path = "/api/v1/sos/validate",
    request_body = CreateMachineRequest,
    responses(
        (status = 200, description = "Validation result", body = ValidateResponse),
    ),
    tag = "SOS"
)]
pub async fn validate_spec(Json(req): Json<CreateMachineRequest>) -> ApiResult<ValidateResponse> {
    let mut errors = Vec::new();
    let mut initial_state = None;
    let mut terminal_count = 0;
    let mut initial_count = 0;

    // Validate states
    for s in &req.states {
        match parse_state_kind(&s.kind) {
            Ok(kind) => {
                if kind == StateKind::Initial {
                    initial_count += 1;
                    initial_state = Some(s.name.clone());
                }
                if kind == StateKind::Terminal || kind == StateKind::Error {
                    terminal_count += 1;
                }
            }
            Err(e) => errors.push(e.message),
        }
    }

    if initial_count == 0 {
        errors.push("No initial state defined".to_string());
    } else if initial_count > 1 {
        errors.push(format!(
            "Multiple initial states defined: {}",
            initial_count
        ));
    }

    // Validate transitions
    let state_names: Vec<_> = req.states.iter().map(|s| s.name.clone()).collect();
    for t in &req.transitions {
        if !state_names.contains(&t.from) {
            errors.push(format!(
                "Transition references unknown source state: {}",
                t.from
            ));
        }
        if !state_names.contains(&t.to) {
            errors.push(format!(
                "Transition references unknown target state: {}",
                t.to
            ));
        }
    }

    Ok(Json(ValidateResponse {
        valid: errors.is_empty(),
        name: req.name,
        states: req.states.len(),
        transitions: req.transitions.len(),
        initial_state,
        terminal_states: terminal_count,
        errors,
    }))
}

/// GET /api/v1/sos/list - List all active machines
#[utoipa::path(
    get,
    path = "/api/v1/sos/list",
    responses(
        (status = 200, description = "Machine list", body = ListMachinesResponse),
    ),
    tag = "SOS"
)]
pub async fn list_machines() -> ApiResult<ListMachinesResponse> {
    let kernel_state = get_kernel()
        .lock()
        .map_err(|e| ApiError::new("INTERNAL_ERROR", format!("Failed to lock kernel: {}", e)))?;

    let aggregate = kernel_state.kernel.aggregate_stats();

    let machines: Vec<MachineSummary> = kernel_state
        .kernel
        .machine_ids()
        .into_iter()
        .filter_map(|id| {
            let name = kernel_state.names.get(&id)?.clone();
            let current_id = kernel_state.kernel.current_state(id).ok()?;
            let spec = kernel_state.specs.get(&id)?;
            let current_state = spec
                .state(current_id)
                .map(|s| s.name.clone())
                .unwrap_or_else(|| format!("state_{}", current_id));
            let is_terminal = kernel_state.kernel.is_terminal(id).unwrap_or(false);

            Some(MachineSummary {
                machine_id: id,
                name,
                current_state,
                is_terminal,
            })
        })
        .collect();

    Ok(Json(ListMachinesResponse {
        total: machines.len(),
        machines,
        aggregate: AggregateStats {
            total_machines: aggregate.total_machines,
            active_machines: aggregate.active_count,
            terminated_machines: aggregate.terminated_count,
        },
    }))
}

// ============================================================================
// Router
// ============================================================================

/// Create SOS router with all endpoints
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/create", post(create_machine))
        .route("/transition", post(execute_transition))
        .route("/state", post(get_state))
        .route("/history", post(get_history))
        .route("/validate", post(validate_spec))
        .route("/list", get(list_machines))
}
