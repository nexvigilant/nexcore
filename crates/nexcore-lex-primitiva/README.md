# nexcore-lex-primitiva

The foundational symbolic system for the NexVigilant platform. This crate provides the irreducible **16 Lex Primitiva** symbols, the **80 Bedrock Atoms**, and the formal grounding to mathematical constants and foundations.

## Intent
To provide a universal vocabulary for grounded computation. Every high-tier type in the NexCore workspace ultimately decomposes into the symbols defined in this crate, enabling verifiable cross-domain reasoning.

## The 16 Lex Primitiva Symbols
| Symbol | Name | Tier | Root? |
| :--- | :--- | :---: | :---: |
| **σ** | Sequence | T1 | No |
| **μ** | Mapping | T1 | No |
| **ς** | State | T1 | No |
| **ρ** | Recursion | T1 | No |
| **∅** | Void | T1 | No |
| **∂** | Boundary | T1 | No |
| **f** | Frequency | T1 | No |
| **∃** | Existence | T1 | No |
| **π** | Persistence | T1 | No |
| **→** | Causality | T1 | **Yes** |
| **κ** | Comparison | T1 | No |
| **N** | Quantity | T1 | **Yes** |
| **λ** | Location | T1 | No |
| **∝** | Proportion | T1 | No |
| **Σ** | Sum | T1 | No |
| **×** | Product | T1 | **Axiomatic** |

## Core Components
- **DependencyGraph**: Traces primitives back to their mathematical roots (Peano, Category Theory).
- **CompositionAlgebra**: Logic for combining T1 primitives into higher-tier types.
- **MolecularWeight**: Shannon information-theoretic mass of a grounded concept.
- **TransferCalculator**: Predicts the confidence of transferring logic across domains.

## SOPs for Use
### Implementing GroundsTo
```rust
use nexcore_lex_primitiva::prelude::*;

impl GroundsTo for MyType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::State, 0.8)
    }
}
```

### Tracing a Primitive
```rust
let traces = DependencyGraph::trace(LexPrimitiva::Void);
// Traces back to the Root Constant 0
```

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
