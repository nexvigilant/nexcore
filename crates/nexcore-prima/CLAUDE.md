# AI Guidance — nexcore-prima

Primitive-first programming language (πρίμα).

## Use When
- Defining formal rules or safety axioms that must ground to T1 primitives.
- Implementing domain-specific logic that requires high transfer confidence.
- Developing interpreters or transpilers for grounded logic.
- Verifying the "primitive weight" of a computational task.

## Grounding Patterns
- **Root Grounding**: All Prima computation must ground to `0` (absence) or `1` (existence).
- **Type Mapping**: Ensure new language features are mapped to the 15 Lex Primitiva symbols in `src/grounding.rs`.
- **T1 Primitives**:
  - `μ + σ`: Root primitives for the parsing and execution pipeline.
  - `Σ + κ`: Root primitives for data processing and branching.

## Maintenance SOPs
- **Lexer/Parser Updates**: When adding new syntax, you MUST update both `lexer.rs` and `parser.rs` and add a corresponding `Expr` or `Stmt` variant in `ast.rs`.
- **Unicode First**: Prefer the `.σ` extension and use the appropriate Unicode symbols in documentation.
- **Panic-Free**: The interpreter MUST not panic on malformed input. Always return `PrimaError`.

## Key Entry Points
- `src/lib.rs`: `eval()`, `parse()`, and `tokenize()` convenience functions.
- `src/interpret.rs`: The main tree-walking interpreter.
- `src/token.rs`: The Lex Primitiva token definitions.
