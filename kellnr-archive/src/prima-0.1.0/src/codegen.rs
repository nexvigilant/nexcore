// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Code Generator
//!
//! Compiles AST to bytecode for the Prima VM.
//!
//! ## Philosophy
//!
//! The compiler transforms the AST tree to linear bytecode:
//! - **μ (Mapping)**: AST → Bytecode transformation
//! - **σ (Sequence)**: Linear instruction stream
//! - **λ (Location)**: Variable resolution and scoping
//!
//! ## Tier: T2-C (μ + σ + λ + →)
//!
//! ## Architecture
//!
//! ```text
//! Program (σ[Stmt]) → Compiler → BytecodeModule
//!                         │
//!                         ├── compile_stmt: Stmt → ()
//!                         ├── compile_expr: Expr → ()
//!                         └── emit_*: generate opcodes
//! ```

use crate::ast::{BinOp, Block, Expr, Literal, MatchArm, Param, Pattern, Program, Stmt, UnOp};
use crate::bytecode::{BytecodeModule, CompiledFunction, OpCode};
use crate::error::PrimaResult;
use crate::stdlib::Stdlib;
use crate::value::Value;

/// Helper to convert usize line to u32.
fn line(span_line: usize) -> u32 {
    span_line as u32
}

// ═══════════════════════════════════════════════════════════════════════════
// LOCAL — λ (Location: variable binding)
// ═══════════════════════════════════════════════════════════════════════════

/// A local variable in the current scope.
#[derive(Debug, Clone)]
struct Local {
    /// Variable name.
    name: String,
    /// Scope depth (0 = global, 1+ = local).
    depth: usize,
    /// Stack slot index.
    slot: u8,
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPILER — μ (Mapping: AST → Bytecode)
// ═══════════════════════════════════════════════════════════════════════════

/// Bytecode compiler for Prima programs.
///
/// Transforms AST directly to bytecode without intermediate IR.
///
/// ## Tier: T2-C (μ + σ + λ + → + ς)
pub struct Compiler {
    /// Output bytecode module.
    module: BytecodeModule,
    /// Current function being compiled.
    current: CompiledFunction,
    /// Local variables in current function.
    locals: Vec<Local>,
    /// Current scope depth.
    scope_depth: usize,
    /// Standard library reference for builtin detection.
    stdlib: Stdlib,
    /// Loop context for break/continue (start address, exit patches).
    loop_stack: Vec<LoopContext>,
}

/// Loop context for break/continue handling.
#[derive(Debug)]
struct LoopContext {
    /// Address of loop start (for continue).
    #[allow(dead_code)]
    start: usize,
    /// Addresses to patch for break statements.
    break_patches: Vec<usize>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    /// Create a new compiler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            module: BytecodeModule::new(),
            current: CompiledFunction::new("<script>", 0),
            locals: Vec::new(),
            scope_depth: 0,
            stdlib: Stdlib::new(),
            loop_stack: Vec::new(),
        }
    }

    /// Compile a program to bytecode.
    pub fn compile(&mut self, program: &Program) -> PrimaResult<BytecodeModule> {
        let n_stmts = program.statements.len();

        // Compile statements
        for (i, stmt) in program.statements.iter().enumerate() {
            let is_last = i == n_stmts - 1;

            if is_last {
                if let Stmt::Expr { expr, .. } = stmt {
                    // Last statement is an expression - don't pop, so it becomes the return value
                    self.compile_expr(expr)?;
                    self.emit_op(OpCode::Return, 0);
                } else {
                    self.compile_stmt(stmt)?;
                    self.emit_op(OpCode::Void, 0);
                    self.emit_op(OpCode::Return, 0);
                }
            } else {
                self.compile_stmt(stmt)?;
            }
        }

        if n_stmts == 0 {
            // Empty program
            self.emit_op(OpCode::Void, 0);
            self.emit_op(OpCode::Return, 0);
        }

        // Finish the main function
        let main_func = std::mem::replace(&mut self.current, CompiledFunction::new("", 0));
        self.module.add_function(main_func);
        self.module.set_entry("<script>");

        Ok(std::mem::take(&mut self.module))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Statement Compilation
    // ─────────────────────────────────────────────────────────────────────────

    /// Compile a statement.
    fn compile_stmt(&mut self, stmt: &Stmt) -> PrimaResult<()> {
        match stmt {
            Stmt::Let { name, value, span } => {
                self.compile_let(name, value, line(span.line))?;
            }
            Stmt::FnDef {
                name,
                params,
                body,
                span,
                ..
            } => {
                self.compile_fn_def(name, params, body, line(span.line))?;
            }
            Stmt::TypeDef { .. } => {
                // Type definitions are compile-time only
            }
            Stmt::Expr { expr, .. } => {
                self.compile_expr(expr)?;
                self.emit_op(OpCode::Pop, 0);
            }
            Stmt::Return { value, span } => {
                if let Some(expr) = value {
                    self.compile_expr(expr)?;
                } else {
                    self.emit_op(OpCode::Void, line(span.line));
                }
                self.emit_op(OpCode::Return, line(span.line));
            }
        }
        Ok(())
    }

    /// Compile a let binding.
    fn compile_let(&mut self, name: &str, value: &Expr, line: u32) -> PrimaResult<()> {
        self.compile_expr(value)?;

        if self.scope_depth == 0 {
            // Global variable
            let idx = self.module.add_global(name.to_string());
            self.emit_op(OpCode::StoreGlobal, line);
            self.emit_u16(idx, line);
        } else {
            // Local variable
            let slot = self.declare_local(name);
            self.emit_op(OpCode::StoreLocal, line);
            self.emit_byte(slot, line);
        }
        self.emit_op(OpCode::Pop, line);
        Ok(())
    }

    /// Compile a function definition.
    fn compile_fn_def(
        &mut self,
        name: &str,
        params: &[Param],
        body: &Block,
        line: u32,
    ) -> PrimaResult<()> {
        // Save current state
        let old_function = std::mem::replace(
            &mut self.current,
            CompiledFunction::new(name, params.len() as u8),
        );
        let old_locals = std::mem::take(&mut self.locals);
        let old_depth = self.scope_depth;

        self.scope_depth = 1;

        // Declare parameters as locals
        for param in params {
            self.declare_local(&param.name);
        }

        // Compile body
        self.compile_block(body)?;

        // Ensure function returns
        if !self.ends_with_return() {
            if body.expr.is_some() {
                self.emit_op(OpCode::Return, line);
            } else {
                self.emit_op(OpCode::Void, line);
                self.emit_op(OpCode::Return, line);
            }
        }

        // Restore state
        let func = std::mem::replace(&mut self.current, old_function);
        self.locals = old_locals;
        self.scope_depth = old_depth;

        // Add function to module
        self.module.add_function(func);

        // Register function name as global
        let idx = self.module.add_global(name.to_string());
        let const_idx = self.add_constant(Value::builtin(name.to_string()));
        self.emit_op(OpCode::Constant, line);
        self.emit_u16(const_idx, line);
        self.emit_op(OpCode::StoreGlobal, line);
        self.emit_u16(idx, line);
        self.emit_op(OpCode::Pop, line);

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Expression Compilation
    // ─────────────────────────────────────────────────────────────────────────

    /// Compile an expression.
    fn compile_expr(&mut self, expr: &Expr) -> PrimaResult<()> {
        match expr {
            Expr::Literal { value, span } => {
                self.compile_literal(value, line(span.line))?;
            }
            Expr::Ident { name, span } => {
                self.compile_ident(name, line(span.line))?;
            }
            Expr::Binary {
                left,
                op,
                right,
                span,
            } => {
                self.compile_binary(left, *op, right, line(span.line))?;
            }
            Expr::Unary { op, operand, span } => {
                self.compile_unary(*op, operand, line(span.line))?;
            }
            Expr::Call { func, args, span } => {
                self.compile_call(func, args, line(span.line))?;
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
                span,
            } => {
                self.compile_if(cond, then_branch, else_branch.as_ref(), line(span.line))?;
            }
            Expr::For {
                var,
                iter,
                body,
                span,
            } => {
                self.compile_for(var, iter, body, line(span.line))?;
            }
            Expr::Match {
                scrutinee,
                arms,
                span,
            } => {
                self.compile_match(scrutinee, arms, line(span.line))?;
            }
            Expr::Lambda { params, body, span } => {
                self.compile_lambda(params, body, line(span.line))?;
            }
            Expr::Sequence { elements, span } => {
                self.compile_sequence(elements, line(span.line))?;
            }
            Expr::Block { block, .. } => {
                self.compile_block(block)?;
            }
            Expr::Mapping { pairs, span } => {
                self.compile_mapping(pairs, line(span.line))?;
            }
            Expr::Member {
                object,
                field,
                span,
            } => {
                self.compile_member(object, field, line(span.line))?;
            }
            Expr::MethodCall {
                object,
                method,
                args,
                span,
            } => {
                self.compile_method_call(object, method, args, line(span.line))?;
            }
            // Homoiconicity expressions (basic support)
            Expr::Quoted { span, .. }
            | Expr::Quasiquoted { span, .. }
            | Expr::Unquoted { span, .. }
            | Expr::UnquotedSplice { span, .. } => {
                // For now, compile as void (homoiconicity needs interpreter)
                self.emit_op(OpCode::Void, line(span.line));
            }
        }
        Ok(())
    }

    /// Compile a literal.
    fn compile_literal(&mut self, lit: &Literal, line: u32) -> PrimaResult<()> {
        match lit {
            Literal::Int(n) => {
                let idx = self.add_constant(Value::int(*n));
                self.emit_op(OpCode::Constant, line);
                self.emit_u16(idx, line);
            }
            Literal::Float(f) => {
                let idx = self.add_constant(Value::float(*f));
                self.emit_op(OpCode::Constant, line);
                self.emit_u16(idx, line);
            }
            Literal::String(s) => {
                let idx = self.add_constant(Value::string(s.clone()));
                self.emit_op(OpCode::Constant, line);
                self.emit_u16(idx, line);
            }
            Literal::Bool(b) => {
                if *b {
                    self.emit_op(OpCode::True, line);
                } else {
                    self.emit_op(OpCode::False, line);
                }
            }
            Literal::Void => {
                self.emit_op(OpCode::Void, line);
            }
            Literal::Symbol(s) => {
                let idx = self.add_constant(Value::symbol(s.clone()));
                self.emit_op(OpCode::Constant, line);
                self.emit_u16(idx, line);
            }
        }
        Ok(())
    }

    /// Compile an identifier reference.
    fn compile_ident(&mut self, name: &str, line: u32) -> PrimaResult<()> {
        // Try local first
        if let Some(slot) = self.resolve_local(name) {
            self.emit_op(OpCode::LoadLocal, line);
            self.emit_byte(slot, line);
            return Ok(());
        }

        // Fall back to global
        let idx = self.module.add_global(name.to_string());
        self.emit_op(OpCode::LoadGlobal, line);
        self.emit_u16(idx, line);
        Ok(())
    }

    /// Compile a binary operation.
    fn compile_binary(
        &mut self,
        left: &Expr,
        op: BinOp,
        right: &Expr,
        line: u32,
    ) -> PrimaResult<()> {
        // Short-circuit evaluation for And/Or
        match op {
            BinOp::And => {
                self.compile_expr(left)?;
                let jump_patch = self.emit_jump(OpCode::JumpIfNot, line);
                self.emit_op(OpCode::Pop, line);
                self.compile_expr(right)?;
                self.patch_jump(jump_patch);
                return Ok(());
            }
            BinOp::Or => {
                self.compile_expr(left)?;
                let jump_patch = self.emit_jump(OpCode::JumpIf, line);
                self.emit_op(OpCode::Pop, line);
                self.compile_expr(right)?;
                self.patch_jump(jump_patch);
                return Ok(());
            }
            _ => {}
        }

        // Standard binary ops
        self.compile_expr(left)?;
        self.compile_expr(right)?;

        let opcode = match op {
            BinOp::Add => OpCode::Add,
            BinOp::Sub => OpCode::Sub,
            BinOp::Mul => OpCode::Mul,
            BinOp::Div => OpCode::Div,
            BinOp::Mod => OpCode::Mod,
            BinOp::Eq | BinOp::KappaEq => OpCode::Eq,
            BinOp::Ne | BinOp::KappaNe => OpCode::Ne,
            BinOp::Lt | BinOp::KappaLt => OpCode::Lt,
            BinOp::Le | BinOp::KappaLe => OpCode::Le,
            BinOp::Gt | BinOp::KappaGt => OpCode::Gt,
            BinOp::Ge | BinOp::KappaGe => OpCode::Ge,
            BinOp::And => OpCode::And, // Won't reach here
            BinOp::Or => OpCode::Or,   // Won't reach here
        };

        self.emit_op(opcode, line);
        Ok(())
    }

    /// Compile a unary operation.
    fn compile_unary(&mut self, op: UnOp, operand: &Expr, line: u32) -> PrimaResult<()> {
        self.compile_expr(operand)?;
        let opcode = match op {
            UnOp::Neg => OpCode::Neg,
            UnOp::Not => OpCode::Not,
        };
        self.emit_op(opcode, line);
        Ok(())
    }

    /// Compile a function call.
    fn compile_call(&mut self, func: &str, args: &[Expr], line: u32) -> PrimaResult<()> {
        // Check if it's a stdlib function
        let is_stdlib = self.stdlib.has(func);

        // Push function reference
        if is_stdlib {
            let idx = self.add_constant(Value::builtin(func.to_string()));
            self.emit_op(OpCode::Constant, line);
            self.emit_u16(idx, line);
        } else {
            // Load from globals
            let idx = self.module.add_global(func.to_string());
            self.emit_op(OpCode::LoadGlobal, line);
            self.emit_u16(idx, line);
        }

        // Push arguments
        for arg in args {
            self.compile_expr(arg)?;
        }

        // Emit call
        self.emit_op(OpCode::Call, line);
        self.emit_byte(args.len() as u8, line);

        Ok(())
    }

    /// Compile an if expression.
    fn compile_if(
        &mut self,
        cond: &Expr,
        then_branch: &Block,
        else_branch: Option<&Block>,
        line: u32,
    ) -> PrimaResult<()> {
        // Compile condition
        self.compile_expr(cond)?;

        // Jump to else if false (JumpIfNot peeks, so we pop after)
        let else_jump = self.emit_jump(OpCode::JumpIfNot, line);
        self.emit_op(OpCode::Pop, line); // Pop condition (we took the then branch)

        // Compile then branch
        self.compile_block(then_branch)?;

        // Jump over else branch
        let end_jump = self.emit_jump(OpCode::Jump, line);

        // Patch else jump
        self.patch_jump(else_jump);
        self.emit_op(OpCode::Pop, line); // Pop condition (we took the else branch)

        // Compile else branch
        if let Some(else_block) = else_branch {
            self.compile_block(else_block)?;
        } else {
            self.emit_op(OpCode::Void, line);
        }

        // Patch end jump
        self.patch_jump(end_jump);

        Ok(())
    }

    /// Compile a for loop.
    fn compile_for(&mut self, var: &str, iter: &Expr, body: &Block, line: u32) -> PrimaResult<()> {
        self.begin_scope();

        // Evaluate iterator
        self.compile_expr(iter)?;

        // Store iterator in a local
        let iter_slot = self.declare_local("__iter");
        self.emit_op(OpCode::StoreLocal, line);
        self.emit_byte(iter_slot, line);
        self.emit_op(OpCode::Pop, line);

        // Initialize index to 0
        let idx_slot = self.declare_local("__idx");
        let zero_idx = self.add_constant(Value::int(0));
        self.emit_op(OpCode::Constant, line);
        self.emit_u16(zero_idx, line);
        self.emit_op(OpCode::StoreLocal, line);
        self.emit_byte(idx_slot, line);
        self.emit_op(OpCode::Pop, line);

        // Declare loop variable
        let var_slot = self.declare_local(var);

        // Loop start
        let loop_start = self.current_offset();
        self.loop_stack.push(LoopContext {
            start: loop_start,
            break_patches: Vec::new(),
        });

        // Check if idx < len(iter)
        self.emit_op(OpCode::LoadLocal, line);
        self.emit_byte(idx_slot, line);
        self.emit_op(OpCode::LoadLocal, line);
        self.emit_byte(iter_slot, line);
        self.emit_op(OpCode::Length, line);
        self.emit_op(OpCode::Lt, line);

        // Exit loop if false
        let exit_jump = self.emit_jump(OpCode::JumpIfNot, line);
        self.emit_op(OpCode::Pop, line);

        // Get current element: iter[idx]
        self.emit_op(OpCode::LoadLocal, line);
        self.emit_byte(iter_slot, line);
        self.emit_op(OpCode::LoadLocal, line);
        self.emit_byte(idx_slot, line);
        self.emit_op(OpCode::Index, line);
        self.emit_op(OpCode::StoreLocal, line);
        self.emit_byte(var_slot, line);
        self.emit_op(OpCode::Pop, line);

        // Compile body
        self.compile_block(body)?;
        self.emit_op(OpCode::Pop, line);

        // Increment index
        self.emit_op(OpCode::LoadLocal, line);
        self.emit_byte(idx_slot, line);
        let one_idx = self.add_constant(Value::int(1));
        self.emit_op(OpCode::Constant, line);
        self.emit_u16(one_idx, line);
        self.emit_op(OpCode::Add, line);
        self.emit_op(OpCode::StoreLocal, line);
        self.emit_byte(idx_slot, line);
        self.emit_op(OpCode::Pop, line);

        // Jump back to loop start
        self.emit_loop(loop_start, line);

        // Patch exit
        self.patch_jump(exit_jump);
        self.emit_op(OpCode::Pop, line);

        // Patch break statements
        if let Some(ctx) = self.loop_stack.pop() {
            for patch in ctx.break_patches {
                self.patch_jump(patch);
            }
        }

        self.end_scope(line);

        // For loop produces void
        self.emit_op(OpCode::Void, line);

        Ok(())
    }

    /// Compile a match expression.
    fn compile_match(&mut self, scrutinee: &Expr, arms: &[MatchArm], line: u32) -> PrimaResult<()> {
        // Compile scrutinee
        self.compile_expr(scrutinee)?;

        // Store in a temporary local
        self.begin_scope();
        let scrutinee_slot = self.declare_local("__match");
        self.emit_op(OpCode::StoreLocal, line);
        self.emit_byte(scrutinee_slot, line);
        self.emit_op(OpCode::Pop, line);

        let mut end_jumps = Vec::new();

        for (i, arm) in arms.iter().enumerate() {
            let is_last = i == arms.len() - 1;

            // Compile pattern check
            let skip_jump = self.compile_pattern(&arm.pattern, scrutinee_slot, line)?;

            // Compile arm body
            self.compile_expr(&arm.body)?;

            // Jump to end
            let end_jump = self.emit_jump(OpCode::Jump, line);
            end_jumps.push(end_jump);

            // Patch skip jump
            if let Some(patch) = skip_jump {
                self.patch_jump(patch);
                self.emit_op(OpCode::Pop, line); // Pop comparison result
            }

            // If last arm and no match, produce void
            if is_last {
                self.emit_op(OpCode::Void, line);
            }
        }

        // Patch all end jumps
        for jump in end_jumps {
            self.patch_jump(jump);
        }

        self.end_scope(line);

        Ok(())
    }

    /// Compile a pattern match, returns optional skip jump address.
    fn compile_pattern(
        &mut self,
        pattern: &Pattern,
        scrutinee_slot: u8,
        line: u32,
    ) -> PrimaResult<Option<usize>> {
        match pattern {
            Pattern::Wildcard { .. } => {
                // Always matches
                Ok(None)
            }
            Pattern::Literal { value, .. } => {
                // Load scrutinee
                self.emit_op(OpCode::LoadLocal, line);
                self.emit_byte(scrutinee_slot, line);

                // Load literal
                self.compile_literal(value, line)?;

                // Compare
                self.emit_op(OpCode::Eq, line);

                // Jump if not equal
                let skip = self.emit_jump(OpCode::JumpIfNot, line);
                self.emit_op(OpCode::Pop, line);

                Ok(Some(skip))
            }
            Pattern::Ident { name, .. } => {
                // Bind scrutinee to name
                let slot = self.declare_local(name);
                self.emit_op(OpCode::LoadLocal, line);
                self.emit_byte(scrutinee_slot, line);
                self.emit_op(OpCode::StoreLocal, line);
                self.emit_byte(slot, line);
                self.emit_op(OpCode::Pop, line);
                Ok(None)
            }
            Pattern::Constructor { .. } => {
                // Constructor patterns need more work
                Ok(None)
            }
        }
    }

    /// Compile a lambda expression.
    fn compile_lambda(&mut self, params: &[Param], body: &Expr, line: u32) -> PrimaResult<()> {
        // Generate unique lambda name
        let lambda_name = format!("__lambda_{}", self.current.chunk.len());

        // Save current state
        let old_function = std::mem::replace(
            &mut self.current,
            CompiledFunction::new(&lambda_name, params.len() as u8),
        );
        let old_locals = std::mem::take(&mut self.locals);
        let old_depth = self.scope_depth;

        self.scope_depth = 1;

        // Declare parameters as locals
        for param in params {
            self.declare_local(&param.name);
        }

        // Compile body expression
        self.compile_expr(body)?;
        self.emit_op(OpCode::Return, line);

        // Restore state
        let func = std::mem::replace(&mut self.current, old_function);
        self.locals = old_locals;
        self.scope_depth = old_depth;

        // Add function to module
        self.module.add_function(func);

        // Push function reference
        let idx = self.add_constant(Value::builtin(lambda_name.clone()));
        self.emit_op(OpCode::Constant, line);
        self.emit_u16(idx, line);

        // Register lambda as global
        let global_idx = self.module.add_global(lambda_name);
        self.emit_op(OpCode::Dup, line);
        self.emit_op(OpCode::StoreGlobal, line);
        self.emit_u16(global_idx, line);
        self.emit_op(OpCode::Pop, line);

        Ok(())
    }

    /// Compile a sequence literal.
    fn compile_sequence(&mut self, elements: &[Expr], line: u32) -> PrimaResult<()> {
        for elem in elements {
            self.compile_expr(elem)?;
        }
        self.emit_op(OpCode::MakeSeq, line);
        self.emit_byte(elements.len() as u8, line);
        Ok(())
    }

    /// Compile a block.
    fn compile_block(&mut self, block: &Block) -> PrimaResult<()> {
        self.begin_scope();

        for stmt in &block.statements {
            self.compile_stmt(stmt)?;
        }

        if let Some(expr) = &block.expr {
            self.compile_expr(expr)?;
        } else {
            self.emit_op(OpCode::Void, line(block.span.line));
        }

        self.end_scope(line(block.span.line));
        Ok(())
    }

    /// Compile a mapping literal.
    fn compile_mapping(&mut self, pairs: &[(Expr, Expr)], line: u32) -> PrimaResult<()> {
        // For now, compile as sequence of pairs
        for (key, value) in pairs {
            self.compile_expr(key)?;
            self.compile_expr(value)?;
            self.emit_op(OpCode::MakeSeq, line);
            self.emit_byte(2, line);
        }
        self.emit_op(OpCode::MakeSeq, line);
        self.emit_byte(pairs.len() as u8, line);
        Ok(())
    }

    /// Compile member access.
    fn compile_member(&mut self, object: &Expr, _field: &str, line: u32) -> PrimaResult<()> {
        // For now, just compile the object (field access needs runtime support)
        self.compile_expr(object)?;
        // TODO: Implement proper field access
        self.emit_op(OpCode::Pop, line);
        self.emit_op(OpCode::Void, line);
        Ok(())
    }

    /// Compile method call.
    fn compile_method_call(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
        line: u32,
    ) -> PrimaResult<()> {
        // Compile as function call with object as first arg
        // method(object, args...)
        let is_stdlib = self.stdlib.has(method);

        if is_stdlib {
            let idx = self.add_constant(Value::builtin(method.to_string()));
            self.emit_op(OpCode::Constant, line);
            self.emit_u16(idx, line);
        } else {
            let idx = self.module.add_global(method.to_string());
            self.emit_op(OpCode::LoadGlobal, line);
            self.emit_u16(idx, line);
        }

        // Object is first argument
        self.compile_expr(object)?;

        // Then other arguments
        for arg in args {
            self.compile_expr(arg)?;
        }

        self.emit_op(OpCode::Call, line);
        self.emit_byte((args.len() + 1) as u8, line);

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Scope Management
    // ─────────────────────────────────────────────────────────────────────────

    /// Begin a new scope.
    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    /// End the current scope.
    fn end_scope(&mut self, _line: u32) {
        self.scope_depth -= 1;

        // Remove locals from this scope
        while let Some(local) = self.locals.last() {
            if local.depth <= self.scope_depth {
                break;
            }
            self.locals.pop();
        }
    }

    /// Declare a local variable.
    fn declare_local(&mut self, name: &str) -> u8 {
        let slot = self.locals.len() as u8;
        self.locals.push(Local {
            name: name.to_string(),
            depth: self.scope_depth,
            slot,
        });
        self.current.local_count = self.current.local_count.max(slot + 1);
        slot
    }

    /// Resolve a local variable.
    fn resolve_local(&self, name: &str) -> Option<u8> {
        for local in self.locals.iter().rev() {
            if local.name == name {
                return Some(local.slot);
            }
        }
        None
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Emit Helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Get current bytecode offset.
    fn current_offset(&self) -> usize {
        self.current.chunk.len()
    }

    /// Emit an opcode.
    fn emit_op(&mut self, op: OpCode, line: u32) {
        self.current.chunk.write_op(op, line);
    }

    /// Emit a byte.
    fn emit_byte(&mut self, byte: u8, line: u32) {
        self.current.chunk.write(byte, line);
    }

    /// Emit a u16.
    fn emit_u16(&mut self, value: u16, line: u32) {
        self.current.chunk.write_u16(value, line);
    }

    /// Add a constant and return its index.
    fn add_constant(&mut self, value: Value) -> u16 {
        self.current.chunk.add_constant(value)
    }

    /// Emit a jump instruction and return the patch address.
    fn emit_jump(&mut self, op: OpCode, line: u32) -> usize {
        self.emit_op(op, line);
        let patch = self.current_offset();
        self.emit_u16(0xFFFF, line); // Placeholder
        patch
    }

    /// Patch a jump instruction.
    fn patch_jump(&mut self, patch: usize) {
        let current = self.current_offset();
        let jump = (current as i32 - patch as i32 - 2) as i16;
        self.current.chunk.patch_u16(patch, jump as u16);
    }

    /// Emit a loop instruction (jump backward).
    fn emit_loop(&mut self, target: usize, line: u32) {
        self.emit_op(OpCode::Loop, line);
        let offset = self.current_offset() - target + 2;
        self.emit_u16(offset as u16, line);
    }

    /// Check if function ends with return.
    fn ends_with_return(&self) -> bool {
        let code = &self.current.chunk.code;
        if code.is_empty() {
            return false;
        }
        let last = code.last().copied().unwrap_or(0);
        last == OpCode::Return as u8
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::vm::VM;

    /// Helper to compile source code.
    fn compile(source: &str) -> PrimaResult<BytecodeModule> {
        let tokens = Lexer::new(source).tokenize()?;
        let program = Parser::new(tokens).parse()?;
        Compiler::new().compile(&program)
    }

    /// Helper to compile and run source code.
    fn run(source: &str) -> PrimaResult<Value> {
        let module = compile(source)?;
        VM::new().run(&module)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Literal Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_compile_int() {
        let result = compile("42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_bool() {
        let result = compile("true");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_string() {
        let result = compile("\"hello\"");
        assert!(result.is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Arithmetic Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_run_add() {
        let result = run("1 + 2");
        assert!(result.is_ok());
        // Note: The result is void because we don't have a final expression
        // In a proper setup, the last expression would be the return value
    }

    #[test]
    fn test_compile_arithmetic() {
        let result = compile("1 + 2 * 3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_comparison() {
        let result = compile("1 < 2");
        assert!(result.is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Variable Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_compile_let() {
        let result = compile("let x = 42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_let_reference() {
        let result = compile("let x = 42\nx");
        assert!(result.is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Function Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_compile_function() {
        let result = compile("fn add(a: N, b: N) → N { a + b }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_function_call() {
        let result = compile("fn f() → N { 42 }\nf()");
        assert!(result.is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Control Flow Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_compile_if() {
        let result = compile("if true { 1 } else { 2 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_if_no_else() {
        let result = compile("if true { 1 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_for() {
        let result = compile("for i in σ[1, 2, 3] { i }");
        assert!(result.is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Match Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_compile_match() {
        // Prima uses → for match arms (or Σ for the keyword)
        let result = compile("match 1 { 1 → 10, _ → 0 }");
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Lambda Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_compile_lambda() {
        let result = compile("|x| x + 1");
        assert!(result.is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Sequence Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_compile_sequence() {
        let result = compile("σ[1, 2, 3]");
        assert!(result.is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Integration Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_compile_module_has_entry() {
        let module = compile("42");
        assert!(module.is_ok());
        let module = module.ok().unwrap_or_else(BytecodeModule::new);
        assert!(module.entry.is_some());
        assert_eq!(module.entry.as_deref(), Some("<script>"));
    }

    #[test]
    fn test_compile_function_in_module() {
        let module = compile("fn answer() → N { 42 }");
        assert!(module.is_ok());
        let module = module.ok().unwrap_or_else(BytecodeModule::new);
        assert!(module.functions.contains_key("answer"));
    }
}
