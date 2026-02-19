//! # PVDSL Abstract Syntax Tree
//!
//! AST node definitions for the PVDSL parser.

use serde::{Deserialize, Serialize};

/// A statement in PVDSL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Statement {
    /// Variable declaration: `x = 10`
    VariableDeclaration {
        /// Variable name
        identifier: String,
        /// Initial value
        value: Expression,
    },
    /// Function definition: `fn foo(a, b) { ... }`
    FunctionDef {
        /// Function name
        identifier: String,
        /// Parameter names
        params: Vec<String>,
        /// Function body
        body: Vec<Statement>,
    },
    /// Return statement: `return x`
    ReturnStatement {
        /// Optional return value
        value: Option<Expression>,
    },
    /// If statement: `if cond { ... } else { ... }`
    IfStatement {
        /// Condition expression
        condition: Expression,
        /// Then branch
        consequent: Vec<Statement>,
        /// Optional else branch
        alternate: Option<Vec<Statement>>,
    },
    /// While loop: `while cond { ... }`
    WhileStatement {
        /// Loop condition
        condition: Expression,
        /// Loop body
        body: Vec<Statement>,
    },
    /// For loop: `for x in items { ... }`
    ForStatement {
        /// Iterator variable name
        iterator: String,
        /// Iterable expression
        iterable: Expression,
        /// Loop body
        body: Vec<Statement>,
    },
    /// Expression statement: `foo()`
    ExpressionStatement {
        /// The expression
        expression: Expression,
    },
}

/// An expression in PVDSL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Expression {
    /// String literal: `"hello"`
    StringLiteral {
        /// String value
        value: String,
    },
    /// Number literal: `42` or `3.14`
    NumberLiteral {
        /// Numeric value
        value: f64,
    },
    /// Boolean literal: `true` or `false`
    BooleanLiteral {
        /// Boolean value
        value: bool,
    },
    /// Identifier: `x`
    Identifier {
        /// Variable name
        name: String,
    },
    /// Function call: `foo(a, b)` or `signal::prr(a, b, c, d)`
    FunctionCall {
        /// Optional namespace (e.g., `signal` in `signal::prr`)
        namespace: Option<String>,
        /// Function name
        identifier: String,
        /// Arguments
        arguments: Vec<Expression>,
    },
    /// Binary expression: `a + b`
    BinaryExpression {
        /// Left operand
        left: Box<Expression>,
        /// Operator (e.g., "+", "-", "==")
        operator: String,
        /// Right operand
        right: Box<Expression>,
    },
    /// List literal: `[1, 2, 3]`
    ListLiteral {
        /// List elements
        elements: Vec<Expression>,
    },
}

/// A parsed PVDSL program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    /// Top-level statements
    pub statements: Vec<Statement>,
    /// Program metadata
    pub metadata: ProgramMetadata,
}

/// Metadata about a parsed program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramMetadata {
    /// Whether the program contains function definitions
    pub has_functions: bool,
}
