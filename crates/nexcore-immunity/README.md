# nexcore-immunity

Self-healing code system for the NexVigilant Core kernel. It uses a biological analogy (antibodies, PAMPs, DAMPs) to detect code antipatterns, apply automated corrections, and prevent regression through a persistent registry of learned threats.

## Intent
To enforce high-quality, panic-free, and idiomatically correct Rust code across the workspace. It enables the system to "learn" from its own failures by creating antibodies for specific antipatterns detected during development.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **∃ (Existence)**: The core primitive for sensing the presence of a threat or pattern.
- **κ (Comparison)**: Validates incoming code against known antibody patterns.
- **μ (Mapping)**: Maps a detected threat to an automated fix transformation.
- **π (Persistence)**: Durable storage of the antibody registry in `~/.claude/immunity/antibodies.yaml`.
- **ρ (Recursion)**: The homeostatic SENSE → DECIDE → RESPOND → LEARN loop.

## Threat Classification
- **PAMPs (Pathogen-Associated Molecular Patterns)**: External threats coming from user input, untrusted templates, or outdated dependency patterns.
- **DAMPs (Damage-Associated Molecular Patterns)**: Internal damage signals such as compilation errors, test failures, or structural "smells."

## SOPs for Use
### Scanning for Threats
```rust
use nexcore_immunity::{load_default_registry, ImmunityScanner};

let registry = load_default_registry()?;
let scanner = ImmunityScanner::new(&registry)?;
let result = scanner.scan("let x = foo.unwrap();", Some("lib.rs"));

if !result.clean {
    // Handle detected threats
}
```

### Adding a New Antibody
1. Define the pattern in the antibody registry (`antibodies.yaml`).
2. Specify the `ThreatType` (PAMP/DAMP) and `Severity`.
3. Provide a `ResponseStrategy` (e.g., `SuggestSafeAlternative`).

## Key Components
- **ImmunityScanner**: The primary engine for pattern matching and threat detection.
- **AntibodyRegistry**: Persistent collection of learned code patterns and their corresponding responses.
- **AutoimmuneDiagnosis**: Tools for detecting when immunity rules are conflicting or overly aggressive.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
