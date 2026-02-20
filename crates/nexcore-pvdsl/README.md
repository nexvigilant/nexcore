# nexcore-pvdsl

Pharmacovigilance Domain-Specific Language (PVDSL) for the NexVigilant Core kernel. This crate provides a high-level scripting environment tailored for drug safety professionals, enabling the definition and execution of safety rules, signal detection algorithms, and causality assessments.

## Intent
To provide a safe, domain-focused abstraction over complex pharmacovigilance calculations. PVDSL source code is compiled into custom bytecode and executed on a virtual machine (VM), ensuring consistent results across molecular, clinical, and regulatory domains.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **μ (Mapping)**: Maps PVDSL source code to bytecode and function identifiers to internal Rust implementations.
- **σ (Sequence)**: Manages the compilation pipeline and the sequential execution of VM instructions.
- **κ (Comparison)**: Used for evaluating logical conditions and safety thresholds within scripts.
- **N (Quantity)**: Primary primitive for all numeric results (signal ratios, scores, dates).

## Built-in Namespaces
| Namespace | Functionality | Key Examples |
| :--- | :--- | :--- |
| `signal::*` | Signal Detection | `prr`, `ror`, `ic`, `ebgm`, `chi_square`. |
| `causality::*` | Causality Assessment | `naranjo`, `who_umc`, `rucam`. |
| `meddra::*` | Medical Coding | `levenshtein`, `similarity`. |
| `chem::*` | Capability Assessment | `arrhenius`, `michaelis_menten`, `hill`. |
| `math::*` | Basic Math | `sqrt`, `pow`, `log`, `ln`, `exp`. |

## SOPs for Use
### Executing a PVDSL Script
```rust
use nexcore_pvdsl::PvdslEngine;

let mut engine = PvdslEngine::new();
let prr = engine.eval_number("return signal::prr(10, 90, 100, 9800)")?;
println!("Calculated PRR: {:.2}", prr);
```

### Adding a new Function
1. Define the function implementation in `src/runtime.rs` or the appropriate domain module.
2. Register the function name and signature in `src/lexer.rs` or the `builtins` table.
3. Update the `VirtualMachine` to handle the new operation if it requires a custom opcode.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
