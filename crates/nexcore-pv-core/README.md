# nexcore-pv-core

The high-performance computation engine for pharmacovigilance (PV) operations within the NexVigilant platform. This crate implements the core mathematical and logical foundations for signal detection, causality assessment, and pharmacokinetic modeling.

## Intent
To provide a consolidated, type-safe library for all safety-critical PV calculations. It grounds domain-specific safety observations into formal axioms defined in the Theory of Vigilance (ToV).

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **N (Quantity)**: The primary primitive for all PV metrics (PRR, ROR, IC, EBGM).
- **κ (Comparison)**: Used for evaluating signal thresholds and comparing drug-event profiles.
- **→ (Causality)**: Formal implementation of Naranjo, WHO-UMC, and RUCAM loops.
- **∂ (Boundary)**: Defines the safety boundaries and intervention thresholds for drugs.
- **ς (State)**: Manages the temporal state and progression of adverse event cases (ICSRs).

## Core Modules
- **signals**: Standard algorithms (PRR, ROR, IC, EBGM) and advanced detection (MaxSPRT, CuSum).
- **causality**: Formal frameworks for assessing the link between exposure and outcome.
- **coding**: O(1) Perfect Hash lookup for MedDRA/ICH terms and fuzzy matching.
- **pk/thermodynamic**: Pharmacokinetics and binding energy calculations for molecular safety.
- **risk**: Predictive analytics including Safety-at-Risk and Monte Carlo simulations.
- **regulatory**: Compliance bridges for FDA, EMA, and ICH reporting standards.

## SOPs for Use
### Signal Detection
```rust
use nexcore_pv_core::signals::calculate_prr;
let prr = calculate_prr(10, 90, 100, 9800); // (a, b, c, d)
```

### Safety Margin Calculation
```rust
use nexcore_pv_core::SafetyMargin;
let margin = SafetyMargin::calculate(prr, ror_lower, ic025, eb05, n);
```

### Regulatory Lookup
```rust
use nexcore_pv_core::lookup_term;
if let Some(term) = lookup_term("Adverse Event") {
    println!("Definition: {}", term.definition);
}
```

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
