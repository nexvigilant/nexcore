# AI Guidance — nexcore-tov-grounded

Concrete implementation of Theory of Vigilance (ToV) primitives.

## Use When
- Implementing runtime safety checks or signal calculations.
- Using standardized units like `Bits` or `QuantityUnit`.
- Implementing system responses via the `Actuator` trait.
- Verifying architectural stability against Complexity Magic Numbers.

## Grounding Patterns
- **Standard Equation**: Always use `SignalStrengthS::calculate()` for signal magnitude.
- **Unit Safety**: Use `UnitId` to ensure properties are not summed across incompatible dimensions (e.g., Mass + Time).
- **T1 Primitives**:
  - `N + κ`: Root primitives for scalar measurement and stability comparisons.
  - `ς + →`: Root primitives for state management and causal response.

## Maintenance SOPs
- **Clamping**: Ensure all factor-based types (Recognition, Temporal) are clamped between `0.0` and `1.0`.
- **Commandment X**: Every new computed type MUST carry its confidence using the `Measured<T>` wrapper.
- **No Unsafe**: Strictly enforce `#![forbid(unsafe_code)]`.

## Key Entry Points
- `src/lib.rs`: The entire primitive catalog and implementation logic.
- `src/grounding.rs`: T1 Lex Primitiva mapping.
