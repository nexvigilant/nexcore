//! Code generator: AST → assembly text → Program.
//!
//! Phase 6: Unified emission with frame-based function calls (enabling recursion),
//! for loops, and built-in function recognition.
//!
//! ## Frame-Based Function Calls
//!
//! Functions use memory as a call stack for variable storage:
//! - `mem[0]` = SP (next free memory address, initialized in prologue)
//! - On function entry: allocate frame at SP, bump SP
//! - On function exit: restore SP, deallocate frame
//!
//! Frame layout for `fn f(a, b)` with local `x`:
//! ```text
//! mem[SP+0] = saved_SP (for restoration on return)
//! mem[SP+1] = a (param)
//! mem[SP+2] = b (param)
//! mem[SP+3] = x (local)
//! frame_size = 4
//! ```
//!
//! Variable access: `addr = mem[0] - (frame_size - slot)`
//!
//! ## Built-in Functions
//!
//! These names map directly to VM opcodes:
//! - `abs(x)` → Abs
//! - `min(a, b)` → Min
//! - `max(a, b)` → Max
//! - `pow(a, b)` → Pow
//! - `sqrt(x)` → Sqrt
//! - `sign(x)` → Sign
//! - `clamp(val, lo, hi)` → Clamp
//!
//! Tier: T3 (→ Causality + μ Mapping + σ Sequence + ∂ Boundary + ρ Recursion + ς State)

use crate::error::{DnaError, Result};
use crate::lang::ast::{BinOp, Expr, Stmt};
use crate::program::Program;

use std::collections::HashMap;
use std::fmt::Write;

// ---------------------------------------------------------------------------
// Helper: count local variables in a function body
// ---------------------------------------------------------------------------

/// Count the number of local variable definitions in a statement list.
/// Includes nested scopes (if/while/for bodies).
/// For loops add 2 locals: the loop variable and the end-cache.
fn count_fn_locals(body: &[Stmt]) -> usize {
    body.iter()
        .map(|stmt| match stmt {
            Stmt::Let { .. } => 1,
            Stmt::For { body: inner, .. } => 2 + count_fn_locals(inner),
            Stmt::If {
                then_body,
                else_body,
                ..
            } => count_fn_locals(then_body) + count_fn_locals(else_body),
            Stmt::While { body: inner, .. } => count_fn_locals(inner),
            _ => 0,
        })
        .sum()
}

// ---------------------------------------------------------------------------
// Assembly generator
// ---------------------------------------------------------------------------

/// Unified assembly code generator.
///
/// Emits assembly text, then pipes through `asm::assemble()`.
/// Main-scope variables use flat addressing (mem[1], mem[2], ...).
/// Function variables use frame-relative addressing via mem[0] as SP.
///
/// Tier: T3 (→ + μ + σ + ∂ + ς + ρ)
struct AsmGen {
    // --- Main scope ---
    /// Variable name → flat memory address (main scope).
    vars: HashMap<String, usize>,
    /// Next available flat memory address (starts at 1, 0 is SP).
    next_addr: usize,

    // --- Functions ---
    /// Function name → label name.
    fns: HashMap<String, String>,
    /// Function name → parameter list.
    fn_params: HashMap<String, Vec<String>>,

    // --- Labels ---
    /// Counter for generating unique labels.
    label_counter: usize,

    // --- Output buffers ---
    /// Main code buffer (between entry and halt).
    main_code: String,
    /// Function bodies buffer (after halt).
    fn_bodies: String,

    // --- Current function context ---
    /// True when emitting inside a function body.
    in_function: bool,
    /// Frame size of the current function (1 + params + locals).
    current_fn_frame_size: usize,
    /// Variable name → frame slot index (current function).
    current_fn_slots: HashMap<String, usize>,
    /// Next available local slot in the current function's frame.
    next_fn_local_slot: usize,
}

impl AsmGen {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
            next_addr: 1, // 0 is reserved for SP
            fns: HashMap::new(),
            fn_params: HashMap::new(),
            label_counter: 0,
            main_code: String::new(),
            fn_bodies: String::new(),
            in_function: false,
            current_fn_frame_size: 0,
            current_fn_slots: HashMap::new(),
            next_fn_local_slot: 0,
        }
    }

    // -----------------------------------------------------------------------
    // Low-level emission
    // -----------------------------------------------------------------------

    /// Emit an instruction line to the active buffer.
    fn emit_line(&mut self, line: &str) {
        let buf = if self.in_function {
            &mut self.fn_bodies
        } else {
            &mut self.main_code
        };
        let _ = writeln!(buf, "    {line}");
    }

    /// Emit a label to the active buffer.
    fn emit_label_line(&mut self, label: &str) {
        let buf = if self.in_function {
            &mut self.fn_bodies
        } else {
            &mut self.main_code
        };
        let _ = writeln!(buf, "{label}:");
    }

    /// Generate a unique label name.
    fn fresh_label(&mut self, prefix: &str) -> String {
        let label = format!("{prefix}_{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    // -----------------------------------------------------------------------
    // Variable management
    // -----------------------------------------------------------------------

    /// Allocate a flat memory address for a main-scope variable.
    fn alloc_var(&mut self, name: &str) -> usize {
        if let Some(&addr) = self.vars.get(name) {
            return addr;
        }
        let addr = self.next_addr;
        self.next_addr += 1;
        self.vars.insert(name.to_string(), addr);
        addr
    }

    /// Define a new variable in the current scope.
    ///
    /// In main scope: allocates a flat memory address.
    /// In function scope: allocates a frame slot.
    fn define_var(&mut self, name: &str) {
        if self.in_function {
            let slot = self.next_fn_local_slot;
            self.next_fn_local_slot += 1;
            self.current_fn_slots.insert(name.to_string(), slot);
        } else {
            self.alloc_var(name);
        }
    }

    /// Emit code to load a variable's value onto the stack.
    fn emit_load_var(&mut self, name: &str) -> Result<()> {
        if self.in_function {
            // Try frame-relative first
            if let Some(&slot) = self.current_fn_slots.get(name) {
                let offset = self.current_fn_frame_size - slot;
                self.emit_line("lit 0");
                self.emit_line("load");
                self.emit_line(&format!("lit {offset}"));
                self.emit_line("sub");
                self.emit_line("load");
                return Ok(());
            }
            // Fall back to main-scope flat addressing
            if let Some(&addr) = self.vars.get(name) {
                self.emit_line(&format!("lit {addr}"));
                self.emit_line("load");
                return Ok(());
            }
            return Err(DnaError::SyntaxError(
                0,
                format!("undefined variable in function: '{name}'"),
            ));
        }

        // Main scope: flat addressing
        let &addr = self
            .vars
            .get(name)
            .ok_or_else(|| DnaError::SyntaxError(0, format!("undefined variable: '{name}'")))?;
        self.emit_line(&format!("lit {addr}"));
        self.emit_line("load");
        Ok(())
    }

    /// Emit code to store the top-of-stack value into a variable.
    fn emit_store_var(&mut self, name: &str) -> Result<()> {
        if self.in_function {
            // Try frame-relative first
            if let Some(&slot) = self.current_fn_slots.get(name) {
                let offset = self.current_fn_frame_size - slot;
                self.emit_line("lit 0");
                self.emit_line("load");
                self.emit_line(&format!("lit {offset}"));
                self.emit_line("sub");
                self.emit_line("store");
                return Ok(());
            }
            // Fall back to main-scope flat addressing
            if let Some(&addr) = self.vars.get(name) {
                self.emit_line(&format!("lit {addr}"));
                self.emit_line("store");
                return Ok(());
            }
            return Err(DnaError::SyntaxError(
                0,
                format!("undefined variable in function: '{name}'"),
            ));
        }

        // Main scope: flat addressing
        let &addr = self
            .vars
            .get(name)
            .ok_or_else(|| DnaError::SyntaxError(0, format!("undefined variable: '{name}'")))?;
        self.emit_line(&format!("lit {addr}"));
        self.emit_line("store");
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Statement emission (unified for main + function context)
    // -----------------------------------------------------------------------

    /// Emit assembly for a single statement.
    fn emit_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::ExprStmt(expr) => {
                // print() is a statement-level output builtin — handles its own output
                if let Expr::Call { name, args } = expr {
                    if name == "print" {
                        for arg in args {
                            self.emit_expr(arg)?;
                            self.emit_line("out");
                        }
                        return Ok(());
                    }
                    if name == "assert" {
                        if let Some(arg) = args.first() {
                            self.emit_expr(arg)?;
                            self.emit_line("assert");
                        }
                        return Ok(());
                    }
                }
                self.emit_expr(expr)?;
                self.emit_line("out");
            }
            Stmt::Let { name, value } => {
                self.emit_expr(value)?;
                self.define_var(name);
                self.emit_store_var(name)?;
            }
            Stmt::Assign { name, value } => {
                self.emit_expr(value)?;
                self.emit_store_var(name)?;
            }
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                self.emit_if(cond, then_body, else_body)?;
            }
            Stmt::While { cond, body } => {
                self.emit_while(cond, body)?;
            }
            Stmt::For {
                var,
                start,
                end,
                body,
            } => {
                self.emit_for(var, start, end, body)?;
            }
            Stmt::FnDef { name, params, body } => {
                self.emit_fn_def(name, params, body)?;
            }
            Stmt::Return(expr) => {
                self.emit_expr(expr)?;
                if self.in_function {
                    self.emit_fn_epilogue();
                }
                self.emit_line("ret");
            }
        }
        Ok(())
    }

    /// Emit a block of statements.
    fn emit_stmts(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            self.emit_stmt(stmt)?;
        }
        Ok(())
    }

    /// Emit if/else.
    fn emit_if(&mut self, cond: &Expr, then_body: &[Stmt], else_body: &[Stmt]) -> Result<()> {
        let else_label = self.fresh_label("else");
        let endif_label = self.fresh_label("endif");

        self.emit_expr(cond)?;

        if else_body.is_empty() {
            self.emit_line(&format!("lit @{endif_label}"));
            self.emit_line("jmpifz");
            self.emit_stmts(then_body)?;
            self.emit_label_line(&endif_label);
        } else {
            self.emit_line(&format!("lit @{else_label}"));
            self.emit_line("jmpifz");
            self.emit_stmts(then_body)?;
            self.emit_line(&format!("lit @{endif_label}"));
            self.emit_line("jmp");
            self.emit_label_line(&else_label);
            self.emit_stmts(else_body)?;
            self.emit_label_line(&endif_label);
        }

        Ok(())
    }

    /// Emit while loop.
    fn emit_while(&mut self, cond: &Expr, body: &[Stmt]) -> Result<()> {
        let while_label = self.fresh_label("while");
        let end_label = self.fresh_label("endwhile");

        self.emit_label_line(&while_label);
        self.emit_expr(cond)?;
        self.emit_line(&format!("lit @{end_label}"));
        self.emit_line("jmpifz");

        self.emit_stmts(body)?;

        self.emit_line(&format!("lit @{while_label}"));
        self.emit_line("jmp");

        self.emit_label_line(&end_label);

        Ok(())
    }

    /// Emit for loop: `for var = start to end do body end`
    ///
    /// Desugars to:
    /// ```text
    /// __end = end_expr
    /// var = start_expr
    /// while var <= __end do
    ///   body
    ///   var = var + 1
    /// end
    /// ```
    fn emit_for(&mut self, var: &str, start: &Expr, end: &Expr, body: &[Stmt]) -> Result<()> {
        // Allocate end-cache and loop variable
        let end_cache = self.fresh_label("__for_end");
        self.define_var(&end_cache);
        self.define_var(var);

        // Cache end value
        self.emit_expr(end)?;
        self.emit_store_var(&end_cache)?;

        // Initialize loop variable
        self.emit_expr(start)?;
        self.emit_store_var(var)?;

        let loop_label = self.fresh_label("for");
        let end_label = self.fresh_label("endfor");

        // Loop header
        self.emit_label_line(&loop_label);

        // Check: var <= end (exit if var > end)
        self.emit_load_var(var)?;
        self.emit_load_var(&end_cache)?;
        // Le: !(var > end) = (var > end) == 0
        self.emit_line("gt");
        self.emit_line("push0");
        self.emit_line("eq");
        self.emit_line(&format!("lit @{end_label}"));
        self.emit_line("jmpifz");

        // Body
        self.emit_stmts(body)?;

        // Increment
        self.emit_load_var(var)?;
        self.emit_line("inc");
        self.emit_store_var(var)?;

        // Loop back
        self.emit_line(&format!("lit @{loop_label}"));
        self.emit_line("jmp");

        self.emit_label_line(&end_label);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Function definition with frame-based calling convention
    // -----------------------------------------------------------------------

    /// Emit a function definition.
    ///
    /// Frame-based calling convention:
    /// - Allocate a frame at mem[SP] with slots for saved_sp, params, and locals
    /// - Pop arguments from data stack into frame slots
    /// - Body accesses variables via frame-relative addressing
    /// - On return, restore SP and deallocate frame
    fn emit_fn_def(&mut self, name: &str, params: &[String], body: &[Stmt]) -> Result<()> {
        if self.in_function {
            return Err(DnaError::SyntaxError(
                0,
                "nested function definitions not supported".into(),
            ));
        }

        let fn_label = format!("fn_{name}");
        self.fns.insert(name.to_string(), fn_label.clone());
        self.fn_params.insert(name.to_string(), params.to_vec());

        // Compute frame size
        let num_locals = count_fn_locals(body);
        let frame_size = 1 + params.len() + num_locals; // slot 0 = saved_sp

        // Build slot mapping for params
        let mut slots = HashMap::new();
        for (i, param) in params.iter().enumerate() {
            slots.insert(param.clone(), 1 + i); // slots 1, 2, ...
        }

        // Switch to function context
        self.in_function = true;
        self.current_fn_frame_size = frame_size;
        self.current_fn_slots = slots;
        self.next_fn_local_slot = 1 + params.len(); // locals start after params

        // Emit function label
        self.emit_label_line(&fn_label);

        // === Frame prologue ===
        self.emit_fn_prologue(params);

        // Emit body
        for stmt in body {
            self.emit_stmt(stmt)?;
        }

        // Implicit return (in case body doesn't end with explicit return)
        self.emit_line("push0");
        self.emit_fn_epilogue();
        self.emit_line("ret");

        // Restore main context
        self.in_function = false;
        self.current_fn_frame_size = 0;
        self.current_fn_slots = HashMap::new();
        self.next_fn_local_slot = 0;

        Ok(())
    }

    /// Emit function prologue: allocate frame, save SP, pop arguments.
    ///
    /// Stack on entry: [arg_0, arg_1, ..., arg_N-1]
    /// Stack after prologue: [] (all args stored in frame)
    fn emit_fn_prologue(&mut self, params: &[String]) {
        let frame_size = self.current_fn_frame_size;

        self.emit_line("; frame prologue");

        // Read current SP (= our frame base)
        self.emit_line("lit 0");
        self.emit_line("load"); // stack: [...args, fb]

        // Save fb at mem[fb+0] (slot 0 = saved_sp)
        self.emit_line("dup"); // [...args, fb, fb]
        self.emit_line("dup"); // [...args, fb, fb, fb]
        self.emit_line("store"); // mem[fb] = fb; [...args, fb]

        // Bump SP: mem[0] = fb + frame_size
        self.emit_line("dup"); // [...args, fb, fb]
        self.emit_line(&format!("lit {frame_size}")); // [...args, fb, fb, fs]
        self.emit_line("add"); // [...args, fb, fb+fs]
        self.emit_line("lit 0"); // [...args, fb, fb+fs, 0]
        self.emit_line("store"); // addr=0, val=fb+fs → mem[0]=fb+fs; [...args, fb]

        // Pop params from stack into frame slots (reverse order)
        // Stack: [arg_0, arg_1, ..., arg_N-1, fb]
        for i in (0..params.len()).rev() {
            let slot = 1 + i;
            self.emit_line("swap"); // [..., fb, arg_i]
            self.emit_line("over"); // [..., fb, arg_i, fb]
            self.emit_line(&format!("lit {slot}")); // [..., fb, arg_i, fb, slot]
            self.emit_line("add"); // [..., fb, arg_i, fb+slot]
            self.emit_line("store"); // mem[fb+slot] = arg_i; [..., fb]
        }

        // Discard frame base from stack
        self.emit_line("pop");
        self.emit_line("; end prologue");
    }

    /// Emit function epilogue: restore SP to deallocate frame.
    ///
    /// Stack on entry: [..., result]
    /// Stack after epilogue: [..., result]
    fn emit_fn_epilogue(&mut self) {
        let frame_size = self.current_fn_frame_size;

        self.emit_line("; frame epilogue");

        // Read saved fb from slot 0: addr = mem[0] - frame_size
        self.emit_line("lit 0");
        self.emit_line("load"); // [result, sp]
        self.emit_line(&format!("lit {frame_size}")); // [result, sp, fs]
        self.emit_line("sub"); // [result, fb]
        self.emit_line("dup"); // [result, fb, fb]
        self.emit_line("load"); // [result, fb, saved_fb]

        // Restore: mem[0] = saved_fb
        self.emit_line("lit 0"); // [result, fb, saved_fb, 0]
        self.emit_line("store"); // addr=0, val=saved_fb → mem[0]=saved_fb; [result, fb]

        // Clean up
        self.emit_line("pop"); // [result]
        self.emit_line("; end epilogue");
    }

    // -----------------------------------------------------------------------
    // Expression emission (unified)
    // -----------------------------------------------------------------------

    /// Emit code for an expression (post-order traversal).
    fn emit_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Lit(n) => {
                self.emit_line(&format!("lit {n}"));
            }
            Expr::Var(name) => {
                self.emit_load_var(name)?;
            }
            Expr::Neg(inner) => {
                self.emit_expr(inner)?;
                self.emit_line("neg");
            }
            Expr::Not(inner) => {
                self.emit_expr(inner)?;
                self.emit_line("push0");
                self.emit_line("eq");
            }
            Expr::BitNot(inner) => {
                self.emit_expr(inner)?;
                self.emit_line("bnot");
            }
            Expr::BinOp { left, op, right } => {
                self.emit_expr(left)?;
                self.emit_expr(right)?;
                self.emit_binop(*op);
            }
            Expr::Call { name, args } => {
                self.emit_call(name, args)?;
            }
        }
        Ok(())
    }

    /// Emit a function call (or built-in).
    fn emit_call(&mut self, name: &str, args: &[Expr]) -> Result<()> {
        // Check for built-in functions first
        if self.try_emit_builtin(name, args)? {
            return Ok(());
        }

        // Regular function call
        for arg in args {
            self.emit_expr(arg)?;
        }

        let fn_label = self
            .fns
            .get(name)
            .cloned()
            .ok_or_else(|| DnaError::SyntaxError(0, format!("undefined function: '{name}'")))?;
        self.emit_line(&format!("lit @{fn_label}"));
        self.emit_line("call");
        Ok(())
    }

    /// Try to emit a built-in function. Returns true if handled.
    fn try_emit_builtin(&mut self, name: &str, args: &[Expr]) -> Result<bool> {
        match (name, args.len()) {
            ("abs", 1) => {
                self.emit_expr(&args[0])?;
                self.emit_line("abs");
                Ok(true)
            }
            ("min", 2) => {
                self.emit_expr(&args[0])?;
                self.emit_expr(&args[1])?;
                self.emit_line("min");
                Ok(true)
            }
            ("max", 2) => {
                self.emit_expr(&args[0])?;
                self.emit_expr(&args[1])?;
                self.emit_line("max");
                Ok(true)
            }
            ("pow", 2) => {
                self.emit_expr(&args[0])?;
                self.emit_expr(&args[1])?;
                self.emit_line("pow");
                Ok(true)
            }
            ("sqrt", 1) => {
                self.emit_expr(&args[0])?;
                self.emit_line("sqrt");
                Ok(true)
            }
            ("sign", 1) => {
                self.emit_expr(&args[0])?;
                self.emit_line("sign");
                Ok(true)
            }
            ("clamp", 3) => {
                self.emit_expr(&args[0])?; // val
                self.emit_expr(&args[1])?; // lo
                self.emit_expr(&args[2])?; // hi
                self.emit_line("clamp");
                Ok(true)
            }
            ("log2", 1) => {
                self.emit_expr(&args[0])?;
                self.emit_line("log2");
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Emit assembly for a binary operator.
    fn emit_binop(&mut self, op: BinOp) {
        match op {
            BinOp::Add => self.emit_line("add"),
            BinOp::Sub => self.emit_line("sub"),
            BinOp::Mul => self.emit_line("mul"),
            BinOp::Div => self.emit_line("div"),
            BinOp::Mod => self.emit_line("mod"),
            BinOp::Eq => self.emit_line("eq"),
            BinOp::Neq => self.emit_line("neq"),
            BinOp::Lt => self.emit_line("lt"),
            BinOp::Gt => self.emit_line("gt"),
            BinOp::Le => {
                // a <= b → !(a > b) → (a > b) == 0
                self.emit_line("gt");
                self.emit_line("push0");
                self.emit_line("eq");
            }
            BinOp::Ge => {
                // a >= b → !(a < b) → (a < b) == 0
                self.emit_line("lt");
                self.emit_line("push0");
                self.emit_line("eq");
            }
            BinOp::BitAnd => self.emit_line("band"),
            BinOp::BitOr => self.emit_line("bor"),
            BinOp::BitXor => self.emit_line("bxor"),
            BinOp::Shl => self.emit_line("shl"),
            BinOp::Shr => self.emit_line("shr"),
            BinOp::And => self.emit_line("and"),
            BinOp::Or => self.emit_line("or"),
        }
    }

    // -----------------------------------------------------------------------
    // Assembly output
    // -----------------------------------------------------------------------

    /// Build the final assembly source text.
    fn build_assembly(&self) -> String {
        let mut asm = String::new();
        let _ = writeln!(asm, "; generated by nexcore-dna compiler");
        let _ = writeln!(asm, ".code");
        let _ = writeln!(asm, "    entry");

        // Initialize frame pointer (SP) past main-scope variables
        if !self.fns.is_empty() {
            let sp_init = self.next_addr; // first free address for frames
            let _ = writeln!(asm, "    lit {sp_init}");
            let _ = writeln!(asm, "    lit 0");
            let _ = writeln!(asm, "    store");
        }

        asm.push_str(&self.main_code);
        let _ = writeln!(asm, "    halt");

        // Function bodies after halt (only reachable via call)
        if !self.fn_bodies.is_empty() {
            let _ = writeln!(asm, "; --- functions ---");
            asm.push_str(&self.fn_bodies);
        }

        asm
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compile a list of statements into a Program.
///
/// Two-phase compilation:
/// 1. First pass: register all function definitions (forward references)
/// 2. Second pass: emit assembly for all statements
/// 3. Pipe assembly text through `asm::assemble()`
///
/// Tier: T3 (→ + μ + σ + ∂ + ς + ρ)
pub fn compile_stmts(stmts: &[Stmt]) -> Result<Program> {
    let mut cg = AsmGen::new();

    // First pass: register function definitions for forward calls
    for stmt in stmts {
        if let Stmt::FnDef { name, params, .. } = stmt {
            let fn_label = format!("fn_{name}");
            cg.fns.insert(name.clone(), fn_label);
            cg.fn_params.insert(name.clone(), params.clone());
        }
    }

    // Second pass: emit code
    for stmt in stmts {
        match stmt {
            Stmt::FnDef { name, params, body } => {
                cg.emit_fn_def(name, params, body)?;
            }
            _ => {
                cg.emit_stmt(stmt)?;
            }
        }
    }

    // Build and assemble
    let asm_source = cg.build_assembly();
    crate::asm::assemble(&asm_source)
}

/// Get the generated assembly source for debugging.
pub fn compile_to_asm(stmts: &[Stmt]) -> Result<String> {
    let mut cg = AsmGen::new();

    // First pass: register functions
    for stmt in stmts {
        if let Stmt::FnDef { name, params, .. } = stmt {
            let fn_label = format!("fn_{name}");
            cg.fns.insert(name.clone(), fn_label);
            cg.fn_params.insert(name.clone(), params.clone());
        }
    }

    // Second pass: emit code
    for stmt in stmts {
        match stmt {
            Stmt::FnDef { name, params, body } => {
                cg.emit_fn_def(name, params, body)?;
            }
            _ => {
                cg.emit_stmt(stmt)?;
            }
        }
    }

    Ok(cg.build_assembly())
}

/// Compile statements into a Genome with gene annotations.
///
/// Returns the assembled program, resolved labels, and gene annotations.
/// Used by `compiler::compile_genome()` to build annotated genomes.
pub fn compile_genome_stmts(
    stmts: &[Stmt],
) -> Result<(
    crate::program::Program,
    std::collections::HashMap<String, usize>,
    Vec<crate::gene::GeneAnnotation>,
)> {
    let mut cg = AsmGen::new();

    // First pass: register function definitions
    for stmt in stmts {
        if let Stmt::FnDef { name, params, .. } = stmt {
            let fn_label = format!("fn_{name}");
            cg.fns.insert(name.clone(), fn_label);
            cg.fn_params.insert(name.clone(), params.clone());
        }
    }

    // Collect function metadata before second pass
    let fn_meta: Vec<(String, Vec<String>, String)> = stmts
        .iter()
        .filter_map(|s| {
            if let Stmt::FnDef { name, params, .. } = s {
                let fn_label = format!("fn_{name}");
                Some((name.clone(), params.clone(), fn_label))
            } else {
                None
            }
        })
        .collect();

    // Second pass: emit code
    for stmt in stmts {
        match stmt {
            Stmt::FnDef { name, params, body } => {
                cg.emit_fn_def(name, params, body)?;
            }
            _ => {
                cg.emit_stmt(stmt)?;
            }
        }
    }

    // Build assembly and assemble with label tracking
    let asm_source = cg.build_assembly();
    let (program, labels) = crate::asm::assemble_with_labels(&asm_source)?;

    // Build gene annotations from labels
    let mut annotations = Vec::with_capacity(fn_meta.len());
    for (name, params, fn_label) in &fn_meta {
        if let Some(&codon_offset) = labels.get(fn_label.as_str()) {
            annotations.push(crate::gene::GeneAnnotation {
                name: name.clone(),
                params: params.clone(),
                start_codon: codon_offset,
            });
        }
    }

    // Sort annotations by codon position (for gene boundary detection)
    annotations.sort_by_key(|a| a.start_codon);

    Ok((program, labels, annotations))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::HaltReason;

    fn run(source: &str) -> Vec<i64> {
        let stmts = crate::lang::parser::parse(source).unwrap_or_default();
        let program = compile_stmts(&stmts)
            .unwrap_or_else(|_| Program::code_only(crate::types::Strand::new(vec![])));
        let result = program.run();
        match result {
            Ok(r) => r.output,
            Err(_) => vec![],
        }
    }

    // --- Basic expression tests (backward compat) ---

    #[test]
    fn compile_literal() {
        assert_eq!(run("42"), vec![42]);
    }

    #[test]
    fn compile_zero() {
        assert_eq!(run("0"), vec![0]);
    }

    #[test]
    fn compile_one() {
        assert_eq!(run("1"), vec![1]);
    }

    #[test]
    fn compile_neg_one() {
        assert_eq!(run("-1"), vec![-1]);
    }

    #[test]
    fn compile_addition() {
        assert_eq!(run("2 + 3"), vec![5]);
    }

    #[test]
    fn compile_subtraction() {
        assert_eq!(run("10 - 4"), vec![6]);
    }

    #[test]
    fn compile_multiplication() {
        assert_eq!(run("7 * 8"), vec![56]);
    }

    #[test]
    fn compile_division() {
        assert_eq!(run("100 / 3"), vec![33]);
    }

    #[test]
    fn compile_modulo() {
        assert_eq!(run("17 % 5"), vec![2]);
    }

    #[test]
    fn compile_precedence() {
        assert_eq!(run("2 + 3 * 4"), vec![14]);
    }

    #[test]
    fn compile_parens() {
        assert_eq!(run("(2 + 3) * 4"), vec![20]);
    }

    #[test]
    fn compile_negation() {
        assert_eq!(run("-5 + 8"), vec![3]);
    }

    #[test]
    fn compile_complex() {
        assert_eq!(run("(10 - 3) * (4 + 2) / 3"), vec![14]);
    }

    #[test]
    fn compile_multiline() {
        assert_eq!(run("2 + 3\n4 * 5"), vec![5, 20]);
    }

    #[test]
    fn compile_large_literal() {
        assert_eq!(run("1000"), vec![1000]);
    }

    #[test]
    fn compile_negative_literal() {
        assert_eq!(run("-42"), vec![-42]);
    }

    // --- Variable tests ---

    #[test]
    fn compile_let_and_use() {
        assert_eq!(run("let x = 42\nx"), vec![42]);
    }

    #[test]
    fn compile_let_arithmetic() {
        assert_eq!(run("let x = 10\nlet y = 20\nx + y"), vec![30]);
    }

    #[test]
    fn compile_assign() {
        assert_eq!(run("let x = 5\nx = 10\nx"), vec![10]);
    }

    #[test]
    fn compile_var_in_expr() {
        assert_eq!(run("let a = 3\nlet b = 4\na * a + b * b"), vec![25]);
    }

    // --- Compound assignment tests ---

    #[test]
    fn compile_plus_eq() {
        assert_eq!(run("let x = 10\nx += 5\nx"), vec![15]);
    }

    #[test]
    fn compile_minus_eq() {
        assert_eq!(run("let x = 10\nx -= 3\nx"), vec![7]);
    }

    #[test]
    fn compile_star_eq() {
        assert_eq!(run("let x = 4\nx *= 3\nx"), vec![12]);
    }

    #[test]
    fn compile_slash_eq() {
        assert_eq!(run("let x = 20\nx /= 4\nx"), vec![5]);
    }

    #[test]
    fn compile_percent_eq() {
        assert_eq!(run("let x = 17\nx %= 5\nx"), vec![2]);
    }

    #[test]
    fn compile_compound_assign_loop() {
        // The SPEC-v3 §9.1 pattern: `total += i * i`
        let source = "
let total = 0
let i = 1
for i = 1 to 5 do
  total += i * i
end
total
";
        assert_eq!(run(source), vec![55]); // 1+4+9+16+25
    }

    #[test]
    fn compile_compound_chain() {
        // Multiple compound assigns in sequence
        assert_eq!(run("let x = 100\nx -= 50\nx *= 2\nx += 1\nx"), vec![101]);
    }

    // --- Comparison tests ---

    #[test]
    fn compile_eq() {
        assert_eq!(run("5 == 5"), vec![1]);
        assert_eq!(run("5 == 3"), vec![0]);
    }

    #[test]
    fn compile_neq() {
        assert_eq!(run("5 != 3"), vec![1]);
        assert_eq!(run("5 != 5"), vec![0]);
    }

    #[test]
    fn compile_lt() {
        assert_eq!(run("3 < 5"), vec![1]);
        assert_eq!(run("5 < 3"), vec![0]);
    }

    #[test]
    fn compile_gt() {
        assert_eq!(run("5 > 3"), vec![1]);
        assert_eq!(run("3 > 5"), vec![0]);
    }

    #[test]
    fn compile_le() {
        assert_eq!(run("3 <= 5"), vec![1]);
        assert_eq!(run("5 <= 5"), vec![1]);
        assert_eq!(run("6 <= 5"), vec![0]);
    }

    #[test]
    fn compile_ge() {
        assert_eq!(run("5 >= 3"), vec![1]);
        assert_eq!(run("5 >= 5"), vec![1]);
        assert_eq!(run("4 >= 5"), vec![0]);
    }

    // --- Logic tests ---

    #[test]
    fn compile_and() {
        assert_eq!(run("1 and 1"), vec![1]);
        assert_eq!(run("1 and 0"), vec![0]);
    }

    #[test]
    fn compile_or() {
        assert_eq!(run("0 or 1"), vec![1]);
        assert_eq!(run("0 or 0"), vec![0]);
    }

    #[test]
    fn compile_not() {
        assert_eq!(run("not 0"), vec![1]);
        assert_eq!(run("not 1"), vec![0]);
        assert_eq!(run("not 42"), vec![0]);
    }

    // --- Control flow tests ---

    #[test]
    fn compile_if_true() {
        let out = run("if 1 > 0 do\n  42\nend");
        assert_eq!(out, vec![42]);
    }

    #[test]
    fn compile_if_false() {
        let out = run("if 0 > 1 do\n  42\nend");
        assert!(out.is_empty());
    }

    #[test]
    fn compile_if_else_true() {
        let out = run("if 1 > 0 do\n  42\nelse\n  0\nend");
        assert_eq!(out, vec![42]);
    }

    #[test]
    fn compile_if_else_false() {
        let out = run("if 0 > 1 do\n  42\nelse\n  99\nend");
        assert_eq!(out, vec![99]);
    }

    #[test]
    fn compile_while_countdown() {
        let source = "
let x = 5
let sum = 0
while x > 0 do
  sum = sum + x
  x = x - 1
end
sum
";
        assert_eq!(run(source), vec![15]); // 5+4+3+2+1
    }

    // --- Function tests ---

    #[test]
    fn compile_fn_simple() {
        let source = "
fn double(x) do
  return x * 2
end
double(21)
";
        assert_eq!(run(source), vec![42]);
    }

    #[test]
    fn compile_fn_two_params() {
        let source = "
fn add(a, b) do
  return a + b
end
add(10, 32)
";
        assert_eq!(run(source), vec![42]);
    }

    #[test]
    fn compile_fn_no_params() {
        let source = "
fn answer() do
  return 42
end
answer()
";
        assert_eq!(run(source), vec![42]);
    }

    #[test]
    fn compile_fn_with_locals() {
        let source = "
fn square_sum(a, b) do
  let sa = a * a
  let sb = b * b
  return sa + sb
end
square_sum(3, 4)
";
        assert_eq!(run(source), vec![25]);
    }

    #[test]
    fn compile_with_halt() {
        let stmts = crate::lang::parser::parse("42").unwrap_or_default();
        let program = compile_stmts(&stmts);
        assert!(program.is_ok());
        if let Ok(p) = program {
            let result = p.run();
            assert!(result.is_ok());
            if let Ok(r) = result {
                assert!(matches!(r.halt_reason, HaltReason::Normal));
            }
        }
    }

    // --- For loop tests ---

    #[test]
    fn compile_for_basic() {
        let source = "
let sum = 0
for i = 1 to 5 do
  sum = sum + i
end
sum
";
        assert_eq!(run(source), vec![15]); // 1+2+3+4+5
    }

    #[test]
    fn compile_for_output() {
        let source = "
for i = 1 to 3 do
  i
end
";
        assert_eq!(run(source), vec![1, 2, 3]);
    }

    #[test]
    fn compile_for_single() {
        let source = "
for i = 5 to 5 do
  i
end
";
        assert_eq!(run(source), vec![5]); // inclusive, single iteration
    }

    #[test]
    fn compile_for_empty() {
        // Start > end → no iterations
        let source = "
for i = 10 to 5 do
  i
end
42
";
        assert_eq!(run(source), vec![42]);
    }

    #[test]
    fn compile_for_nested() {
        let source = "
let sum = 0
for i = 1 to 3 do
  for j = 1 to 3 do
    sum = sum + 1
  end
end
sum
";
        assert_eq!(run(source), vec![9]); // 3*3
    }

    // --- Range syntax for-loop tests ---

    #[test]
    fn compile_for_in_range_basic() {
        let source = "
let sum = 0
for i in 1..5 do
  sum += i
end
sum
";
        assert_eq!(run(source), vec![15]); // 1+2+3+4+5
    }

    #[test]
    fn compile_for_in_range_output() {
        let source = "
for i in 1..3 do
  i
end
";
        assert_eq!(run(source), vec![1, 2, 3]);
    }

    #[test]
    fn compile_for_in_range_with_compound() {
        // The SPEC-v3 §9.1 canonical example — both features combined
        let source = "
fn sum_squares(n) do
  let total = 0
  for i in 1..n do
    total += i * i
  end
  return total
end
sum_squares(5)
";
        assert_eq!(run(source), vec![55]); // 1+4+9+16+25
    }

    // --- Built-in function tests ---

    #[test]
    fn compile_builtin_abs() {
        assert_eq!(run("abs(-42)"), vec![42]);
        assert_eq!(run("abs(42)"), vec![42]);
    }

    #[test]
    fn compile_builtin_min() {
        assert_eq!(run("min(3, 7)"), vec![3]);
        assert_eq!(run("min(10, 2)"), vec![2]);
    }

    #[test]
    fn compile_builtin_max() {
        assert_eq!(run("max(3, 7)"), vec![7]);
        assert_eq!(run("max(10, 2)"), vec![10]);
    }

    #[test]
    fn compile_builtin_pow() {
        assert_eq!(run("pow(2, 10)"), vec![1024]);
        assert_eq!(run("pow(3, 3)"), vec![27]);
    }

    #[test]
    fn compile_builtin_sqrt() {
        assert_eq!(run("sqrt(144)"), vec![12]);
        assert_eq!(run("sqrt(0)"), vec![0]);
    }

    #[test]
    fn compile_builtin_sign() {
        assert_eq!(run("sign(-42)"), vec![-1]);
        assert_eq!(run("sign(0)"), vec![0]);
        assert_eq!(run("sign(100)"), vec![1]);
    }

    #[test]
    fn compile_builtin_clamp() {
        // clamp(val, lo, hi) — 3-arg builtin, maps to AGG opcode
        assert_eq!(run("clamp(5, 0, 10)"), vec![5]); // in range
        assert_eq!(run("clamp(-3, 0, 10)"), vec![0]); // below min
        assert_eq!(run("clamp(99, 0, 10)"), vec![10]); // above max
        assert_eq!(run("clamp(0, 0, 0)"), vec![0]); // degenerate range
    }

    #[test]
    fn compile_builtin_clamp_in_expr() {
        // clamp composes with other expressions
        assert_eq!(run("clamp(50, 0, 100) + clamp(-10, 0, 100)"), vec![50]);
    }

    #[test]
    fn compile_builtin_in_expr() {
        // Built-ins compose with expressions
        assert_eq!(run("abs(-5) + max(3, 7)"), vec![12]);
    }

    #[test]
    fn compile_builtin_in_function() {
        let source = "
fn clamp_val(x) do
  return min(max(x, 0), 100)
end
clamp_val(-50)
";
        assert_eq!(run(source), vec![0]);
    }

    // --- Recursive function tests ---

    #[test]
    fn compile_recursive_factorial() {
        let source = "
fn fact(n) do
  if n <= 1 do
    return 1
  end
  return n * fact(n - 1)
end
fact(5)
";
        assert_eq!(run(source), vec![120]);
    }

    #[test]
    fn compile_recursive_fibonacci() {
        let source = "
fn fib(n) do
  if n <= 0 do
    return 0
  end
  if n == 1 do
    return 1
  end
  return fib(n - 1) + fib(n - 2)
end
fib(10)
";
        assert_eq!(run(source), vec![55]);
    }

    #[test]
    fn compile_recursive_sum() {
        let source = "
fn sum(n) do
  if n <= 0 do
    return 0
  end
  return n + sum(n - 1)
end
sum(10)
";
        assert_eq!(run(source), vec![55]); // 10+9+...+1
    }

    #[test]
    fn compile_mutual_calls() {
        // Not recursion, but two functions calling each other's results
        let source = "
fn double(x) do
  return x * 2
end
fn add_then_double(a, b) do
  return double(a + b)
end
add_then_double(3, 4)
";
        assert_eq!(run(source), vec![14]);
    }

    #[test]
    fn compile_fn_with_for() {
        let source = "
fn sum_to(n) do
  let total = 0
  for i = 1 to n do
    total = total + i
  end
  return total
end
sum_to(10)
";
        assert_eq!(run(source), vec![55]);
    }

    // --- Boolean literal tests ---

    #[test]
    fn compile_true() {
        assert_eq!(run("true"), vec![1]);
    }

    #[test]
    fn compile_false() {
        assert_eq!(run("false"), vec![0]);
    }

    #[test]
    fn compile_true_and_false() {
        assert_eq!(run("true and false"), vec![0]);
    }

    #[test]
    fn compile_true_or_false() {
        assert_eq!(run("true or false"), vec![1]);
    }

    #[test]
    fn compile_not_true() {
        assert_eq!(run("not true"), vec![0]);
    }

    #[test]
    fn compile_not_false() {
        assert_eq!(run("not false"), vec![1]);
    }

    #[test]
    fn compile_bool_in_if() {
        assert_eq!(run("if true do\n  42\nend"), vec![42]);
        assert!(run("if false do\n  42\nend").is_empty());
    }

    #[test]
    fn compile_bool_let() {
        assert_eq!(run("let flag = true\nflag"), vec![1]);
    }

    #[test]
    fn compile_bool_comparison() {
        // true == 1, false == 0
        assert_eq!(run("true == 1"), vec![1]);
        assert_eq!(run("false == 0"), vec![1]);
    }

    // --- Print builtin tests ---

    #[test]
    fn compile_print_single() {
        assert_eq!(run("print(42)"), vec![42]);
    }

    #[test]
    fn compile_print_expr() {
        assert_eq!(run("print(2 + 3)"), vec![5]);
    }

    #[test]
    fn compile_print_multi() {
        assert_eq!(run("print(1, 2, 3)"), vec![1, 2, 3]);
    }

    #[test]
    fn compile_print_var() {
        assert_eq!(run("let x = 99\nprint(x)"), vec![99]);
    }

    #[test]
    fn compile_print_bool() {
        assert_eq!(run("print(true, false)"), vec![1, 0]);
    }

    // --- Bitwise operator tests ---

    #[test]
    fn compile_bitwise_and() {
        assert_eq!(run("12 & 10"), vec![8]); // 1100 & 1010 = 1000
    }

    #[test]
    fn compile_bitwise_or() {
        assert_eq!(run("12 | 10"), vec![14]); // 1100 | 1010 = 1110
    }

    #[test]
    fn compile_bitwise_xor() {
        assert_eq!(run("12 ^ 10"), vec![6]); // 1100 ^ 1010 = 0110
    }

    #[test]
    fn compile_shift_left() {
        assert_eq!(run("1 << 4"), vec![16]);
        assert_eq!(run("3 << 2"), vec![12]);
    }

    #[test]
    fn compile_shift_right() {
        assert_eq!(run("16 >> 4"), vec![1]);
        assert_eq!(run("12 >> 2"), vec![3]);
    }

    #[test]
    fn compile_bitnot() {
        // ~0 = -1 (all bits set, two's complement)
        assert_eq!(run("~0"), vec![-1]);
        assert_eq!(run("~(-1)"), vec![0]);
    }

    #[test]
    fn compile_bitwise_combined() {
        // (0xFF & 0x0F) | (0xF0 ^ 0xFF)
        // = 0x0F | 0x0F = 0x0F = 15
        assert_eq!(run("(255 & 15) | (240 ^ 255)"), vec![15]);
    }

    #[test]
    fn compile_bitwise_in_variable() {
        assert_eq!(run("let mask = 15\nlet val = 255\nval & mask"), vec![15]);
    }

    #[test]
    fn compile_bitwise_precedence() {
        // 2 + 3 & 7 should parse as (2 + 3) & 7 = 5 & 7 = 5
        assert_eq!(run("2 + 3 & 7"), vec![5]);
    }

    // --- Assert builtin tests ---

    #[test]
    fn compile_assert_pass() {
        // assert(1) should not crash, no output
        assert_eq!(run("assert(1)\n42"), vec![42]);
    }

    #[test]
    fn compile_assert_true() {
        assert_eq!(run("assert(true)\n99"), vec![99]);
    }

    #[test]
    fn compile_assert_expression() {
        assert_eq!(run("assert(2 + 2 == 4)\n1"), vec![1]);
    }

    // --- Log2 builtin tests ---

    #[test]
    fn compile_log2_powers() {
        assert_eq!(run("log2(1)"), vec![0]);
        assert_eq!(run("log2(2)"), vec![1]);
        assert_eq!(run("log2(4)"), vec![2]);
        assert_eq!(run("log2(8)"), vec![3]);
        assert_eq!(run("log2(1024)"), vec![10]);
    }

    #[test]
    fn compile_log2_non_power() {
        // log2(7) = 2 (floor)
        assert_eq!(run("log2(7)"), vec![2]);
        assert_eq!(run("log2(100)"), vec![6]);
    }

    #[test]
    fn compile_log2_in_expr() {
        assert_eq!(run("log2(8) + log2(16)"), vec![7]); // 3 + 4
    }

    // --- Elif tests ---

    #[test]
    fn compile_elif_first_true() {
        let source = "
let x = 1
if x == 1 do
  10
elif x == 2 do
  20
else
  30
end
";
        assert_eq!(run(source), vec![10]);
    }

    #[test]
    fn compile_elif_second_true() {
        let source = "
let x = 2
if x == 1 do
  10
elif x == 2 do
  20
else
  30
end
";
        assert_eq!(run(source), vec![20]);
    }

    #[test]
    fn compile_elif_all_false_to_else() {
        let source = "
let x = 99
if x == 1 do
  10
elif x == 2 do
  20
else
  30
end
";
        assert_eq!(run(source), vec![30]);
    }

    #[test]
    fn compile_elif_deep_chain() {
        let source = "
let grade = 75
if grade >= 90 do
  4
elif grade >= 80 do
  3
elif grade >= 70 do
  2
elif grade >= 60 do
  1
else
  0
end
";
        assert_eq!(run(source), vec![2]);
    }

    #[test]
    fn compile_elif_no_else() {
        let source = "
let x = 5
if x == 1 do
  10
elif x == 5 do
  50
end
";
        assert_eq!(run(source), vec![50]);
    }
}
