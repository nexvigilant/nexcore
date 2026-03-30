// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # nexcore-prima — Thin Re-export Wrapper
//!
//! This crate re-exports [`prima`] — the canonical Prima language implementation.
//!
//! ## History
//!
//! `nexcore-prima` was the original interpreter (10 modules). The `prima` crate
//! evolved into the full-featured implementation (30+ modules) with bytecode VM,
//! type inference, effect system, and optimization passes.
//!
//! This crate is retained for backward compatibility. New code should depend on
//! `prima` directly.

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
// NOTE: grounding.rs exists but impls live in canonical prima crate
// (orphan rules prevent implementing GroundsTo on re-exported foreign types here)
#![warn(missing_docs)]
pub use prima::*;

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // Re-export surface smoke tests
    // Verify the wrapper crate correctly surfaces the prima API.
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn reexport_file_extension_is_true() {
        // The canonical file extension is `.true` (code that compiles is true)
        assert_eq!(FILE_EXTENSION, "true");
    }

    #[test]
    fn reexport_file_extension_fallback_is_prima() {
        assert_eq!(FILE_EXTENSION_FALLBACK, "prima");
    }

    #[test]
    fn reexport_file_extension_not_exists() {
        // FILE_EXTENSION_NOT is the `.nottrue` extension
        assert!(!FILE_EXTENSION_NOT.is_empty());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // eval() — tree-walking interpreter pipeline
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn eval_root_constant_zero() {
        let v = eval("0").unwrap();
        assert!(v.is_zero());
    }

    #[test]
    fn eval_root_constant_one() {
        let v = eval("1").unwrap();
        assert!(v.is_one());
    }

    #[test]
    fn eval_integer_arithmetic_add() {
        assert_eq!(eval("3 + 4").unwrap(), Value::int(7));
    }

    #[test]
    fn eval_integer_arithmetic_sub() {
        assert_eq!(eval("10 - 3").unwrap(), Value::int(7));
    }

    #[test]
    fn eval_integer_arithmetic_mul() {
        assert_eq!(eval("6 * 7").unwrap(), Value::int(42));
    }

    #[test]
    fn eval_integer_arithmetic_div() {
        assert_eq!(eval("84 / 2").unwrap(), Value::int(42));
    }

    #[test]
    fn eval_integer_arithmetic_mod() {
        assert_eq!(eval("10 % 3").unwrap(), Value::int(1));
    }

    #[test]
    fn eval_division_by_zero_returns_error() {
        let result = eval("1 / 0");
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Error display must contain the boundary symbol
        assert!(
            err.to_string().contains('∂') || err.to_string().contains("zero"),
            "expected boundary error, got: {}",
            err
        );
    }

    #[test]
    fn eval_boolean_true() {
        assert_eq!(eval("true").unwrap(), Value::bool(true));
    }

    #[test]
    fn eval_boolean_false() {
        assert_eq!(eval("false").unwrap(), Value::bool(false));
    }

    #[test]
    fn eval_string_literal() {
        assert_eq!(eval("\"hello\"").unwrap(), Value::string("hello"));
    }

    #[test]
    fn eval_string_concatenation() {
        assert_eq!(eval("\"foo\" + \"bar\"").unwrap(), Value::string("foobar"));
    }

    #[test]
    fn eval_let_binding() {
        assert_eq!(eval("let x = 99\nx").unwrap(), Value::int(99));
    }

    #[test]
    fn eval_let_used_in_expression() {
        assert_eq!(
            eval("let x = 10\nlet y = 32\nx + y").unwrap(),
            Value::int(42)
        );
    }

    #[test]
    fn eval_function_definition_and_call() {
        let r = eval("fn double(x: N) → N { x * 2 }\ndouble(21)").unwrap();
        assert_eq!(r, Value::int(42));
    }

    #[test]
    fn eval_function_two_params() {
        let r = eval("fn add(a: N, b: N) → N { a + b }\nadd(20, 22)").unwrap();
        assert_eq!(r, Value::int(42));
    }

    #[test]
    fn eval_if_true_branch() {
        assert_eq!(eval("if true { 42 } else { 0 }").unwrap(), Value::int(42));
    }

    #[test]
    fn eval_if_false_branch() {
        assert_eq!(eval("if false { 0 } else { 42 }").unwrap(), Value::int(42));
    }

    #[test]
    fn eval_if_no_else_returns_void() {
        let v = eval("if false { 1 }").unwrap();
        assert!(!v.is_truthy());
    }

    #[test]
    fn eval_kappa_comparison_eq() {
        assert_eq!(eval("5 κ= 5").unwrap(), Value::bool(true));
        assert_eq!(eval("5 κ= 6").unwrap(), Value::bool(false));
    }

    #[test]
    fn eval_kappa_comparison_lt() {
        assert_eq!(eval("3 κ< 5").unwrap(), Value::bool(true));
        assert_eq!(eval("5 κ< 3").unwrap(), Value::bool(false));
    }

    #[test]
    fn eval_kappa_comparison_gt() {
        assert_eq!(eval("5 κ> 3").unwrap(), Value::bool(true));
        assert_eq!(eval("3 κ> 5").unwrap(), Value::bool(false));
    }

    #[test]
    fn eval_sequence_literal() {
        let v = eval("σ[1, 2, 3]").unwrap();
        assert!(v.is_truthy());
    }

    #[test]
    fn eval_sequence_empty_is_falsy() {
        let v = eval("σ[]").unwrap();
        assert!(!v.is_truthy());
    }

    #[test]
    fn eval_symbol_evaluates_to_itself() {
        let v = eval(":foo").unwrap();
        assert!(v.is_truthy()); // symbols are always truthy
    }

    #[test]
    fn eval_undefined_variable_returns_error() {
        let result = eval("undefined_var_xyz");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("undefined_var_xyz") || msg.contains('λ'),
            "expected undefined error, got: {}",
            msg
        );
    }

    #[test]
    fn eval_quoted_expression_produces_value() {
        let v = eval("'42").unwrap();
        assert!(v.is_truthy()); // quoted AST nodes are truthy
    }

    #[test]
    fn eval_eval_builtin_is_disabled() {
        // The prima `eval` built-in is currently disabled (EvalDisabled boundary).
        // Verify this returns an error rather than silently succeeding.
        let result = eval("eval('42)");
        assert!(result.is_err(), "eval() builtin should be disabled");
    }

    #[test]
    fn eval_unquote_outside_quasiquote_is_error() {
        assert!(eval("~42").is_err());
    }

    #[test]
    fn eval_for_loop_over_sequence() {
        // for loop returns last value
        let v = eval("for x in σ[1, 2, 3] { x }").unwrap();
        assert_eq!(v, Value::int(3));
    }

    #[test]
    fn eval_for_loop_over_non_sequence_is_error() {
        assert!(eval("for x in 42 { x }").is_err());
    }

    #[test]
    fn eval_match_wildcard() {
        let v = eval("match 42 { _ → 1 }").unwrap();
        assert_eq!(v, Value::int(1));
    }

    #[test]
    fn eval_match_literal() {
        let v = eval("match 42 { 42 → 99, _ → 0 }").unwrap();
        assert_eq!(v, Value::int(99));
    }

    #[test]
    fn eval_match_no_arm_is_error() {
        // No matching arm — exhaustiveness error
        let result = eval("match 99 { 1 → 0 }");
        assert!(result.is_err());
    }

    #[test]
    fn eval_logical_and() {
        assert_eq!(eval("true && true").unwrap(), Value::bool(true));
        assert_eq!(eval("true && false").unwrap(), Value::bool(false));
    }

    #[test]
    fn eval_logical_or() {
        assert_eq!(eval("false || true").unwrap(), Value::bool(true));
        assert_eq!(eval("false || false").unwrap(), Value::bool(false));
    }

    #[test]
    fn eval_negation() {
        assert_eq!(eval("-42").unwrap(), Value::int(-42));
    }

    #[test]
    fn eval_logical_not() {
        assert_eq!(eval("!false").unwrap(), Value::bool(true));
        assert_eq!(eval("!true").unwrap(), Value::bool(false));
    }

    #[test]
    fn eval_tier_method_on_int() {
        let v = eval("42.tier()").unwrap();
        assert_eq!(v, Value::string("T1"));
    }

    #[test]
    fn eval_len_method_on_string() {
        let v = eval("\"hello\".len()").unwrap();
        assert_eq!(v, Value::int(5));
    }

    #[test]
    fn eval_len_method_on_sequence() {
        let v = eval("σ[1, 2, 3].len()").unwrap();
        assert_eq!(v, Value::int(3));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // tokenize() — lexer pipeline surface
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn tokenize_empty_returns_eof() {
        let tokens = tokenize("").unwrap();
        assert!(!tokens.is_empty()); // always at least EOF token
        assert!(tokens.last().map(|t| t.is_eof()).unwrap_or(false));
    }

    #[test]
    fn tokenize_integer_count() {
        let tokens = tokenize("42").unwrap();
        // integer token + EOF
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn tokenize_unterminated_string_is_error() {
        assert!(tokenize("\"unterminated").is_err());
    }

    #[test]
    fn tokenize_unknown_character_is_error() {
        // Single ampersand is not valid Prima syntax
        assert!(tokenize("&").is_err());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // parse() — parser surface
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn parse_empty_program_has_no_statements() {
        let prog = parse("").unwrap();
        assert_eq!(prog.statements.len(), 0);
    }

    #[test]
    fn parse_let_produces_one_statement() {
        let prog = parse("let x = 42").unwrap();
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn parse_function_def_produces_one_statement() {
        let prog = parse("fn f(x: N) → N { x }").unwrap();
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn parse_multiple_statements() {
        let prog = parse("let x = 1\nlet y = 2\nx + y").unwrap();
        assert_eq!(prog.statements.len(), 3);
    }

    #[test]
    fn parse_invalid_syntax_returns_error() {
        // Missing closing brace
        assert!(parse("fn f( → N { x }").is_err());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // compile_and_run() — bytecode VM pipeline
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn compile_and_run_integer_literal() {
        assert!(compile_and_run("42").is_ok());
    }

    #[test]
    fn compile_and_run_arithmetic() {
        let v = compile_and_run("6 * 7").unwrap();
        assert_eq!(v, Value::int(42));
    }

    #[test]
    fn compile_and_run_boolean() {
        assert!(compile_and_run("true").is_ok());
    }

    #[test]
    fn compile_and_run_sequence() {
        assert!(compile_and_run("σ[1, 2, 3]").is_ok());
    }

    #[test]
    fn compile_and_run_function_def() {
        assert!(compile_and_run("fn answer() → N { 42 }").is_ok());
    }

    #[test]
    fn compile_and_run_if_expr() {
        assert!(compile_and_run("if true { 1 } else { 2 }").is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Value API — accessible via re-export
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn value_void_is_not_truthy() {
        assert!(!Value::void().is_truthy());
    }

    #[test]
    fn value_int_zero_is_falsy() {
        assert!(!Value::int(0).is_truthy());
    }

    #[test]
    fn value_int_nonzero_is_truthy() {
        assert!(Value::int(1).is_truthy());
        assert!(Value::int(-1).is_truthy());
    }

    #[test]
    fn value_bool_true_is_truthy() {
        assert!(Value::bool(true).is_truthy());
    }

    #[test]
    fn value_bool_false_is_falsy() {
        assert!(!Value::bool(false).is_truthy());
    }

    #[test]
    fn value_empty_string_is_falsy() {
        assert!(!Value::string("").is_truthy());
    }

    #[test]
    fn value_nonempty_string_is_truthy() {
        assert!(Value::string("x").is_truthy());
    }

    #[test]
    fn value_is_zero_semantics() {
        assert!(Value::int(0).is_zero());
        assert!(Value::float(0.0).is_zero());
        assert!(!Value::int(1).is_zero());
    }

    #[test]
    fn value_is_one_semantics() {
        assert!(Value::int(1).is_one());
        assert!(Value::float(1.0).is_one());
        assert!(!Value::int(0).is_one());
    }

    #[test]
    fn value_equality_int() {
        assert_eq!(Value::int(42), Value::int(42));
        assert_ne!(Value::int(42), Value::int(43));
    }

    #[test]
    fn value_equality_string() {
        assert_eq!(Value::string("a"), Value::string("a"));
        assert_ne!(Value::string("a"), Value::string("b"));
    }

    #[test]
    fn value_equality_bool() {
        assert_eq!(Value::bool(true), Value::bool(true));
        assert_ne!(Value::bool(true), Value::bool(false));
    }

    #[test]
    fn value_void_equals_void() {
        assert_eq!(Value::void(), Value::void());
    }

    #[test]
    fn value_grounding_constants_include_zero_and_one() {
        let consts = Value::int(42).grounding_constants();
        assert!(consts.contains(&"0"));
        assert!(consts.contains(&"1"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PrimaError — accessible via re-export
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn prima_error_display_contains_boundary_symbol() {
        use prima::token::Span;
        let err = PrimaError::lexer(Span::default(), "test error");
        assert!(err.to_string().contains('∂'));
    }

    #[test]
    fn prima_error_undefined_contains_name() {
        let err = PrimaError::undefined("my_var");
        assert!(err.to_string().contains("my_var"));
    }

    #[test]
    fn prima_error_division_by_zero_display() {
        let err = PrimaError::DivisionByZero;
        assert!(err.to_string().contains('0'));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Lexer/Parser/Interpreter — types accessible via re-export
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn lexer_can_be_constructed_and_tokenize() {
        let mut lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        assert!(!tokens.is_empty());
    }

    #[test]
    fn parser_can_be_constructed_and_parse() {
        let mut lexer = Lexer::new("let x = 1");
        let tokens = lexer.tokenize().unwrap();
        let prog = Parser::new(tokens).parse().unwrap();
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn interpreter_can_be_constructed() {
        let _interp = Interpreter::new();
    }

    #[test]
    fn interpreter_evaluates_program() {
        let mut lexer = Lexer::new("1 + 2");
        let tokens = lexer.tokenize().unwrap();
        let prog = Parser::new(tokens).parse().unwrap();
        let result = Interpreter::new().eval_program(&prog).unwrap();
        assert_eq!(result, Value::int(3));
    }
}
