//! # PVDSL Bytecode Compiler
//!
//! Compiles AST to stack-based bytecode.

use super::ast::{Expression, Program, Statement};
use super::runtime::RuntimeValue;
use serde::{Deserialize, Serialize};

/// Bytecode operation codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpCode {
    /// Push constant from constant pool
    LoadConst(u16),
    /// Load variable from name pool
    LoadVar(u16),
    /// Store top-of-stack into variable
    StoreVar(u16),
    /// Pop and discard top of stack
    PopTop,
    /// Binary addition: pop 2, push sum
    BinaryAdd,
    /// Binary subtraction: pop 2, push difference
    BinarySub,
    /// Binary multiplication: pop 2, push product
    BinaryMul,
    /// Binary division: pop 2, push quotient
    BinaryDiv,
    /// Binary modulo: pop 2, push remainder
    BinaryMod,
    /// Compare equal: pop 2, push bool
    CompareEq,
    /// Compare not equal
    CompareNe,
    /// Compare less than
    CompareLt,
    /// Compare less than or equal
    CompareLe,
    /// Compare greater than
    CompareGt,
    /// Compare greater than or equal
    CompareGe,
    /// Logical AND
    LogicalAnd,
    /// Logical OR
    LogicalOr,
    /// Unconditional jump
    Jump(u16),
    /// Jump if top-of-stack is falsy
    JumpIfFalse(u16),
    /// Call function with N arguments
    CallFunction(u8),
    /// Call namespaced function (namespace_idx, name_idx, arg_count)
    CallNamespaced(u16, u16, u8),
    /// Return from function
    Return,
    /// Build a list from N items on stack
    BuildList(u16),
    /// No operation
    Nop,
}

/// A compiled PVDSL program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledProgram {
    /// Bytecode instructions
    pub instructions: Vec<OpCode>,
    /// Constant pool
    pub constants: Vec<RuntimeValue>,
    /// Name pool (variable/function names)
    pub names: Vec<String>,
}

/// Bytecode generator that compiles AST to bytecode
pub struct BytecodeGenerator {
    instructions: Vec<OpCode>,
    constants: Vec<RuntimeValue>,
    names: Vec<String>,
}

impl BytecodeGenerator {
    /// Create a new bytecode generator
    #[must_use]
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            names: Vec::new(),
        }
    }

    /// Compile a program AST to bytecode
    #[must_use]
    pub fn compile(mut self, program: &Program) -> CompiledProgram {
        for stmt in &program.statements {
            self.generate_statement(stmt);
        }

        CompiledProgram {
            instructions: self.instructions,
            constants: self.constants,
            names: self.names,
        }
    }

    fn generate_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VariableDeclaration { identifier, value } => {
                self.generate_expression(value);
                let name_idx = self.add_name(identifier);
                self.instructions.push(OpCode::StoreVar(name_idx));
            }
            Statement::ReturnStatement { value } => {
                if let Some(expr) = value {
                    self.generate_expression(expr);
                } else {
                    let const_idx = self.add_constant(RuntimeValue::Null);
                    self.instructions.push(OpCode::LoadConst(const_idx));
                }
                self.instructions.push(OpCode::Return);
            }
            Statement::ExpressionStatement { expression } => {
                self.generate_expression(expression);
                self.instructions.push(OpCode::PopTop);
            }
            Statement::IfStatement {
                condition,
                consequent,
                alternate,
            } => {
                self.generate_expression(condition);
                let jump_if_false_idx = self.instructions.len();
                self.instructions.push(OpCode::JumpIfFalse(0)); // placeholder

                for stmt in consequent {
                    self.generate_statement(stmt);
                }

                if let Some(alt) = alternate {
                    let jump_over_else_idx = self.instructions.len();
                    self.instructions.push(OpCode::Jump(0)); // placeholder

                    // Patch jump_if_false to here
                    let else_start = self.instructions.len() as u16;
                    self.instructions[jump_if_false_idx] = OpCode::JumpIfFalse(else_start);

                    for stmt in alt {
                        self.generate_statement(stmt);
                    }

                    // Patch jump_over_else
                    let end = self.instructions.len() as u16;
                    self.instructions[jump_over_else_idx] = OpCode::Jump(end);
                } else {
                    // Patch jump_if_false to end
                    let end = self.instructions.len() as u16;
                    self.instructions[jump_if_false_idx] = OpCode::JumpIfFalse(end);
                }
            }
            Statement::WhileStatement { condition, body } => {
                let loop_start = self.instructions.len() as u16;
                self.generate_expression(condition);
                let jump_if_false_idx = self.instructions.len();
                self.instructions.push(OpCode::JumpIfFalse(0)); // placeholder

                for stmt in body {
                    self.generate_statement(stmt);
                }
                self.instructions.push(OpCode::Jump(loop_start));

                // Patch jump_if_false
                let end = self.instructions.len() as u16;
                self.instructions[jump_if_false_idx] = OpCode::JumpIfFalse(end);
            }
            Statement::FunctionDef { .. } => {
                // Functions are handled differently in a full implementation
                // For now, skip function definitions at top level
            }
            Statement::ForStatement { .. } => {
                // For loops need list iteration support
                // Simplified: skip for now
            }
        }
    }

    fn generate_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::NumberLiteral { value } => {
                let const_idx = self.add_constant(RuntimeValue::Number(*value));
                self.instructions.push(OpCode::LoadConst(const_idx));
            }
            Expression::StringLiteral { value } => {
                let const_idx = self.add_constant(RuntimeValue::String(value.clone()));
                self.instructions.push(OpCode::LoadConst(const_idx));
            }
            Expression::BooleanLiteral { value } => {
                let const_idx = self.add_constant(RuntimeValue::Boolean(*value));
                self.instructions.push(OpCode::LoadConst(const_idx));
            }
            Expression::Identifier { name } => {
                let name_idx = self.add_name(name);
                self.instructions.push(OpCode::LoadVar(name_idx));
            }
            Expression::BinaryExpression {
                left,
                operator,
                right,
            } => {
                self.generate_expression(left);
                self.generate_expression(right);
                let op = match operator.as_str() {
                    "+" => OpCode::BinaryAdd,
                    "-" => OpCode::BinarySub,
                    "*" => OpCode::BinaryMul,
                    "/" => OpCode::BinaryDiv,
                    "%" => OpCode::BinaryMod,
                    "==" => OpCode::CompareEq,
                    "!=" => OpCode::CompareNe,
                    "<" => OpCode::CompareLt,
                    "<=" => OpCode::CompareLe,
                    ">" => OpCode::CompareGt,
                    ">=" => OpCode::CompareGe,
                    "and" | "&&" => OpCode::LogicalAnd,
                    "or" | "||" => OpCode::LogicalOr,
                    _ => OpCode::Nop,
                };
                self.instructions.push(op);
            }
            Expression::FunctionCall {
                namespace,
                identifier,
                arguments,
            } => {
                // Push arguments
                for arg in arguments {
                    self.generate_expression(arg);
                }

                if let Some(ns) = namespace {
                    let ns_idx = self.add_name(ns);
                    let name_idx = self.add_name(identifier);
                    self.instructions.push(OpCode::CallNamespaced(
                        ns_idx,
                        name_idx,
                        arguments.len() as u8,
                    ));
                } else {
                    let name_idx = self.add_name(identifier);
                    self.instructions.push(OpCode::LoadVar(name_idx));
                    self.instructions
                        .push(OpCode::CallFunction(arguments.len() as u8));
                }
            }
            Expression::ListLiteral { elements } => {
                for elem in elements {
                    self.generate_expression(elem);
                }
                self.instructions
                    .push(OpCode::BuildList(elements.len() as u16));
            }
        }
    }

    fn add_constant(&mut self, val: RuntimeValue) -> u16 {
        // Check if constant already exists
        if let Some(idx) = self.constants.iter().position(|c| c == &val) {
            return idx as u16;
        }
        self.constants.push(val);
        (self.constants.len() - 1) as u16
    }

    fn add_name(&mut self, name: &str) -> u16 {
        if let Some(idx) = self.names.iter().position(|n| n == name) {
            idx as u16
        } else {
            self.names.push(name.to_string());
            (self.names.len() - 1) as u16
        }
    }
}

impl Default for BytecodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::lexer::Lexer;
    use super::super::parser::Parser;
    use super::*;

    #[test]
    fn test_compile_basic() {
        let mut lexer = Lexer::new("x = 42");
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        let compiled = BytecodeGenerator::new().compile(&program);

        assert!(!compiled.instructions.is_empty());
        assert!(compiled.constants.contains(&RuntimeValue::Number(42.0)));
    }

    #[test]
    fn test_compile_arithmetic() {
        let mut lexer = Lexer::new("x = 1 + 2 * 3");
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        let compiled = BytecodeGenerator::new().compile(&program);

        // Should have arithmetic ops
        assert!(
            compiled
                .instructions
                .iter()
                .any(|op| matches!(op, OpCode::BinaryAdd | OpCode::BinaryMul))
        );
    }
}
