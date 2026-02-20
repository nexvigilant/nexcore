# nexcore-prima

The primitive-first programming language for the NexVigilant platform. Every construct in the Prima language (πρίμα) is formally grounded in the 15 Lex Primitiva symbols, enabling verifiable and cross-domain transferable logic.

## Intent
To provide a computation environment where complexity is strictly tracked and safety is guaranteed through grounding. Prima code (`.σ`) is used to define behavioral rules, safety axioms, and domain-specific logic that must be transparent to both humans and AI agents.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **μ (Mapping)**: The core primitive for parsing source code to AST and evaluating expressions.
- **σ (Sequence)**: Manages the execution flow of programs and the ordering of tokens.
- **Σ (Sum)**: Root primitive for arithmetic and logical accumulation.
- **κ (Comparison)**: Used for type checking and threshold evaluation within the language.

## The 15 Lex Primitiva Symbols
`σ μ ς ρ ∅ ∂ ν ∃ π → κ N λ ∝ Σ`

## File Extensions
- **.σ** (Sigma): Preferred Unicode extension.
- **.prima**: Standard ASCII fallback.

## SOPs for Use
### Evaluating Source Code
```rust
use nexcore_prima::eval;

let result = eval("1 + 2")?;
assert_eq!(result, Value::int(3));
```

### Defining a Function
```prima
fn f(x: N) → N {
    x * 2
}
f(21) // Returns 42
```

## Language Tiers
| Tier | Primitives | Transfer Confidence |
| :--- | :---: | :--- |
| **T1** | 1 | 1.0 |
| **T2-P** | 2-3 | 0.9 |
| **T2-C** | 4-5 | 0.7 |
| **T3** | 6+ | 0.4 |

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
