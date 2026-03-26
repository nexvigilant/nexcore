//! Compilation Space MCP tools — 7D transform algebra.
//!
//! Pure-function wrappers for nexcore-compilation-space: compilation points,
//! transform catalog, chain validation, axes, and spatial distance.

use nexcore_compilation_space::axis::{
    AbstractionLevel, Axis, BranchConfig, Dimensionality, EvalState, LanguageId, ReflectionDepth,
    TemporalCoord,
};
use nexcore_compilation_space::point::CompilationPoint;
use nexcore_compilation_space::transform::TransformChain;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::compilation_space::{
    CompilationAbstractionLevelsParams, CompilationAxesCatalogParams,
    CompilationCatalogLookupParams, CompilationChainPresetsParams, CompilationChainValidateParams,
    CompilationDistanceParams, CompilationPointCompareParams, CompilationPointInput,
    CompilationPointPresetsParams, CompilationPointSummaryParams,
};

// ── Helpers ──────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

fn parse_abstraction(s: &str) -> Option<AbstractionLevel> {
    match s.to_lowercase().trim() {
        "execution" => Some(AbstractionLevel::Execution),
        "binary" => Some(AbstractionLevel::Binary),
        "ir" => Some(AbstractionLevel::Ir),
        "ast" => Some(AbstractionLevel::Ast),
        "token" | "tokens" => Some(AbstractionLevel::Token),
        "source" => Some(AbstractionLevel::Source),
        "spec" | "specification" => Some(AbstractionLevel::Specification),
        "intent" => Some(AbstractionLevel::Intent),
        _ => None,
    }
}

fn parse_eval_state(s: &str) -> Option<EvalState> {
    match s.to_lowercase().trim() {
        "symbolic" => Some(EvalState::Symbolic),
        "partial" => Some(EvalState::Partial),
        "concrete" => Some(EvalState::Concrete),
        _ => None,
    }
}

fn parse_dimensionality(s: &str) -> Option<Dimensionality> {
    match s.to_lowercase().trim() {
        "scalar" => Some(Dimensionality::Scalar),
        "linear" => Some(Dimensionality::Linear),
        "tree" => Some(Dimensionality::Tree),
        "graph" => Some(Dimensionality::Graph),
        _ => None,
    }
}

fn resolve_language(s: &str) -> LanguageId {
    match s.to_lowercase().trim() {
        "rust" => LanguageId::rust(),
        "prima" => LanguageId::prima(),
        "pvdsl" => LanguageId::pvdsl(),
        "javascript" | "js" => LanguageId::javascript(),
        "wasm" => LanguageId::wasm(),
        "sql" => LanguageId::sql(),
        "c" => LanguageId::c_lang(),
        "llvm" | "llvm_ir" => LanguageId::llvm_ir(),
        "natural" | "english" => LanguageId::natural(),
        other => LanguageId::new(other),
    }
}

fn build_point(input: &CompilationPointInput) -> Result<CompilationPoint, nexcore_error::NexError> {
    let abstraction = parse_abstraction(&input.abstraction).ok_or_else(|| {
        nexcore_error::nexerror!("abstraction must be execution, binary, ir, ast, token, source, specification, or intent")
    })?;
    let language = resolve_language(&input.language);

    let mut point = CompilationPoint::new(abstraction, language);

    if let Some(rev) = input.revision {
        point = point.at_time(TemporalCoord::new(rev));
    }
    if let Some(ref eval) = input.eval_state {
        let es = parse_eval_state(eval).ok_or_else(|| {
            nexcore_error::nexerror!("eval_state must be symbolic, partial, or concrete")
        })?;
        point = point.with_eval(es);
    }
    if let Some(depth) = input.reflection_depth {
        point = point.with_reflection(ReflectionDepth(depth));
    }
    if let Some(ref dim) = input.dimensionality {
        let d = parse_dimensionality(dim).ok_or_else(|| {
            nexcore_error::nexerror!("dimensionality must be scalar, linear, tree, or graph")
        })?;
        point = point.with_dimensionality(d);
    }

    Ok(point)
}

fn resolve_catalog_transform(
    p: &CompilationCatalogLookupParams,
) -> Result<nexcore_compilation_space::transform::Transform, nexcore_error::NexError> {
    use nexcore_compilation_space::catalog;

    let lang = || resolve_language(p.language.as_deref().unwrap_or("rust"));
    let target = || resolve_language(p.target_language.as_deref().unwrap_or("rust"));
    let from_rev = p.from_rev.unwrap_or(0);
    let to_rev = p.to_rev.unwrap_or(1);

    match p.transform.to_lowercase().trim() {
        // Vertical
        "lex" => Ok(catalog::lex(lang())),
        "parse" => Ok(catalog::parse(lang())),
        "lower" => Ok(catalog::lower(lang())),
        "codegen" => Ok(catalog::codegen(lang())),
        "decompile" => Ok(catalog::decompile(lang())),
        "pretty_print" => Ok(catalog::pretty_print(lang())),
        "disassemble" => Ok(catalog::disassemble()),
        // Lateral
        "transpile" => Ok(catalog::transpile(lang(), target())),
        "ast_transpile" => Ok(catalog::ast_transpile(lang(), target())),
        "ir_translate" => Ok(catalog::ir_translate(lang(), target())),
        // Temporal
        "diff" => Ok(catalog::diff(lang(), from_rev, to_rev)),
        "patch" => Ok(catalog::patch(lang(), from_rev, to_rev)),
        "migrate" => Ok(catalog::migrate(lang(), from_rev, to_rev)),
        // Evaluation
        "const_eval" => Ok(catalog::const_eval(lang())),
        "monomorphize" => Ok(catalog::monomorphize(lang())),
        "symbolic_execute" => Ok(catalog::symbolic_execute(lang())),
        // Reflection
        "macro_expand" => Ok(catalog::macro_expand(lang())),
        "schema_codegen" => Ok(catalog::schema_codegen(lang())),
        "meta_lift" => Ok(catalog::meta_lift(lang())),
        // Projection
        "serialize" => Ok(catalog::serialize(lang())),
        "deserialize" => Ok(catalog::deserialize(lang())),
        "flatten" => Ok(catalog::flatten(lang())),
        // Branching
        "cfg_resolve" => {
            let config = BranchConfig::new();
            Ok(catalog::cfg_resolve(lang(), config))
        }
        "feature_gate" => {
            let feature = p.feature.as_deref().unwrap_or("default");
            Ok(catalog::feature_gate(lang(), feature))
        }
        // AI / Intent
        "intent_compile" => Ok(catalog::intent_compile(lang())),
        "intent_explain" => Ok(catalog::intent_explain(lang())),
        "spec_extract" => Ok(catalog::spec_extract(lang())),
        "spec_compile" => Ok(catalog::spec_compile(lang(), target())),
        // Refactoring
        "refactor" => Ok(catalog::refactor(lang())),
        "lint" => Ok(catalog::lint(lang())),
        "optimize" => Ok(catalog::optimize(lang())),
        // NexCore-specific
        "prima_compile" => Ok(catalog::prima_compile()),
        "pvdsl_execute" => Ok(catalog::pvdsl_execute()),
        "intent_to_prima" => Ok(catalog::intent_to_prima()),
        _ => Err(nexcore_error::nexerror!(
            "unknown transform '{}'",
            p.transform
        )),
    }
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Compare two compilation points: differing axes and distance.
pub fn compilation_point_compare(
    p: CompilationPointCompareParams,
) -> Result<CallToolResult, McpError> {
    let a = match build_point(&p.a) {
        Ok(pt) => pt,
        Err(e) => return err_result(&e.to_string()),
    };
    let b = match build_point(&p.b) {
        Ok(pt) => pt,
        Err(e) => return err_result(&e.to_string()),
    };

    let differing = a.differing_axes(&b);
    let axes: Vec<&str> = differing.iter().map(|ax| ax.label()).collect();

    ok_json(json!({
        "differing_axes": axes,
        "axis_distance": a.axis_distance(&b),
        "same_position": a.same_position(&b),
        "a_summary": a.summary(),
        "b_summary": b.summary(),
    }))
}

/// Get a human-readable summary of a compilation point.
pub fn compilation_point_summary(
    p: CompilationPointSummaryParams,
) -> Result<CallToolResult, McpError> {
    let point = match build_point(&p.point) {
        Ok(pt) => pt,
        Err(e) => return err_result(&e.to_string()),
    };
    ok_json(json!({
        "summary": point.summary(),
        "point": serde_json::to_value(&point).unwrap_or_default(),
    }))
}

/// List preset compilation points.
pub fn compilation_point_presets(
    _p: CompilationPointPresetsParams,
) -> Result<CallToolResult, McpError> {
    let presets = [
        ("intent", CompilationPoint::intent()),
        ("source_rust", CompilationPoint::source(LanguageId::rust())),
        ("tokens_rust", CompilationPoint::tokens(LanguageId::rust())),
        ("ast_rust", CompilationPoint::ast(LanguageId::rust())),
        ("ir_rust", CompilationPoint::ir(LanguageId::rust())),
        ("binary", CompilationPoint::binary()),
        ("execution", CompilationPoint::execution()),
    ];

    let items: Vec<serde_json::Value> = presets
        .iter()
        .map(|(name, pt)| {
            json!({
                "name": name,
                "summary": pt.summary(),
            })
        })
        .collect();

    ok_json(json!({ "presets": items }))
}

/// Look up a named transform from the catalog and return its analysis.
pub fn compilation_catalog_lookup(
    p: CompilationCatalogLookupParams,
) -> Result<CallToolResult, McpError> {
    let transform = match resolve_catalog_transform(&p) {
        Ok(t) => t,
        Err(e) => return err_result(&e.to_string()),
    };

    let axes: Vec<&str> = transform
        .axes_traversed()
        .iter()
        .map(|a| a.label())
        .collect();
    let direction = transform.direction();

    ok_json(json!({
        "name": transform.name,
        "description": transform.description,
        "direction": format!("{direction:?}"),
        "axes_traversed": axes,
        "axis_count": transform.axis_count(),
        "is_vertical": transform.is_vertical(),
        "is_lateral": transform.is_lateral(),
        "is_temporal": transform.is_temporal(),
        "is_lowering": transform.is_lowering(),
        "is_raising": transform.is_raising(),
        "preserves_information": transform.preserves_information,
        "reversible": transform.reversible,
        "source_summary": transform.source.summary(),
        "target_summary": transform.target.summary(),
    }))
}

/// Validate a transform chain for step connectivity.
pub fn compilation_chain_validate(
    p: CompilationChainValidateParams,
) -> Result<CallToolResult, McpError> {
    let mut chain = TransformChain::new();

    for step in &p.transforms {
        let transform = match resolve_catalog_transform(step) {
            Ok(t) => t,
            Err(e) => return err_result(&format!("step '{}': {}", step.transform, e)),
        };
        chain.push(transform);
    }

    let errors = chain.validate();
    let axes: Vec<&str> = chain.total_axes().iter().map(|a| a.label()).collect();

    ok_json(json!({
        "steps": chain.len(),
        "total_axes": axes,
        "total_axis_count": chain.total_axis_count(),
        "is_round_trip": chain.is_round_trip(),
        "preserves_information": chain.preserves_information(),
        "is_reversible": chain.is_reversible(),
        "errors": errors.iter().map(|e| json!({
            "step_index": e.step_index,
            "message": e.message,
        })).collect::<Vec<_>>(),
        "valid": errors.is_empty(),
        "start": chain.start().map(|p| p.summary()),
        "end": chain.end().map(|p| p.summary()),
    }))
}

/// Get a preset transform chain.
pub fn compilation_chain_presets(
    p: CompilationChainPresetsParams,
) -> Result<CallToolResult, McpError> {
    use nexcore_compilation_space::catalog;

    let chain = match p.chain.to_lowercase().trim() {
        "compile" => {
            let lang = resolve_language(p.language.as_deref().unwrap_or("rust"));
            catalog::compile_chain(lang)
        }
        "nexcore_ai" | "ai" => catalog::nexcore_ai_pipeline(),
        "rust_compile" | "rust" => catalog::rust_compile_chain(),
        _ => return err_result("chain must be 'compile', 'nexcore_ai', or 'rust_compile'"),
    };

    let axes: Vec<&str> = chain.total_axes().iter().map(|a| a.label()).collect();

    ok_json(json!({
        "steps": chain.len(),
        "total_axes": axes,
        "preserves_information": chain.preserves_information(),
        "is_reversible": chain.is_reversible(),
        "start": chain.start().map(|p| p.summary()),
        "end": chain.end().map(|p| p.summary()),
        "summary": chain.summary(),
    }))
}

/// List all 7 compilation space axes.
pub fn compilation_axes_catalog(
    _p: CompilationAxesCatalogParams,
) -> Result<CallToolResult, McpError> {
    let axes: Vec<serde_json::Value> = Axis::ALL
        .iter()
        .map(|a| {
            json!({
                "label": a.label(),
                "primitive_symbol": a.primitive_symbol(),
                "description": a.description(),
            })
        })
        .collect();

    ok_json(json!({ "axes": axes }))
}

/// List all 8 abstraction levels in compilation order.
pub fn compilation_abstraction_levels(
    _p: CompilationAbstractionLevelsParams,
) -> Result<CallToolResult, McpError> {
    let levels: Vec<serde_json::Value> = AbstractionLevel::ALL
        .iter()
        .enumerate()
        .map(|(i, level)| {
            json!({
                "index": i,
                "label": level.label(),
            })
        })
        .collect();

    ok_json(json!({ "levels": levels }))
}

/// Compute spatial distance between two compilation points.
pub fn compilation_distance(p: CompilationDistanceParams) -> Result<CallToolResult, McpError> {
    let a = match build_point(&p.a) {
        Ok(pt) => pt,
        Err(e) => return err_result(&e.to_string()),
    };
    let b = match build_point(&p.b) {
        Ok(pt) => pt,
        Err(e) => return err_result(&e.to_string()),
    };

    ok_json(json!({
        "axis_distance": a.axis_distance(&b),
        "same_position": a.same_position(&b),
        "a_summary": a.summary(),
        "b_summary": b.summary(),
    }))
}
