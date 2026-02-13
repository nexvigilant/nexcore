# nexcore-state-os

**State Operating System** — A 15-layer runtime for state machine orchestration built on the Universal Theory of State.

## Architecture

STOS follows the Quindecet pattern: 15 layers, each with a unique dominant T1 primitive from the Lex Primitiva.

```
┌─────────────────────────────────────────────────────────────────┐
│                     STATE OPERATING SYSTEM                       │
├─────────────────────────────────────────────────────────────────┤
│  Layer  │ Module              │ Dominant │ Purpose              │
├─────────┼─────────────────────┼──────────┼──────────────────────┤
│ STOS-ST │ state_registry      │ ς State  │ Core state registry  │
│ STOS-TR │ transition_engine   │ → Cause  │ Transition execution │
│ STOS-BD │ boundary_manager    │ ∂ Bound  │ Initial/terminal     │
│ STOS-GD │ guard_evaluator     │ κ Compare│ Guard evaluation     │
│ STOS-CT │ count_metrics       │ N Quant  │ Metrics & counts     │
│ STOS-SQ │ sequence_controller │ σ Seq    │ Transition ordering  │
│ STOS-RC │ recursion_detector  │ ρ Recur  │ Cycle detection      │
│ STOS-VD │ void_cleaner        │ ∅ Void   │ Unreachable cleanup  │
│ STOS-PR │ persist_store       │ π Persist│ Snapshots & storage  │
│ STOS-EX │ existence_validator │ ∃ Exist  │ State validation     │
│ STOS-AG │ aggregate_coord     │ Σ Sum    │ Multi-machine agg    │
│ STOS-TM │ temporal_scheduler  │ ν Freq   │ Time-based sched     │
│ STOS-LC │ location_router     │ λ Loc    │ Distributed state    │
│ STOS-IR │ irreversibility_aud │ ∝ Irrev  │ Audit trails         │
│ STOS-MP │ mapping_transformer │ μ Map    │ State transforms     │
└─────────┴─────────────────────┴──────────┴──────────────────────┘
```

## Usage

```rust
use nexcore_state_os::prelude::*;

// Create the State OS kernel
let mut kernel = StateKernel::new();

// Create a machine
let machine_id = kernel.create_machine(0)?;

// Register states
let s0 = kernel.register_state(machine_id, "pending", StateKind::Initial)?;
let s1 = kernel.register_state(machine_id, "confirmed", StateKind::Normal)?;
let s2 = kernel.register_state(machine_id, "delivered", StateKind::Terminal)?;

// Register transitions
let t0 = kernel.register_transition(machine_id, "confirm", s0, s1)?;
let t1 = kernel.register_transition(machine_id, "deliver", s1, s2)?;

// Execute transitions
kernel.transition(machine_id, t0)?;
kernel.transition(machine_id, t1)?;

// Check terminal
assert!(kernel.is_terminal(machine_id)?);
```

## Builder Pattern

```rust
use nexcore_state_os::prelude::*;

let spec = MachineSpec::builder("order")
    .state("pending", StateKind::Initial)
    .state("confirmed", StateKind::Normal)
    .state("shipped", StateKind::Normal)
    .state("delivered", StateKind::Terminal)
    .transition("pending", "confirmed", "confirm")
    .transition("confirmed", "shipped", "ship")
    .transition("shipped", "delivered", "deliver")
    .build();

let mut instance = MachineInstance::new(1, spec).unwrap();
instance.handle("confirm");
instance.handle("ship");
instance.handle("deliver");
assert!(instance.terminated);
```

## Features

- `std` (default): Standard library support
- `async`: Async runtime integration (planned)

## License

All Rights Reserved. Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
