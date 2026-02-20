# AI Guidance — nexcore-energy

Token budget management via metabolic analogy.

## Use When
- Deciding which AI model to use for a specific task (Opus vs. Haiku).
- Monitoring token burn rates and remaining "Energy Charge."
- Classifying failed operations into `WasteClass` for future avoidance.
- Implementing "ATP Synthase" patterns (recovering budget via compression/caching).

## Grounding Patterns
- **Energy Charge (EC)**: Always monitor EC before starting a `large` (>5k token) operation.
- **Coupling Ratio**: Higher yield tasks (artifact generation) justify a lower EC, while low-yield tasks (file listing) require high EC.
- **T1 Primitives**:
  - `N + ∝`: Budget mathematics and proportions.
  - `κ + ς`: State-based strategy selection.

## Maintenance SOPs
- **Thresholds**: EC < 0.5 is a hard "Checkpoint" signal. Agents should save all artifacts and halt.
- **Waste Tracking**: Always call `spend_waste()` for tool failures to ensure the EC accurately reflects system overhead.
- **Anabolic Policy**: Only use `Opus` in the Anabolic regime (>0.85 EC) or when `coupling_ratio` is >2.0.

## Key Entry Points
- `src/lib.rs`: The core `decide()` algorithm and `TokenPool` struct.
- `src/grounding.rs`: T1 grounding for energy types.
