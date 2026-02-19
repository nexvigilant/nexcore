// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Virtual Machine
//!
//! Stack-based bytecode interpreter for Prima programs.
//!
//! ## Philosophy
//!
//! The VM executes bytecode through a stack-based evaluation model:
//! - **→ (Causality)**: Instruction execution causes state changes
//! - **ρ (Recursion)**: Call stack for function invocation
//! - **ς (State)**: Stack and variables hold current state
//!
//! ## Tier: T2-C (→ + ρ + ς + σ)
//!
//! ## Architecture
//!
//! ```text
//! BytecodeModule → VM → Execution
//!                   │
//!                   ├── Stack (σ[Value]) — operand stack
//!                   ├── Frames (σ[CallFrame]) — call stack
//!                   └── Globals (μ[String → Value]) — global vars
//! ```

use crate::bytecode::{BytecodeModule, Chunk, CompiledFunction, OpCode};
use crate::error::{PrimaError, PrimaResult};
use crate::stdlib::{Stdlib, StdlibKind};
use crate::value::{Value, ValueData};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// CALL FRAME — ρ (Recursion: function invocation context)
// ═══════════════════════════════════════════════════════════════════════════

/// A call frame represents an active function invocation.
///
/// ## Tier: T2-P (ρ + ς)
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// The function being executed.
    pub function: CompiledFunction,
    /// Instruction pointer (offset into chunk.code).
    pub ip: usize,
    /// Base slot for local variables on the stack.
    pub base_slot: usize,
}

impl CallFrame {
    /// Create a new call frame.
    #[must_use]
    pub fn new(function: CompiledFunction, base_slot: usize) -> Self {
        Self {
            function,
            ip: 0,
            base_slot,
        }
    }

    /// Get the current chunk.
    #[must_use]
    pub fn chunk(&self) -> &Chunk {
        &self.function.chunk
    }

    /// Read the next byte and advance IP.
    pub fn read_byte(&mut self) -> u8 {
        let byte = self.chunk().code.get(self.ip).copied().unwrap_or(0);
        self.ip += 1;
        byte
    }

    /// Read a u16 operand (big-endian) and advance IP.
    pub fn read_u16(&mut self) -> u16 {
        let hi = self.read_byte() as u16;
        let lo = self.read_byte() as u16;
        (hi << 8) | lo
    }

    /// Read an i16 operand and advance IP.
    pub fn read_i16(&mut self) -> i16 {
        self.read_u16() as i16
    }

    /// Check if execution is complete.
    #[must_use]
    pub fn is_done(&self) -> bool {
        self.ip >= self.chunk().code.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONTROL FLOW — → (Causality: execution flow control)
// ═══════════════════════════════════════════════════════════════════════════

/// Control flow result from opcode execution.
#[derive(Debug)]
enum ControlFlow {
    /// Continue to next instruction.
    Continue,
    /// Return from current function with value.
    Return(Value),
    /// Halt execution.
    Halt,
}

// ═══════════════════════════════════════════════════════════════════════════
// VM — ς (State: execution state machine)
// ═══════════════════════════════════════════════════════════════════════════

/// The Prima Virtual Machine.
///
/// Executes bytecode through stack-based evaluation.
///
/// ## Tier: T2-C (→ + ρ + ς + σ + μ)
#[derive(Debug)]
pub struct VM {
    /// Operand stack (σ[Value]).
    stack: Vec<Value>,
    /// Call frames (σ[CallFrame]).
    frames: Vec<CallFrame>,
    /// Global variables (μ[String → Value]).
    globals: HashMap<String, Value>,
    /// Captured output for testing (σ[String]).
    output: Vec<String>,
    /// Standard library.
    stdlib: Stdlib,
    /// Maximum stack depth.
    max_stack: usize,
    /// Maximum call depth.
    max_frames: usize,
    /// Cumulative differential tracking (Entropy {-})
    pub total_entropy: f64,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    /// Default stack size.
    pub const DEFAULT_STACK_SIZE: usize = 256;
    /// Default max call depth.
    pub const DEFAULT_MAX_FRAMES: usize = 64;

    /// Create a new VM.
    #[must_use]
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(Self::DEFAULT_STACK_SIZE),
            frames: Vec::with_capacity(Self::DEFAULT_MAX_FRAMES),
            globals: HashMap::new(),
            output: Vec::new(),
            stdlib: Stdlib::new(),
            max_stack: Self::DEFAULT_STACK_SIZE,
            max_frames: Self::DEFAULT_MAX_FRAMES,
            total_entropy: 0.0,
        }
    }

    /// Create a VM with custom limits.
    #[must_use]
    pub fn with_limits(max_stack: usize, max_frames: usize) -> Self {
        Self {
            stack: Vec::with_capacity(max_stack),
            frames: Vec::with_capacity(max_frames),
            globals: HashMap::new(),
            output: Vec::new(),
            stdlib: Stdlib::new(),
            max_stack,
            max_frames,
            total_entropy: 0.0,
        }
    }

    /// Get captured output.
    #[must_use]
    pub fn output(&self) -> &[String] {
        &self.output
    }

    /// Clear captured output.
    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    /// Run a bytecode module.
    pub fn run(&mut self, module: &BytecodeModule) -> PrimaResult<Value> {
        // Reset state
        self.stack.clear();
        self.frames.clear();

        // Register functions as globals
        for (name, func) in &module.functions {
            self.globals
                .insert(name.clone(), Value::builtin(name.clone()));
            // Store compiled function for later lookup
            self.globals
                .insert(format!("__fn_{name}"), self.function_to_value(func));
        }

        // Find entry point
        let entry_name = module
            .entry
            .as_ref()
            .ok_or_else(|| PrimaError::runtime("No entry point in module"))?;

        let entry_func = module.functions.get(entry_name).ok_or_else(|| {
            PrimaError::runtime(format!("Entry function '{entry_name}' not found"))
        })?;

        // Create initial call frame
        let frame = CallFrame::new(entry_func.clone(), 0);
        self.frames.push(frame);

        // Run the execution loop
        self.run_loop(module)
    }

    /// Main execution loop.
    fn run_loop(&mut self, module: &BytecodeModule) -> PrimaResult<Value> {
        while !self.frames.is_empty() {
            let flow = self.execute_instruction(module)?;
            match flow {
                ControlFlow::Continue => {}
                ControlFlow::Return(value) => {
                    // Pop the call frame
                    let frame = self.frames.pop();
                    if let Some(f) = frame {
                        // Truncate stack to base_slot
                        self.stack.truncate(f.base_slot);
                    }

                    if self.frames.is_empty() {
                        // Top-level return
                        return Ok(value);
                    } else {
                        // Push return value for caller
                        self.push(value)?;
                    }
                }
                ControlFlow::Halt => {
                    return Ok(self.pop().unwrap_or_else(|_| Value::void()));
                }
            }
        }

        // If we get here with an empty stack, return void
        Ok(self.pop().unwrap_or_else(|_| Value::void()))
    }

    /// Execute a single instruction.
    fn execute_instruction(&mut self, module: &BytecodeModule) -> PrimaResult<ControlFlow> {
        // Phase 1: Read instruction and operands from frame
        let (op, operands) = {
            let frame_idx = self.frames.len().saturating_sub(1);
            let frame = self
                .frames
                .get_mut(frame_idx)
                .ok_or_else(|| PrimaError::runtime("No active call frame"))?;

            if frame.is_done() {
                return Ok(ControlFlow::Return(Value::void()));
            }

            let byte = frame.read_byte();
            let op = OpCode::from(byte);

            // Read operands based on opcode
            let operands = match op {
                OpCode::Constant
                | OpCode::LoadGlobal
                | OpCode::StoreGlobal
                | OpCode::Jump
                | OpCode::JumpIf
                | OpCode::JumpIfNot => {
                    let u16_val = frame.read_u16();
                    (u16_val, 0u8, frame.ip, frame.base_slot)
                }
                OpCode::LoadLocal
                | OpCode::StoreLocal
                | OpCode::Call
                | OpCode::MakeSeq
                | OpCode::LoadUpvalue
                | OpCode::StoreUpvalue
                | OpCode::CloseUpvalue => {
                    let u8_val = frame.read_byte();
                    (0u16, u8_val, frame.ip, frame.base_slot)
                }
                OpCode::Loop => {
                    let u16_val = frame.read_u16();
                    (u16_val, 0u8, frame.ip, frame.base_slot)
                }
                OpCode::Closure => {
                    let func_idx = frame.read_u16();
                    let upvalue_count = frame.read_byte();
                    (func_idx, upvalue_count, frame.ip, frame.base_slot)
                }
                _ => (0u16, 0u8, frame.ip, frame.base_slot),
            };

            (op, operands)
        };

        let (u16_operand, u8_operand, _ip, base_slot) = operands;

        // Phase 2: Execute instruction using pre-read operands
        match op {
            // ─────────────────────────────────────────────────────────────────
            // Constants & Variables
            // ─────────────────────────────────────────────────────────────────
            OpCode::Constant => {
                let idx = u16_operand as usize;
                let value = self
                    .frames
                    .last()
                    .and_then(|f| f.chunk().constants.get(idx).cloned())
                    .unwrap_or_else(Value::void);
                self.push(value)?;
            }
            OpCode::True => self.push(Value::bool(true))?,
            OpCode::False => self.push(Value::bool(false))?,
            OpCode::Void => self.push(Value::void())?,

            OpCode::LoadLocal => {
                let slot = u8_operand as usize;
                let value = self
                    .stack
                    .get(base_slot + slot)
                    .cloned()
                    .unwrap_or_else(Value::void);
                self.push(value)?;
            }
            OpCode::StoreLocal => {
                let slot = u8_operand as usize;
                let value = self.pop()?;
                let idx = base_slot + slot;
                // Extend stack if needed
                while self.stack.len() <= idx {
                    self.stack.push(Value::void());
                }
                // Push the value back for expression result first (assignment returns value)
                self.push(value.clone())?;
                // Then store to local slot
                self.stack[idx] = value;
            }
            OpCode::LoadGlobal => {
                let idx = u16_operand as usize;
                let name = module.globals.get(idx).cloned().unwrap_or_default();
                let value = self.globals.get(&name).cloned().unwrap_or_else(Value::void);
                self.push(value)?;
            }
            OpCode::StoreGlobal => {
                let idx = u16_operand as usize;
                let name = module.globals.get(idx).cloned().unwrap_or_default();
                let value = self.peek(0)?.clone();
                self.globals.insert(name, value);
            }

            // ─────────────────────────────────────────────────────────────────
            // Stack Operations
            // ─────────────────────────────────────────────────────────────────
            OpCode::Pop => {
                self.pop()?;
            }
            OpCode::Dup => {
                let value = self.peek(0)?.clone();
                self.push(value)?;
            }
            OpCode::Swap => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.push(a)?;
                self.push(b)?;
            }

            // ─────────────────────────────────────────────────────────────────
            // Arithmetic
            // ─────────────────────────────────────────────────────────────────
            OpCode::Add => self.binary_op(Self::add)?,
            OpCode::Sub => {
                let b = self.pop()?;
                let a = self.pop()?;
                // Track differential (Entropy {-})
                let diff = match &b.data {
                    ValueData::Int(n) => n.abs() as f64,
                    ValueData::Float(f) => f.abs(),
                    _ => 0.0,
                };
                self.total_entropy += diff;
                // Sync with stdlib for built-in entropy() calls
                crate::stdlib::set_entropy(self.total_entropy);

                let result = Self::sub(a, b)?;
                self.push(result)?;
            }
            OpCode::Mul => self.binary_op(Self::mul)?,
            OpCode::Div => self.binary_op(Self::div)?,
            OpCode::Mod => self.binary_op(Self::modulo)?,
            OpCode::Neg => {
                let value = self.pop()?;
                self.push(Self::negate(value)?)?;
            }

            // ─────────────────────────────────────────────────────────────────
            // Comparison
            // ─────────────────────────────────────────────────────────────────
            OpCode::Eq => self.binary_op(|a, b| Ok(Value::bool(a == b)))?,
            OpCode::Ne => self.binary_op(|a, b| Ok(Value::bool(a != b)))?,
            OpCode::Lt => self.binary_op(Self::less_than)?,
            OpCode::Le => self.binary_op(Self::less_equal)?,
            OpCode::Gt => self.binary_op(Self::greater_than)?,
            OpCode::Ge => self.binary_op(Self::greater_equal)?,

            // ─────────────────────────────────────────────────────────────────
            // Logic
            // ─────────────────────────────────────────────────────────────────
            OpCode::Not => {
                let value = self.pop()?;
                self.push(Value::bool(!value.is_truthy()))?;
            }
            OpCode::And => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(Value::bool(a.is_truthy() && b.is_truthy()))?;
            }
            OpCode::Or => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(Value::bool(a.is_truthy() || b.is_truthy()))?;
            }

            // ─────────────────────────────────────────────────────────────────
            // Control Flow
            // ─────────────────────────────────────────────────────────────────
            OpCode::Jump => {
                let offset = u16_operand as i16;
                if let Some(f) = self.frames.last_mut() {
                    f.ip = Self::safe_jump(f.ip, offset, f.chunk().code.len())?;
                }
            }
            OpCode::JumpIf => {
                let offset = u16_operand as i16;
                let cond = self.peek(0)?;
                if cond.is_truthy() {
                    if let Some(f) = self.frames.last_mut() {
                        f.ip = Self::safe_jump(f.ip, offset, f.chunk().code.len())?;
                    }
                }
            }
            OpCode::JumpIfNot => {
                let offset = u16_operand as i16;
                let cond = self.peek(0)?;
                if !cond.is_truthy() {
                    if let Some(f) = self.frames.last_mut() {
                        f.ip = Self::safe_jump(f.ip, offset, f.chunk().code.len())?;
                    }
                }
            }
            OpCode::Loop => {
                let offset = u16_operand as usize;
                if let Some(f) = self.frames.last_mut() {
                    f.ip = f.ip.saturating_sub(offset);
                }
            }

            OpCode::Call => {
                let arg_count = u8_operand as usize;
                return self.call_value(arg_count, module);
            }
            OpCode::Return => {
                let value = self.pop().unwrap_or_else(|_| Value::void());
                return Ok(ControlFlow::Return(value));
            }

            // ─────────────────────────────────────────────────────────────────
            // Sequences
            // ─────────────────────────────────────────────────────────────────
            OpCode::MakeSeq => {
                let count = u8_operand as usize;
                let mut elements = Vec::with_capacity(count);
                for _ in 0..count {
                    elements.push(self.pop()?);
                }
                elements.reverse();
                self.push(Value::sequence(elements))?;
            }
            OpCode::Index => {
                let idx = self.pop()?;
                let seq = self.pop()?;
                self.push(Self::index_value(seq, idx)?)?;
            }
            OpCode::Length => {
                let seq = self.pop()?;
                self.push(Self::length_value(seq)?)?;
            }
            OpCode::Push => {
                let val = self.pop()?;
                let seq = self.pop()?;
                self.push(Self::push_value(seq, val)?)?;
            }
            OpCode::Concat => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(Self::concat_values(a, b)?)?;
            }

            // ─────────────────────────────────────────────────────────────────
            // I/O
            // ─────────────────────────────────────────────────────────────────
            OpCode::Print => {
                let value = self.pop()?;
                let s = format!("{value}");
                print!("{s}");
                self.output.push(s);
                self.push(Value::void())?;
            }
            OpCode::Println => {
                let value = self.pop()?;
                let s = format!("{value}");
                println!("{s}");
                self.output.push(s);
                self.push(Value::void())?;
            }

            // ─────────────────────────────────────────────────────────────────
            // Closures (basic support)
            // ─────────────────────────────────────────────────────────────────
            OpCode::Closure => {
                let func_idx = u16_operand;
                let _upvalue_count = u8_operand;
                // For now, just push the function index as a reference
                self.push(Value::int(func_idx as i64))?;
            }
            OpCode::LoadUpvalue => {
                let _slot = u8_operand;
                // TODO: Implement upvalue loading
                self.push(Value::void())?;
            }
            OpCode::StoreUpvalue => {
                let _slot = u8_operand;
                // TODO: Implement upvalue storing
            }
            OpCode::CloseUpvalue => {
                // TODO: Implement upvalue closing
            }

            // ─────────────────────────────────────────────────────────────────
            // Special
            // ─────────────────────────────────────────────────────────────────
            OpCode::Nop => {}
            OpCode::Halt => {
                return Ok(ControlFlow::Halt);
            }
        }

        Ok(ControlFlow::Continue)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Stack Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// Push a value onto the stack.
    fn push(&mut self, value: Value) -> PrimaResult<()> {
        if self.stack.len() >= self.max_stack {
            return Err(PrimaError::runtime("Stack overflow"));
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pop a value from the stack.
    fn pop(&mut self) -> PrimaResult<Value> {
        self.stack
            .pop()
            .ok_or_else(|| PrimaError::runtime("Stack underflow"))
    }

    /// Peek at a value on the stack.
    fn peek(&self, distance: usize) -> PrimaResult<&Value> {
        let idx = self
            .stack
            .len()
            .checked_sub(1 + distance)
            .ok_or_else(|| PrimaError::runtime("Stack underflow on peek"))?;
        self.stack
            .get(idx)
            .ok_or_else(|| PrimaError::runtime("Invalid stack index"))
    }

    /// Apply a binary operation.
    fn binary_op<F>(&mut self, op: F) -> PrimaResult<()>
    where
        F: FnOnce(Value, Value) -> PrimaResult<Value>,
    {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = op(a, b)?;
        self.push(result)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Function Calls
    // ─────────────────────────────────────────────────────────────────────────

    /// Call a value as a function.
    fn call_value(
        &mut self,
        arg_count: usize,
        module: &BytecodeModule,
    ) -> PrimaResult<ControlFlow> {
        // Get the callee (arg_count + 1 from top)
        let callee_idx = self
            .stack
            .len()
            .checked_sub(arg_count + 1)
            .ok_or_else(|| PrimaError::runtime("Stack underflow in call"))?;

        let callee = self.stack[callee_idx].clone();

        match &callee.data {
            ValueData::Builtin(name) => {
                // Check if it's a user-defined function
                if let Some(func) = module.functions.get(name) {
                    return self.call_function(func.clone(), arg_count);
                }

                // Check stdlib
                if let Some(stdlib_fn) = self.stdlib.get(name) {
                    return self.call_builtin(stdlib_fn.kind, arg_count);
                }

                Err(PrimaError::runtime(format!("Unknown function: {name}")))
            }
            ValueData::Function(fv) => {
                // Create a compiled function from FnValue (for interpreted functions)
                // This is a compatibility path
                Err(PrimaError::runtime(format!(
                    "Cannot call interpreted function '{}' from VM",
                    fv.name
                )))
            }
            _ => Err(PrimaError::runtime(format!(
                "Cannot call value: {}",
                callee
            ))),
        }
    }

    /// Call a compiled function.
    fn call_function(
        &mut self,
        func: CompiledFunction,
        arg_count: usize,
    ) -> PrimaResult<ControlFlow> {
        if arg_count != func.arity as usize {
            return Err(PrimaError::runtime(format!(
                "Expected {} arguments but got {}",
                func.arity, arg_count
            )));
        }

        if self.frames.len() >= self.max_frames {
            return Err(PrimaError::runtime("Call stack overflow"));
        }

        // Calculate base slot (where args start, minus the callee)
        let base_slot = self.stack.len() - arg_count - 1;

        // Remove the callee from stack position
        // Args are at base_slot+1..base_slot+1+arg_count
        // We want locals to start at base_slot
        self.stack.remove(base_slot);

        let frame = CallFrame::new(func, base_slot);
        self.frames.push(frame);

        Ok(ControlFlow::Continue)
    }

    /// Call a builtin function.
    fn call_builtin(&mut self, kind: StdlibKind, arg_count: usize) -> PrimaResult<ControlFlow> {
        // Collect arguments
        let mut args = Vec::with_capacity(arg_count);
        for _ in 0..arg_count {
            args.push(self.pop()?);
        }
        args.reverse();

        // Pop the callee
        self.pop()?;

        // Execute builtin
        let result = crate::stdlib::execute(kind, args)?;
        self.push(result)?;

        Ok(ControlFlow::Continue)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Value Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// Compute jump target with bounds validation.
    /// Returns error if the resulting IP would be out of bounds.
    fn safe_jump(ip: usize, offset: i16, code_len: usize) -> PrimaResult<usize> {
        let new_ip = (ip as i64) + (offset as i64);
        if new_ip < 0 || new_ip as usize > code_len {
            return Err(PrimaError::runtime(format!(
                "Jump offset {offset} from IP {ip} produces invalid target {new_ip}"
            )));
        }
        Ok(new_ip as usize)
    }

    fn function_to_value(&self, _func: &CompiledFunction) -> Value {
        // Create a placeholder value for the function
        Value::void()
    }

    fn add(a: Value, b: Value) -> PrimaResult<Value> {
        match (&a.data, &b.data) {
            (ValueData::Int(x), ValueData::Int(y)) => x
                .checked_add(*y)
                .map(Value::int)
                .ok_or(PrimaError::Overflow),
            (ValueData::Float(x), ValueData::Float(y)) => Ok(Value::float(x + y)),
            (ValueData::Int(x), ValueData::Float(y)) => Ok(Value::float(*x as f64 + y)),
            (ValueData::Float(x), ValueData::Int(y)) => Ok(Value::float(x + *y as f64)),
            (ValueData::String(x), ValueData::String(y)) => Ok(Value::string(format!("{x}{y}"))),
            _ => Err(PrimaError::runtime(format!("Cannot add {a} and {b}"))),
        }
    }

    fn sub(a: Value, b: Value) -> PrimaResult<Value> {
        match (&a.data, &b.data) {
            (ValueData::Int(x), ValueData::Int(y)) => x
                .checked_sub(*y)
                .map(Value::int)
                .ok_or(PrimaError::Overflow),
            (ValueData::Float(x), ValueData::Float(y)) => Ok(Value::float(x - y)),
            (ValueData::Int(x), ValueData::Float(y)) => Ok(Value::float(*x as f64 - y)),
            (ValueData::Float(x), ValueData::Int(y)) => Ok(Value::float(x - *y as f64)),
            _ => Err(PrimaError::runtime(format!("Cannot subtract {b} from {a}"))),
        }
    }

    fn mul(a: Value, b: Value) -> PrimaResult<Value> {
        match (&a.data, &b.data) {
            (ValueData::Int(x), ValueData::Int(y)) => x
                .checked_mul(*y)
                .map(Value::int)
                .ok_or(PrimaError::Overflow),
            (ValueData::Float(x), ValueData::Float(y)) => Ok(Value::float(x * y)),
            (ValueData::Int(x), ValueData::Float(y)) => Ok(Value::float(*x as f64 * y)),
            (ValueData::Float(x), ValueData::Int(y)) => Ok(Value::float(x * *y as f64)),
            _ => Err(PrimaError::runtime(format!("Cannot multiply {a} and {b}"))),
        }
    }

    fn div(a: Value, b: Value) -> PrimaResult<Value> {
        match (&a.data, &b.data) {
            (ValueData::Int(x), ValueData::Int(y)) => {
                if *y == 0 {
                    return Err(PrimaError::DivisionByZero);
                }
                Ok(Value::int(x / y))
            }
            (ValueData::Float(x), ValueData::Float(y)) => {
                if *y == 0.0 {
                    return Err(PrimaError::DivisionByZero);
                }
                Ok(Value::float(x / y))
            }
            (ValueData::Int(x), ValueData::Float(y)) => {
                if *y == 0.0 {
                    return Err(PrimaError::DivisionByZero);
                }
                Ok(Value::float(*x as f64 / y))
            }
            (ValueData::Float(x), ValueData::Int(y)) => {
                if *y == 0 {
                    return Err(PrimaError::DivisionByZero);
                }
                Ok(Value::float(x / *y as f64))
            }
            _ => Err(PrimaError::runtime(format!("Cannot divide {a} by {b}"))),
        }
    }

    fn modulo(a: Value, b: Value) -> PrimaResult<Value> {
        match (&a.data, &b.data) {
            (ValueData::Int(x), ValueData::Int(y)) => {
                if *y == 0 {
                    return Err(PrimaError::DivisionByZero);
                }
                Ok(Value::int(x % y))
            }
            _ => Err(PrimaError::runtime(format!(
                "Cannot compute modulo of {a} and {b}"
            ))),
        }
    }

    fn negate(value: Value) -> PrimaResult<Value> {
        match &value.data {
            ValueData::Int(n) => n.checked_neg().map(Value::int).ok_or(PrimaError::Overflow),
            ValueData::Float(f) => Ok(Value::float(-f)),
            _ => Err(PrimaError::runtime(format!("Cannot negate {value}"))),
        }
    }

    fn less_than(a: Value, b: Value) -> PrimaResult<Value> {
        match (&a.data, &b.data) {
            (ValueData::Int(x), ValueData::Int(y)) => Ok(Value::bool(x < y)),
            (ValueData::Float(x), ValueData::Float(y)) => Ok(Value::bool(x < y)),
            (ValueData::Int(x), ValueData::Float(y)) => Ok(Value::bool((*x as f64) < *y)),
            (ValueData::Float(x), ValueData::Int(y)) => Ok(Value::bool(*x < (*y as f64))),
            _ => Err(PrimaError::runtime(format!("Cannot compare {a} and {b}"))),
        }
    }

    fn less_equal(a: Value, b: Value) -> PrimaResult<Value> {
        match (&a.data, &b.data) {
            (ValueData::Int(x), ValueData::Int(y)) => Ok(Value::bool(x <= y)),
            (ValueData::Float(x), ValueData::Float(y)) => Ok(Value::bool(x <= y)),
            (ValueData::Int(x), ValueData::Float(y)) => Ok(Value::bool((*x as f64) <= *y)),
            (ValueData::Float(x), ValueData::Int(y)) => Ok(Value::bool(*x <= (*y as f64))),
            _ => Err(PrimaError::runtime(format!("Cannot compare {a} and {b}"))),
        }
    }

    fn greater_than(a: Value, b: Value) -> PrimaResult<Value> {
        match (&a.data, &b.data) {
            (ValueData::Int(x), ValueData::Int(y)) => Ok(Value::bool(x > y)),
            (ValueData::Float(x), ValueData::Float(y)) => Ok(Value::bool(x > y)),
            (ValueData::Int(x), ValueData::Float(y)) => Ok(Value::bool((*x as f64) > *y)),
            (ValueData::Float(x), ValueData::Int(y)) => Ok(Value::bool(*x > (*y as f64))),
            _ => Err(PrimaError::runtime(format!("Cannot compare {a} and {b}"))),
        }
    }

    fn greater_equal(a: Value, b: Value) -> PrimaResult<Value> {
        match (&a.data, &b.data) {
            (ValueData::Int(x), ValueData::Int(y)) => Ok(Value::bool(x >= y)),
            (ValueData::Float(x), ValueData::Float(y)) => Ok(Value::bool(x >= y)),
            (ValueData::Int(x), ValueData::Float(y)) => Ok(Value::bool((*x as f64) >= *y)),
            (ValueData::Float(x), ValueData::Int(y)) => Ok(Value::bool(*x >= (*y as f64))),
            _ => Err(PrimaError::runtime(format!("Cannot compare {a} and {b}"))),
        }
    }

    fn index_value(seq: Value, idx: Value) -> PrimaResult<Value> {
        match (&seq.data, &idx.data) {
            (ValueData::Sequence(v), ValueData::Int(i)) => {
                let i = *i as usize;
                v.get(i)
                    .cloned()
                    .ok_or_else(|| PrimaError::runtime(format!("Index {i} out of bounds")))
            }
            (ValueData::String(s), ValueData::Int(i)) => {
                let i = *i as usize;
                s.chars()
                    .nth(i)
                    .map(|c| Value::string(c.to_string()))
                    .ok_or_else(|| PrimaError::runtime(format!("Index {i} out of bounds")))
            }
            _ => Err(PrimaError::runtime(format!(
                "Cannot index {seq} with {idx}"
            ))),
        }
    }

    fn length_value(seq: Value) -> PrimaResult<Value> {
        match &seq.data {
            ValueData::Sequence(v) => Ok(Value::int(v.len() as i64)),
            ValueData::String(s) => Ok(Value::int(s.len() as i64)),
            _ => Err(PrimaError::runtime(format!("Cannot get length of {seq}"))),
        }
    }

    fn push_value(seq: Value, val: Value) -> PrimaResult<Value> {
        match seq.data {
            ValueData::Sequence(mut v) => {
                v.push(val);
                Ok(Value::sequence(v))
            }
            _ => Err(PrimaError::runtime(format!("Cannot push to {seq}"))),
        }
    }

    fn concat_values(a: Value, b: Value) -> PrimaResult<Value> {
        match (a.data, b.data) {
            (ValueData::Sequence(mut va), ValueData::Sequence(vb)) => {
                va.extend(vb);
                Ok(Value::sequence(va))
            }
            (ValueData::String(sa), ValueData::String(sb)) => {
                Ok(Value::string(format!("{sa}{sb}")))
            }
            _ => Err(PrimaError::runtime("Cannot concatenate these values")),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::CompiledFunction;

    /// Helper to create a simple function with bytecode.
    fn make_function(
        name: &str,
        arity: u8,
        code: Vec<u8>,
        constants: Vec<Value>,
    ) -> CompiledFunction {
        let mut func = CompiledFunction::new(name, arity);
        let code_len = code.len();
        func.chunk.code = code;
        func.chunk.lines = vec![1; code_len];
        func.chunk.constants = constants;
        func
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Basic Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_vm_creation() {
        let vm = VM::new();
        assert!(vm.stack.is_empty());
        assert!(vm.frames.is_empty());
    }

    #[test]
    fn test_vm_constant() {
        let func = make_function(
            "main",
            0,
            vec![
                OpCode::Constant as u8,
                0,
                0, // constant 0
                OpCode::Return as u8,
            ],
            vec![Value::int(42)],
        );

        let mut module = BytecodeModule::new();
        module.add_function(func);
        module.set_entry("main");

        let mut vm = VM::new();
        let result = vm.run(&module);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::int(42));
    }

    #[test]
    fn test_vm_arithmetic_add() {
        let func = make_function(
            "main",
            0,
            vec![
                OpCode::Constant as u8,
                0,
                0, // 10
                OpCode::Constant as u8,
                0,
                1, // 32
                OpCode::Add as u8,
                OpCode::Return as u8,
            ],
            vec![Value::int(10), Value::int(32)],
        );

        let mut module = BytecodeModule::new();
        module.add_function(func);
        module.set_entry("main");

        let mut vm = VM::new();
        let result = vm.run(&module);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::int(42));
    }

    #[test]
    fn test_vm_arithmetic_sub() {
        let func = make_function(
            "main",
            0,
            vec![
                OpCode::Constant as u8,
                0,
                0, // 50
                OpCode::Constant as u8,
                0,
                1, // 8
                OpCode::Sub as u8,
                OpCode::Return as u8,
            ],
            vec![Value::int(50), Value::int(8)],
        );

        let mut module = BytecodeModule::new();
        module.add_function(func);
        module.set_entry("main");

        let mut vm = VM::new();
        let result = vm.run(&module);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::int(42));
    }

    #[test]
    fn test_vm_arithmetic_mul() {
        let func = make_function(
            "main",
            0,
            vec![
                OpCode::Constant as u8,
                0,
                0, // 6
                OpCode::Constant as u8,
                0,
                1, // 7
                OpCode::Mul as u8,
                OpCode::Return as u8,
            ],
            vec![Value::int(6), Value::int(7)],
        );

        let mut module = BytecodeModule::new();
        module.add_function(func);
        module.set_entry("main");

        let mut vm = VM::new();
        let result = vm.run(&module);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::int(42));
    }

    #[test]
    fn test_vm_comparison() {
        let func = make_function(
            "main",
            0,
            vec![
                OpCode::Constant as u8,
                0,
                0, // 5
                OpCode::Constant as u8,
                0,
                1, // 10
                OpCode::Lt as u8,
                OpCode::Return as u8,
            ],
            vec![Value::int(5), Value::int(10)],
        );

        let mut module = BytecodeModule::new();
        module.add_function(func);
        module.set_entry("main");

        let mut vm = VM::new();
        let result = vm.run(&module);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::bool(true));
    }

    #[test]
    fn test_vm_logic() {
        let func = make_function(
            "main",
            0,
            vec![
                OpCode::True as u8,
                OpCode::False as u8,
                OpCode::Or as u8,
                OpCode::Return as u8,
            ],
            vec![],
        );

        let mut module = BytecodeModule::new();
        module.add_function(func);
        module.set_entry("main");

        let mut vm = VM::new();
        let result = vm.run(&module);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::bool(true));
    }

    #[test]
    fn test_vm_sequence() {
        let func = make_function(
            "main",
            0,
            vec![
                OpCode::Constant as u8,
                0,
                0, // 1
                OpCode::Constant as u8,
                0,
                1, // 2
                OpCode::Constant as u8,
                0,
                2, // 3
                OpCode::MakeSeq as u8,
                3,
                OpCode::Length as u8,
                OpCode::Return as u8,
            ],
            vec![Value::int(1), Value::int(2), Value::int(3)],
        );

        let mut module = BytecodeModule::new();
        module.add_function(func);
        module.set_entry("main");

        let mut vm = VM::new();
        let result = vm.run(&module);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::int(3));
    }

    #[test]
    fn test_vm_jump() {
        // Jump over a constant push, just return 42
        // Bytecode layout:
        // 0-2: Constant 42
        // 3: Jump opcode
        // 4-5: Jump offset (after reading, IP=6; offset +3 lands at 9)
        // 6-8: Constant 100 (skipped)
        // 9: Return
        let func = make_function(
            "main",
            0,
            vec![
                OpCode::Constant as u8,
                0,
                0, // 42 (offset 0-2)
                OpCode::Jump as u8,
                0,
                3, // Jump +3 bytes (IP=6 + 3 = 9, lands on Return)
                OpCode::Constant as u8,
                0,
                1,                    // 100 (offset 6-8, skipped)
                OpCode::Return as u8, // offset 9
            ],
            vec![Value::int(42), Value::int(100)],
        );

        let mut module = BytecodeModule::new();
        module.add_function(func);
        module.set_entry("main");

        let mut vm = VM::new();
        let result = vm.run(&module);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::int(42));
    }

    #[test]
    fn test_vm_jump_if_not() {
        // If false, jump to return 42, else return 100
        // JumpIfNot peeks (doesn't pop), so we need explicit Pop
        // Bytecode layout:
        // 0: False
        // 1: JumpIfNot opcode
        // 2-3: offset (after reading, IP=4; offset +5 lands at 9)
        // 4: Pop (we didn't jump, pop the false)
        // 5-7: Constant 100
        // 8: Return (skipped)
        // 9: Pop (we jumped here, pop the false)
        // 10-12: Constant 42
        // 13: Return
        let func = make_function(
            "main",
            0,
            vec![
                OpCode::False as u8,     // offset 0
                OpCode::JumpIfNot as u8, // offset 1
                0,
                5,                      // offset 2-3: jump +5 (IP=4 + 5 = 9)
                OpCode::Pop as u8,      // offset 4: pop condition (didn't jump)
                OpCode::Constant as u8, // offset 5
                0,
                1,                      // offset 6-7: constant 100
                OpCode::Return as u8,   // offset 8
                OpCode::Pop as u8,      // offset 9: pop condition (jumped here)
                OpCode::Constant as u8, // offset 10
                0,
                0,                    // offset 11-12: constant 42
                OpCode::Return as u8, // offset 13
            ],
            vec![Value::int(42), Value::int(100)],
        );

        let mut module = BytecodeModule::new();
        module.add_function(func);
        module.set_entry("main");

        let mut vm = VM::new();
        let result = vm.run(&module);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::int(42));
    }

    #[test]
    fn test_vm_division_by_zero() {
        let func = make_function(
            "main",
            0,
            vec![
                OpCode::Constant as u8,
                0,
                0, // 42
                OpCode::Constant as u8,
                0,
                1, // 0
                OpCode::Div as u8,
                OpCode::Return as u8,
            ],
            vec![Value::int(42), Value::int(0)],
        );

        let mut module = BytecodeModule::new();
        module.add_function(func);
        module.set_entry("main");

        let mut vm = VM::new();
        let result = vm.run(&module);
        assert!(result.is_err());
    }

    #[test]
    fn test_vm_locals() {
        // Store and load a local variable
        // StoreLocal peeks (doesn't pop) so we can chain, but we need
        // to understand that locals share the stack with base_slot offset.
        // For this test, just verify store+load round trips correctly.
        let func = make_function(
            "main",
            0,
            vec![
                // Push placeholder for local slot 0
                OpCode::Void as u8,
                // Push the value we want to store
                OpCode::Constant as u8,
                0,
                0, // 42
                // Store to slot 0 (this peeks and copies to slot, leaving value on stack)
                OpCode::StoreLocal as u8,
                0,
                // Pop the extra copy we pushed for the store expression result
                OpCode::Pop as u8,
                // Load from slot 0
                OpCode::LoadLocal as u8,
                0,
                OpCode::Return as u8,
            ],
            vec![Value::int(42)],
        );

        let mut module = BytecodeModule::new();
        module.add_function(func);
        module.set_entry("main");

        let mut vm = VM::new();
        let result = vm.run(&module);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::int(42));
    }

    #[test]
    fn test_vm_negation() {
        let func = make_function(
            "main",
            0,
            vec![
                OpCode::Constant as u8,
                0,
                0, // 42
                OpCode::Neg as u8,
                OpCode::Return as u8,
            ],
            vec![Value::int(42)],
        );

        let mut module = BytecodeModule::new();
        module.add_function(func);
        module.set_entry("main");

        let mut vm = VM::new();
        let result = vm.run(&module);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::int(-42));
    }

    #[test]
    fn test_vm_not() {
        let func = make_function(
            "main",
            0,
            vec![OpCode::True as u8, OpCode::Not as u8, OpCode::Return as u8],
            vec![],
        );

        let mut module = BytecodeModule::new();
        module.add_function(func);
        module.set_entry("main");

        let mut vm = VM::new();
        let result = vm.run(&module);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::bool(false));
    }

    #[test]
    fn test_vm_halt() {
        let func = make_function(
            "main",
            0,
            vec![
                OpCode::Constant as u8,
                0,
                0, // 42
                OpCode::Halt as u8,
                OpCode::Constant as u8,
                0,
                1, // 100 (never reached)
                OpCode::Return as u8,
            ],
            vec![Value::int(42), Value::int(100)],
        );

        let mut module = BytecodeModule::new();
        module.add_function(func);
        module.set_entry("main");

        let mut vm = VM::new();
        let result = vm.run(&module);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::int(42));
    }
}
