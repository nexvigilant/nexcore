// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Built-in Functions
//!
//! Core functions grounded in primitives.
//!
//! ## Tier: T2-P (→ + various)

use crate::error::{PrimaError, PrimaResult};
use crate::value::{Value, ValueData};
use crate::vocabulary;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static CURRENT_ENTROPY: RefCell<f64> = const { RefCell::new(0.0) };
    static EXIT_CODE: RefCell<Option<i32>> = const { RefCell::new(None) };
    static CLI_ARGS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Set the current entropy value (for VM synchronization)
pub fn set_entropy(val: f64) {
    CURRENT_ENTROPY.with(|e| *e.borrow_mut() = val);
}

/// Get the current entropy value
pub fn get_entropy() -> f64 {
    CURRENT_ENTROPY.with(|e| *e.borrow())
}

/// Set the exit code (for hook support)
pub fn set_exit_code(code: i32) {
    EXIT_CODE.with(|c| *c.borrow_mut() = Some(code));
}

/// Get the exit code (None if not set)
pub fn get_exit_code() -> Option<i32> {
    EXIT_CODE.with(|c| *c.borrow())
}

/// Clear the exit code (for reuse)
pub fn clear_exit_code() {
    EXIT_CODE.with(|c| *c.borrow_mut() = None);
}

/// Set CLI arguments (call from main before eval)
pub fn set_cli_args(args: Vec<String>) {
    CLI_ARGS.with(|a| *a.borrow_mut() = args);
}

/// Get CLI arguments
pub fn get_cli_args() -> Vec<String> {
    CLI_ARGS.with(|a| a.borrow().clone())
}

/// Built-in function registry.
pub type BuiltinFn = fn(&[Value]) -> PrimaResult<Value>;

// ═══════════════════════════════════════════════════════════════════════════
// BUILTIN REGISTRATION — σ (sequence of mappings)
// Data driven from vocabulary.rs — the single source of truth.
// ═══════════════════════════════════════════════════════════════════════════

/// Register a builtin with both verbose and compressed names.
fn reg(map: &mut HashMap<String, Value>, verbose: &str, compressed: &str) {
    map.insert(verbose.into(), Value::builtin(verbose));
    map.insert(compressed.into(), Value::builtin(verbose));
}

/// Get all built-in functions (verbose + compressed).
///
/// Loads from [`vocabulary`] — the single source of truth.
#[must_use]
pub fn builtins() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    // Register all canonical builtins (IO, SEQ, STRING, MATH, TYPE, GROUNDING, VERIFY, CONVERT)
    for group in vocabulary::ALL_BUILTINS {
        for (verbose, compressed) in *group {
            reg(&mut m, verbose, compressed);
        }
    }
    // Register higher-order functions (handled specially in interpret.rs)
    for (verbose, compressed) in vocabulary::HOFS {
        reg(&mut m, verbose, compressed);
    }
    m
}

/// Call a built-in function.
pub fn call_builtin(name: &str, args: &[Value]) -> PrimaResult<Value> {
    match name {
        // I/O
        "print" => builtin_print(args),
        "println" => builtin_println(args),
        // Sequence operations (σ)
        "len" => builtin_len(args),
        "push" => builtin_push(args),
        "pop" => builtin_pop(args),
        "head" => builtin_head(args),
        "tail" => builtin_tail(args),
        "concat" => builtin_concat(args),
        "range" => builtin_range(args),
        // Math (N)
        "abs" => builtin_abs(args),
        "min" => builtin_min(args),
        "max" => builtin_max(args),
        // Type introspection
        "typeof" => builtin_type(args),
        "is_int" => builtin_is_int(args),
        "is_float" => builtin_is_float(args),
        "is_string" => builtin_is_string(args),
        "is_seq" => builtin_is_seq(args),
        // Grounding introspection
        "tier" => builtin_tier(args),
        "composition" => builtin_composition(args),
        "constants" => builtin_constants(args),
        "transfer" => builtin_transfer(args),
        // Verification (∂)
        "assert" => builtin_assert(args),
        // String operations
        "chars" => builtin_chars(args),
        "split" => builtin_split(args),
        "join" => builtin_join(args),
        "upper" => builtin_upper(args),
        "lower" => builtin_lower(args),
        "trim" => builtin_trim(args),
        "contains" => builtin_contains(args),
        "replace" => builtin_replace(args),
        // Conversions
        "to_string" => builtin_to_string(args),
        "to_int" => builtin_to_int(args),
        "to_float" => builtin_to_float(args),
        "entropy" => builtin_entropy(args),
        // System builtins (hook support)
        "stdin" => builtin_stdin(args),
        "readline" => builtin_readline(args),
        "exit" => builtin_exit(args),
        "env" => builtin_env(args),
        "args" => builtin_args(args),
        "json_parse" => builtin_json_parse(args),
        _ => Err(PrimaError::undefined(name)),
    }
}

/// entropy() → N — get current cumulative differential magnitude
fn builtin_entropy(args: &[Value]) -> PrimaResult<Value> {
    if !args.is_empty() {
        return Err(PrimaError::runtime("entropy requires 0 arguments"));
    }
    Ok(Value::float(CURRENT_ENTROPY.with(|e| *e.borrow())))
}

/// print(args...) — output without newline
fn builtin_print(args: &[Value]) -> PrimaResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    Ok(Value::void())
}

/// println(args...) — output with newline
fn builtin_println(args: &[Value]) -> PrimaResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
    Ok(Value::void())
}

/// len(σ) → N — sequence length
fn builtin_len(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::Sequence(v),
            ..
        }) => Ok(Value::int(v.len() as i64)),
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => Ok(Value::int(s.len() as i64)),
        Some(Value {
            data: ValueData::Mapping(m),
            ..
        }) => Ok(Value::int(m.len() as i64)),
        _ => Err(PrimaError::runtime(
            "len requires a sequence, string, or mapping",
        )),
    }
}

/// push(σ, v) → σ — append to sequence
fn builtin_push(args: &[Value]) -> PrimaResult<Value> {
    match (args.first(), args.get(1)) {
        (
            Some(Value {
                data: ValueData::Sequence(v),
                ..
            }),
            Some(elem),
        ) => {
            let mut new_v = v.clone();
            new_v.push(elem.clone());
            Ok(Value::sequence(new_v))
        }
        _ => Err(PrimaError::runtime("push requires (sequence, element)")),
    }
}

/// pop(σ) → (σ, v) — remove last element
fn builtin_pop(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::Sequence(v),
            ..
        }) => {
            let mut new_v = v.clone();
            let elem = new_v.pop().unwrap_or(Value::void());
            let mut result = HashMap::new();
            result.insert("seq".into(), Value::sequence(new_v));
            result.insert("elem".into(), elem);
            Ok(Value::mapping(result))
        }
        _ => Err(PrimaError::runtime("pop requires a sequence")),
    }
}

/// tier(v) → String — get tier classification
fn builtin_tier(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(v) => Ok(Value::string(v.tier().code())),
        None => Err(PrimaError::runtime("tier requires an argument")),
    }
}

/// composition(v) → String — get primitive composition
fn builtin_composition(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(v) => Ok(Value::string(format!("{}", v.composition))),
        None => Err(PrimaError::runtime("composition requires an argument")),
    }
}

/// constants(v) → σ[String] — get grounding constants
fn builtin_constants(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(v) => {
            let constants: Vec<Value> = v
                .grounding_constants()
                .into_iter()
                .map(Value::string)
                .collect();
            Ok(Value::sequence(constants))
        }
        None => Err(PrimaError::runtime("constants requires an argument")),
    }
}

/// transfer(v) → N — get transfer confidence
fn builtin_transfer(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(v) => Ok(Value::float(v.transfer_confidence())),
        None => Err(PrimaError::runtime("transfer requires an argument")),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SEQUENCE OPERATIONS (σ)
// ═══════════════════════════════════════════════════════════════════════════

/// head(σ) → v — first element
fn builtin_head(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::Sequence(v),
            ..
        }) => v
            .first()
            .cloned()
            .ok_or_else(|| PrimaError::runtime("head of empty sequence")),
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => s
            .chars()
            .next()
            .map(|c| Value::string(c.to_string()))
            .ok_or_else(|| PrimaError::runtime("head of empty string")),
        _ => Err(PrimaError::runtime("head requires a sequence or string")),
    }
}

/// tail(σ) → σ — all but first element
fn builtin_tail(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::Sequence(v),
            ..
        }) => {
            if v.is_empty() {
                Ok(Value::sequence(vec![]))
            } else {
                Ok(Value::sequence(v[1..].to_vec()))
            }
        }
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => {
            let tail: String = s.chars().skip(1).collect();
            Ok(Value::string(tail))
        }
        _ => Err(PrimaError::runtime("tail requires a sequence or string")),
    }
}

/// concat(σ, σ) → σ — concatenate sequences
fn builtin_concat(args: &[Value]) -> PrimaResult<Value> {
    match (args.first(), args.get(1)) {
        (
            Some(Value {
                data: ValueData::Sequence(a),
                ..
            }),
            Some(Value {
                data: ValueData::Sequence(b),
                ..
            }),
        ) => {
            let mut result = a.clone();
            result.extend(b.iter().cloned());
            Ok(Value::sequence(result))
        }
        (
            Some(Value {
                data: ValueData::String(a),
                ..
            }),
            Some(Value {
                data: ValueData::String(b),
                ..
            }),
        ) => Ok(Value::string(format!("{}{}", a, b))),
        _ => Err(PrimaError::runtime(
            "concat requires two sequences or strings",
        )),
    }
}

/// range(start, end) → σ[N] — integer range [start, end)
fn builtin_range(args: &[Value]) -> PrimaResult<Value> {
    match (args.first(), args.get(1)) {
        (
            Some(Value {
                data: ValueData::Int(start),
                ..
            }),
            Some(Value {
                data: ValueData::Int(end),
                ..
            }),
        ) => {
            let seq: Vec<Value> = (*start..*end).map(Value::int).collect();
            Ok(Value::sequence(seq))
        }
        (
            Some(Value {
                data: ValueData::Int(end),
                ..
            }),
            None,
        ) => {
            let seq: Vec<Value> = (0..*end).map(Value::int).collect();
            Ok(Value::sequence(seq))
        }
        _ => Err(PrimaError::runtime("range requires integer arguments")),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MATH OPERATIONS (N)
// ═══════════════════════════════════════════════════════════════════════════

/// abs(n) → N — absolute value
fn builtin_abs(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::Int(n),
            ..
        }) => Ok(Value::int(n.abs())),
        Some(Value {
            data: ValueData::Float(n),
            ..
        }) => Ok(Value::float(n.abs())),
        _ => Err(PrimaError::runtime("abs requires a number")),
    }
}

/// min(a, b) → N — minimum value
fn builtin_min(args: &[Value]) -> PrimaResult<Value> {
    match (args.first(), args.get(1)) {
        (
            Some(Value {
                data: ValueData::Int(a),
                ..
            }),
            Some(Value {
                data: ValueData::Int(b),
                ..
            }),
        ) => Ok(Value::int(*a.min(b))),
        (
            Some(Value {
                data: ValueData::Float(a),
                ..
            }),
            Some(Value {
                data: ValueData::Float(b),
                ..
            }),
        ) => Ok(Value::float(a.min(*b))),
        _ => Err(PrimaError::runtime("min requires two numbers of same type")),
    }
}

/// max(a, b) → N — maximum value
fn builtin_max(args: &[Value]) -> PrimaResult<Value> {
    match (args.first(), args.get(1)) {
        (
            Some(Value {
                data: ValueData::Int(a),
                ..
            }),
            Some(Value {
                data: ValueData::Int(b),
                ..
            }),
        ) => Ok(Value::int(*a.max(b))),
        (
            Some(Value {
                data: ValueData::Float(a),
                ..
            }),
            Some(Value {
                data: ValueData::Float(b),
                ..
            }),
        ) => Ok(Value::float(a.max(*b))),
        _ => Err(PrimaError::runtime("max requires two numbers of same type")),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TYPE INTROSPECTION
// ═══════════════════════════════════════════════════════════════════════════

/// type(v) → String — get type name
fn builtin_type(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(v) => {
            let name = match &v.data {
                ValueData::Void => "∅",
                ValueData::Int(_) => "N",
                ValueData::Float(_) => "N.float",
                ValueData::Bool(_) => "Bool",
                ValueData::String(_) => "String",
                ValueData::Symbol(_) => "Symbol",
                ValueData::Sequence(_) => "σ",
                ValueData::Mapping(_) => "μ",
                ValueData::Function { .. } | ValueData::Builtin(_) => "μ.fn",
                ValueData::Quoted(_) => "ρ.quoted",
            };
            Ok(Value::string(name))
        }
        None => Err(PrimaError::runtime("type requires an argument")),
    }
}

/// is_int(v) → Bool
fn builtin_is_int(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(v) => Ok(Value::bool(matches!(v.data, ValueData::Int(_)))),
        None => Err(PrimaError::runtime("is_int requires an argument")),
    }
}

/// is_float(v) → Bool
fn builtin_is_float(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(v) => Ok(Value::bool(matches!(v.data, ValueData::Float(_)))),
        None => Err(PrimaError::runtime("is_float requires an argument")),
    }
}

/// is_string(v) → Bool
fn builtin_is_string(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(v) => Ok(Value::bool(matches!(v.data, ValueData::String(_)))),
        None => Err(PrimaError::runtime("is_string requires an argument")),
    }
}

/// is_seq(v) → Bool
fn builtin_is_seq(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(v) => Ok(Value::bool(matches!(v.data, ValueData::Sequence(_)))),
        None => Err(PrimaError::runtime("is_seq requires an argument")),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// VERIFICATION (∂)
// ═══════════════════════════════════════════════════════════════════════════

/// assert(cond, msg?) → ∅ — verify condition, halt if false
fn builtin_assert(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(v) if v.is_truthy() => Ok(Value::void()),
        Some(_) => {
            let msg = args
                .get(1)
                .map(|v| format!("{}", v))
                .unwrap_or_else(|| "assertion failed".into());
            Err(PrimaError::runtime(format!("∂[assert]: {}", msg)))
        }
        None => Err(PrimaError::runtime("assert requires a condition")),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// STRING OPERATIONS
// ═══════════════════════════════════════════════════════════════════════════

/// chars(s) → σ[String] — split string into characters
fn builtin_chars(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => {
            let chars: Vec<Value> = s.chars().map(|c| Value::string(c.to_string())).collect();
            Ok(Value::sequence(chars))
        }
        _ => Err(PrimaError::runtime("chars requires a string")),
    }
}

/// split(s, delim) → σ[String] — split string by delimiter
fn builtin_split(args: &[Value]) -> PrimaResult<Value> {
    match (args.first(), args.get(1)) {
        (
            Some(Value {
                data: ValueData::String(s),
                ..
            }),
            Some(Value {
                data: ValueData::String(delim),
                ..
            }),
        ) => {
            let parts: Vec<Value> = s.split(delim.as_str()).map(Value::string).collect();
            Ok(Value::sequence(parts))
        }
        _ => Err(PrimaError::runtime("split requires (string, delimiter)")),
    }
}

/// join(σ[String], delim) → String — join strings with delimiter
fn builtin_join(args: &[Value]) -> PrimaResult<Value> {
    match (args.first(), args.get(1)) {
        (
            Some(Value {
                data: ValueData::Sequence(items),
                ..
            }),
            Some(Value {
                data: ValueData::String(delim),
                ..
            }),
        ) => {
            let strings: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
            Ok(Value::string(strings.join(delim)))
        }
        _ => Err(PrimaError::runtime("join requires (sequence, delimiter)")),
    }
}

/// upper(s) → String — uppercase
fn builtin_upper(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => Ok(Value::string(s.to_uppercase())),
        _ => Err(PrimaError::runtime("upper requires a string")),
    }
}

/// lower(s) → String — lowercase
fn builtin_lower(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => Ok(Value::string(s.to_lowercase())),
        _ => Err(PrimaError::runtime("lower requires a string")),
    }
}

/// trim(s) → String — remove whitespace
fn builtin_trim(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => Ok(Value::string(s.trim())),
        _ => Err(PrimaError::runtime("trim requires a string")),
    }
}

/// contains(s, substr) → Bool — check if string contains substring
fn builtin_contains(args: &[Value]) -> PrimaResult<Value> {
    match (args.first(), args.get(1)) {
        (
            Some(Value {
                data: ValueData::String(s),
                ..
            }),
            Some(Value {
                data: ValueData::String(substr),
                ..
            }),
        ) => Ok(Value::bool(s.contains(substr.as_str()))),
        (
            Some(Value {
                data: ValueData::Sequence(v),
                ..
            }),
            Some(elem),
        ) => Ok(Value::bool(v.contains(elem))),
        _ => Err(PrimaError::runtime(
            "contains requires (string, substr) or (seq, elem)",
        )),
    }
}

/// replace(s, from, to) → String — replace all occurrences
fn builtin_replace(args: &[Value]) -> PrimaResult<Value> {
    match (args.first(), args.get(1), args.get(2)) {
        (
            Some(Value {
                data: ValueData::String(s),
                ..
            }),
            Some(Value {
                data: ValueData::String(from),
                ..
            }),
            Some(Value {
                data: ValueData::String(to),
                ..
            }),
        ) => Ok(Value::string(s.replace(from.as_str(), to.as_str()))),
        _ => Err(PrimaError::runtime("replace requires (string, from, to)")),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TYPE CONVERSIONS
// ═══════════════════════════════════════════════════════════════════════════

/// to_string(v) → String — convert to string
fn builtin_to_string(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(v) => Ok(Value::string(format!("{}", v))),
        None => Err(PrimaError::runtime("to_string requires an argument")),
    }
}

/// to_int(v) → N — convert to integer
fn builtin_to_int(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::Int(n),
            ..
        }) => Ok(Value::int(*n)),
        Some(Value {
            data: ValueData::Float(n),
            ..
        }) => Ok(Value::int(*n as i64)),
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => s
            .parse::<i64>()
            .map(Value::int)
            .map_err(|_| PrimaError::runtime(format!("cannot parse '{}' as int", s))),
        Some(Value {
            data: ValueData::Bool(b),
            ..
        }) => Ok(Value::int(if *b { 1 } else { 0 })),
        _ => Err(PrimaError::runtime("to_int requires a convertible value")),
    }
}

/// to_float(v) → N.float — convert to float
fn builtin_to_float(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::Int(n),
            ..
        }) => Ok(Value::float(*n as f64)),
        Some(Value {
            data: ValueData::Float(n),
            ..
        }) => Ok(Value::float(*n)),
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => s
            .parse::<f64>()
            .map(Value::float)
            .map_err(|_| PrimaError::runtime(format!("cannot parse '{}' as float", s))),
        _ => Err(PrimaError::runtime("to_float requires a convertible value")),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM BUILTINS (π + → + ∃) — Hook Support
// ═══════════════════════════════════════════════════════════════════════════

/// stdin() → String — read all of stdin
fn builtin_stdin(args: &[Value]) -> PrimaResult<Value> {
    if !args.is_empty() {
        return Err(PrimaError::runtime("stdin requires 0 arguments"));
    }
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|e| PrimaError::runtime(format!("stdin read error: {}", e)))?;
    Ok(Value::string(buffer))
}

/// readline() → String — read one line from stdin
fn builtin_readline(args: &[Value]) -> PrimaResult<Value> {
    if !args.is_empty() {
        return Err(PrimaError::runtime("readline requires 0 arguments"));
    }
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .map_err(|e| PrimaError::runtime(format!("readline error: {}", e)))?;
    // Remove trailing newline if present
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(Value::string(line))
}

/// exit(N) → ∅ — set process exit code
fn builtin_exit(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::Int(code),
            ..
        }) => {
            set_exit_code(*code as i32);
            Ok(Value::void())
        }
        None => {
            set_exit_code(0);
            Ok(Value::void())
        }
        _ => Err(PrimaError::runtime("exit requires an integer code")),
    }
}

/// env(String) → String|∅ — get environment variable
fn builtin_env(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::String(name),
            ..
        }) => match std::env::var(name.as_str()) {
            Ok(val) => Ok(Value::string(val)),
            Err(_) => Ok(Value::void()),
        },
        _ => Err(PrimaError::runtime("env requires a string argument")),
    }
}

/// args() → σ[String] — get CLI arguments
fn builtin_args(args: &[Value]) -> PrimaResult<Value> {
    if !args.is_empty() {
        return Err(PrimaError::runtime("args requires 0 arguments"));
    }
    let cli_args = get_cli_args();
    let values: Vec<Value> = cli_args.into_iter().map(Value::string).collect();
    Ok(Value::sequence(values))
}

/// json_parse(String) → μ|σ|N|String|Bool|∅ — parse JSON string
fn builtin_json_parse(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::String(s),
            ..
        }) => {
            let json: serde_json::Value = serde_json::from_str(s)
                .map_err(|e| PrimaError::runtime(format!("JSON parse error: {}", e)))?;
            json_to_prima(json)
        }
        _ => Err(PrimaError::runtime("json_parse requires a string argument")),
    }
}

/// Convert serde_json::Value to Prima Value
fn json_to_prima(json: serde_json::Value) -> PrimaResult<Value> {
    match json {
        serde_json::Value::Null => Ok(Value::void()),
        serde_json::Value::Bool(b) => Ok(Value::bool(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::float(f))
            } else {
                Err(PrimaError::runtime("unsupported JSON number"))
            }
        }
        serde_json::Value::String(s) => Ok(Value::string(s)),
        serde_json::Value::Array(arr) => {
            let values: Result<Vec<Value>, _> = arr.into_iter().map(json_to_prima).collect();
            Ok(Value::sequence(values?))
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_prima(v)?);
            }
            Ok(Value::mapping(map))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtins_exist() {
        let b = builtins();
        assert!(b.contains_key("print"));
        assert!(b.contains_key("tier"));
        assert!(b.contains_key("composition"));
    }

    #[test]
    fn test_len() {
        let seq = Value::sequence(vec![Value::int(1), Value::int(2), Value::int(3)]);
        let result = builtin_len(&[seq]).unwrap();
        assert_eq!(result, Value::int(3));
    }

    #[test]
    fn test_tier() {
        let int = Value::int(42);
        let result = builtin_tier(&[int]).unwrap();
        assert_eq!(result, Value::string("T1"));
    }

    #[test]
    fn test_transfer() {
        let int = Value::int(42);
        let result = builtin_transfer(&[int]).unwrap();
        if let ValueData::Float(conf) = result.data {
            assert!((conf - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_constants() {
        let int = Value::int(42);
        let result = builtin_constants(&[int]).unwrap();
        if let ValueData::Sequence(v) = result.data {
            assert!(
                v.iter()
                    .any(|c| matches!(&c.data, ValueData::String(s) if s == "0"))
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // NEW BUILTIN TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_head() {
        let seq = Value::sequence(vec![Value::int(1), Value::int(2), Value::int(3)]);
        let result = builtin_head(&[seq]).ok();
        assert!(result.is_some());
        assert_eq!(result, Some(Value::int(1)));
    }

    #[test]
    fn test_tail() {
        let seq = Value::sequence(vec![Value::int(1), Value::int(2), Value::int(3)]);
        let result = builtin_tail(&[seq]).ok();
        assert!(result.is_some());
        if let Some(Value {
            data: ValueData::Sequence(v),
            ..
        }) = result
        {
            assert_eq!(v.len(), 2);
        }
    }

    #[test]
    fn test_concat() {
        let a = Value::sequence(vec![Value::int(1)]);
        let b = Value::sequence(vec![Value::int(2)]);
        let result = builtin_concat(&[a, b]).ok();
        assert!(result.is_some());
        if let Some(Value {
            data: ValueData::Sequence(v),
            ..
        }) = result
        {
            assert_eq!(v.len(), 2);
        }
    }

    #[test]
    fn test_range() {
        let result = builtin_range(&[Value::int(3)]).ok();
        assert!(result.is_some());
        if let Some(Value {
            data: ValueData::Sequence(v),
            ..
        }) = result
        {
            assert_eq!(v.len(), 3);
        }
    }

    #[test]
    fn test_abs() {
        let result = builtin_abs(&[Value::int(-42)]).ok();
        assert_eq!(result, Some(Value::int(42)));
    }

    #[test]
    fn test_min_max() {
        let min_result = builtin_min(&[Value::int(3), Value::int(7)]).ok();
        let max_result = builtin_max(&[Value::int(3), Value::int(7)]).ok();
        assert_eq!(min_result, Some(Value::int(3)));
        assert_eq!(max_result, Some(Value::int(7)));
    }

    #[test]
    fn test_typeof() {
        let result = builtin_type(&[Value::int(42)]).ok();
        assert!(result.is_some());
        if let Some(Value {
            data: ValueData::String(s),
            ..
        }) = result
        {
            assert_eq!(s, "N");
        }
    }

    #[test]
    fn test_is_predicates() {
        assert_eq!(
            builtin_is_int(&[Value::int(1)]).ok(),
            Some(Value::bool(true))
        );
        assert_eq!(
            builtin_is_int(&[Value::string("x")]).ok(),
            Some(Value::bool(false))
        );
        assert_eq!(
            builtin_is_seq(&[Value::sequence(vec![])]).ok(),
            Some(Value::bool(true))
        );
    }

    #[test]
    fn test_assert_success() {
        let result = builtin_assert(&[Value::bool(true)]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assert_failure() {
        let result = builtin_assert(&[Value::bool(false)]);
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SYSTEM BUILTIN TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_exit_code() {
        clear_exit_code();
        assert!(get_exit_code().is_none());

        let result = builtin_exit(&[Value::int(2)]);
        assert!(result.is_ok());
        assert_eq!(get_exit_code(), Some(2));

        clear_exit_code();
        assert!(get_exit_code().is_none());
    }

    #[test]
    fn test_env() {
        // PATH should always exist
        let result = builtin_env(&[Value::string("PATH")]);
        assert!(result.is_ok());
        let val = result.ok();
        assert!(val.is_some());
        // PATH value should be non-void
        if let Some(v) = val {
            assert!(!matches!(v.data, ValueData::Void));
        }

        // Non-existent env var returns void
        let result = builtin_env(&[Value::string("PRIMA_NONEXISTENT_VAR_12345")]);
        assert!(result.is_ok());
        if let Some(v) = result.ok() {
            assert!(matches!(v.data, ValueData::Void));
        }
    }

    #[test]
    fn test_args() {
        set_cli_args(vec!["arg1".into(), "arg2".into()]);
        let result = builtin_args(&[]);
        assert!(result.is_ok());
        if let Some(Value {
            data: ValueData::Sequence(v),
            ..
        }) = result.ok()
        {
            assert_eq!(v.len(), 2);
        }
        set_cli_args(vec![]); // cleanup
    }

    #[test]
    fn test_json_parse_object() {
        let json = r#"{"name": "test", "value": 42}"#;
        let result = builtin_json_parse(&[Value::string(json)]);
        assert!(result.is_ok());
        if let Some(Value {
            data: ValueData::Mapping(m),
            ..
        }) = result.ok()
        {
            assert_eq!(m.len(), 2);
            assert!(m.contains_key("name"));
            assert!(m.contains_key("value"));
        }
    }

    #[test]
    fn test_json_parse_array() {
        let json = r#"[1, 2, 3]"#;
        let result = builtin_json_parse(&[Value::string(json)]);
        assert!(result.is_ok());
        if let Some(Value {
            data: ValueData::Sequence(v),
            ..
        }) = result.ok()
        {
            assert_eq!(v.len(), 3);
        }
    }

    #[test]
    fn test_json_parse_nested() {
        let json = r#"{"tool": "Write", "params": {"path": "/tmp/test.txt"}}"#;
        let result = builtin_json_parse(&[Value::string(json)]);
        assert!(result.is_ok());
        if let Some(Value {
            data: ValueData::Mapping(m),
            ..
        }) = result.ok()
        {
            assert!(m.contains_key("tool"));
            assert!(m.contains_key("params"));
        }
    }

    #[test]
    fn test_json_parse_primitives() {
        // null -> void
        let result = builtin_json_parse(&[Value::string("null")]);
        assert!(result.is_ok());
        if let Some(v) = result.ok() {
            assert!(matches!(v.data, ValueData::Void));
        }

        // true -> bool
        let result = builtin_json_parse(&[Value::string("true")]);
        assert!(result.is_ok());
        if let Some(v) = result.ok() {
            assert!(matches!(v.data, ValueData::Bool(true)));
        }

        // number -> int
        let result = builtin_json_parse(&[Value::string("42")]);
        assert!(result.is_ok());
        if let Some(v) = result.ok() {
            assert!(matches!(v.data, ValueData::Int(42)));
        }

        // string -> string
        let result = builtin_json_parse(&[Value::string(r#""hello""#)]);
        assert!(result.is_ok());
    }
}
