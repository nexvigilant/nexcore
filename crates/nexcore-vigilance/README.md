# nexcore-vigilance

The single source of truth for all consolidated pharmacovigilance modules within the NexVigilant platform. This crate serves as the **domain monolith**, implementing the core Theory of Vigilance (ToV) axioms, Guardian-AV risk detection, and regulatory signal processing.

## Intent
To provide a formal, axiomatic computation kernel for safety-critical signal detection across molecular, physiological, and clinical domains. It grounds domain-specific observations into universal safety primitives.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **ς (State)**: Manages the internal safety manifold and transition between safe/harm states.
- **∂ (Boundary)**: Defines the stratified manifold boundaries for the 8 ToV harm types.
- **μ (Mapping)**: Maps raw signals (PRR, ROR, IC, EBGM) to the safety manifold.
- **→ (Causality)**: Implements Naranjo and WHO-UMC causality assessment loops.

## Core Frameworks
1. **Theory of Vigilance (ToV)**: Implementing the 8 Harm Types (A-H) and the Safety Manifold.
2. **Guardian-AV**: Homeostatic control loops for real-time threat sensing and response.
3. **Pharmacovigilance Operating System (PVOS)**: The substrate for executing safety-critical workflows.
4. **Skills Registry**: Consolidated competency-based assessment and tool registry.

## The 8 Harm Types (§9 ToV)
| Type | Name | Conservation Law | Hierarchy Levels |
|------|------|------------------|------------------|
| A | Acute | Law 1 (Mass) | 4-6 |
| B | Cumulative | Law 1 (Mass) | 5-7 |
| C | Off-Target | Law 2 (Energy) | 3-5 |
| D | Cascade | Law 4 (Flux) | 4-7 |
| E | Idiosyncratic | θ-space | 3-6 |
| F | Saturation | Law 8 (Capacity) | 3-5 |
| G | Interaction | Law 5 (Catalyst) | 4-6 |
| H | Population | θ-distribution | 6-8 |

## SOPs for Use
### Adding a new Domain Module
1. Create a module directory under `src/`.
2. Implement the standard Gold Standard structure: `primitives.rs`, `composites.rs`, `grounding.rs`, `transfer.rs`.
3. Register the module in `src/lib.rs`.

### Implementing a new Safety Axiom
1. Add the formal definition to `src/tov/`.
2. Update `SafetyMargin::calculate` in `src/tov/mod.rs` if the axiom affects the distance calculation.
3. Add a corresponding test case to `src/tests/`.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
