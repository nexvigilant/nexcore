# AI Guidance — nexcore-lex-primitiva

Foundational symbolic system and T1 primitive registry.

## Use When
- Grounding new Rust types in Lex Primitiva symbols.
- Calculating transfer confidence between disparate domains.
- Analyzing the structural complexity or "molecular weight" of a concept.
- Tracing the mathematical derivation of a primitive.

## Grounding Patterns
- **Dominant Selection**: When implementing `GroundsTo`, always specify the `dominant` primitive to enable accurate phase-transition analysis.
- **Pure Primitives**: Use `is_pure()` to check if a type grounds to exactly one T1 symbol.
- **T1 Primitives**:
  - `→ + N`: The two active root primitives for decision and value.
  - `×`: The axiomatic product primitive present in all multi-primitive compositions.

## Maintenance SOPs
- **Symbol Uniqueness**: Never reuse a symbol (Unicode or ASCII) for a new primitive.
- **Bedrock Atoms**: Each primitive MUST have exactly 5 Bedrock Atoms in `src/bedrock.rs`.
- **DAG Invariant**: The primitive dependency graph MUST remain a Directed Acyclic Graph (DAG). Run tests to verify no cycles.

## Key Entry Points
- `src/primitiva.rs`: `LexPrimitiva` and `PrimitiveComposition` definitions.
- `src/tier.rs`: The 4-tier grounding system.
- `src/grounding.rs`: The `GroundsTo` trait.
- `src/graph.rs`: `DependencyGraph` and mathematical foundations.
