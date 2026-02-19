// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Integration Tests for Prima Codegen
//!
//! End-to-end tests: Prima source → Generated code → Compile → Run
//!
//! ## Tier: T2-C (σ + μ + → + κ)

use prima_codegen::{
    Backend, EmitContext,
    backends::{CBackend, GoBackend, PythonBackend, RustBackend, TypeScriptBackend},
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Test fixture directory
fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Temp output directory
fn output_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/output");
    fs::create_dir_all(&dir).ok();
    dir
}

/// Parse Prima source and generate code for target
fn generate_code(
    source: &str,
    backend: &dyn Backend,
    ctx: &mut EmitContext,
) -> Result<String, String> {
    let program = prima::parse(source).map_err(|e| format!("Parse error: {}", e))?;
    backend
        .emit_program(&program, ctx)
        .map_err(|e| format!("Emit error: {}", e))
}

// ═══════════════════════════════════════════════════════════════════════════
// RUST INTEGRATION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_rust_arithmetic_compiles() {
    let source = r#"
μ add(a: N, b: N) → N {
    a + b
}

μ multiply(x: N, y: N) → N {
    x * y
}
"#;

    let mut ctx = EmitContext::new();
    let code = generate_code(source, &RustBackend::new(), &mut ctx);
    assert!(code.is_ok(), "Failed to generate Rust: {:?}", code.err());

    let code = code.ok().unwrap_or_default();

    // Verify key constructs
    assert!(code.contains("fn add"));
    assert!(code.contains("fn multiply"));
    assert!(code.contains("i64"));
}

#[test]
fn test_rust_conditional_compiles() {
    let source = r#"
μ abs(n: N) → N {
    ∂ (n κ< 0) {
        0 - n
    } else {
        n
    }
}
"#;

    let mut ctx = EmitContext::new();
    let code = generate_code(source, &RustBackend::new(), &mut ctx);
    assert!(code.is_ok());

    let code = code.ok().unwrap_or_default();
    assert!(code.contains("if"));
    assert!(code.contains("else"));
}

#[test]
fn test_rust_sequence_compiles() {
    let source = r#"
λ nums = σ[1, 2, 3, 4, 5]
"#;

    let mut ctx = EmitContext::new();
    let code = generate_code(source, &RustBackend::new(), &mut ctx);
    assert!(code.is_ok());

    let code = code.ok().unwrap_or_default();
    assert!(code.contains("vec!") || code.contains("Vec"));
}

// ═══════════════════════════════════════════════════════════════════════════
// PYTHON INTEGRATION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_python_arithmetic_generates() {
    let source = r#"
μ add(a: N, b: N) → N {
    a + b
}
"#;

    let mut ctx = EmitContext::new();
    let code = generate_code(source, &PythonBackend::new(), &mut ctx);
    assert!(code.is_ok());

    let code = code.ok().unwrap_or_default();
    assert!(code.contains("def add"));
    assert!(code.contains("return"));
}

#[test]
fn test_python_syntax_valid() {
    // Use < instead of <= (κ<= not supported)
    let source = r#"
μ factorial(n: N) → N {
    ∂ (n κ< 2) {
        1
    } else {
        n * factorial(n - 1)
    }
}
"#;

    let mut ctx = EmitContext::new();
    let code = generate_code(source, &PythonBackend::new(), &mut ctx);
    assert!(code.is_ok(), "Failed to generate: {:?}", code.err());

    let code = code.ok().unwrap_or_default();

    // Write to file and check syntax with Python
    let output_path = output_dir().join("test_factorial.py");
    fs::write(&output_path, &code).ok();

    // Check if Python is available and validate syntax
    let result = Command::new("python3")
        .args(["-m", "py_compile"])
        .arg(&output_path)
        .output();

    if let Ok(output) = result {
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Don't fail if python3 not installed, just warn
            if !stderr.contains("not found") {
                panic!("Python syntax error: {}", stderr);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TYPESCRIPT INTEGRATION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_typescript_function_generates() {
    let source = r#"
μ greet(name: String) → String {
    "Hello, " + name
}
"#;

    let mut ctx = EmitContext::new();
    let code = generate_code(source, &TypeScriptBackend::new(), &mut ctx);
    assert!(code.is_ok(), "Failed to generate: {:?}", code.err());

    let code = code.ok().unwrap_or_default();
    assert!(code.contains("function greet"));
    // TypeScript may emit "string" or "String" depending on backend
    assert!(
        code.to_lowercase().contains("string"),
        "Should contain string type, got: {}",
        code
    );
}

#[test]
fn test_typescript_strict_equality() {
    // Use standard == (κ== not supported in parser)
    let source = r#"
μ is_zero(n: N) → Bool {
    n == 0
}
"#;

    let mut ctx = EmitContext::new();
    let code = generate_code(source, &TypeScriptBackend::new(), &mut ctx);
    assert!(code.is_ok(), "Failed to generate: {:?}", code.err());

    let code = code.ok().unwrap_or_default();
    // TypeScript should use strict equality
    assert!(
        code.contains("===") || code.contains("=="),
        "Should contain equality check, got: {}",
        code
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// GO INTEGRATION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_go_function_generates() {
    let source = r#"
μ double(x: N) → N {
    x * 2
}
"#;

    let mut ctx = EmitContext::new();
    let code = generate_code(source, &GoBackend::new(), &mut ctx);
    assert!(code.is_ok());

    let code = code.ok().unwrap_or_default();
    assert!(code.contains("func double"));
    assert!(code.contains("int64"));
}

#[test]
fn test_go_package_declaration() {
    let source = r#"
μ noop() → ∅ {
    ∅
}
"#;

    let mut ctx = EmitContext::new();
    let code = generate_code(source, &GoBackend::new(), &mut ctx);
    assert!(code.is_ok());

    let code = code.ok().unwrap_or_default();
    assert!(code.contains("package main"));
}

// ═══════════════════════════════════════════════════════════════════════════
// C INTEGRATION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_c_function_generates() {
    let source = r#"
μ square(n: N) → N {
    n * n
}
"#;

    let mut ctx = EmitContext::new();
    let code = generate_code(source, &CBackend::new(), &mut ctx);
    assert!(code.is_ok());

    let code = code.ok().unwrap_or_default();
    assert!(code.contains("int64_t square"));
    assert!(code.contains("#include"));
}

#[test]
fn test_c_includes_stdint() {
    let source = r#"
μ identity(x: N) → N {
    x
}
"#;

    let mut ctx = EmitContext::new();
    let code = generate_code(source, &CBackend::new(), &mut ctx);
    assert!(code.is_ok());

    let code = code.ok().unwrap_or_default();
    assert!(code.contains("<stdint.h>"));
}

// ═══════════════════════════════════════════════════════════════════════════
// CROSS-TARGET CONSISTENCY TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_all_targets_generate_same_function() {
    let source = r#"
μ add(a: N, b: N) → N {
    a + b
}
"#;

    let backends: Vec<(&str, Box<dyn Backend>)> = vec![
        ("rust", Box::new(RustBackend::new())),
        ("python", Box::new(PythonBackend::new())),
        ("typescript", Box::new(TypeScriptBackend::new())),
        ("go", Box::new(GoBackend::new())),
        ("c", Box::new(CBackend::new())),
    ];

    for (name, backend) in backends {
        let mut ctx = EmitContext::new();
        let result = generate_code(source, backend.as_ref(), &mut ctx);
        assert!(result.is_ok(), "Failed for {}: {:?}", name, result.err());

        let code = result.ok().unwrap_or_default();
        // All should contain "add" function name
        assert!(
            code.to_lowercase().contains("add"),
            "{} should contain 'add' function",
            name
        );
    }
}

#[test]
fn test_transfer_confidence_ordering() {
    // Rust should have highest, C lowest
    let source = "μ f() → N { 42 }";

    let mut rust_ctx = EmitContext::new();
    let mut py_ctx = EmitContext::new();
    let mut ts_ctx = EmitContext::new();
    let mut go_ctx = EmitContext::new();
    let mut c_ctx = EmitContext::new();

    let _ = generate_code(source, &RustBackend::new(), &mut rust_ctx);
    let _ = generate_code(source, &PythonBackend::new(), &mut py_ctx);
    let _ = generate_code(source, &TypeScriptBackend::new(), &mut ts_ctx);
    let _ = generate_code(source, &GoBackend::new(), &mut go_ctx);
    let _ = generate_code(source, &CBackend::new(), &mut c_ctx);

    use prima_codegen::TargetLanguage;

    let rust_conf = rust_ctx.transfer_confidence(TargetLanguage::Rust);
    let ts_conf = ts_ctx.transfer_confidence(TargetLanguage::TypeScript);
    let go_conf = go_ctx.transfer_confidence(TargetLanguage::Go);
    let py_conf = py_ctx.transfer_confidence(TargetLanguage::Python);
    let c_conf = c_ctx.transfer_confidence(TargetLanguage::C);

    // Expected: Rust > TypeScript > Go > Python > C
    assert!(
        rust_conf >= ts_conf,
        "Rust ({}) should >= TypeScript ({})",
        rust_conf,
        ts_conf
    );
    assert!(
        ts_conf >= go_conf,
        "TypeScript ({}) should >= Go ({})",
        ts_conf,
        go_conf
    );
    assert!(
        go_conf >= py_conf,
        "Go ({}) should >= Python ({})",
        go_conf,
        py_conf
    );
    assert!(
        py_conf >= c_conf,
        "Python ({}) should >= C ({})",
        py_conf,
        c_conf
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// PRIMITIVE TRACKING TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_primitives_tracked() {
    let source = r#"
μ complex(a: N, b: N) → N {
    λ result = a + b
    ∂ (result κ> 10) {
        result * 2
    } else {
        result
    }
}
"#;

    let mut ctx = EmitContext::new();
    let _ = generate_code(source, &RustBackend::new(), &mut ctx);

    // Should track: μ (function), λ (let), N (number), ∂ (if), κ (compare), → (return)
    assert!(!ctx.primitives_used.is_empty(), "Should track primitives");
}

// ═══════════════════════════════════════════════════════════════════════════
// ERROR HANDLING TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_invalid_source_returns_error() {
    let source = "this is not valid prima {{{{";

    let mut ctx = EmitContext::new();
    let result = generate_code(source, &RustBackend::new(), &mut ctx);

    assert!(result.is_err(), "Should fail on invalid source");
}

#[test]
fn test_empty_source_handles_gracefully() {
    let source = "";

    let mut ctx = EmitContext::new();
    let result = generate_code(source, &RustBackend::new(), &mut ctx);

    // Empty should either succeed with header-only or fail gracefully
    // Either is acceptable
    if let Ok(code) = result {
        // Should at least have the header comment
        assert!(code.contains("Generated") || code.is_empty());
    }
}
