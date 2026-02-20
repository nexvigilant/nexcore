# disney-loop

A forward-only causal discovery pipeline for the NexVigilant Core kernel. It implements the **Disney Loop** model: `ρ(t) → ∂(¬σ⁻¹) → ∃(ν) → ρ(t+1)`, which ensures that the system only advances through novelty discovery and strictly rejects any state regression.

## Intent
To provide a structured, irreversible pipeline for knowledge and compound discovery. It automates the process of assessing current state, filtering out regressive "backward" movements, and aggregating new discoveries across multiple domains.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **ρ (Recursion)**: Manages the iterative transition between state snapshots (t → t+1).
- **∂ (Boundary)**: Implements the **Anti-Regression Gate** (rejects backward movement).
- **∃ (Existence)**: The core primitive for the **Curiosity Search** (detecting new discoveries).
- **ν (Frequency)**: Tracks the rate of discovery and novelty accumulation per domain.
- **σ (Sequence)**: Enforces the unidirectional "forward-only" flow of the pipeline.

## Pipeline Stages
1. **ρ(t)**: Ingest current state snapshot.
2. **∂(¬σ⁻¹)**: Anti-Regression Gate — Filter out any records indicating backward movement.
3. **∃(ν)**: Curiosity Search — Aggregate novelty scores and count new discoveries by domain.
4. **ρ(t+1)**: New State Sink — Persist the transformed forward-only state to JSON.

## SOPs for Use
### Running the Discovery Pipeline
```rust
use disney_loop::{transform_anti_regression_gate, transform_curiosity_search, sink_new_state};

let df = // load initial state DataFrame
let filtered = transform_anti_regression_gate(df.lazy())?;
let aggregated = transform_curiosity_search(filtered)?;
let row_count = sink_new_state(aggregated, Path::new("output/state_next.json"))?;

println!("Pipeline complete. {} domains advanced.", row_count);
```

## Key Components
- **Anti-Regression Gate**: The logical filter ensuring `direction != 'backward'`.
- **Curiosity Search**: The aggregation logic for novelty scoring.
- **New State Sink**: The irreversible persistence layer.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
