# nexcore-skill-verify

Validation engine for the Diamond v2 skill standard. This crate provides a Rust-based framework for verifying that Claude Code skills are structurally correct, properly grounded, and meet all safety and documentation requirements.

## Intent
To enforce a high standard of quality across the skill ecosystem. It replaces the legacy Python-based verification system with a faster, type-safe Rust implementation that can be easily integrated into CI/CD pipelines and the Code Forge.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **κ (Comparison)**: The core primitive for matching skill metadata against the Diamond v2 spec.
- **∂ (Boundary)**: Defines the validation gates (PASS/FAIL) for skill compliance.
- **μ (Mapping)**: Maps YAML frontmatter and file content into structured `VerifyContext`.
- **∃ (Existence)**: Checks for the presence of mandatory files like `SKILL.md`.

## Core Checks
1. **File Integrity**: Verifies `SKILL.md` existence and path validity.
2. **YAML Compliance**: Ensures frontmatter is valid and contains required fields (Intent, Primitives).
3. **Grounding Verification**: Validates that the T1 primitives declared in the skill are actually manifested in the tool logic.
4. **SMST Scoring**: (Structural, Methodical, Semantic, Transferable) — computes the overall quality grade of the skill.

## SOPs for Use
### Verifying a Skill
```rust
use nexcore_skill_verify::{Verifier, VerifyContext};
use std::path::Path;

let ctx = VerifyContext::new(Path::new("~/.claude/skills/my-skill"))?;
let verifier = Verifier::default();
let report = verifier.verify(&ctx)?;

if report.passed() {
    println!("Skill is Diamond v2 compliant.");
} else {
    println!("Validation failed: {}", report.summary());
}
```

### Adding a new Check
1. Implement the `Check` trait in `src/checks/`.
2. Register the check in `Verifier::default()` or the appropriate check-suite.
3. Ensure you provide clear `Remediation` suggestions for failures.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
