# AI Guidance — nexcore-pvdsl

Pharmacovigilance Domain-Specific Language (PVDSL) engine.

## Use When
- Implementing complex drug safety algorithms that must be configurable at runtime.
- Translating regulatory guidelines (GVP, ICH) into executable rules.
- Performing batch signal detection on large datasets.
- Bridging the gap between non-technical PV experts and the Rust kernel.

## Grounding Patterns
- **Namespacing**: Always use the fully qualified namespace (e.g., `signal::prr` rather than just `prr`) to ensure correct function mapping.
- **Bytecode Integrity**: Never modify the `OpCode` enum without a corresponding update to the `BytecodeGenerator` and `VirtualMachine`.
- **T1 Primitives**:
  - `μ + σ`: Root primitives for the compiler and VM loop.
  - `κ + N`: Root primitives for arithmetic and logical branching.

## Maintenance SOPs
- **Wolfram Validation**: All new mathematical or signal functions MUST be validated against Wolfram Alpha results. Use the `wolfram_validated` test module as a template.
- **Strict Typing**: While PVDSL is flexible, the runtime values are strictly grounded in `f64` (Number) or `String`. Always use `RuntimeValue` for boundary crossing.
- **No Unsafe/Panic**: The VM must be robust against stack overflow or division by zero. Always return `PvdslError`.

## Key Entry Points
- `src/lib.rs`: `PvdslEngine` facade.
- `src/vm.rs`: The virtual machine and instruction executor.
- `src/transpiler.rs`: Support for higher-level regulatory rule translation.
