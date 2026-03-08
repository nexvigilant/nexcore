//! Integration tests for nexcore-molt.
//!
//! Tests the full engine workflow: register commands, execute multi-line
//! Tcl scripts, verify sandbox policies, and round-trip JSON values.

use molt::Interp;
use molt::molt_ok;
use molt::types::{ContextID, MoltResult, Value};
use nexcore_molt::{Engine, SandboxPolicy};

// ── Custom commands for integration testing ──────────────────────────

fn cmd_add(_interp: &mut Interp, _ctx: ContextID, args: &[Value]) -> MoltResult {
    molt::check_args(1, args, 3, 3, "a b")?;
    let a = args[1].as_int()?;
    let b = args[2].as_int()?;
    molt_ok!(a + b)
}

fn cmd_mul(_interp: &mut Interp, _ctx: ContextID, args: &[Value]) -> MoltResult {
    molt::check_args(1, args, 3, 3, "a b")?;
    let a = args[1].as_int()?;
    let b = args[2].as_int()?;
    molt_ok!(a * b)
}

fn cmd_neg(_interp: &mut Interp, _ctx: ContextID, args: &[Value]) -> MoltResult {
    molt::check_args(1, args, 2, 2, "n")?;
    let n = args[1].as_int()?;
    molt_ok!(-n)
}

fn cmd_len(_interp: &mut Interp, _ctx: ContextID, args: &[Value]) -> MoltResult {
    molt::check_args(1, args, 2, 2, "s")?;
    let s = args[1].as_str();
    molt_ok!(s.len() as i64)
}

fn cmd_greet(_interp: &mut Interp, _ctx: ContextID, args: &[Value]) -> MoltResult {
    molt::check_args(1, args, 2, 2, "name")?;
    let name = args[1].as_str();
    molt_ok!(format!("Hello, {name}!"))
}

// ── Integration tests ───────────────────────────────────────────────

/// Register 5 custom commands, execute a multi-line Tcl script that calls
/// them, and verify the results.
#[test]
fn five_commands_multi_line_script() {
    let mut engine = Engine::safe();
    let mut reg = nexcore_molt::CommandRegistry::new();
    reg.add("add", cmd_add);
    reg.add("mul", cmd_mul);
    reg.add("neg", cmd_neg);
    reg.add("len", cmd_len);
    reg.add("greet", cmd_greet);
    engine.register(&reg);

    // Multi-line script that exercises all 5 commands
    let script = r#"
        set sum [add 10 32]
        set product [mul $sum 2]
        set negated [neg $product]
        set greeting [greet "World"]
        set name_len [len $greeting]
        list $sum $product $negated $greeting $name_len
    "#;

    let result = engine
        .execute(script)
        .unwrap_or_else(|e| panic!("script failed: {e}"));
    // sum=42, product=84, negated=-84, greeting="Hello, World!", name_len=13
    assert_eq!(result, "42 84 -84 {Hello, World!} 13");
}

/// Sandbox: verify `exit` and `source` are absent under Safe policy.
#[test]
fn sandbox_safe_excludes_dangerous_commands() {
    let mut engine = Engine::safe();
    let names = engine.command_names();
    assert!(!names.contains(&"exit".to_string()));
    assert!(!names.contains(&"source".to_string()));

    // Attempting to call exit should produce an error, not terminate
    let result = engine.execute("exit");
    assert!(result.is_err());
}

/// Sandbox: Allowlist prevents access to non-listed commands.
#[test]
fn sandbox_allowlist_restricts() {
    let policy = SandboxPolicy::Allowlist(vec!["expr".into(), "set".into()]);
    let mut engine = Engine::new(policy);

    // Allowed commands work
    let result = engine
        .execute("expr {2 + 3}")
        .unwrap_or_else(|e| panic!("expr failed: {e}"));
    assert_eq!(result, "5");

    // Non-allowed commands fail
    assert!(engine.execute("if {1} {set x 1}").is_err());
    assert!(engine.execute("exit").is_err());
}

/// Context: inject state, mutate from command, verify mutation persists.
#[test]
fn context_mutation_persists() {
    let mut engine = Engine::safe();
    let id = engine.inject_context(vec![1i64, 2, 3]);

    // The context is mutable through the engine
    let data = engine.context::<Vec<i64>>(id);
    data.push(4);
    data.push(5);
    assert_eq!(engine.context::<Vec<i64>>(id).len(), 5);
    assert_eq!(engine.context::<Vec<i64>>(id)[4], 5);
}

/// ValueAdapter: JSON -> Molt vars -> script access -> results back to JSON.
#[test]
fn json_round_trip_through_script() {
    use serde_json::json;

    let mut engine = Engine::safe();

    // Set JSON values as Tcl variables
    engine
        .set_var("count", &json!(42))
        .unwrap_or_else(|e| panic!("set count: {e}"));
    engine
        .set_var("name", &json!("Alice"))
        .unwrap_or_else(|e| panic!("set name: {e}"));
    engine
        .set_var("pi", &json!(3.14))
        .unwrap_or_else(|e| panic!("set pi: {e}"));
    engine
        .set_var("flag", &json!(null))
        .unwrap_or_else(|e| panic!("set flag: {e}"));

    // Access from Tcl script
    let result = engine
        .execute("expr {$count + 8}")
        .unwrap_or_else(|e| panic!("expr: {e}"));
    assert_eq!(result, "50");

    let result = engine
        .execute("string length $name")
        .unwrap_or_else(|e| panic!("strlen: {e}"));
    assert_eq!(result, "5");

    // Read back as JSON
    let count = engine
        .get_var("count")
        .unwrap_or_else(|e| panic!("get count: {e}"));
    assert_eq!(count, json!(42));

    let name = engine
        .get_var("name")
        .unwrap_or_else(|e| panic!("get name: {e}"));
    assert_eq!(name, json!("Alice"));

    let flag = engine
        .get_var("flag")
        .unwrap_or_else(|e| panic!("get flag: {e}"));
    assert_eq!(flag, json!(null));
}

/// Proc definition and invocation through the engine.
#[test]
fn proc_definition_and_call() {
    let mut engine = Engine::safe();

    engine
        .execute("proc double {x} { expr {$x * 2} }")
        .unwrap_or_else(|e| panic!("proc def: {e}"));

    let result = engine
        .execute("double 21")
        .unwrap_or_else(|e| panic!("call: {e}"));
    assert_eq!(result, "42");
}
