# nexcore-energy

Token-as-energy system for the NexVigilant Core kernel, modeled on ATP/ADP biochemistry. It manages AI token budgets by classifying consumption into metabolic pools and deriving execution strategies based on the current "Energy Charge."

## Intent
To provide a biologically-grounded framework for resource allocation under scarcity. It ensures that expensive AI models (Opus) are used for high-yield operations, while conserving energy (Haiku/Cache) during low-yield or resource-constrained periods.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **N (Quantity)**: Represents the discrete token counts in each metabolic pool.
- **κ (Comparison)**: Used for classifying energy states into metabolic regimes (Anabolic, Catabolic, etc.).
- **∝ (Proportion)**: Calculations for Energy Charge (EC) and Coupling Ratios (Value/Cost).
- **ς (State)**: Management of the current `EnergyState` and its transition over time.

## Metabolic Pools (Token Analogs)
| Pool | Biological Analog | AI Token Analog |
|------|-------------------|-----------------|
| **tATP** | Ready Energy | Tokens remaining in the active budget. |
| **tADP** | Productive Spend | Tokens spent on work that generated value (artifacts, successful edits). |
| **tAMP** | Degraded Waste | Tokens wasted on failures, retries, or verbose output. |

## Energy Charge (EC) Formula
Derived from Atkinson (1968):
`EC = (tATP + 0.5 * tADP) / (tATP + tADP + tAMP)`

## SOPs for Use
### Deciding on a Strategy
```rust
use nexcore_energy::{TokenPool, Operation, decide};

let pool = TokenPool::new(100_000);
let op = Operation::builder("deep-audit")
    .cost(5_000)
    .value(20_000.0)
    .build();

let strategy = decide(&pool, &op);
// Returns Strategy::Opus, Strategy::Sonnet, etc.
```

### Tracking Consumption
```rust
pool.spend_productive(2000); // For successful tasks
pool.spend_waste(500);       // For failed tool calls
pool.recycle(1000);          // For effective context compression (ATP Synthase)
```

## Key Components
- **TokenPool**: The state container for the three metabolic pools.
- **Regime**: Classification of energy state (Anabolic >0.85, Homeostatic 0.70-0.85, Catabolic 0.50-0.70, Crisis <0.50).
- **WasteClass**: Categorization of tAMP sources (Futile Cycling, Heat Loss, etc.).

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
