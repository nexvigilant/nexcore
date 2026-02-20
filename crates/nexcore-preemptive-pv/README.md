# nexcore-preemptive-pv

Three-tier signal detection system for the NexVigilant platform. This crate implements the **Preemptive PV Equation**, enabling the system to transition from reactive detection (did harm occur?) to predictive and preemptive intervention (can I prevent irreversible harm?).

## Intent
To provide an advanced analytical framework for early safety intervention. It uses thermodynamic and kinetic analogies (Gibbs energy, Hill amplification, Nernst noise) to model the feasibility and trajectory of safety signals, allowing for decisions that anticipate and block harm before it becomes irreversible.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **→ (Causality)**: The primary primitive for the Tier 3 preemptive decision engine.
- **∝ (Proportion)**: Used for calculating Hill amplification (Gamma) and noise correction (eta).
- **κ (Comparison)**: Evaluates signal emergence feasibility (DeltaG) against intervention costs.
- **σ (Sequence)**: Manages the temporal trajectory and three-tier detection pipeline.
- **N (Quantity)**: Represents scalar inputs like reporting counts and seriousness weights.

## The Three Tiers
| Tier | Name | Key Question | Calculation |
| :--- | :--- | :--- | :--- |
| **Tier 1** | Reactive | Did harm occur? | `S = N/E` (Reporting Ratio) |
| **Tier 2** | Predictive | Will harm occur? | `Psi = DeltaG * Gamma * (1-eta)` |
| **Tier 3** | Preemptive | Can I prevent it? | `Pi = Psi * Omega - C(I)` |

## Core Components
- **Gibbs**: Models signal emergence feasibility using thermodynamic energy analogies.
- **Trajectory**: Tracks signal growth over time using Hill amplification curves.
- **Severity (Omega)**: Weights signals by their inherent seriousness and irreversibility.
- **Intervention**: Models the impact of "competitive inhibition" strategies on signal propagation.

## SOPs for Use
### Evaluating a Preemptive Decision
```rust
use nexcore_preemptive_pv::preemptive;
use nexcore_preemptive_pv::types::*;

let result = preemptive::evaluate_default(&gibbs, &data, &noise, Seriousness::Fatal);
if result.decision.requires_intervention() {
    println!("Preemptive intervention recommended!");
}
```

### Analyzing Signal Trajectory
Use the `trajectory` module to determine if a signal is in a state of logarithmic growth or approaching saturation.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
