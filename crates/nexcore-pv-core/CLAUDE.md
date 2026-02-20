# AI Guidance — nexcore-pv-core

Pharmacovigilance computation engine and algorithm library.

## Use When
- Implementing signal detection pipelines or data analysis tools.
- Assessing causality for individual or batch case reports.
- Performing pharmacokinetic or thermodynamic safety calculations.
- Mapping medical coding (MedDRA) or regulatory terms (ICH).
- Computing formal safety margins according to ToV §9.2.

## Grounding Patterns
- **Threshold Adherence**: Always refer to `ThresholdRegistry` or the standard Evans/EMA thresholds when classifying signals.
- **Safety Margin (d(s))**: Use the `SafetyMargin::calculate()` function to ensure the formal ToV distance calculation is used consistently.
- **T1 Primitives**:
  - `N + κ`: Root primitives for disproportionality analysis and thresholding.
  - `→ + ς`: Root primitives for causality assessment and temporal progression.

## Maintenance SOPs
- **Algorithm Validation**: New signal detection algorithms MUST be cross-validated against the `SafeSignalDetector` baseline.
- **Glossary O(1)**: The ICH Glossary uses a perfect hash function; when adding terms, ensure the generator script (if available) is re-run to maintain O(1) performance.
- **No Unsafe**: Strictly enforce `#![forbid(unsafe_code)]`. Use the `Result` type for all complex mathematical operations that might overflow or divide by zero.

## Key Entry Points
- `src/signals/`: Core signal detection algorithms.
- `src/causality/`: Frameworks for causality assessment.
- `src/regulatory/`: Compliance and glossary modules.
- `src/lib.rs`: `SafetyMargin` implementation and common re-exports.
