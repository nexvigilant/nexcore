# AI Guidance — nexcore-immunity

Self-healing code and antipattern detection system.

## Use When
- Implementing new code quality checks or "antibodies."
- Validating incoming code against security or idiomatic standards.
- Automating the fix-reponse loop for common errors (e.g., `unwrap` usage).
- Learning from internal "damage" signals (DAMPs) like test failures.

## Grounding Patterns
- **Threat Sensing (∃)**: Use `ImmunityScanner` to check for the existence of patterns before committing changes.
- **Pattern Match (κ)**: All antibodies must have a clear `CodePattern` grounded in string comparison or regex.
- **T1 Primitives**:
  - `∃ + κ`: Root primitives for detection.
  - `μ + ρ`: Root primitives for the self-healing response loop.

## Maintenance SOPs
- **Antibody Creation**: When a new bug is fixed, ask: "Can we create an antibody for this?" If yes, add a new `Antibody` to the registry.
- **Response Strategy**: Favor `SuggestSafeAlternative` over direct deletion to ensure the human-in-the-loop remains informed.
- **No Unsafe**: Immunity itself must be 100% safe. Strictly enforce the `#![forbid(unsafe_code)]` rule.

## Key Entry Points
- `src/scanner.rs`: The pattern matching engine.
- `src/types.rs`: `Antibody`, `ThreatLevel`, and `PAMP/DAMP` definitions.
- `src/loader.rs`: YAML registry loading logic.
