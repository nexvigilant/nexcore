// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Built-in Functions
//!
//! Core functions grounded in primitives.
//!
//! ## Tier: T2-P (→ + various)

use crate::error::{PrimaError, PrimaResult};
use crate::value::{Value, ValueData};
use nexcore_lex_primitiva::{LexPrimitiva, Tier};
use std::collections::HashMap;

use std::cell::RefCell;

thread_local! {
    static CURRENT_ENTROPY: RefCell<f64> = const { RefCell::new(0.0) };
}

/// Set the current entropy value (for interpreter synchronization)
pub fn set_entropy(val: f64) {
    CURRENT_ENTROPY.with(|e| *e.borrow_mut() = val);
}

/// Built-in function registry.
pub type BuiltinFn = fn(&[Value]) -> PrimaResult<Value>;

/// Get all built-in functions.
#[must_use]
pub fn builtins() -> HashMap<String, Value> {
    let mut map = HashMap::new();
    map.insert("print".into(), Value::builtin("print"));
    map.insert("println".into(), Value::builtin("println"));
    map.insert("len".into(), Value::builtin("len"));
    map.insert("push".into(), Value::builtin("push"));
    map.insert("pop".into(), Value::builtin("pop"));
    map.insert("map".into(), Value::builtin("map"));
    map.insert("fold".into(), Value::builtin("fold"));
    map.insert("filter".into(), Value::builtin("filter"));
    map.insert("tier".into(), Value::builtin("tier"));
    map.insert("composition".into(), Value::builtin("composition"));
    map.insert("constants".into(), Value::builtin("constants"));
    map.insert("transfer".into(), Value::builtin("transfer"));
    map.insert("entropy".into(), Value::builtin("entropy"));
    map.insert("{-}".into(), Value::builtin("entropy"));

    // Synthesis Primitives (T2-P / T2-C)
    map.insert("stratify".into(), Value::builtin("stratify"));
    map.insert("Δ".into(), Value::builtin("stratify"));
    map.insert("accumulate".into(), Value::builtin("accumulate"));
    map.insert("⟲".into(), Value::builtin("accumulate"));
    map.insert("snapshot".into(), Value::builtin("snapshot"));
    map.insert("◊".into(), Value::builtin("snapshot"));
    map
}

/// Call a built-in function.
pub fn call_builtin(name: &str, args: &[Value]) -> PrimaResult<Value> {
    match name {
        "print" => builtin_print(args),
        "println" => builtin_println(args),
        "len" => builtin_len(args),
        "push" => builtin_push(args),
        "pop" => builtin_pop(args),
        "tier" => builtin_tier(args),
        "composition" => builtin_composition(args),
        "constants" => builtin_constants(args),
        "transfer" => builtin_transfer(args),
        "entropy" => builtin_entropy(args),
        // Synthesis
        "stratify" | "Δ" => builtin_stratify(args),
        "accumulate" | "⟲" => builtin_accumulate(args),
        "snapshot" | "◊" => builtin_snapshot(args),
        _ => Err(PrimaError::undefined(name)),
    }
}

/// entropy() → N — get current cumulative differential {-}
fn builtin_entropy(_args: &[Value]) -> PrimaResult<Value> {
    Ok(Value::float(CURRENT_ENTROPY.with(|e| *e.borrow())))
}

/// stratify(σ[A], A→K) → μ[K → σ[A]] — T2-P (Δ)
/// Groups sequence into layers by key function.
fn builtin_stratify(args: &[Value]) -> PrimaResult<Value> {
    // Note: Full implementation requires access to the interpreter for function calls.
    // For now, we provide a placeholder or return an error until integrated.
    Err(PrimaError::runtime("stratify requires interpreter context"))
}

/// accumulate(σ[N]) → N — T2-P (⟲)
/// Persistent sum of quantities.
fn builtin_accumulate(args: &[Value]) -> PrimaResult<Value> {
    match args.first() {
        Some(Value {
            data: ValueData::Sequence(v),
            ..
        }) => {
            let sum: i64 = v
                .iter()
                .filter_map(|e| {
                    if let ValueData::Int(n) = e.data {
                        Some(n)
                    } else {
                        None
                    }
                })
                .sum();
            Ok(Value::int(sum))
        }
        _ => Err(PrimaError::runtime(
            "accumulate requires a sequence of integers",
        )),
    }
}

/// snapshot() → μ — T2-C (◊)
/// Captures current values as mapping.
fn builtin_snapshot(args: &[Value]) -> PrimaResult<Value> {
    // Placeholders for environment-dependent synthesis
    let mut map = HashMap::new();
    map.insert(
        "timestamp".into(),
        Value::int(nexcore_chrono::DateTime::now().timestamp()),
    );
    Ok(Value::mapping(map))
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
                .map(|c| Value::string(c))
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
}
