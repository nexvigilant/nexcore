// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Intermediate Representation
//!
//! A flattened, linear representation for optimization and codegen.
//!
//! ## Philosophy
//!
//! The IR sits between AST and bytecode/machine code. It enables:
//! - Optimization passes (constant folding, dead code elimination)
//! - Effect-aware transformations (pure functions can be reordered)
//! - Clean separation of concerns (parsing → IR → codegen)
//!
//! ## Tier: T2-C (σ + μ + → + ς + ∂)
//!
//! ## Architecture
//!
//! ```text
//! Module
//! └── Function*
//!     └── BasicBlock*
//!         └── Instruction*
//! ```
//!
//! Uses SSA (Static Single Assignment) form with explicit phi nodes.

use crate::ast::{BinOp, UnOp};
use crate::effects::EffectSet;
use lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// REGISTER — λ + ς (Location holding State)
// ═══════════════════════════════════════════════════════════════════════════

/// A virtual register (SSA value).
///
/// In SSA form, each register is assigned exactly once.
///
/// ## Tier: T2-P (λ + ς)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Reg(pub u32);

impl Reg {
    /// The void/unit register (always contains ∅).
    pub const VOID: Self = Self(0);

    /// Create a new register.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the register ID.
    #[must_use]
    pub const fn id(self) -> u32 {
        self.0
    }
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Self::VOID {
            write!(f, "%void")
        } else {
            write!(f, "%{}", self.0)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// BASIC BLOCK — σ[Instruction] (Sequence of operations)
// ═══════════════════════════════════════════════════════════════════════════

/// Basic block identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockId(pub u32);

impl BlockId {
    /// Entry block ID.
    pub const ENTRY: Self = Self(0);

    /// Create a new block ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

/// A basic block — straight-line code with a terminator.
///
/// ## Invariants
/// - No control flow within (linear σ)
/// - Ends with exactly one terminator
/// - Entry block has id 0
///
/// ## Tier: T2-P (σ + →)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    /// Block identifier.
    pub id: BlockId,
    /// Instructions (excluding terminator).
    pub instructions: Vec<Instruction>,
    /// Block terminator (required).
    pub terminator: Terminator,
    /// Accumulated effects in this block.
    pub effects: EffectSet,
}

impl BasicBlock {
    /// Create a new basic block.
    #[must_use]
    pub fn new(id: BlockId) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            terminator: Terminator::Unreachable,
            effects: EffectSet::empty(),
        }
    }

    /// Add an instruction to the block.
    pub fn push(&mut self, inst: Instruction) {
        // Track effects
        self.effects = self.effects | inst.effects();
        self.instructions.push(inst);
    }

    /// Set the terminator.
    pub fn terminate(&mut self, term: Terminator) {
        self.terminator = term;
    }

    /// Check if block is terminated.
    #[must_use]
    pub fn is_terminated(&self) -> bool {
        !matches!(self.terminator, Terminator::Unreachable)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// INSTRUCTION — → (Causality: inputs → output)
// ═══════════════════════════════════════════════════════════════════════════

/// IR instruction.
///
/// Each instruction produces a value in a destination register.
/// SSA form: each register assigned exactly once.
///
/// ## Tier: T2-C (→ + ς + Σ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    /// Load a constant value.
    /// `dst = const`
    LoadConst { dst: Reg, value: IrConst },

    /// Copy a value.
    /// `dst = src`
    Copy { dst: Reg, src: Reg },

    /// Binary operation.
    /// `dst = left op right`
    BinOp {
        dst: Reg,
        op: BinOp,
        left: Reg,
        right: Reg,
    },

    /// Unary operation.
    /// `dst = op src`
    UnOp { dst: Reg, op: UnOp, src: Reg },

    /// Function call.
    /// `dst = func(args...)`
    Call {
        dst: Reg,
        func: String,
        args: Vec<Reg>,
    },

    /// SSA phi node — merges values from predecessor blocks.
    /// `dst = phi [(block1, val1), (block2, val2), ...]`
    Phi {
        dst: Reg,
        incoming: Vec<(BlockId, Reg)>,
    },

    /// Create a sequence.
    /// `dst = [elements...]`
    MakeSeq { dst: Reg, elements: Vec<Reg> },

    /// Index into a sequence.
    /// `dst = seq[idx]`
    Index { dst: Reg, seq: Reg, idx: Reg },

    /// Get sequence length.
    /// `dst = #seq`
    Length { dst: Reg, seq: Reg },
}

impl Instruction {
    /// Get the destination register (if any).
    #[must_use]
    pub fn dst(&self) -> Reg {
        match self {
            Self::LoadConst { dst, .. }
            | Self::Copy { dst, .. }
            | Self::BinOp { dst, .. }
            | Self::UnOp { dst, .. }
            | Self::Call { dst, .. }
            | Self::Phi { dst, .. }
            | Self::MakeSeq { dst, .. }
            | Self::Index { dst, .. }
            | Self::Length { dst, .. } => *dst,
        }
    }

    /// Get effects of this instruction.
    #[must_use]
    pub fn effects(&self) -> EffectSet {
        match self {
            // Calls may have effects (check builtin_effect in caller)
            Self::Call { func, .. } => crate::effects::builtin_effect(func).effects,
            // All other instructions are pure
            _ => EffectSet::empty(),
        }
    }

    /// Check if this instruction is pure.
    #[must_use]
    pub fn is_pure(&self) -> bool {
        self.effects().is_pure()
    }

    /// Get primitive composition for this instruction.
    #[must_use]
    pub fn composition(&self) -> PrimitiveComposition {
        let prims = match self {
            Self::LoadConst { .. } => vec![LexPrimitiva::Quantity, LexPrimitiva::Causality],
            Self::Copy { .. } => vec![LexPrimitiva::State, LexPrimitiva::Causality],
            Self::BinOp { .. } => {
                vec![
                    LexPrimitiva::Quantity,
                    LexPrimitiva::Causality,
                    LexPrimitiva::Mapping,
                ]
            }
            Self::UnOp { .. } => vec![LexPrimitiva::Causality, LexPrimitiva::Mapping],
            Self::Call { .. } => {
                vec![
                    LexPrimitiva::Causality,
                    LexPrimitiva::Mapping,
                    LexPrimitiva::Persistence,
                ]
            }
            Self::Phi { .. } => vec![LexPrimitiva::Sum, LexPrimitiva::State],
            Self::MakeSeq { .. } => vec![LexPrimitiva::Sequence, LexPrimitiva::Causality],
            Self::Index { .. } => {
                vec![
                    LexPrimitiva::Sequence,
                    LexPrimitiva::Location,
                    LexPrimitiva::Causality,
                ]
            }
            Self::Length { .. } => {
                vec![
                    LexPrimitiva::Sequence,
                    LexPrimitiva::Quantity,
                    LexPrimitiva::Causality,
                ]
            }
        };
        PrimitiveComposition::new(prims)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TERMINATOR — ∂ (Boundary: control flow decisions)
// ═══════════════════════════════════════════════════════════════════════════

/// Block terminator — ends a basic block with control flow.
///
/// ## Tier: T2-P (∂ + →)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Terminator {
    /// Unconditional jump.
    Jump { target: BlockId },

    /// Conditional branch.
    Branch {
        cond: Reg,
        then_block: BlockId,
        else_block: BlockId,
    },

    /// Return from function.
    Return { value: Reg },

    /// Unreachable (placeholder, error if reached).
    Unreachable,
}

impl Terminator {
    /// Get successor block IDs.
    #[must_use]
    pub fn successors(&self) -> Vec<BlockId> {
        match self {
            Self::Jump { target } => vec![*target],
            Self::Branch {
                then_block,
                else_block,
                ..
            } => vec![*then_block, *else_block],
            Self::Return { .. } | Self::Unreachable => vec![],
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// IR CONSTANT — ν (Invariant values)
// ═══════════════════════════════════════════════════════════════════════════

/// Compile-time constant value.
///
/// ## Tier: T2-P (ν + Σ)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IrConst {
    /// ∅ — Void/unit
    Void,
    /// N — Integer
    Int(i64),
    /// N — Float
    Float(f64),
    /// Σ(0,1) — Boolean
    Bool(bool),
    /// σ[char] — String
    String(String),
}

impl IrConst {
    /// Get root constant ({0, 1}).
    #[must_use]
    pub fn root_constant(&self) -> u8 {
        match self {
            Self::Void => 0,
            Self::Bool(b) => u8::from(*b),
            Self::Int(n) => {
                if *n == 0 {
                    0
                } else {
                    1
                }
            }
            Self::Float(f) => {
                if *f == 0.0 {
                    0
                } else {
                    1
                }
            }
            Self::String(s) => {
                if s.is_empty() {
                    0
                } else {
                    1
                }
            }
        }
    }
}

impl fmt::Display for IrConst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Void => write!(f, "∅"),
            Self::Int(n) => write!(f, "{n}"),
            Self::Float(n) => write!(f, "{n}"),
            Self::Bool(b) => write!(f, "{}", if *b { "1" } else { "0" }),
            Self::String(s) => write!(f, "\"{s}\""),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FUNCTION — μ (Mapping: params → result)
// ═══════════════════════════════════════════════════════════════════════════

/// IR function.
///
/// ## Tier: T2-C (μ + σ + ∂ + →)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrFunction {
    /// Function name.
    pub name: String,
    /// Parameter registers (in order).
    pub params: Vec<Reg>,
    /// Basic blocks (entry is blocks[0]).
    pub blocks: Vec<BasicBlock>,
    /// Total effects of this function.
    pub effects: EffectSet,
    /// Next register ID for allocation.
    next_reg: u32,
    /// Next block ID for allocation.
    next_block: u32,
}

impl IrFunction {
    /// Create a new function.
    #[must_use]
    pub fn new(name: impl Into<String>, param_count: usize) -> Self {
        // Reserve reg 0 for void, params start at 1
        let params: Vec<Reg> = (1..=param_count as u32).map(Reg::new).collect();
        let next_reg = (param_count as u32) + 1;

        let mut func = Self {
            name: name.into(),
            params,
            blocks: Vec::new(),
            effects: EffectSet::empty(),
            next_reg,
            next_block: 0,
        };

        // Create entry block
        func.new_block();
        func
    }

    /// Allocate a new register.
    pub fn alloc_reg(&mut self) -> Reg {
        let reg = Reg::new(self.next_reg);
        self.next_reg += 1;
        reg
    }

    /// Create a new basic block.
    pub fn new_block(&mut self) -> BlockId {
        let id = BlockId::new(self.next_block);
        self.next_block += 1;
        self.blocks.push(BasicBlock::new(id));
        id
    }

    /// Get entry block ID.
    #[must_use]
    pub const fn entry(&self) -> BlockId {
        BlockId::ENTRY
    }

    /// Get a block by ID.
    #[must_use]
    pub fn block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(id.0 as usize)
    }

    /// Get a mutable block by ID.
    pub fn block_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(id.0 as usize)
    }

    /// Compute total effects from all blocks.
    pub fn compute_effects(&mut self) {
        self.effects = self
            .blocks
            .iter()
            .fold(EffectSet::empty(), |acc, b| acc | b.effects);
    }

    /// Check if function is pure.
    #[must_use]
    pub fn is_pure(&self) -> bool {
        self.effects.is_pure()
    }

    /// Get register count.
    #[must_use]
    pub const fn reg_count(&self) -> u32 {
        self.next_reg
    }

    /// Get block count.
    #[must_use]
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MODULE — σ[Function] (Sequence of mappings)
// ═══════════════════════════════════════════════════════════════════════════

/// IR module — collection of functions.
///
/// ## Tier: T2-C (σ + μ)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IrModule {
    /// Functions by name.
    pub functions: HashMap<String, IrFunction>,
    /// Entry point (main function).
    pub entry: Option<String>,
}

impl IrModule {
    /// Create a new empty module.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a function to the module.
    pub fn add_function(&mut self, func: IrFunction) {
        self.functions.insert(func.name.clone(), func);
    }

    /// Get a function by name.
    #[must_use]
    pub fn function(&self, name: &str) -> Option<&IrFunction> {
        self.functions.get(name)
    }

    /// Get a mutable function by name.
    pub fn function_mut(&mut self, name: &str) -> Option<&mut IrFunction> {
        self.functions.get_mut(name)
    }

    /// Set the entry point.
    pub fn set_entry(&mut self, name: impl Into<String>) {
        self.entry = Some(name.into());
    }

    /// Get total effects of the module.
    #[must_use]
    pub fn total_effects(&self) -> EffectSet {
        self.functions
            .values()
            .fold(EffectSet::empty(), |acc, f| acc | f.effects)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// IR BUILDER — ς (State machine for construction)
// ═══════════════════════════════════════════════════════════════════════════

/// Builder for constructing IR from AST.
///
/// ## Tier: T2-C (ς + μ + σ)
pub struct IrBuilder {
    /// Current function being built.
    current_func: Option<IrFunction>,
    /// Current block ID.
    current_block: BlockId,
    /// Variable → Register mapping (scoped).
    scopes: Vec<HashMap<String, Reg>>,
}

impl Default for IrBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl IrBuilder {
    /// Create a new IR builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_func: None,
            current_block: BlockId::ENTRY,
            scopes: vec![HashMap::new()],
        }
    }

    /// Start building a new function.
    pub fn start_function(&mut self, name: &str, params: &[String]) {
        let func = IrFunction::new(name, params.len());

        // Bind parameters to registers
        self.scopes.push(HashMap::new());
        for (i, param) in params.iter().enumerate() {
            let reg = func.params[i];
            self.bind(param.clone(), reg);
        }

        self.current_func = Some(func);
        self.current_block = BlockId::ENTRY;
    }

    /// Finish building the current function.
    pub fn finish_function(&mut self) -> Option<IrFunction> {
        self.scopes.pop();
        let mut func = self.current_func.take()?;
        func.compute_effects();
        Some(func)
    }

    /// Push a new scope.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop current scope.
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Bind a variable to a register.
    pub fn bind(&mut self, name: String, reg: Reg) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, reg);
        }
    }

    /// Look up a variable.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<Reg> {
        for scope in self.scopes.iter().rev() {
            if let Some(&reg) = scope.get(name) {
                return Some(reg);
            }
        }
        None
    }

    /// Allocate a new register.
    pub fn alloc_reg(&mut self) -> Reg {
        self.current_func
            .as_mut()
            .map(IrFunction::alloc_reg)
            .unwrap_or(Reg::VOID)
    }

    /// Create a new basic block.
    pub fn new_block(&mut self) -> BlockId {
        self.current_func
            .as_mut()
            .map(IrFunction::new_block)
            .unwrap_or(BlockId::ENTRY)
    }

    /// Switch to a block.
    pub fn switch_to(&mut self, block: BlockId) {
        self.current_block = block;
    }

    /// Emit an instruction to current block.
    pub fn emit(&mut self, inst: Instruction) {
        if let Some(func) = &mut self.current_func {
            if let Some(block) = func.block_mut(self.current_block) {
                block.push(inst);
            }
        }
    }

    /// Terminate current block.
    pub fn terminate(&mut self, term: Terminator) {
        if let Some(func) = &mut self.current_func {
            if let Some(block) = func.block_mut(self.current_block) {
                block.terminate(term);
            }
        }
    }

    /// Emit a load constant instruction.
    pub fn emit_const(&mut self, value: IrConst) -> Reg {
        let dst = self.alloc_reg();
        self.emit(Instruction::LoadConst { dst, value });
        dst
    }

    /// Emit a binary operation.
    pub fn emit_binop(&mut self, op: BinOp, left: Reg, right: Reg) -> Reg {
        let dst = self.alloc_reg();
        self.emit(Instruction::BinOp {
            dst,
            op,
            left,
            right,
        });
        dst
    }

    /// Emit a unary operation.
    pub fn emit_unop(&mut self, op: UnOp, src: Reg) -> Reg {
        let dst = self.alloc_reg();
        self.emit(Instruction::UnOp { dst, op, src });
        dst
    }

    /// Emit a function call.
    pub fn emit_call(&mut self, func: &str, args: Vec<Reg>) -> Reg {
        let dst = self.alloc_reg();
        self.emit(Instruction::Call {
            dst,
            func: func.to_string(),
            args,
        });
        dst
    }

    /// Emit a sequence construction.
    pub fn emit_seq(&mut self, elements: Vec<Reg>) -> Reg {
        let dst = self.alloc_reg();
        self.emit(Instruction::MakeSeq { dst, elements });
        dst
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// IR PRINTER — Display IR in human-readable form
// ═══════════════════════════════════════════════════════════════════════════

/// Format an IR module as text.
#[must_use]
pub fn format_module(module: &IrModule) -> String {
    let mut out = String::new();

    for func in module.functions.values() {
        out.push_str(&format_function(func));
        out.push('\n');
    }

    out
}

/// Format an IR function as text.
#[must_use]
pub fn format_function(func: &IrFunction) -> String {
    let mut out = String::new();

    // Function signature
    let params: Vec<String> = func.params.iter().map(|r| format!("{r}")).collect();
    out.push_str(&format!(
        "fn {}({}) ! {} {{\n",
        func.name,
        params.join(", "),
        func.effects
    ));

    // Blocks
    for block in &func.blocks {
        out.push_str(&format!("  {}:\n", block.id));

        for inst in &block.instructions {
            out.push_str(&format!("    {}\n", format_instruction(inst)));
        }

        out.push_str(&format!("    {}\n", format_terminator(&block.terminator)));
    }

    out.push_str("}\n");
    out
}

/// Format a single instruction.
#[must_use]
pub fn format_instruction(inst: &Instruction) -> String {
    match inst {
        Instruction::LoadConst { dst, value } => {
            format!("{dst} = const {value}")
        }
        Instruction::Copy { dst, src } => {
            format!("{dst} = copy {src}")
        }
        Instruction::BinOp {
            dst,
            op,
            left,
            right,
        } => {
            format!("{dst} = {left} {op:?} {right}")
        }
        Instruction::UnOp { dst, op, src } => {
            format!("{dst} = {op:?} {src}")
        }
        Instruction::Call { dst, func, args } => {
            let args_str: Vec<String> = args.iter().map(|r| format!("{r}")).collect();
            format!("{dst} = call {func}({})", args_str.join(", "))
        }
        Instruction::Phi { dst, incoming } => {
            let pairs: Vec<String> = incoming
                .iter()
                .map(|(b, r)| format!("[{b}: {r}]"))
                .collect();
            format!("{dst} = phi {}", pairs.join(", "))
        }
        Instruction::MakeSeq { dst, elements } => {
            let elems: Vec<String> = elements.iter().map(|r| format!("{r}")).collect();
            format!("{dst} = seq [{}]", elems.join(", "))
        }
        Instruction::Index { dst, seq, idx } => {
            format!("{dst} = index {seq}[{idx}]")
        }
        Instruction::Length { dst, seq } => {
            format!("{dst} = len {seq}")
        }
    }
}

/// Format a terminator.
#[must_use]
pub fn format_terminator(term: &Terminator) -> String {
    match term {
        Terminator::Jump { target } => format!("jump {target}"),
        Terminator::Branch {
            cond,
            then_block,
            else_block,
        } => {
            format!("branch {cond} ? {then_block} : {else_block}")
        }
        Terminator::Return { value } => format!("return {value}"),
        Terminator::Unreachable => "unreachable".to_string(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS — ν (Invariant verification)
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // Register tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_reg_display() {
        assert_eq!(format!("{}", Reg::VOID), "%void");
        assert_eq!(format!("{}", Reg::new(1)), "%1");
        assert_eq!(format!("{}", Reg::new(42)), "%42");
    }

    #[test]
    fn test_reg_equality() {
        assert_eq!(Reg::new(1), Reg::new(1));
        assert_ne!(Reg::new(1), Reg::new(2));
        assert_eq!(Reg::VOID, Reg::new(0));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BlockId tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_block_id_display() {
        assert_eq!(format!("{}", BlockId::ENTRY), "bb0");
        assert_eq!(format!("{}", BlockId::new(5)), "bb5");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // IrConst tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_const_display() {
        assert_eq!(format!("{}", IrConst::Void), "∅");
        assert_eq!(format!("{}", IrConst::Int(42)), "42");
        assert_eq!(format!("{}", IrConst::Bool(true)), "1");
        assert_eq!(format!("{}", IrConst::Bool(false)), "0");
        assert_eq!(format!("{}", IrConst::String("hi".into())), "\"hi\"");
    }

    #[test]
    fn test_const_root() {
        assert_eq!(IrConst::Void.root_constant(), 0);
        assert_eq!(IrConst::Int(0).root_constant(), 0);
        assert_eq!(IrConst::Int(1).root_constant(), 1);
        assert_eq!(IrConst::Int(-5).root_constant(), 1);
        assert_eq!(IrConst::Bool(false).root_constant(), 0);
        assert_eq!(IrConst::Bool(true).root_constant(), 1);
        assert_eq!(IrConst::String("".into()).root_constant(), 0);
        assert_eq!(IrConst::String("a".into()).root_constant(), 1);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BasicBlock tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_basic_block_construction() {
        let mut block = BasicBlock::new(BlockId::ENTRY);
        assert!(!block.is_terminated());

        block.push(Instruction::LoadConst {
            dst: Reg::new(1),
            value: IrConst::Int(42),
        });
        assert_eq!(block.instructions.len(), 1);

        block.terminate(Terminator::Return { value: Reg::new(1) });
        assert!(block.is_terminated());
    }

    #[test]
    fn test_block_effects() {
        let mut block = BasicBlock::new(BlockId::ENTRY);

        // Pure instruction
        block.push(Instruction::LoadConst {
            dst: Reg::new(1),
            value: IrConst::Int(5),
        });
        assert!(block.effects.is_pure());

        // IO instruction
        block.push(Instruction::Call {
            dst: Reg::new(2),
            func: "print".to_string(),
            args: vec![Reg::new(1)],
        });
        assert!(block.effects.has_io());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // IrFunction tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_function_creation() {
        let func = IrFunction::new("test", 2);

        assert_eq!(func.name, "test");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.params[0], Reg::new(1));
        assert_eq!(func.params[1], Reg::new(2));
        assert_eq!(func.block_count(), 1); // Entry block
    }

    #[test]
    fn test_function_reg_alloc() {
        let mut func = IrFunction::new("f", 1);
        // Params use regs 1
        // Next reg should be 2

        let r1 = func.alloc_reg();
        assert_eq!(r1, Reg::new(2));

        let r2 = func.alloc_reg();
        assert_eq!(r2, Reg::new(3));
    }

    #[test]
    fn test_function_blocks() {
        let mut func = IrFunction::new("f", 0);

        let entry = func.entry();
        assert_eq!(entry, BlockId::ENTRY);

        let bb1 = func.new_block();
        assert_eq!(bb1, BlockId::new(1));

        assert_eq!(func.block_count(), 2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // IrBuilder tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_builder_simple_function() {
        let mut builder = IrBuilder::new();

        builder.start_function("add", &["a".into(), "b".into()]);

        // a is %1, b is %2
        let a = builder.lookup("a");
        let b = builder.lookup("b");
        assert_eq!(a, Some(Reg::new(1)));
        assert_eq!(b, Some(Reg::new(2)));

        // result = a + b
        let result = builder.emit_binop(BinOp::Add, a.unwrap_or(Reg::VOID), b.unwrap_or(Reg::VOID));

        // return result
        builder.terminate(Terminator::Return { value: result });

        let func = builder.finish_function();
        assert!(func.is_some());

        let func = func.unwrap_or_else(|| IrFunction::new("", 0));
        assert_eq!(func.name, "add");
        assert!(func.is_pure());
    }

    #[test]
    fn test_builder_with_io() {
        let mut builder = IrBuilder::new();

        builder.start_function("greet", &[]);

        let msg = builder.emit_const(IrConst::String("hello".into()));
        builder.emit_call("print", vec![msg]);
        builder.terminate(Terminator::Return { value: Reg::VOID });

        let func = builder.finish_function();
        let func = func.unwrap_or_else(|| IrFunction::new("", 0));

        assert!(!func.is_pure()); // Has IO effect
        assert!(func.effects.has_io());
    }

    #[test]
    fn test_builder_scopes() {
        let mut builder = IrBuilder::new();
        builder.start_function("test", &[]);

        // Outer scope
        let x = builder.alloc_reg();
        builder.bind("x".into(), x);
        assert_eq!(builder.lookup("x"), Some(x));

        // Inner scope
        builder.push_scope();
        let y = builder.alloc_reg();
        builder.bind("y".into(), y);
        assert_eq!(builder.lookup("x"), Some(x)); // Still visible
        assert_eq!(builder.lookup("y"), Some(y));

        builder.pop_scope();
        assert_eq!(builder.lookup("x"), Some(x));
        assert_eq!(builder.lookup("y"), None); // Out of scope

        builder.terminate(Terminator::Return { value: Reg::VOID });
        let _ = builder.finish_function();
    }

    // ─────────────────────────────────────────────────────────────────────────
    // IrModule tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_module_construction() {
        let mut module = IrModule::new();

        let func = IrFunction::new("main", 0);
        module.add_function(func);
        module.set_entry("main");

        assert!(module.function("main").is_some());
        assert_eq!(module.entry, Some("main".into()));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Terminator tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_terminator_successors() {
        let jump = Terminator::Jump {
            target: BlockId::new(1),
        };
        assert_eq!(jump.successors(), vec![BlockId::new(1)]);

        let branch = Terminator::Branch {
            cond: Reg::new(1),
            then_block: BlockId::new(2),
            else_block: BlockId::new(3),
        };
        assert_eq!(branch.successors(), vec![BlockId::new(2), BlockId::new(3)]);

        let ret = Terminator::Return { value: Reg::VOID };
        assert!(ret.successors().is_empty());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Instruction tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_instruction_dst() {
        let inst = Instruction::LoadConst {
            dst: Reg::new(5),
            value: IrConst::Int(10),
        };
        assert_eq!(inst.dst(), Reg::new(5));

        let inst = Instruction::BinOp {
            dst: Reg::new(3),
            op: BinOp::Add,
            left: Reg::new(1),
            right: Reg::new(2),
        };
        assert_eq!(inst.dst(), Reg::new(3));
    }

    #[test]
    fn test_instruction_effects() {
        let pure_inst = Instruction::LoadConst {
            dst: Reg::new(1),
            value: IrConst::Int(5),
        };
        assert!(pure_inst.is_pure());

        let io_inst = Instruction::Call {
            dst: Reg::new(2),
            func: "println".to_string(),
            args: vec![],
        };
        assert!(!io_inst.is_pure());
        assert!(io_inst.effects().has_io());
    }

    #[test]
    fn test_instruction_composition() {
        let inst = Instruction::BinOp {
            dst: Reg::new(1),
            op: BinOp::Add,
            left: Reg::new(2),
            right: Reg::new(3),
        };
        let comp = inst.composition();
        let unique = comp.unique();

        assert!(unique.contains(&LexPrimitiva::Quantity));
        assert!(unique.contains(&LexPrimitiva::Causality));
        assert!(unique.contains(&LexPrimitiva::Mapping));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Printer tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_format_function() {
        let mut builder = IrBuilder::new();
        builder.start_function("double", &["x".into()]);

        let x = builder.lookup("x").unwrap_or(Reg::VOID);
        let two = builder.emit_const(IrConst::Int(2));
        let result = builder.emit_binop(BinOp::Mul, x, two);
        builder.terminate(Terminator::Return { value: result });

        let func = builder
            .finish_function()
            .unwrap_or_else(|| IrFunction::new("", 0));
        let text = format_function(&func);

        assert!(text.contains("fn double"));
        assert!(text.contains("bb0:"));
        assert!(text.contains("const 2"));
        assert!(text.contains("Mul"));
        assert!(text.contains("return"));
    }

    #[test]
    fn test_format_instruction() {
        let inst = Instruction::LoadConst {
            dst: Reg::new(1),
            value: IrConst::Int(42),
        };
        assert_eq!(format_instruction(&inst), "%1 = const 42");

        let inst = Instruction::Call {
            dst: Reg::new(2),
            func: "print".into(),
            args: vec![Reg::new(1)],
        };
        assert_eq!(format_instruction(&inst), "%2 = call print(%1)");
    }
}
