#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

//! # NexVigilant Core — PVDSL
//!
//! Pharmacovigilance Domain-Specific Language with custom bytecode compiler.
//!
//! Extracted from `nexcore-vigilance::pvdsl` for independent compilation.
//!
//! PVDSL provides a simple, domain-focused scripting language for
//! pharmacovigilance workflows with built-in support for:
//!
//! - Signal detection (PRR, ROR, IC, EBGM)
//! - Causality assessment (Naranjo)
//! - String similarity (Levenshtein)
//! - Mathematical operations
//!
//! ## Example
//!
//! ```ignore
//! use nexcore_pvdsl::PvdslEngine;
//!
//! let mut engine = PvdslEngine::new();
//!
//! // Simple arithmetic
//! let result = engine.eval_number("x = 2 + 3\nreturn x").unwrap();
//! assert_eq!(result, 5.0);
//!
//! // Signal detection
//! let prr = engine.eval_number("return signal::prr(10, 90, 100, 9800)").unwrap();
//! assert!(prr > 9.0);
//! ```
//!
//! ## Namespaces
//!
//! - `signal::*` - Signal detection (prr, ror, ic, ebgm, chi_square, fisher, sprt, maxsprt, cusum, mgps)
//! - `causality::*` - Causality assessment (naranjo, who_umc, rucam)
//! - `meddra::*` - Medical coding (levenshtein, similarity)
//! - `risk::*` - Risk analytics (sar, es, monte_carlo)
//! - `date::*` - Date operations (now, diff_days)
//! - `classify::*` - Classification (hartwig_siegel)
//! - `math::*` - Mathematical functions (abs, sqrt, pow, log, ln, exp, min, max, floor, ceil, round)
//! - `chem::*` - Chemistry-based capability assessment (arrhenius, michaelis, hill, henderson, halflife, sqi)

pub mod ast;
pub mod bytecode;
pub mod chemistry;
pub mod engine;
pub mod error;
pub mod grounding;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod text;
pub mod transpiler;
pub mod vm;

// Re-exports for convenient usage
pub use ast::{Expression, Program, Statement};
pub use bytecode::{BytecodeGenerator, CompiledProgram, OpCode};
pub use engine::PvdslEngine;
pub use error::{PvdslError, PvdslResult};
pub use lexer::{Lexer, Token, TokenType};
pub use parser::Parser;
pub use runtime::RuntimeValue;
pub use transpiler::{GvpTranspiler, RegulatoryRule};
pub use vm::VirtualMachine;

/// SQI weight constants (Skill Quality Index).
///
/// Inlined from vigilance::capabilities::sqi to avoid circular dependency.
pub mod sqi_weights {
    /// Adoption weight (Arrhenius barrier)
    pub const WEIGHT_ADOPTION: f64 = 0.20;
    /// Capacity weight (Michaelis-Menten saturation)
    pub const WEIGHT_CAPACITY: f64 = 0.25;
    /// Synergy weight (Hill cooperativity)
    pub const WEIGHT_SYNERGY: f64 = 0.20;
    /// Stability weight (Henderson-Hasselbalch)
    pub const WEIGHT_STABILITY: f64 = 0.20;
    /// Freshness weight (half-life decay)
    pub const WEIGHT_FRESHNESS: f64 = 0.15;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e2e_execution() {
        let source = "x = 42\nreturn x";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        let generator = BytecodeGenerator::new();
        let compiled = generator.compile(&program);

        let mut vm = VirtualMachine::new();
        let result = vm.run(&compiled).unwrap().unwrap();

        assert_eq!(result, RuntimeValue::Number(42.0));
    }

    #[test]
    fn test_engine_convenience() {
        let mut engine = PvdslEngine::new();
        let result = engine.eval_number("return 2 * 21").unwrap();
        assert!((result - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_signal_detection_integration() {
        let mut engine = PvdslEngine::new();

        // Test PRR calculation via PVDSL
        let prr = engine
            .eval_number("return signal::prr(10, 90, 100, 9800)")
            .unwrap();

        // PRR = (a/(a+b)) / (c/(c+d)) = (10/100) / (100/9900) ≈ 9.9
        assert!(prr > 9.0 && prr < 11.0, "PRR was {prr}");
    }

    #[test]
    fn test_string_functions() {
        let mut engine = PvdslEngine::new();
        let dist = engine
            .eval_number("return meddra::levenshtein(\"kitten\", \"sitting\")")
            .unwrap();
        assert_eq!(dist, 3.0);
    }
}

/// Wolfram Alpha validated tests (2026-01-28)
#[cfg(test)]
mod wolfram_validated {
    use super::*;

    const EPSILON: f64 = 0.01;

    fn approx(actual: f64, expected: f64, name: &str) {
        let diff = (actual - expected).abs();
        assert!(diff < EPSILON, "{name}: expected {expected}, got {actual}");
    }

    /// PRR: Wolfram `(10/100) / (100/9900)` = 9.9
    #[test]
    fn wolfram_prr() {
        let mut engine = PvdslEngine::new();
        let prr = engine
            .eval_number("return signal::prr(10, 90, 100, 9800)")
            .unwrap();
        approx(prr, 9.9, "PRR");
    }

    /// ROR: Wolfram `(10 * 9800) / (90 * 100)` = 10.889
    #[test]
    fn wolfram_ror() {
        let mut engine = PvdslEngine::new();
        let ror = engine
            .eval_number("return signal::ror(10, 90, 100, 9800)")
            .unwrap();
        approx(ror, 10.889, "ROR");
    }

    /// sqrt: Wolfram `sqrt(16)` = 4
    #[test]
    fn wolfram_sqrt() {
        let mut engine = PvdslEngine::new();
        approx(
            engine.eval_number("return math::sqrt(16)").unwrap(),
            4.0,
            "sqrt",
        );
    }

    /// pow: Wolfram `2^10` = 1024
    #[test]
    fn wolfram_pow() {
        let mut engine = PvdslEngine::new();
        approx(
            engine.eval_number("return math::pow(2, 10)").unwrap(),
            1024.0,
            "pow",
        );
    }

    /// SE: Wolfram `sqrt(1/10 - 1/100 + 1/100 - 1/9900)` = 0.316
    #[test]
    fn wolfram_prr_se() {
        let mut engine = PvdslEngine::new();
        let se = engine
            .eval_number("return math::sqrt(1/10 - 1/100 + 1/100 - 1/9900)")
            .unwrap();
        approx(se, 0.316, "PRR SE");
    }
}
