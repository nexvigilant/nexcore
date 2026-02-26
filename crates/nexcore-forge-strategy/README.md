# nexcore-forge-strategy

Evolved decision parameters and strategy engine for the NexCore Code Forge. It implements a decision-making framework discovered through evolutionary training, mapping game-domain topologies to the technical construction process.

## Intent
To provide AI agents with optimal thresholds and priority chains for code generation. It answers the question: "What should the Forge do next?" (e.g., Abandon, Fix Blocker, Refactor, or Decompose) based on current quality metrics and confidence.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **→ (Causality)**: The primary primitive for the decision engine loop.
- **κ (Comparison)**: Evaluates current state against evolved thresholds (e.g., `quality_floor`).
- **ς (State)**: Manages the mutable `ForgeState` across the generation lifecycle.

## Decision Priority Chain
1. **ABANDON**: Confidence below threshold (0.027).
2. **FIX_BLOCKER**: Compiler errors present.
3. **REFACTOR**: Quality below floor (0.313) AND safe to do so (no blocker within 5 hops).
4. **LINT_FIX**: Warnings within `lint_radius` (2 hops) AND pedantic mode active (strictness > 0.5).
5. **DECOMPOSE**: Primitives available to mine.
6. **PROMOTE**: Current tier complete, advance eagerly.
7. **EXPLORE**: Speculative alternative search (when `speculative_generation = true`).
8. **STUCK**: No actionable state — all paths blocked; requires external intervention.

## Evolved Parameters (Sample)
| Parameter | Value | Interpretation |
| :--- | :---: | :--- |
| `quality_floor` | 0.313 | Accept low quality mid-gen to maintain momentum. |
| `abandon_threshold` | 0.027 | Persistence beats perfection; almost never give up. |
| `tier_promotion_eagerness` | 0.890 | Advance tiers quickly once primitives are mined. |
| `boundary_caution` | 0.718 | Be extremely careful at API and module boundaries. |

## SOPs for Use
### Taking a Decision
```rust
use nexcore_forge_strategy::{ForgeStrategy, ForgeState};

let strategy = ForgeStrategy::default();
let state = ForgeState { ... };
let decision = strategy.decide(&state);
// Execute the corresponding ForgeDecision action
```

### Computing Fitness
Use `compute_fitness(&metrics)` to evaluate the quality of a generation attempt based on primitive collection, compiler success, and turn efficiency.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
