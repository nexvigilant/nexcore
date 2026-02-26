# AI Guidance — nexcore-forge-strategy

Decision engine and strategy parameters for the Code Forge.

## Use When
- Determining the next high-level action in a code generation loop.
- Modulating agent behavior between "Pragmatic" and "Pedantic" linting modes.
- Evaluating the fitness of a generation attempt.
- Managing tier-to-tier transitions in primitive-first construction.

## Grounding Patterns
- **Decision Chain**: Always follow the `decide()` priority (8 steps: ABANDON→FIX_BLOCKER→REFACTOR→LINT_FIX→DECOMPOSE→PROMOTE→EXPLORE→STUCK). Never refactor if there are blocking errors within the `safe_refactor_distance` (5). LINT_FIX only fires when `lint_strictness > 0.5` AND `warning_distance <= lint_radius` — both conditions must hold.
- **Quality Floor**: Respect the `0.313` floor. Do not spend excessive tokens on mid-process polish if progress can still be made.
- **T1 Primitives**:
  - `→ + κ`: Root primitives for the decision-threshold logic.
  - `ς + N`: Root primitives for managing the accumulated forge metrics.

## Maintenance SOPs
- **Fitness Mapping**: If the Forge protocol adds a new step (e.g., `EMIT`), a corresponding `ForgeDecision` variant and fitness weight must be added.
- **Genetic Transfer**: Remember that these values are *evolved*. Do not manually tweak them without significant empirical justification from a new generation run.
- **Boundary Checks**: Use `tier_confidence` to reduce expected success probability when crossing module boundaries.

## Key Entry Points
- `src/lib.rs`: `ForgeStrategy`, `ForgeDecision`, and `ForgeState` definitions.
- `src/scoring.rs`: Logic for computing generation quality ratios.
- `src/game.rs`: Mapping of evolved roguelike parameters to technical forge actions.
