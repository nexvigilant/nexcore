//! # PVDSL Engine
//!
//! High-level API for compiling and executing PVDSL scripts.

use super::bytecode::{BytecodeGenerator, CompiledProgram};
use super::error::{PvdslError, PvdslResult};
use super::lexer::Lexer;
use super::parser::Parser;
use super::runtime::RuntimeValue;
use super::vm::VirtualMachine;

/// High-level PVDSL engine for script execution
pub struct PvdslEngine {
    vm: VirtualMachine,
}

impl PvdslEngine {
    /// Create a new PVDSL engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            vm: VirtualMachine::new(),
        }
    }

    /// Compile PVDSL source to bytecode
    ///
    /// # Errors
    ///
    /// Returns an error if lexing or parsing fails.
    pub fn compile(&self, source: &str) -> PvdslResult<CompiledProgram> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;
        Ok(BytecodeGenerator::new().compile(&program))
    }

    /// Execute a compiled program
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails.
    pub fn execute(&mut self, program: &CompiledProgram) -> PvdslResult<Option<RuntimeValue>> {
        self.vm.run(program)
    }

    /// Compile and execute PVDSL source in one step
    ///
    /// # Errors
    ///
    /// Returns an error if compilation or execution fails.
    pub fn eval(&mut self, source: &str) -> PvdslResult<Option<RuntimeValue>> {
        let program = self.compile(source)?;
        self.execute(&program)
    }

    /// Set a variable in the VM
    pub fn set_variable(&mut self, name: &str, value: RuntimeValue) {
        self.vm.set_variable(name, value);
    }

    /// Get a variable from the VM
    #[must_use]
    pub fn get_variable(&self, name: &str) -> Option<&RuntimeValue> {
        self.vm.get_variable(name)
    }

    /// Evaluate an expression and return as f64
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails or result is not a number.
    pub fn eval_number(&mut self, source: &str) -> PvdslResult<f64> {
        match self.eval(source)? {
            Some(RuntimeValue::Number(n)) => Ok(n),
            Some(other) => Err(PvdslError::TypeError {
                expected: "number".into(),
                actual: format!("{other:?}"),
            }),
            None => Err(PvdslError::Execution("No return value".into())),
        }
    }

    /// Evaluate an expression and return as String
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails or result is not a string.
    pub fn eval_string(&mut self, source: &str) -> PvdslResult<String> {
        match self.eval(source)? {
            Some(RuntimeValue::String(s)) => Ok(s),
            Some(other) => Err(PvdslError::TypeError {
                expected: "string".into(),
                actual: format!("{other:?}"),
            }),
            None => Err(PvdslError::Execution("No return value".into())),
        }
    }

    /// Evaluate an expression and return as bool
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails or result is not a boolean.
    pub fn eval_bool(&mut self, source: &str) -> PvdslResult<bool> {
        match self.eval(source)? {
            Some(RuntimeValue::Boolean(b)) => Ok(b),
            Some(other) => Err(PvdslError::TypeError {
                expected: "boolean".into(),
                actual: format!("{other:?}"),
            }),
            None => Err(PvdslError::Execution("No return value".into())),
        }
    }
}

impl Default for PvdslEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_eval() {
        let mut engine = PvdslEngine::new();
        let result = engine.eval("x = 42\nreturn x").unwrap().unwrap();
        assert_eq!(result, RuntimeValue::Number(42.0));
    }

    #[test]
    fn test_engine_eval_number() {
        let mut engine = PvdslEngine::new();
        let result = engine.eval_number("x = 2 + 2\nreturn x").unwrap();
        assert!((result - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_engine_set_variable() {
        let mut engine = PvdslEngine::new();
        engine.set_variable("x", RuntimeValue::Number(100.0));
        let result = engine.eval_number("return x").unwrap();
        assert!((result - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_engine_signal_prr() {
        let mut engine = PvdslEngine::new();
        let prr = engine
            .eval_number("return signal::prr(10, 90, 100, 9800)")
            .unwrap();
        // PRR ≈ 10.89
        assert!(prr > 9.0 && prr < 12.0, "PRR was {prr}");
    }
}
