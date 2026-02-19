# Verus Specifications for Theory of Vigilance

This directory contains formal Verus specifications for key ToV theorems.

## What is Verus?

[Verus](https://github.com/verus-lang/verus) is a verification toolchain for Rust that allows writing proofs and specifications in Rust syntax. Unlike testing (which checks specific cases), Verus uses SMT solvers to mathematically prove properties hold for ALL possible inputs.

## Files

| File                     | Theorem           | Status    |
| ------------------------ | ----------------- | --------- |
| `attenuation_theorem.rs` | T10.2 Attenuation | Specified |

## Running Verus Proofs

### Installation

```bash
# Clone Verus
git clone https://github.com/verus-lang/verus.git
cd verus

# Get Z3 SMT solver
./tools/get-z3.sh

# Build Verus
source ./tools/activate
```

### Verification

```bash
# Verify a file
verus attenuation_theorem.rs

# With verbose output
verus --verbose attenuation_theorem.rs
```

## Standard Rust Fallback

Each Verus file includes a `standard_rust` module that compiles with regular `rustc`. This provides:

- Documentation of the proof structure
- Runnable tests for the theorem statements
- IDE support without Verus toolchain

To run the standard Rust tests:

```bash
cd /home/matthew/ToV/rust-proof
cargo test --test verus_standard
```

## Theorem Coverage

### T10.2 Attenuation Theorem

**Statement**: Under Markov assumption, ℙ(H|δs₁) = e^{-α(H-1)}

**Proof Structure**:

1. `product_formula` - ℙ(H) = ∏ᵢPᵢ (from Axiom 5)
2. `log_representation` - log(ℙ(H)) = Σlog(Pᵢ)
3. `attenuation_uniform_bound` - ℙ(H) ≤ P_max^{H-1}
4. `attenuation_exponential` - ℙ(H) = e^{-α(H-1)}
5. `attenuation_monotonic` - ∂ℙ/∂H < 0
6. `alpha_positive` - α > 0 when all Pᵢ < 1

**Corollaries**:

- `protective_depth` - H ≥ 1 + log(1/ε)/α for target probability ε
- `buffering_monotonicity` - ∂P/∂b ≤ 0 (T10.3 P3)

## Future Work

Additional theorems to specify:

- T10.1 Predictability (Kolmogorov backward equation)
- T10.3 Intervention (full monotonicity properties)
- T10.4 Conservation (diagnostic completeness)
- T10.5 Manifold Equivalence
- LEMMA 8: Foundation Grounding Completeness (proven in Rust 2026-02-04)

## Related Proofs (Rust-Verified)

| Theorem | Location | Status |
|---------|----------|--------|
| LEMMA 8 (Foundation Grounding) | `nexcore-lex-primitiva/src/graph.rs:349-356` | ✓ Proved |

**LEMMA 8:** All 10 mathematical foundations reach at least one root constant (0 or 1). Fixed 2026-02-04 by adding root constants to SignalTheory (1), Thermodynamics (0), FixedPointTheory (1).
