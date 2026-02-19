// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Transform Catalog — the standard library of known compilation transforms.
//!
//! Every familiar tool — lexer, parser, transpiler, code generator, linter,
//! AI assistant — is a named transform in the 7-axis compilation space.
//! This module provides factory functions for constructing them.

use crate::axis::{
    AbstractionLevel, BranchConfig, Dimensionality, EvalState, LanguageId, ReflectionDepth,
    TemporalCoord,
};
use crate::point::CompilationPoint;
use crate::transform::{Transform, TransformChain};

// ═══════════════════════════════════════════════════════════════════
// Axis 1: Abstraction (Vertical) — σ Sequence
// ═══════════════════════════════════════════════════════════════════

/// Lexer: Source → Token (one abstraction step down).
///
/// Tier: T3 (domain transform — σ + μ + ∂)
///
/// Information-preserving and reversible — you can reconstruct source
/// from tokens (whitespace/comment tokens preserved).
pub fn lex(language: LanguageId) -> Transform {
    Transform::new(
        "lex",
        "Tokenize source code into a token stream",
        CompilationPoint::source(language.clone()),
        CompilationPoint::tokens(language),
    )
    .information_preserving()
    .reversible()
}

/// Parser: Token → AST (one abstraction step down).
///
/// Information-preserving (concrete syntax tree) or lossy (abstract syntax tree).
/// Dimensionality changes: Linear → Tree.
pub fn parse(language: LanguageId) -> Transform {
    Transform::new(
        "parse",
        "Parse token stream into abstract syntax tree",
        CompilationPoint::tokens(language.clone()),
        CompilationPoint::ast(language),
    )
    .information_preserving()
    .reversible()
}

/// Lowering: AST → IR (one abstraction step down).
///
/// Generally NOT information-preserving — sugar, names, and structure
/// are desugared/flattened. Dimensionality: Tree → Graph.
pub fn lower(language: LanguageId) -> Transform {
    Transform::new(
        "lower",
        "Lower AST to intermediate representation",
        CompilationPoint::ast(language.clone()),
        CompilationPoint::ir(language),
    )
}

/// Code generation: IR → Binary.
///
/// Not reversible — many IR forms collapse to the same binary.
pub fn codegen(language: LanguageId) -> Transform {
    Transform::new(
        "codegen",
        "Generate binary from intermediate representation",
        CompilationPoint::ir(language),
        CompilationPoint::binary(),
    )
}

/// Full compilation pipeline: Source → Binary (multi-step chain).
///
/// Composes: lex → parse → lower → codegen.
pub fn compile_chain(language: LanguageId) -> TransformChain {
    let mut chain = TransformChain::new();
    chain.push(lex(language.clone()));
    chain.push(parse(language.clone()));
    chain.push(lower(language.clone()));
    chain.push(codegen(language));
    chain
}

/// Decompiler: Binary → Source (reverse direction, lossy).
///
/// Not information-preserving — variable names, comments, structure are lost.
pub fn decompile(target_language: LanguageId) -> Transform {
    Transform::new(
        "decompile",
        "Decompile binary back to (approximate) source code",
        CompilationPoint::binary(),
        CompilationPoint::source(target_language),
    )
}

/// Pretty-printer: Token → Source (reverse of lex).
///
/// Reconstructs formatted source from token stream.
pub fn pretty_print(language: LanguageId) -> Transform {
    Transform::new(
        "pretty_print",
        "Format token stream back into source text",
        CompilationPoint::tokens(language.clone()),
        CompilationPoint::source(language),
    )
    .reversible()
}

/// Disassembler: Binary → IR (reverse of codegen, partial).
pub fn disassemble() -> Transform {
    Transform::new(
        "disassemble",
        "Disassemble binary into IR / assembly",
        CompilationPoint::binary(),
        CompilationPoint::ir(LanguageId::new("assembly")),
    )
}

// ═══════════════════════════════════════════════════════════════════
// Axis 2: Language (Lateral) — μ Mapping
// ═══════════════════════════════════════════════════════════════════

/// Transpiler: Source(A) → Source(B) at the same abstraction level.
///
/// Lateral movement — language changes, abstraction stays.
pub fn transpile(from: LanguageId, to: LanguageId) -> Transform {
    Transform::new(
        "transpile",
        format!("Transpile {} → {}", from.as_str(), to.as_str()),
        CompilationPoint::source(from),
        CompilationPoint::source(to),
    )
}

/// AST-level transpilation: AST(A) → AST(B).
///
/// More precise than source transpilation — operates on structure.
/// Dimensionality stays Tree.
pub fn ast_transpile(from: LanguageId, to: LanguageId) -> Transform {
    Transform::new(
        "ast_transpile",
        format!("Transpile AST: {} → {}", from.as_str(), to.as_str()),
        CompilationPoint::ast(from),
        CompilationPoint::ast(to),
    )
}

/// IR-level cross-compilation: IR(A) → IR(B).
///
/// Example: LLVM IR → WASM IR.
pub fn ir_translate(from: LanguageId, to: LanguageId) -> Transform {
    Transform::new(
        "ir_translate",
        format!("Translate IR: {} → {}", from.as_str(), to.as_str()),
        CompilationPoint::ir(from),
        CompilationPoint::ir(to),
    )
}

// ═══════════════════════════════════════════════════════════════════
// Axis 3: Time (Temporal) — ν Frequency
// ═══════════════════════════════════════════════════════════════════

/// Diff: Source(v1) → Source(v2) — a temporal transform.
///
/// Records what changed between two versions.
pub fn diff(language: LanguageId, from_rev: u64, to_rev: u64) -> Transform {
    Transform::new(
        "diff",
        format!("Diff revision {} → {}", from_rev, to_rev),
        CompilationPoint::source(language.clone()).at_time(TemporalCoord::new(from_rev)),
        CompilationPoint::source(language).at_time(TemporalCoord::new(to_rev)),
    )
    .information_preserving()
    .reversible()
}

/// Patch: Apply a diff to advance a version.
pub fn patch(language: LanguageId, from_rev: u64, to_rev: u64) -> Transform {
    Transform::new(
        "patch",
        format!("Apply patch: revision {} → {}", from_rev, to_rev),
        CompilationPoint::source(language.clone()).at_time(TemporalCoord::new(from_rev)),
        CompilationPoint::source(language).at_time(TemporalCoord::new(to_rev)),
    )
    .reversible()
}

/// Migration: automated code modification across versions.
pub fn migrate(language: LanguageId, from_rev: u64, to_rev: u64) -> Transform {
    Transform::new(
        "migrate",
        format!("Migrate codebase: revision {} → {}", from_rev, to_rev),
        CompilationPoint::source(language.clone()).at_time(TemporalCoord::new(from_rev)),
        CompilationPoint::source(language).at_time(TemporalCoord::new(to_rev)),
    )
}

// ═══════════════════════════════════════════════════════════════════
// Axis 4: Evaluation — ∂ Boundary
// ═══════════════════════════════════════════════════════════════════

/// Const evaluation: Symbolic → Partially Evaluated.
///
/// Resolves `const fn`, `const` generics, known values at compile time.
pub fn const_eval(language: LanguageId) -> Transform {
    Transform::new(
        "const_eval",
        "Evaluate constant expressions at compile time",
        CompilationPoint::source(language.clone()).with_eval(EvalState::Symbolic),
        CompilationPoint::source(language).with_eval(EvalState::Partial),
    )
}

/// Monomorphization: Partial → Concrete (for generics).
///
/// Stamps out concrete instances of generic code.
pub fn monomorphize(language: LanguageId) -> Transform {
    Transform::new(
        "monomorphize",
        "Stamp out concrete instances of generic types/functions",
        CompilationPoint::ir(language.clone()).with_eval(EvalState::Partial),
        CompilationPoint::ir(language).with_eval(EvalState::Concrete),
    )
}

/// Symbolic execution: Concrete → Symbolic (reverse direction).
///
/// Lifts concrete values back to symbolic constraints for analysis.
pub fn symbolic_execute(language: LanguageId) -> Transform {
    Transform::new(
        "symbolic_exec",
        "Execute symbolically for program analysis",
        CompilationPoint::source(language.clone()).with_eval(EvalState::Concrete),
        CompilationPoint::source(language).with_eval(EvalState::Symbolic),
    )
}

// ═══════════════════════════════════════════════════════════════════
// Axis 5: Reflection — ρ Recursion
// ═══════════════════════════════════════════════════════════════════

/// Proc macro expansion: META → GROUND.
///
/// Code-that-writes-code descends one meta-level.
pub fn macro_expand(language: LanguageId) -> Transform {
    Transform::new(
        "macro_expand",
        "Expand procedural macros (meta → ground code)",
        CompilationPoint::source(language.clone()).with_reflection(ReflectionDepth::META),
        CompilationPoint::source(language).with_reflection(ReflectionDepth::GROUND),
    )
}

/// Code generation from schema: META → GROUND.
///
/// Derives, schema codegen, build.rs — all meta-to-ground transforms.
pub fn schema_codegen(language: LanguageId) -> Transform {
    Transform::new(
        "schema_codegen",
        "Generate code from schema/derive attributes",
        CompilationPoint::source(language.clone()).with_reflection(ReflectionDepth::META),
        CompilationPoint::source(language).with_reflection(ReflectionDepth::GROUND),
    )
}

/// Self-modifying code / quine generation: GROUND → META.
///
/// Raises code from ground to meta-level (rare, powerful).
pub fn meta_lift(language: LanguageId) -> Transform {
    Transform::new(
        "meta_lift",
        "Lift code to meta-level (generate code that generates code)",
        CompilationPoint::source(language.clone()).with_reflection(ReflectionDepth::GROUND),
        CompilationPoint::source(language).with_reflection(ReflectionDepth::META),
    )
}

// ═══════════════════════════════════════════════════════════════════
// Axis 6: Projection (Dimensionality) — Σ→σ
// ═══════════════════════════════════════════════════════════════════

/// Serialization: Tree/Graph → Linear (dimension reduction).
///
/// Every serialization format (JSON, protobuf, s-expressions) is a projection.
pub fn serialize(language: LanguageId) -> Transform {
    Transform::new(
        "serialize",
        "Serialize structured data to linear format",
        CompilationPoint::ast(language.clone()),
        CompilationPoint::source(language).with_dimensionality(Dimensionality::Linear),
    )
    .reversible()
}

/// Deserialization: Linear → Tree (dimension expansion).
pub fn deserialize(language: LanguageId) -> Transform {
    Transform::new(
        "deserialize",
        "Parse linear format back to structured tree",
        CompilationPoint::source(language.clone()).with_dimensionality(Dimensionality::Linear),
        CompilationPoint::ast(language),
    )
    .reversible()
}

/// Flattening: Graph → Tree (lose cycles).
pub fn flatten(language: LanguageId) -> Transform {
    Transform::new(
        "flatten",
        "Flatten graph IR to tree representation (lose cycles)",
        CompilationPoint::ir(language.clone()),
        CompilationPoint::ast(language),
    )
}

// ═══════════════════════════════════════════════════════════════════
// Axis 7: Branching — Σ Sum
// ═══════════════════════════════════════════════════════════════════

/// cfg resolution: collapse conditional compilation branches.
///
/// Selects one universe from the superposition.
pub fn cfg_resolve(language: LanguageId, config: BranchConfig) -> Transform {
    Transform::new(
        "cfg_resolve",
        format!(
            "Resolve cfg for target={}",
            config.target.as_deref().unwrap_or("default")
        ),
        CompilationPoint::source(language.clone()),
        CompilationPoint::source(language).with_branch(config),
    )
}

/// Feature gate: add a feature flag to the branch configuration.
pub fn feature_gate(language: LanguageId, feature: &str) -> Transform {
    Transform::new(
        "feature_gate",
        format!("Gate on feature: {feature}"),
        CompilationPoint::source(language.clone()),
        CompilationPoint::source(language).with_branch(BranchConfig::new().with_feature(feature)),
    )
}

// ═══════════════════════════════════════════════════════════════════
// Multi-Axis: AI / Intent Transforms
// ═══════════════════════════════════════════════════════════════════

/// Intent compilation: natural language → source code.
///
/// Traverses Abstraction + Language + potentially Evaluation.
/// The AI-first transform — this IS what LLM coding assistants do.
pub fn intent_compile(target_language: LanguageId) -> Transform {
    Transform::new(
        "intent_compile",
        "Compile human intent into source code (AI-assisted)",
        CompilationPoint::intent(),
        CompilationPoint::source(target_language),
    )
}

/// Intent explanation: source code → natural language.
///
/// The reverse of intent_compile — raises code to human understanding.
pub fn intent_explain(source_language: LanguageId) -> Transform {
    Transform::new(
        "intent_explain",
        "Explain source code in natural language (AI-assisted)",
        CompilationPoint::source(source_language),
        CompilationPoint::intent(),
    )
}

/// Specification extraction: source → formal specification.
///
/// Lifts implementation to specification (types, contracts, invariants).
pub fn spec_extract(language: LanguageId) -> Transform {
    Transform::new(
        "spec_extract",
        "Extract formal specification from implementation",
        CompilationPoint::source(language),
        CompilationPoint::new(AbstractionLevel::Specification, LanguageId::new("spec")),
    )
}

/// Specification compilation: specification → source.
///
/// From formal spec to implementation.
pub fn spec_compile(spec_language: LanguageId, target_language: LanguageId) -> Transform {
    Transform::new(
        "spec_compile",
        "Compile specification to implementation",
        CompilationPoint::new(AbstractionLevel::Specification, spec_language),
        CompilationPoint::source(target_language),
    )
}

// ═══════════════════════════════════════════════════════════════════
// Multi-Axis: Refactoring (same point, better code)
// ═══════════════════════════════════════════════════════════════════

/// Refactor: Source → Source (same point, different code).
///
/// Uniquely, this transform has source == target. The code changes
/// but occupies the same position in the compilation space. This is
/// an identity transform on the space but a non-trivial code mutation.
pub fn refactor(language: LanguageId) -> Transform {
    Transform::new(
        "refactor",
        "Restructure code while preserving behavior",
        CompilationPoint::source(language.clone()),
        CompilationPoint::source(language),
    )
    .information_preserving()
    .reversible()
}

/// Lint/format: Source → Source (cosmetic identity transform).
pub fn lint(language: LanguageId) -> Transform {
    Transform::new(
        "lint",
        "Apply linting and formatting rules",
        CompilationPoint::source(language.clone()),
        CompilationPoint::source(language),
    )
    .information_preserving()
    .reversible()
}

/// Optimization: IR → IR (same level, better code).
pub fn optimize(language: LanguageId) -> Transform {
    Transform::new(
        "optimize",
        "Optimize intermediate representation",
        CompilationPoint::ir(language.clone()),
        CompilationPoint::ir(language),
    )
}

// ═══════════════════════════════════════════════════════════════════
// NexCore-Specific Transforms
// ═══════════════════════════════════════════════════════════════════

/// Prima evaluation: Prima source → PVDSL bytecode.
///
/// NexCore's domain-specific compilation path.
pub fn prima_compile() -> Transform {
    Transform::new(
        "prima_compile",
        "Compile Prima source to PVDSL bytecode",
        CompilationPoint::source(LanguageId::prima()),
        CompilationPoint::new(AbstractionLevel::Binary, LanguageId::pvdsl()),
    )
}

/// PVDSL execution: bytecode → concrete runtime values.
pub fn pvdsl_execute() -> Transform {
    Transform::new(
        "pvdsl_execute",
        "Execute PVDSL bytecode on the VM",
        CompilationPoint::new(AbstractionLevel::Binary, LanguageId::pvdsl()),
        CompilationPoint::new(AbstractionLevel::Execution, LanguageId::pvdsl())
            .with_eval(EvalState::Concrete),
    )
}

/// Intent → Prima: AI compiles human intent into Prima DSL.
pub fn intent_to_prima() -> Transform {
    Transform::new(
        "intent_to_prima",
        "Compile natural language intent into Prima DSL",
        CompilationPoint::intent(),
        CompilationPoint::source(LanguageId::prima()),
    )
}

/// Full NexCore AI pipeline: Intent → Prima → PVDSL → Execution.
pub fn nexcore_ai_pipeline() -> TransformChain {
    let mut chain = TransformChain::new();
    chain.push(intent_to_prima());
    chain.push(prima_compile());
    chain.push(pvdsl_execute());
    chain
}

/// Rust full compilation: Source → Binary.
pub fn rust_compile_chain() -> TransformChain {
    compile_chain(LanguageId::rust())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::Axis;

    // ── Vertical transforms ──

    #[test]
    fn lex_is_vertical() {
        let t = lex(LanguageId::rust());
        assert!(t.is_lowering());
        assert!(t.preserves_information);
        assert!(t.reversible);
    }

    #[test]
    fn parse_is_vertical() {
        let t = parse(LanguageId::rust());
        assert!(t.is_lowering());
        assert!(t.traverses(Axis::Abstraction));
        assert!(t.traverses(Axis::Projection)); // Linear → Tree
    }

    #[test]
    fn lower_changes_dimensionality() {
        let t = lower(LanguageId::rust());
        assert!(t.traverses(Axis::Abstraction));
        assert!(t.traverses(Axis::Projection)); // Tree → Graph
    }

    #[test]
    fn decompile_is_raising() {
        let t = decompile(LanguageId::c_lang());
        assert!(t.is_raising());
        assert!(!t.preserves_information);
    }

    // ── Lateral transforms ──

    #[test]
    fn transpile_is_lateral() {
        let t = transpile(LanguageId::rust(), LanguageId::javascript());
        assert!(t.is_lateral());
        assert!(t.traverses(Axis::Language));
        assert!(!t.traverses(Axis::Abstraction));
    }

    #[test]
    fn ast_transpile_is_lateral() {
        let t = ast_transpile(LanguageId::rust(), LanguageId::c_lang());
        assert!(t.is_lateral());
    }

    // ── Temporal transforms ──

    #[test]
    fn diff_is_temporal() {
        let t = diff(LanguageId::rust(), 1, 2);
        assert!(t.is_temporal());
        assert!(t.traverses(Axis::Time));
        assert!(t.preserves_information);
    }

    #[test]
    fn patch_is_temporal() {
        let t = patch(LanguageId::rust(), 5, 6);
        assert!(t.is_temporal());
        assert!(t.reversible);
    }

    // ── Evaluation transforms ──

    #[test]
    fn const_eval_changes_eval_state() {
        let t = const_eval(LanguageId::rust());
        assert!(t.traverses(Axis::Evaluation));
    }

    #[test]
    fn monomorphize_changes_eval_state() {
        let t = monomorphize(LanguageId::rust());
        assert!(t.traverses(Axis::Evaluation));
    }

    // ── Reflection transforms ──

    #[test]
    fn macro_expand_changes_reflection() {
        let t = macro_expand(LanguageId::rust());
        assert!(t.traverses(Axis::Reflection));
    }

    #[test]
    fn meta_lift_raises_reflection() {
        let t = meta_lift(LanguageId::rust());
        assert!(t.traverses(Axis::Reflection));
    }

    // ── Projection transforms ──

    #[test]
    fn serialize_changes_projection() {
        let t = serialize(LanguageId::new("json"));
        assert!(t.traverses(Axis::Projection));
        assert!(t.reversible);
    }

    // ── Branching transforms ──

    #[test]
    fn cfg_resolve_changes_branch() {
        let config = BranchConfig::new()
            .with_target("aarch64-linux-gnu")
            .with_feature("serde");
        let t = cfg_resolve(LanguageId::rust(), config);
        assert!(t.traverses(Axis::Branching));
    }

    // ── AI / Intent transforms ──

    #[test]
    fn intent_compile_multi_axis() {
        let t = intent_compile(LanguageId::rust());
        assert!(t.traverses(Axis::Abstraction));
        assert!(t.traverses(Axis::Language));
        assert!(t.is_lowering());
    }

    #[test]
    fn intent_explain_is_raising() {
        let t = intent_explain(LanguageId::rust());
        assert!(t.is_raising());
        assert!(t.traverses(Axis::Language));
    }

    // ── Refactoring transforms ──

    #[test]
    fn refactor_is_identity_in_space() {
        let t = refactor(LanguageId::rust());
        assert_eq!(t.axis_count(), 0);
        assert!(t.preserves_information);
    }

    #[test]
    fn lint_is_identity() {
        let t = lint(LanguageId::rust());
        assert_eq!(t.axis_count(), 0);
    }

    // ── Chain tests ──

    #[test]
    fn compile_chain_connected() {
        let chain = compile_chain(LanguageId::rust());
        assert_eq!(chain.len(), 4);
        assert!(chain.validate().is_empty());
    }

    #[test]
    fn compile_chain_source_to_binary() {
        let chain = compile_chain(LanguageId::rust());
        assert_eq!(
            chain.start().map(|p| p.abstraction),
            Some(AbstractionLevel::Source)
        );
        assert_eq!(
            chain.end().map(|p| p.abstraction),
            Some(AbstractionLevel::Binary)
        );
    }

    #[test]
    fn nexcore_ai_pipeline_connected() {
        let chain = nexcore_ai_pipeline();
        assert_eq!(chain.len(), 3);
        // Intent → ... → Execution
        assert_eq!(
            chain.start().map(|p| p.abstraction),
            Some(AbstractionLevel::Intent)
        );
        assert_eq!(
            chain.end().map(|p| p.abstraction),
            Some(AbstractionLevel::Execution)
        );
    }

    #[test]
    fn rust_compile_chain_is_compile_chain() {
        let chain = rust_compile_chain();
        assert_eq!(chain.len(), 4);
        assert!(chain.validate().is_empty());
    }

    // ── NexCore-specific transforms ──

    #[test]
    fn prima_compile_languages() {
        let t = prima_compile();
        assert_eq!(t.source.language, LanguageId::prima());
        assert_eq!(t.target.language, LanguageId::pvdsl());
    }

    #[test]
    fn intent_to_prima_axes() {
        let t = intent_to_prima();
        assert!(t.traverses(Axis::Abstraction));
        assert!(t.traverses(Axis::Language));
    }
}
