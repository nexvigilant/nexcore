# AI Guidance — nexcore-skill-verify

Validation engine for Claude Code skill compliance.

## Use When
- Creating or updating a skill in `~/.claude/skills/`.
- Verifying "Stage 2" (Efficacy) in a `crate_xray` audit.
- Enforcing the Diamond v2 standard for new agents or tools.
- Automating quality feedback during the Code Forge process.

## Grounding Patterns
- **Context Awareness**: Always use `VerifyContext` to access skill files; do not use raw `std::fs` calls to ensure paths are resolved correctly relative to the skill root.
- **Fail Early**: Gating checks (like `file_exists`) should run before complex semantic checks.
- **T1 Primitives**:
  - `κ + ∂`: Root primitives for the comparison-threshold validation.
  - `μ + ∃`: Root primitives for metadata extraction and presence checks.

## Maintenance SOPs
- **Check Granularity**: Each `Check` should focus on exactly one compliance rule to provide granular error reporting.
- **Remediation**: Every `CheckResult::failed()` MUST include a `suggestion` field that tells the agent or user exactly how to fix the issue.
- **No Side Effects**: Verification must be read-only. Never modify a skill file during a `verify()` run.

## Key Entry Points
- `src/verifier.rs`: The main execution loop for running a suite of checks.
- `src/checks/`: Built-in compliance logic (YAML, Grounding, SMST).
- `src/result.rs`: `CheckResult` and `Outcome` definitions.
