# Lean 4 Formalization of Theory of Vigilance

This directory contains a Lean 4 formalization of the Theory of Vigilance axioms and key theorems.

## Why Lean 4?

Lean 4 provides:

- **Dependent types**: Types can depend on values, enabling precise specifications
- **Proof assistant**: Machine-checked proofs with full mathematical rigor
- **Mathlib**: Extensive library of formalized mathematics
- **Extraction**: Can extract verified Rust code via Aeneas

## Structure

```
lean4/
├── lakefile.lean     # Lake build configuration
├── ToV/
│   └── Basic.lean    # Core axioms and definitions
└── README.md
```

## Building

```bash
# Install Lean 4 (via elan)
curl https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh -sSf | sh

# Build the project
cd lean4
lake update
lake build
```

## Formalized Content

### Definitions

| Name                     | Type      | Description                           |
| ------------------------ | --------- | ------------------------------------- |
| `VigilanceSystem`        | structure | Core 𝒱 = (S, U, Θ, H_spec)            |
| `StateSpace`             | structure | Measurable state space S              |
| `Constraint`             | def       | g : S × U × Θ → ℝ                     |
| `SafetyManifold`         | def       | M = ⋂ᵢ{s : gᵢ ≤ 0}                    |
| `PropagationProbability` | structure | Pᵢ ∈ (0, 1)                           |
| `HarmType`               | inductive | 8 harm types A-H                      |
| `HarmCharacteristics`    | structure | (Multiplicity, Temporal, Determinism) |

### Axioms

| Axiom                  | Statement                         |
| ---------------------- | --------------------------------- | -------------------- | ---------------------- |
| `axiom1_decomposition` | ∃E, Φ.                            | E                    | < ∞ ∧ Φ : 𝒫(E) ↠ S_acc |
| `axiom2_hierarchy`     | ∃N≤8, {πᵢ}. S ≅ S₁ ∧ Sᵢ₊₁ ≅ Sᵢ/~ᵢ |
| `axiom3_conservation`  | H ⟺ ∃i. gᵢ(s,u,θ) > 0             |
| `axiom4_manifold`      | int(M) ≠ ∅ ∧ H ⟺ s ∉ M            |
| `axiom5_emergence`     | ℙ(H                               | δs₁) = ∏ᵢPᵢ (Markov) |

### Theorems

| Theorem                     | Status   | Description                              |
| --------------------------- | -------- | ---------------------------------------- |
| `harm_exhaustiveness`       | ✓ Proved | classifyHarm is bijective                |
| `attenuation_theorem`       | Stated   | ℙ(H) decreases with depth                |
| `protectiveDepth`           | Defined  | H ≥ 1 + log(1/ε)/α                       |
| `foundation_grounding`      | ✓ Proved | All 10 foundations reach root (0 or 1)   |

### LEMMA 8: Foundation Grounding Completeness (2026-02-04)

**Statement:** Every mathematical foundation in the Lex Primitiva hierarchy reaches at least one root constant (0 or 1).

**Proof:** By case analysis on all 10 foundations. Each foundation's `terminal_constants()` contains 0 or 1:
- PeanoAxioms → {0, 1, ω} ✓ (0, 1)
- MeasureTheory → {0, ∞} ✓ (0)
- OrderTheory → {0, 1} ✓ (0, 1)
- CategoryTheory → {0, 1} ✓ (0, 1)
- SetTheory → {0, 1} ✓ (0, 1)
- Analysis → {0, ε, ∞} ✓ (0)
- SignalTheory → {1, π, e} ✓ (1)
- InformationTheory → {0, ln(2)} ✓ (0)
- Thermodynamics → {0, kᵦ, ln(2)} ✓ (0)
- FixedPointTheory → {1, φ, e} ✓ (1)

**Rust enforcement test:** `nexcore-lex-primitiva/src/graph.rs:349-356`

**Note:** SignalTheory, Thermodynamics, and FixedPointTheory were fixed on 2026-02-04 to include root constants (previously missing).

## Proof Strategy

### Harm Exhaustiveness (Theorem 9.0.1)

The proof that `classifyHarm` is bijective follows from:

1. **Injectivity**: Case analysis on all 8 combinations shows no collisions
2. **Surjectivity**: Each HarmType has a preimage in HarmCharacteristics

```lean
theorem harm_exhaustiveness : Function.Bijective classifyHarm := by
  constructor
  · intro ⟨m₁, t₁, d₁⟩ ⟨m₂, t₂, d₂⟩ h
    cases m₁ <;> cases t₁ <;> cases d₁ <;>
    cases m₂ <;> cases t₂ <;> cases d₂ <;>
    simp [classifyHarm] at h <;> rfl
  · intro ht
    cases ht <;> exact ⟨_, rfl⟩
```

### Attenuation Theorem (T10.2)

Full proof requires:

1. Measure theory for probability spaces
2. Product measure construction
3. Logarithmic transformation lemmas
4. Exponential decay bounds

## Future Work

### Near-term

- [ ] Formalize `ConservationLaw` constraints mathematically
- [ ] Prove attenuation theorem with measure theory
- [ ] Add cross-domain isomorphism proofs

### Long-term

- [ ] Aeneas extraction to verified Rust
- [ ] Integration with Rust codebase via FFI
- [ ] Full Mathlib-based topology for safety manifold

## References

- [Lean 4 Documentation](https://leanprover.github.io/lean4/doc/)
- [Mathlib4](https://github.com/leanprover-community/mathlib4)
- [Aeneas](https://github.com/AeneasVerif/aeneas) - Rust-to-Lean extraction
