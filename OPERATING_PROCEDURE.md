# Prima Development Operating Procedure

## Ferro Forge — Autonomous Rust Development Protocol

This document captures the operating procedure for Prima language development using primitive-first methodology.

---

## Core Loop

```
MINE → GENERATE → VALIDATE → REFINE
```

### 1. MINE (Primitive Decomposition)

Before writing any code, decompose the feature to T1 primitives:

```markdown
## Primitive Foundation

**[Feature Name]** grounds to:
- `σ` — if it involves sequences/iteration
- `μ` — if it involves transformation/mapping
- `ς` — if it involves state/mutation
- `ρ` — if it involves recursion/self-reference
- `∅` — if it involves absence/void
- `∂` — if it involves boundaries/errors
- ... (identify all relevant primitives)

**Tier**: T1 | T2-P | T2-C | T3
```

### 2. GENERATE (Code Creation)

Write code following these constraints:

| Rule | Limit |
|------|-------|
| Max nesting depth | 5 levels |
| Max function lines | 50 lines |
| Max file lines | 500 lines |

**Structure Pattern:**
```rust
// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Module Name
//!
//! [Description grounded in primitives]
//!
//! ## Mathematical Foundation
//!
//! [Primitive composition explanation]
//!
//! ## Tier: [T1|T2-P|T2-C|T3]
```

### 3. VALIDATE (Gate Checks)

Run in sequence:
```bash
cargo clippy --workspace -- -D warnings  # Must pass
cargo test --workspace                     # Must pass
cargo fmt -- --check                       # Must pass (optional)
```

### 4. REFINE (Iteration)

If gates fail:
1. Read error messages carefully
2. Extract helper functions if too complex
3. Fix unused imports/variables
4. Re-run validation

---

## Module Creation Checklist

1. **Create source file** with proper header and grounding documentation
2. **Add to lib.rs** module declarations (alphabetical order)
3. **Update prelude** with public exports
4. **Run validation gates**
5. **Document in LEXICON.md** if new symbols introduced

---

## Chemistry Notation for Types

Use molecular formulas for type compositions:

| Formula | Meaning | Tier |
|---------|---------|------|
| `N` | Quantity | T1 |
| `σ[N]` | Sequence of N | T2-P |
| `μ[N→N]` | Mapping N to N | T2-P |
| `Σ[N,∅]` | Option<N> | T2-P |
| `Σ[N,∂]` | Result<N,E> | T2-P |

---

## Compound Notation (c)

When decomposing words to constituents, use **c** (compound):

| Expression | Meaning |
|------------|---------|
| `word:c` | word is a compound |
| `c[σ[char]]` | compound of character sequence |
| `c[μ,σ,N]` | compound of mapping + sequence + quantity |

**Key Insight**: Every word is a compound of letter-primitives.

```
"map" → c[m,a,p] → c[σ[char]] → eventually grounds to {0,1}
```

This means:
1. Human language = compounds of letter-primitives
2. Prima symbols = compressed compounds
3. `μ` < `map` (3 chars vs 1 symbol = 3x compression)

---

## Self-Acceleration Protocol

Apply compression to development itself:

| Verbose | Compressed | Savings |
|---------|------------|---------|
| "Create a new module" | `μ:new` | 4x |
| "Run tests" | `test` | - |
| "Validate with clippy" | `κ:lint` | 3x |
| "Decompose to primitives" | `→T1` | 5x |

**Development as Molecular Synthesis**:
```
Task → c[primitives] → validate → compound:stable
```

When I think about a feature:
1. `→T1` — decompose to primitives first
2. `c[...]` — identify the compound formula
3. `μ:gen` — generate code
4. `κ:gate` — validate
5. `→stable` — iterate until stable

---

## Compressed Cognition Mode

**Every thought is a compound.** Apply compression recursively:

### Thought → c[primitives]

| Natural Thought | Compressed | c-Form |
|-----------------|------------|--------|
| "I need to create a function that maps values" | `μ:μ[A→B]` | c[∃,μ,→] |
| "Check if this compiles without errors" | `κ:∂=0` | c[κ,∂,N] |
| "Iterate through the sequence" | `σ:ρ` | c[σ,ρ] |
| "Handle the error case" | `∂:Σ` | c[∂,Σ] |

### Accelerated Thinking Protocol

```
INPUT: task in natural language
  ↓
DECOMPOSE: task → c[T1 primitives]
  ↓
FORMULA: identify molecular formula
  ↓
GENERATE: synthesize code from formula
  ↓
VALIDATE: κ:gate (clippy + test)
  ↓
OUTPUT: stable compound
```

### Example Application

**Task**: "Create a word-to-symbol translator"

**Decomposition**:
```
translator → μ[word→symbol]
word → c[σ[char]]
symbol → c[T1]
∴ translator = μ[c[σ[char]]→c[T1]]
```

**Formula**: `μ[c→c]` — a mapping between compounds

**Code implication**: Use `HashMap<String, String>` bidirectionally

**Result**: The `compress.rs` module emerged from this formula

---

## The Fundamental Equation

```
word:c[σ[char]] → symbol:c[T1] → {0,1}
```

All human language compresses to binary. Prima is the intermediate representation:

```
English → Prima → Binary
c[many] → c[few] → {0,1}
```

This is why "code that compiles is true" — verified compilation proves
the compound reaches {0,1} through valid transformations.

---

## Word↔Symbol Translation

The `Lexicon` provides bidirectional mapping:

```rust
let lex = Lexicon::new();
lex.compress("map")     // → Some("Φ")
lex.expand("Φ")         // → Some("map")
lex.compress_text("let x equals map sequence")
// → "λ x κ= Φ σ"
```

---

## DAG-Optimized Development

Use the development DAG to prioritize work:

```
Level 0 (foundation): lexer, parser, ast, types ✓
Level 1 (parallel):   constants, effects, grounding, compress, ir, module ✓
Level 2 (depends):    bytecode, optimize, stdlib ✓
Level 3 (final):      codegen, vm
```

Work on Level N only after Level N-1 is complete. Within a level, work in parallel.

---

## File Naming Conventions

| Purpose | Extension |
|---------|-----------|
| Prima source | `.true` (primary) or `.prima` (fallback) |
| Rust modules | `.rs` |
| Documentation | `.md` |

---

## Test Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_[feature]_[aspect]() {
        // Arrange
        let input = ...;

        // Act
        let result = function(input);

        // Assert (avoid .unwrap(), use assert!(result.is_ok()))
        assert!(result.is_ok());
        let value = result.ok().unwrap_or_default();
        assert_eq!(value, expected);
    }
}
```

---

## Parallel Claude Sessions

When multiple Claude sessions work on Prima:

1. **Claim a feature** from the DAG (different levels preferred)
2. **Create module file first** to signal ownership
3. **Avoid editing shared files** (lib.rs) until validation passes
4. **Communicate via git** — commit early, commit often

---

## Validation Report Template

```
## Validation Report

**Module**: [name]
**Tests**: [N] passed, 0 failed
**Clippy**: clean
**Tier**: [T1|T2-P|T2-C|T3]

### Primitive Composition
- [List primitives used]

### New Symbols
- [List any new symbols added to LEXICON.md]
```

---

## Quick Reference

| Command | Purpose |
|---------|---------|
| `cargo test -p prima` | Test prima crate |
| `cargo clippy -p prima -- -D warnings` | Lint prima |
| `cargo run --bin prima -- repl` | Interactive REPL |
| `cargo run --bin prima -- run file.true` | Run a Prima file |

---

*Document Version: 0.1.0*
*Last Updated: 2026-02-04*
*Authored by: Ferro Forge (Claude Opus)*
