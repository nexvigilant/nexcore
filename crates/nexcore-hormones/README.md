# nexcore-hormones

Endocrine system for the NexVigilant Core kernel. It manages persistent state modulators (hormones) that affect system-wide behavior, risk tolerance, and exploration rates across sessions.

## Intent
To provide a bio-analogous mechanism for long-term behavioral state. Unlike transient cytokines, hormones are persistent and decay slowly, providing a "mood" or "disposition" to the AI agent based on historical interactions.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **ς (State)**: The core primitive for managing the complex endocrine state.
- **π (Persistence)**: Durable storage of hormone levels in `~/.claude/hormones/state.json`.
- **N (Quantity)**: Quantified hormone levels (0.0 to 1.0) and their specific decay rates.
- **μ (Mapping)**: Mapping external stimuli to internal hormone shifts.

## Core Hormone Types
| Hormone | AI Agent Function |
|---------|--------------------|
| **Cortisol** | Stress response - increases risk aversion and validation depth. |
| **Dopamine** | Reward - reinforces patterns and increases exploration rate. |
| **Serotonin** | Stability - promotes consistency and predictable outcomes. |
| **Adrenaline** | Crisis mode - unlocks high-stakes capabilities, spikes on critical errors. |
| **Oxytocin** | Trust - strengthens partnership signals and increases verbosity. |
| **Melatonin** | Rest - tracks session fatigue and recommends pacing. |

## SOPs for Use
### Loading and Saving State
```rust
use nexcore_hormones::EndocrineState;

let mut state = EndocrineState::load();
state.apply_decay(); // Typically called at session start
state.save()?;
```

### Applying a Stimulus
```rust
use nexcore_hormones::{EndocrineState, Stimulus};

let mut state = EndocrineState::load();
let stimulus = Stimulus::TaskCompleted { complexity: 0.8 };
stimulus.apply(&mut state);
state.save()?;
```

## Behavioral Modifiers
The `BehavioralModifiers` struct derives actionable settings from the hormone state:
- `risk_tolerance`: Derived from Cortisol/Dopamine balance.
- `validation_depth`: Increased by Cortisol (stress-induced caution).
- `crisis_mode`: Triggered by high Adrenaline (>0.7).

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
