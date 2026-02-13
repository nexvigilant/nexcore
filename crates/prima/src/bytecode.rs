// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Bytecode
//!
//! Stack-based bytecode for the Prima VM.
//!
//! ## Philosophy
//!
//! The bytecode provides a portable, executable representation:
//! - **Stack machine**: Simpler than register allocation
//! - **Compact**: Byte-sized opcodes minimize size
//! - **Verifiable**: Can validate before execution
//!
//! ## Tier: T2-P (σ + ρ + N)
//!
//! ## Architecture
//!
//! ```text
//! IR → Bytecode Compiler → Chunk
//!                            │
//!                            ├── Constants (σ[Value])
//!                            ├── Code (σ[u8])
//!                            └── Lines (σ[u32]) for debug
//! ```
//!
//! Stack operations follow LIFO (ρ: recursive structure).

use crate::ir::{BasicBlock, Instruction, IrConst, IrFunction, IrModule, Terminator};
use crate::value::Value;
use lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// OPCODE — N (Numeric instruction codes)
// ═══════════════════════════════════════════════════════════════════════════

/// Bytecode operation codes.
///
/// Each opcode is a single byte followed by optional operands.
///
/// ## Tier: T1 (N — Quantity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpCode {
    // ─────────────────────────────────────────────────────────────────────
    // Constants & Variables (λ + ς)
    // ─────────────────────────────────────────────────────────────────────
    /// Push constant from pool: [idx:u16]
    Constant = 0x00,
    /// Push true (1)
    True = 0x01,
    /// Push false (0)
    False = 0x02,
    /// Push void (∅)
    Void = 0x03,

    /// Load local variable: [slot:u8]
    LoadLocal = 0x10,
    /// Store to local variable: [slot:u8]
    StoreLocal = 0x11,
    /// Load global variable: [name_idx:u16]
    LoadGlobal = 0x12,
    /// Store to global variable: [name_idx:u16]
    StoreGlobal = 0x13,

    // ─────────────────────────────────────────────────────────────────────
    // Stack Operations (ρ — Recursion/Stack)
    // ─────────────────────────────────────────────────────────────────────
    /// Pop top of stack
    Pop = 0x20,
    /// Duplicate top of stack
    Dup = 0x21,
    /// Swap top two stack elements
    Swap = 0x22,

    // ─────────────────────────────────────────────────────────────────────
    // Arithmetic (μ — Mapping: N×N→N)
    // ─────────────────────────────────────────────────────────────────────
    /// Add: pop b, pop a, push a+b
    Add = 0x30,
    /// Subtract: pop b, pop a, push a-b
    Sub = 0x31,
    /// Multiply: pop b, pop a, push a*b
    Mul = 0x32,
    /// Divide: pop b, pop a, push a/b
    Div = 0x33,
    /// Modulo: pop b, pop a, push a%b
    Mod = 0x34,
    /// Negate: pop a, push -a
    Neg = 0x35,

    // ─────────────────────────────────────────────────────────────────────
    // Comparison (κ — Compare)
    // ─────────────────────────────────────────────────────────────────────
    /// Equal: pop b, pop a, push a == b
    Eq = 0x40,
    /// Not equal: pop b, pop a, push a != b
    Ne = 0x41,
    /// Less than: pop b, pop a, push a < b
    Lt = 0x42,
    /// Less or equal: pop b, pop a, push a <= b
    Le = 0x43,
    /// Greater than: pop b, pop a, push a > b
    Gt = 0x44,
    /// Greater or equal: pop b, pop a, push a >= b
    Ge = 0x45,

    // ─────────────────────────────────────────────────────────────────────
    // Logic (∂ — Boundary decisions)
    // ─────────────────────────────────────────────────────────────────────
    /// Logical not: pop a, push !a
    Not = 0x50,
    /// Logical and: pop b, pop a, push a && b
    And = 0x51,
    /// Logical or: pop b, pop a, push a || b
    Or = 0x52,

    // ─────────────────────────────────────────────────────────────────────
    // Control Flow (→ — Causality)
    // ─────────────────────────────────────────────────────────────────────
    /// Unconditional jump: [offset:i16]
    Jump = 0x60,
    /// Jump if top is truthy: [offset:i16]
    JumpIf = 0x61,
    /// Jump if top is falsy: [offset:i16]
    JumpIfNot = 0x62,
    /// Jump back (loop): [offset:u16]
    Loop = 0x63,

    /// Call function: [arg_count:u8]
    Call = 0x70,
    /// Return from function
    Return = 0x71,

    // ─────────────────────────────────────────────────────────────────────
    // Sequences (σ — Sequence)
    // ─────────────────────────────────────────────────────────────────────
    /// Make sequence: [count:u8] - pops count elements, pushes sequence
    MakeSeq = 0x80,
    /// Index sequence: pop idx, pop seq, push seq[idx]
    Index = 0x81,
    /// Get length: pop seq, push len(seq)
    Length = 0x82,
    /// Push to sequence: pop val, pop seq, push seq with val appended
    Push = 0x83,
    /// Concatenate: pop b, pop a, push a ++ b
    Concat = 0x84,

    // ─────────────────────────────────────────────────────────────────────
    // I/O (π — Persistence)
    // ─────────────────────────────────────────────────────────────────────
    /// Print: pop value, print it
    Print = 0x90,
    /// Print with newline
    Println = 0x91,

    // ─────────────────────────────────────────────────────────────────────
    // Closures (λ + μ)
    // ─────────────────────────────────────────────────────────────────────
    /// Create closure: [func_idx:u16, upvalue_count:u8]
    Closure = 0xA0,
    /// Load upvalue: [slot:u8]
    LoadUpvalue = 0xA1,
    /// Store upvalue: [slot:u8]
    StoreUpvalue = 0xA2,
    /// Close upvalues up to slot
    CloseUpvalue = 0xA3,

    // ─────────────────────────────────────────────────────────────────────
    // Special
    // ─────────────────────────────────────────────────────────────────────
    /// No operation
    Nop = 0xFE,
    /// Halt execution
    Halt = 0xFF,
}

impl OpCode {
    /// Get the number of operand bytes this opcode expects.
    #[must_use]
    pub const fn operand_width(self) -> usize {
        match self {
            Self::Constant | Self::LoadGlobal | Self::StoreGlobal => 2,
            Self::Jump | Self::JumpIf | Self::JumpIfNot | Self::Loop => 2,
            Self::Closure => 3, // func_idx:u16 + upvalue_count:u8
            Self::LoadLocal
            | Self::StoreLocal
            | Self::Call
            | Self::MakeSeq
            | Self::LoadUpvalue
            | Self::StoreUpvalue
            | Self::CloseUpvalue => 1,
            _ => 0,
        }
    }

    /// Get primitive composition for this opcode.
    #[must_use]
    pub fn composition(self) -> PrimitiveComposition {
        let prims = match self {
            Self::Constant | Self::True | Self::False | Self::Void => {
                vec![LexPrimitiva::Quantity, LexPrimitiva::Causality]
            }
            Self::LoadLocal | Self::StoreLocal | Self::LoadGlobal | Self::StoreGlobal => {
                vec![LexPrimitiva::Location, LexPrimitiva::State]
            }
            Self::Pop | Self::Dup | Self::Swap => {
                vec![LexPrimitiva::Recursion, LexPrimitiva::State]
            }
            Self::Add | Self::Sub | Self::Mul | Self::Div | Self::Mod | Self::Neg => {
                vec![LexPrimitiva::Quantity, LexPrimitiva::Mapping]
            }
            Self::Eq | Self::Ne | Self::Lt | Self::Le | Self::Gt | Self::Ge => {
                vec![LexPrimitiva::Comparison]
            }
            Self::Not | Self::And | Self::Or => {
                vec![LexPrimitiva::Boundary]
            }
            Self::Jump | Self::JumpIf | Self::JumpIfNot | Self::Loop => {
                vec![LexPrimitiva::Causality, LexPrimitiva::Boundary]
            }
            Self::Call | Self::Return => {
                vec![LexPrimitiva::Mapping, LexPrimitiva::Causality]
            }
            Self::MakeSeq | Self::Index | Self::Length | Self::Push | Self::Concat => {
                vec![LexPrimitiva::Sequence]
            }
            Self::Print | Self::Println => {
                vec![LexPrimitiva::Persistence]
            }
            Self::Closure | Self::LoadUpvalue | Self::StoreUpvalue | Self::CloseUpvalue => {
                vec![LexPrimitiva::Location, LexPrimitiva::Mapping]
            }
            Self::Nop | Self::Halt => vec![LexPrimitiva::Void],
        };
        PrimitiveComposition::new(prims)
    }
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        // Safety: we handle unknown bytes as Nop
        match byte {
            0x00 => Self::Constant,
            0x01 => Self::True,
            0x02 => Self::False,
            0x03 => Self::Void,
            0x10 => Self::LoadLocal,
            0x11 => Self::StoreLocal,
            0x12 => Self::LoadGlobal,
            0x13 => Self::StoreGlobal,
            0x20 => Self::Pop,
            0x21 => Self::Dup,
            0x22 => Self::Swap,
            0x30 => Self::Add,
            0x31 => Self::Sub,
            0x32 => Self::Mul,
            0x33 => Self::Div,
            0x34 => Self::Mod,
            0x35 => Self::Neg,
            0x40 => Self::Eq,
            0x41 => Self::Ne,
            0x42 => Self::Lt,
            0x43 => Self::Le,
            0x44 => Self::Gt,
            0x45 => Self::Ge,
            0x50 => Self::Not,
            0x51 => Self::And,
            0x52 => Self::Or,
            0x60 => Self::Jump,
            0x61 => Self::JumpIf,
            0x62 => Self::JumpIfNot,
            0x63 => Self::Loop,
            0x70 => Self::Call,
            0x71 => Self::Return,
            0x80 => Self::MakeSeq,
            0x81 => Self::Index,
            0x82 => Self::Length,
            0x83 => Self::Push,
            0x84 => Self::Concat,
            0x90 => Self::Print,
            0x91 => Self::Println,
            0xA0 => Self::Closure,
            0xA1 => Self::LoadUpvalue,
            0xA2 => Self::StoreUpvalue,
            0xA3 => Self::CloseUpvalue,
            0xFF => Self::Halt,
            _ => Self::Nop,
        }
    }
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CHUNK — σ[u8] + σ[Value] (Code + Constants)
// ═══════════════════════════════════════════════════════════════════════════

/// A chunk of bytecode with constants and debug info.
///
/// ## Tier: T2-C (σ + N + ν)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Chunk {
    /// Bytecode instructions.
    pub code: Vec<u8>,
    /// Constant pool.
    pub constants: Vec<Value>,
    /// Line numbers for each byte (debug info).
    pub lines: Vec<u32>,
    /// Function name (for debug).
    pub name: String,
}

impl Chunk {
    /// Create a new empty chunk.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
            name: name.into(),
        }
    }

    /// Write a byte to the chunk.
    pub fn write(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    /// Write an opcode.
    pub fn write_op(&mut self, op: OpCode, line: u32) {
        self.write(op as u8, line);
    }

    /// Write a u16 operand (big-endian).
    pub fn write_u16(&mut self, value: u16, line: u32) {
        self.write((value >> 8) as u8, line);
        self.write((value & 0xFF) as u8, line);
    }

    /// Write an i16 operand (big-endian).
    pub fn write_i16(&mut self, value: i16, line: u32) {
        self.write_u16(value as u16, line);
    }

    /// Add a constant and return its index.
    pub fn add_constant(&mut self, value: Value) -> u16 {
        // Check for duplicate
        for (i, c) in self.constants.iter().enumerate() {
            if c == &value {
                return i as u16;
            }
        }
        let idx = self.constants.len();
        self.constants.push(value);
        idx as u16
    }

    /// Get current code length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.code.len()
    }

    /// Check if chunk is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    /// Read a u16 at offset.
    #[must_use]
    pub fn read_u16(&self, offset: usize) -> u16 {
        let hi = self.code.get(offset).copied().unwrap_or(0) as u16;
        let lo = self.code.get(offset + 1).copied().unwrap_or(0) as u16;
        (hi << 8) | lo
    }

    /// Read an i16 at offset.
    #[must_use]
    pub fn read_i16(&self, offset: usize) -> i16 {
        self.read_u16(offset) as i16
    }

    /// Patch a u16 at offset.
    pub fn patch_u16(&mut self, offset: usize, value: u16) {
        if offset + 1 < self.code.len() {
            self.code[offset] = (value >> 8) as u8;
            self.code[offset + 1] = (value & 0xFF) as u8;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPILED FUNCTION — μ (Callable bytecode)
// ═══════════════════════════════════════════════════════════════════════════

/// A compiled function.
///
/// ## Tier: T2-C (μ + σ + λ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledFunction {
    /// Function name.
    pub name: String,
    /// Number of parameters.
    pub arity: u8,
    /// Number of local variables (including params).
    pub local_count: u8,
    /// Number of upvalues (captured variables).
    pub upvalue_count: u8,
    /// The bytecode chunk.
    pub chunk: Chunk,
}

impl CompiledFunction {
    /// Create a new compiled function.
    #[must_use]
    pub fn new(name: impl Into<String>, arity: u8) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            arity,
            local_count: arity,
            upvalue_count: 0,
            chunk: Chunk::new(name),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// BYTECODE MODULE — σ[CompiledFunction]
// ═══════════════════════════════════════════════════════════════════════════

/// A compiled module containing multiple functions.
///
/// ## Tier: T2-C (σ + μ)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BytecodeModule {
    /// Functions by name.
    pub functions: HashMap<String, CompiledFunction>,
    /// Entry function name.
    pub entry: Option<String>,
    /// Global variable names.
    pub globals: Vec<String>,
}

impl BytecodeModule {
    /// Create a new bytecode module.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a function.
    pub fn add_function(&mut self, func: CompiledFunction) {
        self.functions.insert(func.name.clone(), func);
    }

    /// Get a function by name.
    #[must_use]
    pub fn function(&self, name: &str) -> Option<&CompiledFunction> {
        self.functions.get(name)
    }

    /// Set entry point.
    pub fn set_entry(&mut self, name: impl Into<String>) {
        self.entry = Some(name.into());
    }

    /// Add a global variable name.
    pub fn add_global(&mut self, name: String) -> u16 {
        for (i, g) in self.globals.iter().enumerate() {
            if g == &name {
                return i as u16;
            }
        }
        let idx = self.globals.len();
        self.globals.push(name);
        idx as u16
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPILER — μ[IR → Bytecode]
// ═══════════════════════════════════════════════════════════════════════════

/// Bytecode compiler: transforms IR to bytecode.
///
/// ## Tier: T2-C (μ + → + σ)
pub struct BytecodeCompiler {
    /// Output module.
    module: BytecodeModule,
    /// Current function being compiled.
    current: Option<CompiledFunction>,
    /// Register to stack slot mapping.
    reg_slots: HashMap<u32, u8>,
    /// Block offsets for jump resolution.
    block_offsets: HashMap<u32, usize>,
    /// Pending jumps to patch.
    pending_jumps: Vec<(usize, u32)>,
}

impl Default for BytecodeCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl BytecodeCompiler {
    /// Create a new compiler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            module: BytecodeModule::new(),
            current: None,
            reg_slots: HashMap::new(),
            block_offsets: HashMap::new(),
            pending_jumps: Vec::new(),
        }
    }

    /// Compile an IR module to bytecode.
    pub fn compile(&mut self, ir: &IrModule) -> BytecodeModule {
        // Compile each function
        for func in ir.functions.values() {
            let compiled = self.compile_function(func);
            self.module.add_function(compiled);
        }

        // Set entry point
        if let Some(ref entry) = ir.entry {
            self.module.set_entry(entry.clone());
        }

        std::mem::take(&mut self.module)
    }

    /// Compile a single IR function.
    fn compile_function(&mut self, func: &IrFunction) -> CompiledFunction {
        self.reg_slots.clear();
        self.block_offsets.clear();
        self.pending_jumps.clear();

        let arity = func.params.len() as u8;
        let compiled = CompiledFunction::new(&func.name, arity);

        // Map parameters to slots
        for (i, &reg) in func.params.iter().enumerate() {
            self.reg_slots.insert(reg.0, i as u8);
        }

        self.current = Some(compiled);

        // First pass: record block offsets
        self.record_block_offsets(func);

        // Second pass: emit code
        for block in &func.blocks {
            self.compile_block(block);
        }

        // Patch jumps
        self.patch_jumps();

        self.current
            .take()
            .unwrap_or_else(|| CompiledFunction::new("", 0))
    }

    /// Record starting offset of each block.
    fn record_block_offsets(&mut self, func: &IrFunction) {
        let mut offset = 0;
        for block in &func.blocks {
            self.block_offsets.insert(block.id.0, offset);
            offset += self.estimate_block_size(block);
        }
    }

    /// Estimate bytecode size of a block.
    fn estimate_block_size(&self, block: &BasicBlock) -> usize {
        let mut size = 0;
        for inst in &block.instructions {
            size += self.estimate_instruction_size(inst);
        }
        size += self.estimate_terminator_size(&block.terminator);
        size
    }

    /// Estimate bytecode size of an instruction.
    fn estimate_instruction_size(&self, inst: &Instruction) -> usize {
        match inst {
            Instruction::LoadConst { .. } => 3, // Constant + u16
            Instruction::Copy { .. } => 4,      // LoadLocal + StoreLocal
            Instruction::BinOp { .. } => 5,     // 2 loads + op + store
            Instruction::UnOp { .. } => 3,      // load + op + store
            Instruction::Call { args, .. } => 2 + args.len(), // loads + call
            Instruction::Phi { .. } => 0,       // Handled by predecessors
            Instruction::MakeSeq { elements, .. } => 2 + elements.len(),
            Instruction::Index { .. } => 5,
            Instruction::Length { .. } => 3,
        }
    }

    /// Estimate bytecode size of a terminator.
    fn estimate_terminator_size(&self, term: &Terminator) -> usize {
        match term {
            Terminator::Jump { .. } => 3,
            Terminator::Branch { .. } => 6,
            Terminator::Return { .. } => 2,
            Terminator::Unreachable => 1,
        }
    }

    /// Compile a basic block.
    fn compile_block(&mut self, block: &BasicBlock) {
        // Update actual offset
        if let Some(ref mut func) = self.current {
            self.block_offsets.insert(block.id.0, func.chunk.len());
        }

        for inst in &block.instructions {
            self.compile_instruction(inst);
        }

        self.compile_terminator(&block.terminator);
    }

    /// Compile an instruction.
    fn compile_instruction(&mut self, inst: &Instruction) {
        let line = 1; // TODO: Track actual line numbers

        match inst {
            Instruction::LoadConst { dst, value } => {
                self.emit_const(value, line);
                self.assign_slot(dst.0);
                self.emit_store_local(self.get_slot(dst.0), line);
            }
            Instruction::Copy { dst, src } => {
                self.emit_load_local(self.get_slot(src.0), line);
                self.assign_slot(dst.0);
                self.emit_store_local(self.get_slot(dst.0), line);
            }
            Instruction::BinOp {
                dst,
                op,
                left,
                right,
            } => {
                self.emit_load_local(self.get_slot(left.0), line);
                self.emit_load_local(self.get_slot(right.0), line);
                self.emit_binop(op, line);
                self.assign_slot(dst.0);
                self.emit_store_local(self.get_slot(dst.0), line);
            }
            Instruction::UnOp { dst, op, src } => {
                self.emit_load_local(self.get_slot(src.0), line);
                self.emit_unop(op, line);
                self.assign_slot(dst.0);
                self.emit_store_local(self.get_slot(dst.0), line);
            }
            Instruction::Call { dst, func, args } => {
                // Push function name as constant
                self.emit_global_load(func, line);
                // Push arguments
                for arg in args {
                    self.emit_load_local(self.get_slot(arg.0), line);
                }
                self.emit_call(args.len() as u8, line);
                self.assign_slot(dst.0);
                self.emit_store_local(self.get_slot(dst.0), line);
            }
            Instruction::Phi { .. } => {
                // PHI nodes are handled by predecessor blocks
            }
            Instruction::MakeSeq { dst, elements } => {
                for elem in elements {
                    self.emit_load_local(self.get_slot(elem.0), line);
                }
                self.emit_make_seq(elements.len() as u8, line);
                self.assign_slot(dst.0);
                self.emit_store_local(self.get_slot(dst.0), line);
            }
            Instruction::Index { dst, seq, idx } => {
                self.emit_load_local(self.get_slot(seq.0), line);
                self.emit_load_local(self.get_slot(idx.0), line);
                self.emit_op(OpCode::Index, line);
                self.assign_slot(dst.0);
                self.emit_store_local(self.get_slot(dst.0), line);
            }
            Instruction::Length { dst, seq } => {
                self.emit_load_local(self.get_slot(seq.0), line);
                self.emit_op(OpCode::Length, line);
                self.assign_slot(dst.0);
                self.emit_store_local(self.get_slot(dst.0), line);
            }
        }
    }

    /// Compile a terminator.
    fn compile_terminator(&mut self, term: &Terminator) {
        let line = 1;

        match term {
            Terminator::Jump { target } => {
                self.emit_jump(target.0, line);
            }
            Terminator::Branch {
                cond,
                then_block,
                else_block,
            } => {
                self.emit_load_local(self.get_slot(cond.0), line);
                self.emit_branch(then_block.0, else_block.0, line);
            }
            Terminator::Return { value } => {
                self.emit_load_local(self.get_slot(value.0), line);
                self.emit_op(OpCode::Return, line);
            }
            Terminator::Unreachable => {
                self.emit_op(OpCode::Halt, line);
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Emit helpers
    // ─────────────────────────────────────────────────────────────────────

    fn emit_op(&mut self, op: OpCode, line: u32) {
        if let Some(ref mut func) = self.current {
            func.chunk.write_op(op, line);
        }
    }

    fn emit_const(&mut self, value: &IrConst, line: u32) {
        let val = match value {
            IrConst::Void => Value::void(),
            IrConst::Int(n) => Value::int(*n),
            IrConst::Float(f) => Value::float(*f),
            IrConst::Bool(b) => Value::bool(*b),
            IrConst::String(s) => Value::string(s.clone()),
        };

        if let Some(ref mut func) = self.current {
            let idx = func.chunk.add_constant(val);
            func.chunk.write_op(OpCode::Constant, line);
            func.chunk.write_u16(idx, line);
        }
    }

    fn emit_load_local(&mut self, slot: u8, line: u32) {
        if let Some(ref mut func) = self.current {
            func.chunk.write_op(OpCode::LoadLocal, line);
            func.chunk.write(slot, line);
        }
    }

    fn emit_store_local(&mut self, slot: u8, line: u32) {
        if let Some(ref mut func) = self.current {
            func.chunk.write_op(OpCode::StoreLocal, line);
            func.chunk.write(slot, line);
        }
    }

    fn emit_global_load(&mut self, name: &str, line: u32) {
        let idx = self.module.add_global(name.to_string());
        if let Some(ref mut func) = self.current {
            func.chunk.write_op(OpCode::LoadGlobal, line);
            func.chunk.write_u16(idx, line);
        }
    }

    fn emit_binop(&mut self, op: &crate::ast::BinOp, line: u32) {
        use crate::ast::BinOp::*;
        let opcode = match op {
            Add => OpCode::Add,
            Sub => OpCode::Sub,
            Mul => OpCode::Mul,
            Div => OpCode::Div,
            Mod => OpCode::Mod,
            Eq | KappaEq => OpCode::Eq,
            Ne | KappaNe => OpCode::Ne,
            Lt | KappaLt => OpCode::Lt,
            Le | KappaLe => OpCode::Le,
            Gt | KappaGt => OpCode::Gt,
            Ge | KappaGe => OpCode::Ge,
            And => OpCode::And,
            Or => OpCode::Or,
        };
        self.emit_op(opcode, line);
    }

    fn emit_unop(&mut self, op: &crate::ast::UnOp, line: u32) {
        use crate::ast::UnOp::*;
        let opcode = match op {
            Neg => OpCode::Neg,
            Not => OpCode::Not,
        };
        self.emit_op(opcode, line);
    }

    fn emit_call(&mut self, arg_count: u8, line: u32) {
        if let Some(ref mut func) = self.current {
            func.chunk.write_op(OpCode::Call, line);
            func.chunk.write(arg_count, line);
        }
    }

    fn emit_make_seq(&mut self, count: u8, line: u32) {
        if let Some(ref mut func) = self.current {
            func.chunk.write_op(OpCode::MakeSeq, line);
            func.chunk.write(count, line);
        }
    }

    fn emit_jump(&mut self, target: u32, line: u32) {
        if let Some(ref mut func) = self.current {
            func.chunk.write_op(OpCode::Jump, line);
            let patch_addr = func.chunk.len();
            func.chunk.write_u16(0, line); // Placeholder
            self.pending_jumps.push((patch_addr, target));
        }
    }

    fn emit_branch(&mut self, then_block: u32, else_block: u32, line: u32) {
        if let Some(ref mut func) = self.current {
            // JumpIfNot -> else_block
            func.chunk.write_op(OpCode::JumpIfNot, line);
            let else_patch = func.chunk.len();
            func.chunk.write_u16(0, line);
            self.pending_jumps.push((else_patch, else_block));

            // Fall through or jump to then_block
            func.chunk.write_op(OpCode::Jump, line);
            let then_patch = func.chunk.len();
            func.chunk.write_u16(0, line);
            self.pending_jumps.push((then_patch, then_block));
        }
    }

    fn patch_jumps(&mut self) {
        for (patch_addr, target_block) in &self.pending_jumps {
            if let Some(ref mut func) = self.current {
                let target_offset = self.block_offsets.get(target_block).copied().unwrap_or(0);
                let offset = target_offset as i16 - (*patch_addr as i16 + 2);
                func.chunk.patch_u16(*patch_addr, offset as u16);
            }
        }
    }

    fn get_slot(&self, reg: u32) -> u8 {
        self.reg_slots.get(&reg).copied().unwrap_or(0)
    }

    fn assign_slot(&mut self, reg: u32) {
        if !self.reg_slots.contains_key(&reg) {
            let slot = self.reg_slots.len() as u8;
            self.reg_slots.insert(reg, slot);
            if let Some(ref mut func) = self.current {
                func.local_count = func.local_count.max(slot + 1);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DISASSEMBLER — Display bytecode
// ═══════════════════════════════════════════════════════════════════════════

/// Disassemble a chunk to human-readable form.
#[must_use]
pub fn disassemble(chunk: &Chunk) -> String {
    let mut out = String::new();
    out.push_str(&format!("== {} ==\n", chunk.name));

    let mut offset = 0;
    while offset < chunk.code.len() {
        let (s, next) = disassemble_instruction(chunk, offset);
        out.push_str(&s);
        out.push('\n');
        offset = next;
    }

    out
}

/// Disassemble a single instruction.
#[must_use]
pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> (String, usize) {
    let byte = chunk.code.get(offset).copied().unwrap_or(0);
    let op = OpCode::from(byte);
    let line = chunk.lines.get(offset).copied().unwrap_or(0);

    let mut out = format!("{offset:04} {line:4} {op:16}");
    let mut next = offset + 1;

    match op.operand_width() {
        1 => {
            let operand = chunk.code.get(next).copied().unwrap_or(0);
            out.push_str(&format!(" {operand}"));
            next += 1;
        }
        2 => {
            let operand = chunk.read_u16(next);
            if op == OpCode::Constant {
                if let Some(val) = chunk.constants.get(operand as usize) {
                    out.push_str(&format!(" {operand} ({val})"));
                } else {
                    out.push_str(&format!(" {operand}"));
                }
            } else {
                out.push_str(&format!(" {operand}"));
            }
            next += 2;
        }
        3 => {
            let a = chunk.read_u16(next);
            let b = chunk.code.get(next + 2).copied().unwrap_or(0);
            out.push_str(&format!(" {a} {b}"));
            next += 3;
        }
        _ => {}
    }

    (out, next)
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────
    // OpCode tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_opcode_roundtrip() {
        for byte in 0..=255u8 {
            let op = OpCode::from(byte);
            if op != OpCode::Nop || byte == 0xFE {
                assert_eq!(op as u8, byte, "Roundtrip failed for {byte}");
            }
        }
    }

    #[test]
    fn test_opcode_operand_width() {
        assert_eq!(OpCode::Add.operand_width(), 0);
        assert_eq!(OpCode::Constant.operand_width(), 2);
        assert_eq!(OpCode::LoadLocal.operand_width(), 1);
        assert_eq!(OpCode::Jump.operand_width(), 2);
        assert_eq!(OpCode::Closure.operand_width(), 3);
    }

    #[test]
    fn test_opcode_composition() {
        let add_comp = OpCode::Add.composition();
        let unique = add_comp.unique();
        assert!(unique.contains(&LexPrimitiva::Quantity));
        assert!(unique.contains(&LexPrimitiva::Mapping));

        let jump_comp = OpCode::Jump.composition();
        let unique = jump_comp.unique();
        assert!(unique.contains(&LexPrimitiva::Causality));
        assert!(unique.contains(&LexPrimitiva::Boundary));
    }

    // ─────────────────────────────────────────────────────────────────────
    // Chunk tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_chunk_write() {
        let mut chunk = Chunk::new("test");
        chunk.write_op(OpCode::Constant, 1);
        chunk.write_u16(42, 1);
        chunk.write_op(OpCode::Return, 1);

        assert_eq!(chunk.len(), 4);
        assert_eq!(chunk.code[0], OpCode::Constant as u8);
        assert_eq!(chunk.read_u16(1), 42);
        assert_eq!(chunk.code[3], OpCode::Return as u8);
    }

    #[test]
    fn test_chunk_constants() {
        let mut chunk = Chunk::new("test");
        let idx1 = chunk.add_constant(Value::int(42));
        let idx2 = chunk.add_constant(Value::int(100));
        let idx3 = chunk.add_constant(Value::int(42)); // Duplicate

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 0); // Should reuse
        assert_eq!(chunk.constants.len(), 2);
    }

    #[test]
    fn test_chunk_patch() {
        let mut chunk = Chunk::new("test");
        chunk.write_op(OpCode::Jump, 1);
        let patch_addr = chunk.len();
        chunk.write_u16(0, 1);

        chunk.patch_u16(patch_addr, 1234);
        assert_eq!(chunk.read_u16(patch_addr), 1234);
    }

    // ─────────────────────────────────────────────────────────────────────
    // CompiledFunction tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_compiled_function() {
        let func = CompiledFunction::new("add", 2);
        assert_eq!(func.name, "add");
        assert_eq!(func.arity, 2);
        assert_eq!(func.local_count, 2);
    }

    // ─────────────────────────────────────────────────────────────────────
    // BytecodeModule tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_bytecode_module() {
        let mut module = BytecodeModule::new();
        let func = CompiledFunction::new("main", 0);
        module.add_function(func);
        module.set_entry("main");

        assert!(module.function("main").is_some());
        assert_eq!(module.entry, Some("main".into()));
    }

    #[test]
    fn test_module_globals() {
        let mut module = BytecodeModule::new();
        let idx1 = module.add_global("print".into());
        let idx2 = module.add_global("println".into());
        let idx3 = module.add_global("print".into()); // Duplicate

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 0);
        assert_eq!(module.globals.len(), 2);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Compiler tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_compile_empty_module() {
        let ir = IrModule::new();
        let mut compiler = BytecodeCompiler::new();
        let module = compiler.compile(&ir);

        assert!(module.functions.is_empty());
        assert!(module.entry.is_none());
    }

    #[test]
    fn test_compile_simple_function() {
        use crate::ir::{IrBuilder, IrConst, Terminator};

        let mut builder = IrBuilder::new();
        builder.start_function("answer", &[]);
        let val = builder.emit_const(IrConst::Int(42));
        builder.terminate(Terminator::Return { value: val });

        let func = builder
            .finish_function()
            .unwrap_or_else(|| crate::ir::IrFunction::new("", 0));

        let mut ir_module = IrModule::new();
        ir_module.add_function(func);
        ir_module.set_entry("answer");

        let mut compiler = BytecodeCompiler::new();
        let module = compiler.compile(&ir_module);

        assert!(module.function("answer").is_some());
        let answer = module.function("answer").unwrap();
        assert_eq!(answer.arity, 0);
        assert!(!answer.chunk.is_empty());
    }

    // ─────────────────────────────────────────────────────────────────────
    // Disassembler tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_disassemble() {
        let mut chunk = Chunk::new("test");
        chunk.write_op(OpCode::Constant, 1);
        let idx = chunk.add_constant(Value::int(42));
        chunk.write_u16(idx, 1);
        chunk.write_op(OpCode::Return, 1);

        let output = disassemble(&chunk);
        assert!(output.contains("test"));
        assert!(output.contains("Constant"));
        assert!(output.contains("42"));
        assert!(output.contains("Return"));
    }

    #[test]
    fn test_disassemble_instruction() {
        let mut chunk = Chunk::new("test");
        chunk.write_op(OpCode::Add, 5);

        let (text, next) = disassemble_instruction(&chunk, 0);
        assert!(text.contains("Add"));
        assert!(text.contains("5")); // Line number
        assert_eq!(next, 1);
    }
}
