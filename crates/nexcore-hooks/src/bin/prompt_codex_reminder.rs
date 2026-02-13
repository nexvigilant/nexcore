//! Primitive Codex Reminder Hook
//! Injects the Twelve Laws reminder at session start for context binding.

use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap_or_default();

    // Only inject on first prompt (check if this is early in session)
    // This is a UserPromptSubmit hook that adds context

    let reminder = r#"
📜 **PRIMITIVE CODEX BINDING** ────────────────────────────

Before proceeding, confirm:

**1. The Twelve Laws** (single words):
   Irreducibility, Composability, Universality, Testability,
   Provenance, Minimality, Independence, Stratification,
   Bidirectionality, Confidence, Documentation, Transfer

**2. The Four Tiers:**
   • **T1**: Universal primitives (sequence, mapping, recursion, state, void)
   • **T2-P**: Cross-domain primitives (reusable across 2+ domains)
   • **T2-C**: Cross-domain composites (built from T2-P primitives)
   • **T3**: Domain-specific (only meaningful in one domain)

**3. The Three Unfixable Limits:**
   • Observation changes the observed (Heisenberg)
   • Self-reference creates paradox (Gödel)
   • Compression has irreducible loss (Shannon)

You are bound. Proceed.
────────────────────────────────────────────────────────────
"#;

    println!("{}", reminder);
}
