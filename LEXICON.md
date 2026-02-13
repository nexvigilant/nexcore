# Prima Lexicon
## πρίμα — The Compiled Language

*"Humans need to learn to compile their language."*

A dictionary of Prima's symbolic vocabulary, organized by category. Each entry traces back to the 15 Lex Primitiva and ultimately to `{0, 1}`.

---

## Root Constants

| Symbol | Meaning | Foundation |
|--------|---------|------------|
| `0` | Absence, false, zero | Mathematical zero |
| `1` | Existence, true, one | Mathematical one |

---

## The 15 Lex Primitiva (T1-Universal)

| Symbol | Name | Meaning | Rust Manifestation |
|--------|------|---------|-------------------|
| `σ` | Sequence | Ordered collection | `Vec<T>`, `[T]`, iteration |
| `μ` | Mapping | Key→Value transformation | `HashMap<K,V>`, `fn(A) → B` |
| `ς` | State | Mutable data at a point | `mut`, `Cell<T>`, `RefCell<T>` |
| `ρ` | Recursion | Self-reference | `fn f() { f() }`, recursive types |
| `∅` | Void | Absence of value | `()`, `None`, null |
| `∂` | Boundary | Edge/limit | Error handling, conditionals |
| `ν` | Invariant | Unchanging truth | `const`, assertions |
| `∃` | Existence | Something exists | `Option<T>::Some`, `is_some()` |
| `π` | Persistence | Duration/storage | Files, database, heap |
| `→` | Causality | Cause produces effect | Function application |
| `κ` | Comparison | Ordering relation | `<`, `>`, `==`, `Ord` |
| `N` | Quantity | Numeric value | Numbers, counts |
| `λ` | Location | Reference point | Variables, bindings, pointers |
| `∝` | Proportion | Ratio relationship | Scaling, percentages |
| `Σ` | Sum | One of many (enum) | `enum`, `Result`, `match` |

---

## Keywords (Primitive Aliases)

| Keyword | Symbol | Grounding |
|---------|--------|-----------|
| `let` | `λ` | Location binding |
| `fn` | `μ` | Mapping definition |
| `if` | `∂` | Boundary condition |
| `for` | `σ` | Sequence iteration |
| `match` | `Σ` | Sum decomposition |

---

## Higher-Order Functions (T2-C)

| Symbol | Name | Type Signature | Grounding |
|--------|------|----------------|-----------|
| `Φ` | map | `(σ[A], A→B) → σ[B]` | σ + μ + → |
| `Ψ` | filter | `(σ[A], A→Bool) → σ[A]` | σ + μ + Σ |
| `Ω` | fold | `(I, (I,A)→I, σ[A]) → I` | σ + μ + ρ |
| `∃?` | any | `(σ[A], A→Bool) → Bool` | σ + ∃ + Σ |
| `∀?` | all | `(σ[A], A→Bool) → Bool` | σ + ν + Σ |
| `⊃` | find | `(σ[A], A→Bool) → A\|∅` | σ + ∃ + ∅ |
| `⊠` | zip | `(σ[A], σ[B]) → σ[(A,B)]` | σ + σ + μ |

---

## Comparison Operators

| Symbol | ASCII | Operation | Grounding |
|--------|-------|-----------|-----------|
| `κ=` | `==` | Equals | κ → {0, 1} |
| `κ<` | `<` | Less than | κ → {0, 1} |
| `κ>` | `>` | Greater than | κ → {0, 1} |

---

## I/O Functions

| Symbol | Name | Type | Grounding |
|--------|------|------|-----------|
| `ω` | print | `A → ∅` | → + π + ∅ |
| `ωn` | println | `A → ∅` | → + π + ∅ + σ |

---

## Sequence Operations

| Symbol | Name | Type | Grounding |
|--------|------|------|-----------|
| `#` | len | `σ[A] → N` | σ + N |
| `↑` | head | `σ[A] → A` | σ + ∂ |
| `↓` | tail | `σ[A] → σ[A]` | σ + σ |
| `⊕` | push | `(σ[A], A) → σ[A]` | σ + ς |
| `⊖` | pop | `σ[A] → (σ[A], A)` | σ + ς + Σ |
| `⊙` | concat | `(σ[A], σ[A]) → σ[A]` | σ + σ + μ |
| `‥` | range | `(N, N) → σ[N]` | N + σ |

---

## String Operations

| Symbol | Name | Type | Grounding |
|--------|------|------|-----------|
| `χ` | chars | `String → σ[Char]` | σ[N] → σ[N] |
| `⊘` | split | `(String, String) → σ[String]` | σ + ∂ |
| `⊗` | join | `(σ[String], String) → String` | σ + μ |
| `⇑` | upper | `String → String` | μ |
| `⇓` | lower | `String → String` | μ |
| `⊢` | trim | `String → String` | ∂ |
| `↔` | replace | `(String, String, String) → String` | μ |

---

## Math Operations

| Symbol | Name | Type | Grounding |
|--------|------|------|-----------|
| `±` | abs | `N → N` | N + ∂ |
| `⌊` | min | `(N, N) → N` | N + κ |
| `⌈` | max | `(N, N) → N` | N + κ |

---

## Type Introspection

| Symbol | Name | Type | Grounding |
|--------|------|------|-----------|
| `τ` | typeof | `A → String` | ∃ + σ |
| `T` | tier | `A → String` | ∃ + N + σ |

---

## Verification

| Symbol | Name | Type | Grounding |
|--------|------|------|-----------|
| `‼` | assert | `Bool → ∅` | ν + ∂ + ∅ |
| `∈` | contains | `(σ[A], A) → Bool` | σ + κ + Σ |

---

## Grounding Functions

| Symbol | Name | Type | Purpose |
|--------|------|------|---------|
| `K` | constants | `A → σ[Symbol]` | Show root constants |
| `C` | composition | `A → σ[Symbol]` | Show primitive composition |
| `X` | transfer | `(A, Domain) → N` | Cross-domain confidence |

---

## Type Constructors

| Syntax | Meaning | Example | Tier |
|--------|---------|---------|------|
| `N` | Quantity | `42` | T1 |
| `Bool` | Boolean | `true` | T1 |
| `String` | Text | `"hello"` | T2-P |
| `σ[T]` | Sequence | `σ[1,2,3]` | T2-P |
| `μ[K→V]` | Mapping | `μ("a"→1)` | T2-P |
| `T\|E` | Sum/Result | `N\|∅` | T2-P |
| `(A,B)→C` | Function | `(N,N)→N` | T2-P |

---

## Homoiconicity (Code as Data)

| Syntax | Name | Meaning | Grounding |
|--------|------|---------|-----------|
| `'expr` | Quote | AST as data | ρ |
| `` `expr `` | Quasiquote | Template | ρ + σ |
| `~expr` | Unquote | Evaluate | → |
| `~@expr` | Splice | Evaluate & flatten | → + σ |
| `:name` | Symbol | Interned identifier | λ |

---

## Tier System

| Tier | Primitives | Transfer Confidence | Example |
|------|------------|---------------------|---------|
| **T1** | 1 | 1.0 (universal) | `N`, `σ`, `μ` |
| **T2-P** | 2-3 | 0.9 (cross-domain primitive) | `σ[N]`, `μ[String→N]` |
| **T2-C** | 4-5 | 0.7 (cross-domain composite) | `Result[T,E]` |
| **T3** | 6+ | 0.4 (domain-specific) | `PatientRecord` |

---

## Pipeline Operator

| Syntax | Meaning | Grounding |
|--------|---------|-----------|
| `a \|> f` | `f(a)` | → (causality flows left-to-right) |
| `a \|> f \|> g` | `g(f(a))` | → + → (composition) |

**Example:**
```prima
σ[1,2,3,4,5]
    |> Φ(|x| x * 2)     // σ → map → σ
    |> Ψ(|x| x κ> 4)    // σ → filter → σ
    |> Ω(0, |a,b| a+b)  // σ → fold → N
// Result: 24
```

---

## Sum Decomposition

Every `Σ` (sum/fold) can be decomposed to primitives:

```
Σ(σ) = ρ(+, 0, σ)
     = fold(combine, identity, sequence)

Grounding chain:
  Σ → ρ (recursion)
    → σ (sequence to iterate)
    → N (accumulator)
    → + (binary operation)
    → κ= (termination check)
    → 0 (identity element)
    → 1 (existence check)
    → = (equality)
```

---

## Philosophy

### On Flattening

Only flatten expressions when mathematically equivalent:

```prima
// Valid flattening (preserves semantics):
Φ(σ[1,2,3], |x| x*2) ≡ σ[1,2,3] |> Φ(|x| x*2)

// Invalid flattening (loses structure):
∂ x κ> 0 { f(x) } else { g(x) }
≠ f(x) // Cannot flatten conditional
```

When flattening is not possible, introduce new symbols for constant concepts rather than expanding to words.

### Code That Compiles Is True

```
1 compiles → mathematically true
0 fails → mathematically false
```

The compiler is a proof verifier. If your Prima code compiles, it traces to `{0, 1}`.

---

## Adding New Symbols

When introducing a new concept:

1. **Check if existing primitive suffices** — most concepts compose from the 15
2. **Determine tier** — how many primitives compose this concept?
3. **Choose symbol** — prefer Unicode mathematical symbols
4. **Document grounding** — show composition chain to `{0, 1}`
5. **Add to lexicon** — update this file

---

*Document Version: 0.1.0*
*Language Version: Prima 0.1.0*
*Last Updated: 2026-02-04*
