# nexcore-cytokine

Typed event signaling system for the NexVigilant Core kernel, based on biological immune system cytokine patterns. It provides a fire-and-forget messaging substrate for inter-crate communication, threat detection, and agent orchestration.

## Intent
To enable asynchronous, non-blocking coordination between disparate system components using a rich, biological analogy for signal types, concentration, and receptor binding.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **→ (Causality)**: Primary for signal emission causing receptor activation.
- **μ (Mapping)**: Typed matching of cytokine families to receptor handlers.
- **ρ (Recursion)**: Implements amplification cascades (one signal triggers multiple responses).
- **N (Quantity)**: Represents signal concentration and potency.

## Core Biological Analogs
| Family | Biological Function | AI Agent Analog |
|--------|--------------------|-----------------|
| **IL (Interleukin)** | Cell communication | Inter-component messaging |
| **TNF (Tumor Necrosis)** | Destroy threats | Blocking/Terminating harmful actions |
| **IFN (Interferon)** | Activate defenses | Amplication of safety responses |
| **TGF (Transforming)** | Regulate growth | Behavioral modulation |
| **CSF (Colony Stimulating)** | Spawn cells | Subagent creation (Mitosis) |

## SOPs for Use
### Emitting a Signal
```rust
use nexcore_cytokine::{Cytokine, CytokineFamily, Emitter};

let signal = Cytokine::new(CytokineFamily::Il1, "task_started")
    .with_potency(Potency::High);
emitter.emit(signal).await;
```

### Listening for Signals
```rust
use nexcore_cytokine::{CytokineFamily, Receptor};

receptor.on(CytokineFamily::Il1, |signal| async move {
    // Handle signal logic here
}).await;
```

## Key Components
- **CytokineBus**: The central relay for all system-wide signals.
- **Chemotaxis**: Gradient-based signal tracking for locating resources or errors.
- **Apoptosis**: Managed component shutdown and cleanup patterns.
- **Mitosis/Differentiation**: Patterns for agent specialization and spawning.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
