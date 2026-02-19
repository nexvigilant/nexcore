# State Operating System (SOS) Manual

## Overview

The State Operating System (SOS) is a 15-layer state machine runtime built on the Lex Primitiva foundation. It provides MCP tools and CLI commands for creating, executing, and inspecting finite state machines.

**Tier**: T3 (ς + → + μ + ∂ + N + σ + ρ + π + ∃ + Σ + ν + λ + ∝ + κ)
**Dominant Primitive**: ς (State)

---

## Quick Start

### MCP Tools

```bash
# Create a machine
mcp__nexcore__sos_create(
  name="order",
  states=[
    {name: "pending", kind: "initial"},
    {name: "processing", kind: "normal"},
    {name: "shipped", kind: "terminal"}
  ],
  transitions=[
    {from: "pending", to: "processing", event: "confirm"},
    {from: "processing", to: "shipped", event: "ship"}
  ]
)
# Returns: {"machine_id": 1, "name": "order", ...}

# Execute transition
mcp__nexcore__sos_transition(machine_id=1, event="confirm")
# Returns: {"from_state": "pending", "to_state": "processing", ...}

# Query state
mcp__nexcore__sos_state(machine_id=1)
# Returns: {"current_state": "processing", "available_transitions": [...]}
```

### CLI Commands

```bash
# Generate template
nexcore sos new my-machine -o spec.json

# Validate
nexcore sos validate spec.json

# Interactive REPL
nexcore sos run spec.json
```

---

## MCP Tools Reference

### sos_create

Create a new state machine from specification.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | Yes | Machine name |
| `states` | array | Yes | List of state specs |
| `transitions` | array | Yes | List of transition specs |

**State Spec:**
```json
{
  "name": "state_name",
  "kind": "initial|normal|terminal|error"
}
```

**Transition Spec:**
```json
{
  "from": "source_state",
  "to": "target_state",
  "event": "event_name"
}
```

**Returns:**
```json
{
  "machine_id": 1,
  "name": "order",
  "states": 3,
  "transitions": 2,
  "status": "created"
}
```

**Errors:**
- Invalid state kind (not initial/normal/terminal/error)
- No initial state defined
- Multiple initial states
- Transition references unknown state

---

### sos_transition

Execute a transition by event name.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `machine_id` | u64 | Yes | Machine ID |
| `event` | string | Yes | Event name to trigger |

**Returns:**
```json
{
  "machine_id": 1,
  "event": "confirm",
  "from_state": "pending",
  "to_state": "processing",
  "is_terminal": false,
  "transition_id": 0
}
```

**Errors:**
- Machine not found
- No transition for event from current state
- Machine in terminal state

---

### sos_state

Get current state and available transitions.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `machine_id` | u64 | Yes | Machine ID |

**Returns:**
```json
{
  "machine_id": 1,
  "name": "order",
  "current_state": "processing",
  "state_kind": "normal",
  "is_terminal": false,
  "available_transitions": [
    {"event": "ship", "to_state": "shipped"},
    {"event": "cancel", "to_state": "cancelled"}
  ]
}
```

---

### sos_history

Get transition history (boundary crossings).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `machine_id` | u64 | Yes | - | Machine ID |
| `limit` | usize | No | 50 | Max entries |

**Returns:**
```json
{
  "machine_id": 1,
  "history": [
    {"state": "pending", "is_entry": true, "timestamp": 0},
    {"state": "pending", "is_entry": false, "timestamp": 1},
    {"state": "processing", "is_entry": true, "timestamp": 1}
  ],
  "total_crossings": 3,
  "metrics": {
    "state_visits": 2,
    "total_executions": 1
  }
}
```

---

### sos_validate

Validate specification without creating machine.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | Yes | Machine name |
| `states` | array | Yes | State specs |
| `transitions` | array | Yes | Transition specs |

**Returns:**
```json
{
  "valid": true,
  "name": "order",
  "states": 3,
  "transitions": 2,
  "initial_state": "pending",
  "terminal_states": 1,
  "errors": []
}
```

---

### sos_list

List all active machines.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `filter` | string | No | null | Name filter pattern |

**Returns:**
```json
{
  "total": 2,
  "machines": [
    {"machine_id": 1, "name": "order-1", "current_state": "processing", "is_terminal": false},
    {"machine_id": 2, "name": "order-2", "current_state": "shipped", "is_terminal": true}
  ],
  "aggregate": {
    "total_machines": 2,
    "active_machines": 1,
    "terminated_machines": 1
  }
}
```

---

### sos_cycles

Detect cycles in transition graph (Layer 7: ρ Recursion).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `machine_id` | u64 | Yes | - | Machine ID |
| `include_self_loops` | bool | No | true | Include self-loops |

**Returns:**
```json
{
  "machine_id": 1,
  "cycle_count": 1,
  "cycles": [
    {
      "states": ["failed", "pending"],
      "intentional": false,
      "detected_at": 5
    }
  ],
  "has_cycles": true
}
```

---

### sos_audit

Get irreversibility audit trail (Layer 14: ∝ Irreversibility).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `machine_id` | u64 | Yes | - | Machine ID |
| `limit` | usize | No | 100 | Max entries |

**Returns:**
```json
{
  "machine_id": 1,
  "trail_valid": true,
  "trail_length": 5,
  "audit_entries": [
    {"state": "pending", "action": "enter", "timestamp": 0, "boundary_kind": "Initial"},
    {"state": "pending", "action": "exit", "timestamp": 1, "boundary_kind": "Initial"},
    {"state": "processing", "action": "enter", "timestamp": 1, "boundary_kind": "Normal"}
  ],
  "total_entries": 5
}
```

---

### sos_schedule

Schedule a delayed transition (Layer 12: ν Frequency).

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `machine_id` | u64 | Yes | Machine ID |
| `event` | string | Yes | Event to schedule |
| `delay_ticks` | u64 | Yes | Delay in ticks |

**Returns:**
```json
{
  "machine_id": 1,
  "event": "timeout",
  "transition_id": 2,
  "delay_ticks": 100,
  "from_state": "processing",
  "to_state": "failed",
  "status": "scheduled"
}
```

---

### sos_route

Route machine to location (Layer 13: λ Location).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `machine_id` | u64 | Yes | - | Machine ID |
| `location_id` | u64 | No | auto | Target location |

**Returns:**
```json
{
  "machine_id": 1,
  "name": "order-1",
  "location_id": 3,
  "routing": "auto"
}
```

---

## CLI Commands Reference

### nexcore sos new

Generate a template specification file.

```bash
nexcore sos new <name> [-o <output>]
```

**Arguments:**
- `name` - Machine name for template
- `-o, --output` - Output file path (default: stdout)

**Example:**
```bash
nexcore sos new order-workflow -o order.json
```

**Output Template:**
```json
{
  "name": "order-workflow",
  "states": [
    {"name": "pending", "kind": "initial"},
    {"name": "processing", "kind": "normal"},
    {"name": "completed", "kind": "terminal"},
    {"name": "failed", "kind": "error"}
  ],
  "transitions": [
    {"from": "pending", "to": "processing", "event": "start"},
    {"from": "processing", "to": "completed", "event": "complete"},
    {"from": "processing", "to": "failed", "event": "fail"},
    {"from": "failed", "to": "pending", "event": "retry"}
  ]
}
```

---

### nexcore sos validate

Validate a specification file.

```bash
nexcore sos validate <spec>
```

**Arguments:**
- `spec` - Path to JSON spec file

**Example:**
```bash
nexcore sos validate order.json
```

---

### nexcore sos run

Run machine interactively (REPL mode).

```bash
nexcore sos run <spec>
```

**REPL Commands:**
| Command | Description |
|---------|-------------|
| `<event>` | Fire transition by event name |
| `state` | Show current state details |
| `history` | Show transition history |
| `help` | Show commands |
| `quit` | Exit REPL |

**Example Session:**
```
$ nexcore sos run order.json
Loaded machine 'order' (ID: 1)
Type 'help' for commands, 'quit' to exit.

[pending] Available: start -> processing
> start
Transitioned: pending --[start]--> processing

[processing] Available: complete -> completed, fail -> failed
> complete
Transitioned: processing --[complete]--> completed

Machine is in terminal state 'completed'.
No further transitions possible.
```

---

### nexcore sos status

Show machine status (future IPC).

```bash
nexcore sos status <machine_id>
```

**Note:** Currently returns placeholder. Full implementation requires IPC for cross-process queries.

---

## State Kinds

| Kind | Symbol | Description |
|------|--------|-------------|
| `initial` | ∂→ | Entry point (exactly one required) |
| `normal` | ς | Standard operational state |
| `terminal` | ∂← | Final state (no outgoing transitions) |
| `error` | ∂! | Error/failure state |

---

## 15-Layer Architecture

The SOS kernel implements the complete Lex Primitiva quindecet:

| Layer | Module | Primitive | MCP Exposure |
|-------|--------|-----------|--------------|
| 1 | state_registry | ς State | `sos_state` |
| 2 | transition_engine | → Causality | `sos_transition` |
| 3 | boundary_manager | ∂ Boundary | `sos_state`, `sos_history` |
| 4 | guard_evaluator | κ Comparison | `sos_transition` |
| 5 | count_metrics | N Quantity | `sos_history` |
| 6 | sequence_controller | σ Sequence | `sos_history` |
| 7 | recursion_detector | ρ Recursion | `sos_cycles` |
| 8 | void_cleaner | ∅ Void | (internal) |
| 9 | persist_store | π Persistence | `sos_create` |
| 10 | existence_validator | ∃ Existence | `sos_validate` |
| 11 | aggregate_coordinator | Σ Sum | `sos_list` |
| 12 | temporal_scheduler | ν Frequency | `sos_schedule` |
| 13 | location_router | λ Location | `sos_route` |
| 14 | irreversibility_auditor | ∝ Irreversibility | `sos_audit` |
| 15 | mapping_transformer | μ Mapping | all tools |

---

## Examples

### Order Processing Workflow

```json
{
  "name": "order-processing",
  "states": [
    {"name": "cart", "kind": "initial"},
    {"name": "checkout", "kind": "normal"},
    {"name": "payment_pending", "kind": "normal"},
    {"name": "paid", "kind": "normal"},
    {"name": "fulfillment", "kind": "normal"},
    {"name": "shipped", "kind": "terminal"},
    {"name": "cancelled", "kind": "terminal"},
    {"name": "payment_failed", "kind": "error"}
  ],
  "transitions": [
    {"from": "cart", "to": "checkout", "event": "proceed_to_checkout"},
    {"from": "checkout", "to": "payment_pending", "event": "submit_order"},
    {"from": "checkout", "to": "cart", "event": "back_to_cart"},
    {"from": "payment_pending", "to": "paid", "event": "payment_success"},
    {"from": "payment_pending", "to": "payment_failed", "event": "payment_declined"},
    {"from": "payment_failed", "to": "payment_pending", "event": "retry_payment"},
    {"from": "paid", "to": "fulfillment", "event": "start_fulfillment"},
    {"from": "fulfillment", "to": "shipped", "event": "ship"},
    {"from": "cart", "to": "cancelled", "event": "abandon"},
    {"from": "checkout", "to": "cancelled", "event": "cancel"},
    {"from": "payment_pending", "to": "cancelled", "event": "cancel"}
  ]
}
```

### CI/CD Pipeline

```json
{
  "name": "ci-pipeline",
  "states": [
    {"name": "pending", "kind": "initial"},
    {"name": "building", "kind": "normal"},
    {"name": "testing", "kind": "normal"},
    {"name": "deploying", "kind": "normal"},
    {"name": "deployed", "kind": "terminal"},
    {"name": "failed", "kind": "error"}
  ],
  "transitions": [
    {"from": "pending", "to": "building", "event": "start"},
    {"from": "building", "to": "testing", "event": "build_success"},
    {"from": "building", "to": "failed", "event": "build_failed"},
    {"from": "testing", "to": "deploying", "event": "tests_passed"},
    {"from": "testing", "to": "failed", "event": "tests_failed"},
    {"from": "deploying", "to": "deployed", "event": "deploy_success"},
    {"from": "deploying", "to": "failed", "event": "deploy_failed"},
    {"from": "failed", "to": "pending", "event": "retry"}
  ]
}
```

---

## Error Handling

All MCP tools return structured errors via `McpError::invalid_params`:

```json
{
  "error": {
    "code": -32602,
    "message": "Machine not found: 99"
  }
}
```

Common error conditions:
- Machine ID not found
- Invalid state kind
- Missing initial state
- Transition from wrong state
- Machine already terminal

---

## Best Practices

1. **Single Initial State**: Exactly one state with `kind: "initial"`
2. **Terminal States**: At least one terminal or error state
3. **No Orphan States**: All states reachable from initial
4. **Event Naming**: Use verb phrases (`start`, `complete`, `fail`)
5. **State Naming**: Use noun phrases (`pending`, `processing`, `shipped`)

---

## Version

- SOS Version: 0.1.0
- MCP Tools: 10
- Primitive Coverage: 15/15 (100%)
- Tests: 115 passing

---

*Authored by: Matthew Campion, PharmD; NexVigilant*
