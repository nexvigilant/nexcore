// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Gemini Forge Harness
//!
//! API harness that connects Gemini to the primitive forge engine.
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol |
//! |---------|-----------|--------|
//! | Forge Loop | Recursion | ρ |
//! | Primitive Extraction | Mapping | μ |
//! | Code Generation | Causality | → |
//! | Validation Gate | Boundary | ∂ |
//! | Technology Output | Sum | Σ |
//!
//! ## Forge Protocol
//!
//! ```text
//! MINE → DECOMPOSE → GENERATE → VALIDATE → REFINE
//!   ↑                                        │
//!   └────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The 15 Lex Primitiva symbols
pub const LEX_PRIMITIVA: &[(&str, &str, &str)] = &[
    ("σ", "Sequence", "Ordered collection"),
    ("μ", "Mapping", "Key→Value association"),
    ("ς", "State", "Mutable container"),
    ("ρ", "Recursion", "Self-reference"),
    ("∅", "Void", "Absence/null"),
    ("∂", "Boundary", "Condition/edge"),
    ("ν", "Frequency", "Count/occurrence"),
    ("∃", "Existence", "Present/defined"),
    ("π", "Persistence", "Storage/I/O"),
    ("→", "Causality", "Transformation"),
    ("κ", "Comparison", "Equality/ordering"),
    ("N", "Quantity", "Numeric value"),
    ("λ", "Location", "Position/address"),
    ("∝", "Irreversibility", "One-way/entropy"),
    ("Σ", "Sum", "Aggregation"),
];

/// Tier classification based on primitive count
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Tier {
    T1,  // 1 primitive - 100% transfer
    T2P, // 2-3 primitives - 90% transfer
    T2C, // 4-5 primitives - 70% transfer
    T3,  // 6+ primitives - 40% transfer
}

impl Tier {
    pub fn from_count(count: usize) -> Self {
        match count {
            0 | 1 => Tier::T1,
            2 | 3 => Tier::T2P,
            4 | 5 => Tier::T2C,
            _ => Tier::T3,
        }
    }

    pub fn transfer_confidence(&self) -> f64 {
        match self {
            Tier::T1 => 1.0,
            Tier::T2P => 0.9,
            Tier::T2C => 0.7,
            Tier::T3 => 0.4,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Tier::T1 => "T1",
            Tier::T2P => "T2-P",
            Tier::T2C => "T2-C",
            Tier::T3 => "T3",
        }
    }
}

/// A primitive extraction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveExtraction {
    pub concept: String,
    pub primitives: Vec<String>,
    pub tier: Tier,
    pub transfer_confidence: f64,
    pub decomposition: String,
    pub rust_manifestation: Option<String>,
}

/// Forge task specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeTask {
    pub name: String,
    pub description: String,
    pub domain: String,
    pub target_tier: Option<Tier>,
}

/// Forge output - generated technology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeOutput {
    pub task: ForgeTask,
    pub primitives_mined: Vec<PrimitiveExtraction>,
    pub rust_code: String,
    pub validation_status: ValidationStatus,
    pub refinement_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    Pending,
    Compiles,
    TestsPass,
    ClippyClean,
    Production,
    Failed(String),
}

/// The Forge Harness API
#[derive(Debug)]
pub struct ForgeHarness {
    pub session_id: String,
    pub mined_primitives: Vec<PrimitiveExtraction>,
    pub generated_code: HashMap<String, String>,
}

impl ForgeHarness {
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            mined_primitives: Vec::new(),
            generated_code: HashMap::new(),
        }
    }

    /// Get the primitive reference card (for Gemini context)
    pub fn primitive_reference() -> String {
        let mut output = String::new();
        output.push_str("# The 15 Lex Primitiva\n\n");
        output.push_str("| Symbol | Name | Description |\n");
        output.push_str("|--------|------|-------------|\n");
        for (symbol, name, desc) in LEX_PRIMITIVA {
            output.push_str(&format!("| {} | {} | {} |\n", symbol, name, desc));
        }
        output.push_str("\n## Tier Classification\n\n");
        output.push_str("- **T1** (1 primitive): 100% transfer confidence\n");
        output.push_str("- **T2-P** (2-3 primitives): 90% transfer confidence\n");
        output.push_str("- **T2-C** (4-5 primitives): 70% transfer confidence\n");
        output.push_str("- **T3** (6+ primitives): 40% transfer confidence\n");
        output
    }

    /// Generate the forge prompt for Gemini
    pub fn forge_prompt(task: &ForgeTask) -> String {
        format!(
            r#"# FORGE TASK: {}

## Description
{}

## Domain
{}

## Forge Protocol

You are the FORGE - an autonomous technology constructor.

### Phase 1: MINE
Extract the irreducible T1/T2 primitives from the task requirements.
Use only the 15 Lex Primitiva symbols: σ μ ς ρ ∅ ∂ ν ∃ π → κ N λ ∝ Σ

### Phase 2: DECOMPOSE
For each concept in the task, identify:
- Which primitives compose it
- The tier (T1/T2-P/T2-C/T3)
- The Rust type that manifests this composition

### Phase 3: GENERATE
Write idiomatic Rust code that:
- Grounds every type to its primitives via GroundsTo trait
- Uses newtypes for domain values (no raw u32 for IDs)
- Forbids unsafe, unwrap, expect, panic in production paths
- Follows Edition 2024 patterns

### Phase 4: VALIDATE
The code must:
- `cargo check` - compiles
- `cargo clippy -- -D warnings` - no warnings
- `cargo test` - all tests pass

### Phase 5: REFINE
If validation fails, analyze the error and fix.
Loop back to GENERATE until VALIDATE passes.

## Output Format

```rust
// FORGE OUTPUT: [task name]
// Primitives: [list of symbols]
// Tier: [T1/T2-P/T2-C/T3]

[Rust code here]
```

## Begin Forging

Task: {}
"#,
            task.name, task.description, task.domain, task.name
        )
    }

    /// Mine primitives from a concept
    pub fn mine(
        &mut self,
        concept: &str,
        primitives: Vec<&str>,
        decomposition: &str,
    ) -> PrimitiveExtraction {
        let tier = Tier::from_count(primitives.len());
        let extraction = PrimitiveExtraction {
            concept: concept.to_string(),
            primitives: primitives.into_iter().map(String::from).collect(),
            tier,
            transfer_confidence: tier.transfer_confidence(),
            decomposition: decomposition.to_string(),
            rust_manifestation: None,
        };
        self.mined_primitives.push(extraction.clone());
        extraction
    }

    /// Store generated code
    pub fn store_code(&mut self, name: &str, code: &str) {
        self.generated_code
            .insert(name.to_string(), code.to_string());
    }

    /// Get session summary
    pub fn summary(&self) -> String {
        let mut output = format!("# Forge Session: {}\n\n", self.session_id);

        output.push_str("## Primitives Mined\n\n");
        for ext in &self.mined_primitives {
            output.push_str(&format!(
                "- **{}**: [{}] → {} ({})\n",
                ext.concept,
                ext.primitives.join(", "),
                ext.tier.code(),
                ext.transfer_confidence
            ));
        }

        output.push_str("\n## Code Generated\n\n");
        for name in self.generated_code.keys() {
            output.push_str(&format!("- {}\n", name));
        }

        output
    }
}

/// Create the system prompt for Gemini to act as Forge
pub fn gemini_forge_system_prompt() -> String {
    format!(
        r#"You are FORGE, an autonomous Rust technology constructor.

{}

## Your Protocol

1. **MINE**: Extract T1/T2 primitives from requirements
2. **DECOMPOSE**: Map concepts to primitive compositions
3. **GENERATE**: Write grounded Rust code
4. **VALIDATE**: Ensure it compiles and passes tests
5. **REFINE**: Fix errors, loop until valid

## Action Format

To execute forge operations, use:

[ACTION: forge_mine]
{{
  "concept": "name",
  "primitives": ["σ", "μ"],
  "decomposition": "explanation"
}}
[/ACTION]

[ACTION: forge_generate]
{{
  "name": "module_name",
  "code": "rust code here"
}}
[/ACTION]

[ACTION: forge_validate]
{{
  "crate": "crate_name"
}}
[/ACTION]

[ACTION: shell]
cargo check
[/ACTION]

## Remember

- Every type grounds to {{0, 1}} through the 15 primitives
- No unsafe, unwrap, expect, panic in production paths
- Prefer T2-P (high transfer) over T3 (domain-locked)
- The compiler is your verification oracle
"#,
        ForgeHarness::primitive_reference()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_classification() {
        assert_eq!(Tier::from_count(1), Tier::T1);
        assert_eq!(Tier::from_count(2), Tier::T2P);
        assert_eq!(Tier::from_count(4), Tier::T2C);
        assert_eq!(Tier::from_count(7), Tier::T3);
    }

    #[test]
    fn test_transfer_confidence() {
        assert!((Tier::T1.transfer_confidence() - 1.0).abs() < f64::EPSILON);
        assert!((Tier::T2P.transfer_confidence() - 0.9).abs() < f64::EPSILON);
        assert!((Tier::T2C.transfer_confidence() - 0.7).abs() < f64::EPSILON);
        assert!((Tier::T3.transfer_confidence() - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn test_harness_mine() {
        let mut harness = ForgeHarness::new("test-session");
        let ext = harness.mine("Parser", vec!["σ", "μ", "→"], "Input → AST transformation");

        assert_eq!(ext.concept, "Parser");
        assert_eq!(ext.tier, Tier::T2P);
        assert_eq!(harness.mined_primitives.len(), 1);
    }

    #[test]
    fn test_primitive_reference() {
        let ref_card = ForgeHarness::primitive_reference();
        assert!(ref_card.contains("σ"));
        assert!(ref_card.contains("Sequence"));
        assert!(ref_card.contains("T2-P"));
    }
}
