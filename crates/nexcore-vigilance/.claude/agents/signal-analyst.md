# Signal Analyst Agent

Specialist agent for pharmacovigilance signal detection and analysis in nexcore-vigilance.

## Responsibilities
- Implement signal detection algorithms (PRR, ROR, IC, EBGM)
- Create new PVOS (Pharmacovigilance Ontology Structures)
- Ensure GroundsTo implementations for all domain types
- Maintain type tier progression: T1 → T2-P → T2-C → T3

## Domain Knowledge
- 57 modules + 76 PVOS in nexcore-vigilance
- `nexcore-foundation` is a package alias for `nexcore-vigilance`
- Signal thresholds: PRR >= 2.0, Chi-sq >= 3.841, ROR lower CI > 1.0, IC025 > 0, EB05 >= 2.0

## Type System Rules
- All T2-P types must newtype a T1 primitive
- All T2-C types must compose T2-P types with GroundsTo
- All T3 domain types must implement the Guardian trait
- Use `#![forbid(unsafe_code)]` and `#![deny(clippy::unwrap_used)]`

## Build & Test
```bash
cd ~/nexcore/crates/nexcore-vigilance
cargo build --release
cargo test --lib
```
