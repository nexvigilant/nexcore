// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima IR Optimizer
//!
//! Optimization passes for Prima IR.
//!
//! ## Philosophy
//!
//! Optimizations preserve semantics while improving performance:
//! - **Constant Folding** (ν): Evaluate constants at compile time
//! - **Dead Code Elimination** (∅): Remove unreachable code
//! - **Copy Propagation** (μ): Replace copies with originals
//! - **Common Subexpression Elimination** (κ): Reuse computed values
//!
//! ## Tier: T2-C (μ + κ + ν + ∅)
//!
//! ## Pass Pipeline
//!
//! ```text
//! IR → ConstFold → CopyProp → DCE → CSE → Optimized IR
//! ```

use crate::ast::BinOp;
use crate::ir::{BlockId, Instruction, IrConst, IrFunction, IrModule, Reg, Terminator};
use lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition};
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════════════
// OPTIMIZATION PASS TRAIT — μ[IR → IR]
// ═══════════════════════════════════════════════════════════════════════════

/// An optimization pass.
///
/// ## Tier: T2-P (μ + →)
pub trait OptimizationPass {
    /// Pass name for debugging.
    fn name(&self) -> &'static str;

    /// Run the pass on a function.
    fn run_function(&self, func: &mut IrFunction);

    /// Run the pass on a module.
    fn run_module(&self, module: &mut IrModule) {
        for func in module.functions.values_mut() {
            self.run_function(func);
        }
    }

    /// Get primitive composition of this pass.
    fn composition(&self) -> PrimitiveComposition;
}

// ═══════════════════════════════════════════════════════════════════════════
// CONSTANT FOLDING — ν (Invariant evaluation)
// ═══════════════════════════════════════════════════════════════════════════

/// Constant folding: evaluate constant expressions at compile time.
///
/// ## Tier: T2-P (ν + μ)
///
/// ## Examples
///
/// ```text
/// %1 = const 2
/// %2 = const 3
/// %3 = %1 Add %2
/// →
/// %3 = const 5
/// ```
#[derive(Debug, Default)]
pub struct ConstantFolding;

impl ConstantFolding {
    /// Create a new constant folding pass.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Try to fold a binary operation.
    fn fold_binop(&self, op: &BinOp, left: &IrConst, right: &IrConst) -> Option<IrConst> {
        match (op, left, right) {
            // Integer arithmetic
            (BinOp::Add, IrConst::Int(a), IrConst::Int(b)) => Some(IrConst::Int(a + b)),
            (BinOp::Sub, IrConst::Int(a), IrConst::Int(b)) => Some(IrConst::Int(a - b)),
            (BinOp::Mul, IrConst::Int(a), IrConst::Int(b)) => Some(IrConst::Int(a * b)),
            (BinOp::Div, IrConst::Int(a), IrConst::Int(b)) if *b != 0 => Some(IrConst::Int(a / b)),
            (BinOp::Mod, IrConst::Int(a), IrConst::Int(b)) if *b != 0 => Some(IrConst::Int(a % b)),

            // Float arithmetic
            (BinOp::Add, IrConst::Float(a), IrConst::Float(b)) => Some(IrConst::Float(a + b)),
            (BinOp::Sub, IrConst::Float(a), IrConst::Float(b)) => Some(IrConst::Float(a - b)),
            (BinOp::Mul, IrConst::Float(a), IrConst::Float(b)) => Some(IrConst::Float(a * b)),
            (BinOp::Div, IrConst::Float(a), IrConst::Float(b)) if *b != 0.0 => {
                Some(IrConst::Float(a / b))
            }

            // Integer comparisons
            (BinOp::Eq | BinOp::KappaEq, IrConst::Int(a), IrConst::Int(b)) => {
                Some(IrConst::Bool(a == b))
            }
            (BinOp::Ne, IrConst::Int(a), IrConst::Int(b)) => Some(IrConst::Bool(a != b)),
            (BinOp::Lt | BinOp::KappaLt, IrConst::Int(a), IrConst::Int(b)) => {
                Some(IrConst::Bool(a < b))
            }
            (BinOp::Le, IrConst::Int(a), IrConst::Int(b)) => Some(IrConst::Bool(a <= b)),
            (BinOp::Gt | BinOp::KappaGt, IrConst::Int(a), IrConst::Int(b)) => {
                Some(IrConst::Bool(a > b))
            }
            (BinOp::Ge, IrConst::Int(a), IrConst::Int(b)) => Some(IrConst::Bool(a >= b)),

            // Boolean operations
            (BinOp::And, IrConst::Bool(a), IrConst::Bool(b)) => Some(IrConst::Bool(*a && *b)),
            (BinOp::Or, IrConst::Bool(a), IrConst::Bool(b)) => Some(IrConst::Bool(*a || *b)),

            // String concatenation
            (BinOp::Add, IrConst::String(a), IrConst::String(b)) => {
                Some(IrConst::String(format!("{a}{b}")))
            }

            _ => None,
        }
    }
}

impl OptimizationPass for ConstantFolding {
    fn name(&self) -> &'static str {
        "constant-folding"
    }

    fn run_function(&self, func: &mut IrFunction) {
        let mut constants: HashMap<Reg, IrConst> = HashMap::new();

        for block in &mut func.blocks {
            let mut new_instructions = Vec::with_capacity(block.instructions.len());

            for inst in &block.instructions {
                match inst {
                    Instruction::LoadConst { dst, value } => {
                        constants.insert(*dst, value.clone());
                        new_instructions.push(inst.clone());
                    }
                    Instruction::BinOp {
                        dst,
                        op,
                        left,
                        right,
                    } => {
                        let left_const = constants.get(left);
                        let right_const = constants.get(right);

                        if let (Some(l), Some(r)) = (left_const, right_const) {
                            if let Some(folded) = self.fold_binop(op, l, r) {
                                constants.insert(*dst, folded.clone());
                                new_instructions.push(Instruction::LoadConst {
                                    dst: *dst,
                                    value: folded,
                                });
                                continue;
                            }
                        }
                        new_instructions.push(inst.clone());
                    }
                    _ => {
                        new_instructions.push(inst.clone());
                    }
                }
            }

            block.instructions = new_instructions;
        }
    }

    fn composition(&self) -> PrimitiveComposition {
        // ν (Frequency) represents invariants in computation
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // ν — invariant values
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
        ])
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEAD CODE ELIMINATION — ∅ (Removal of unused)
// ═══════════════════════════════════════════════════════════════════════════

/// Dead code elimination: remove instructions whose results are never used.
///
/// ## Tier: T2-P (∅ + κ)
///
/// ## Strategy
///
/// 1. Mark all registers used by terminators as live
/// 2. Walk instructions backwards, marking uses
/// 3. Remove instructions with dead destinations
#[derive(Debug, Default)]
pub struct DeadCodeElimination;

impl DeadCodeElimination {
    /// Create a new DCE pass.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Collect registers used in an instruction.
    fn uses(inst: &Instruction) -> Vec<Reg> {
        match inst {
            Instruction::LoadConst { .. } => vec![],
            Instruction::Copy { src, .. } => vec![*src],
            Instruction::BinOp { left, right, .. } => vec![*left, *right],
            Instruction::UnOp { src, .. } => vec![*src],
            Instruction::Call { args, .. } => args.clone(),
            Instruction::Phi { incoming, .. } => incoming.iter().map(|(_, r)| *r).collect(),
            Instruction::MakeSeq { elements, .. } => elements.clone(),
            Instruction::Index { seq, idx, .. } => vec![*seq, *idx],
            Instruction::Length { seq, .. } => vec![*seq],
        }
    }

    /// Collect registers used in a terminator.
    fn terminator_uses(term: &Terminator) -> Vec<Reg> {
        match term {
            Terminator::Return { value } => vec![*value],
            Terminator::Branch { cond, .. } => vec![*cond],
            _ => vec![],
        }
    }
}

impl OptimizationPass for DeadCodeElimination {
    fn name(&self) -> &'static str {
        "dead-code-elimination"
    }

    fn run_function(&self, func: &mut IrFunction) {
        // Collect all live registers
        let mut live: HashSet<Reg> = HashSet::new();

        // Mark terminator uses as live
        for block in &func.blocks {
            for reg in Self::terminator_uses(&block.terminator) {
                live.insert(reg);
            }
        }

        // Iterate until fixed point
        loop {
            let old_size = live.len();

            for block in &func.blocks {
                for inst in block.instructions.iter().rev() {
                    let dst = inst.dst();
                    if live.contains(&dst) {
                        for used in Self::uses(inst) {
                            live.insert(used);
                        }
                    }
                }
            }

            if live.len() == old_size {
                break;
            }
        }

        // Remove dead instructions
        for block in &mut func.blocks {
            block.instructions.retain(|inst| {
                let dst = inst.dst();
                // Keep if destination is live or instruction has side effects
                live.contains(&dst) || !inst.is_pure()
            });
        }
    }

    fn composition(&self) -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,
            LexPrimitiva::Comparison,
            LexPrimitiva::Sequence,
        ])
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COPY PROPAGATION — μ (Substitute copies)
// ═══════════════════════════════════════════════════════════════════════════

/// Copy propagation: replace uses of copied values with originals.
///
/// ## Tier: T2-P (μ + λ)
///
/// ## Example
///
/// ```text
/// %1 = const 5
/// %2 = copy %1
/// %3 = %2 Add %2
/// →
/// %1 = const 5
/// %2 = copy %1
/// %3 = %1 Add %1
/// ```
#[derive(Debug, Default)]
pub struct CopyPropagation {
    /// Copy chain: dst → src
    copies: HashMap<Reg, Reg>,
}

impl CopyPropagation {
    /// Create a new copy propagation pass.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Find the root of a copy chain.
    fn root(&self, reg: Reg) -> Reg {
        let mut current = reg;
        while let Some(&src) = self.copies.get(&current) {
            current = src;
        }
        current
    }

    /// Replace register uses in an instruction.
    fn propagate_instruction(&self, inst: &Instruction) -> Instruction {
        match inst {
            Instruction::LoadConst { .. } => inst.clone(),
            Instruction::Copy { dst, src } => Instruction::Copy {
                dst: *dst,
                src: self.root(*src),
            },
            Instruction::BinOp {
                dst,
                op,
                left,
                right,
            } => Instruction::BinOp {
                dst: *dst,
                op: *op,
                left: self.root(*left),
                right: self.root(*right),
            },
            Instruction::UnOp { dst, op, src } => Instruction::UnOp {
                dst: *dst,
                op: *op,
                src: self.root(*src),
            },
            Instruction::Call { dst, func, args } => Instruction::Call {
                dst: *dst,
                func: func.clone(),
                args: args.iter().map(|r| self.root(*r)).collect(),
            },
            Instruction::Phi { dst, incoming } => Instruction::Phi {
                dst: *dst,
                incoming: incoming.iter().map(|(b, r)| (*b, self.root(*r))).collect(),
            },
            Instruction::MakeSeq { dst, elements } => Instruction::MakeSeq {
                dst: *dst,
                elements: elements.iter().map(|r| self.root(*r)).collect(),
            },
            Instruction::Index { dst, seq, idx } => Instruction::Index {
                dst: *dst,
                seq: self.root(*seq),
                idx: self.root(*idx),
            },
            Instruction::Length { dst, seq } => Instruction::Length {
                dst: *dst,
                seq: self.root(*seq),
            },
        }
    }
}

impl OptimizationPass for CopyPropagation {
    fn name(&self) -> &'static str {
        "copy-propagation"
    }

    fn run_function(&self, func: &mut IrFunction) {
        // Build copy map
        let mut copies: HashMap<Reg, Reg> = HashMap::new();

        for block in &func.blocks {
            for inst in &block.instructions {
                if let Instruction::Copy { dst, src } = inst {
                    copies.insert(*dst, *src);
                }
            }
        }

        // Create a propagator with the copy map
        let propagator = CopyPropagation { copies };

        // Propagate copies
        for block in &mut func.blocks {
            for inst in &mut block.instructions {
                *inst = propagator.propagate_instruction(inst);
            }

            // Also propagate in terminators
            match &block.terminator {
                Terminator::Branch {
                    cond,
                    then_block,
                    else_block,
                } => {
                    block.terminator = Terminator::Branch {
                        cond: propagator.root(*cond),
                        then_block: *then_block,
                        else_block: *else_block,
                    };
                }
                Terminator::Return { value } => {
                    block.terminator = Terminator::Return {
                        value: propagator.root(*value),
                    };
                }
                _ => {}
            }
        }
    }

    fn composition(&self) -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Location])
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COMMON SUBEXPRESSION ELIMINATION — κ (Detect duplicates)
// ═══════════════════════════════════════════════════════════════════════════

/// Common subexpression elimination: reuse already-computed values.
///
/// ## Tier: T2-C (κ + μ + ς)
///
/// ## Example
///
/// ```text
/// %1 = %a Add %b
/// %2 = %a Add %b
/// →
/// %1 = %a Add %b
/// %2 = copy %1
/// ```
#[derive(Debug, Default)]
pub struct CommonSubexpressionElimination;

impl CommonSubexpressionElimination {
    /// Create a new CSE pass.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Create a hash key for an instruction (without dst).
    fn instruction_key(inst: &Instruction) -> Option<String> {
        match inst {
            Instruction::BinOp {
                op, left, right, ..
            } => Some(format!("binop:{op:?}:{left:?}:{right:?}")),
            Instruction::UnOp { op, src, .. } => Some(format!("unop:{op:?}:{src:?}")),
            Instruction::MakeSeq { elements, .. } => Some(format!("seq:{elements:?}")),
            Instruction::Index { seq, idx, .. } => Some(format!("index:{seq:?}:{idx:?}")),
            Instruction::Length { seq, .. } => Some(format!("len:{seq:?}")),
            _ => None,
        }
    }
}

impl OptimizationPass for CommonSubexpressionElimination {
    fn name(&self) -> &'static str {
        "common-subexpression-elimination"
    }

    fn run_function(&self, func: &mut IrFunction) {
        // Map instruction key → first register that computed it
        let mut computed: HashMap<String, Reg> = HashMap::new();

        for block in &mut func.blocks {
            let mut new_instructions = Vec::with_capacity(block.instructions.len());

            for inst in &block.instructions {
                if let Some(key) = Self::instruction_key(inst) {
                    if let Some(&existing) = computed.get(&key) {
                        // Replace with copy
                        new_instructions.push(Instruction::Copy {
                            dst: inst.dst(),
                            src: existing,
                        });
                        continue;
                    } else {
                        computed.insert(key, inst.dst());
                    }
                }
                new_instructions.push(inst.clone());
            }

            block.instructions = new_instructions;
        }
    }

    fn composition(&self) -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
        ])
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// UNREACHABLE BLOCK ELIMINATION — ∅ + ∂ (Remove unreachable)
// ═══════════════════════════════════════════════════════════════════════════

/// Remove unreachable basic blocks.
///
/// ## Tier: T2-P (∅ + ∂)
#[derive(Debug, Default)]
pub struct UnreachableBlockElimination;

impl UnreachableBlockElimination {
    /// Create a new unreachable block elimination pass.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl OptimizationPass for UnreachableBlockElimination {
    fn name(&self) -> &'static str {
        "unreachable-block-elimination"
    }

    fn run_function(&self, func: &mut IrFunction) {
        // Find reachable blocks via BFS from entry
        let mut reachable: HashSet<u32> = HashSet::new();
        let mut worklist = vec![BlockId::ENTRY.0];

        while let Some(block_id) = worklist.pop() {
            if reachable.contains(&block_id) {
                continue;
            }
            reachable.insert(block_id);

            if let Some(block) = func.blocks.get(block_id as usize) {
                for succ in block.terminator.successors() {
                    worklist.push(succ.0);
                }
            }
        }

        // Remove unreachable blocks
        func.blocks.retain(|b| reachable.contains(&b.id.0));
    }

    fn composition(&self) -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Void, LexPrimitiva::Boundary])
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// OPTIMIZER — σ[Pass] (Pass pipeline)
// ═══════════════════════════════════════════════════════════════════════════

/// Optimization level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptLevel {
    /// No optimization.
    #[default]
    O0,
    /// Basic optimizations.
    O1,
    /// Standard optimizations.
    O2,
    /// Aggressive optimizations.
    O3,
}

/// The optimizer: runs a pipeline of optimization passes.
///
/// ## Tier: T2-C (σ + μ)
pub struct Optimizer {
    /// Optimization level.
    level: OptLevel,
    /// Passes to run.
    passes: Vec<Box<dyn OptimizationPass>>,
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new(OptLevel::O2)
    }
}

impl Optimizer {
    /// Create an optimizer with the given level.
    #[must_use]
    pub fn new(level: OptLevel) -> Self {
        let passes = Self::passes_for_level(level);
        Self { level, passes }
    }

    /// Get passes for an optimization level.
    fn passes_for_level(level: OptLevel) -> Vec<Box<dyn OptimizationPass>> {
        match level {
            OptLevel::O0 => vec![],
            OptLevel::O1 => vec![
                Box::new(ConstantFolding::new()),
                Box::new(DeadCodeElimination::new()),
            ],
            OptLevel::O2 => vec![
                Box::new(ConstantFolding::new()),
                Box::new(CopyPropagation::new()),
                Box::new(DeadCodeElimination::new()),
                Box::new(CommonSubexpressionElimination::new()),
                Box::new(UnreachableBlockElimination::new()),
            ],
            OptLevel::O3 => {
                // Run passes multiple times for better results
                vec![
                    Box::new(ConstantFolding::new()),
                    Box::new(CopyPropagation::new()),
                    Box::new(CommonSubexpressionElimination::new()),
                    Box::new(DeadCodeElimination::new()),
                    Box::new(UnreachableBlockElimination::new()),
                    // Second pass
                    Box::new(ConstantFolding::new()),
                    Box::new(CopyPropagation::new()),
                    Box::new(DeadCodeElimination::new()),
                ]
            }
        }
    }

    /// Get the optimization level.
    #[must_use]
    pub fn level(&self) -> OptLevel {
        self.level
    }

    /// Optimize a module.
    pub fn optimize(&self, module: &mut IrModule) {
        for pass in &self.passes {
            pass.run_module(module);
        }
    }

    /// Optimize a single function.
    pub fn optimize_function(&self, func: &mut IrFunction) {
        for pass in &self.passes {
            pass.run_function(func);
        }
    }

    /// Get pass names.
    #[must_use]
    pub fn pass_names(&self) -> Vec<&'static str> {
        self.passes.iter().map(|p| p.name()).collect()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::IrBuilder;

    // ─────────────────────────────────────────────────────────────────────
    // Constant Folding tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_constant_folding_add() {
        let mut builder = IrBuilder::new();
        builder.start_function("test", &[]);

        let a = builder.emit_const(IrConst::Int(2));
        let b = builder.emit_const(IrConst::Int(3));
        let _c = builder.emit_binop(BinOp::Add, a, b);
        builder.terminate(Terminator::Return { value: Reg::VOID });

        let mut func = builder
            .finish_function()
            .unwrap_or_else(|| IrFunction::new("", 0));

        let pass = ConstantFolding::new();
        pass.run_function(&mut func);

        // Check that the result is folded
        let block = func.blocks.first();
        assert!(block.is_some());
        let block = block.unwrap();

        // Should have a LoadConst with value 5
        let has_five = block.instructions.iter().any(|inst| {
            matches!(
                inst,
                Instruction::LoadConst {
                    value: IrConst::Int(5),
                    ..
                }
            )
        });
        assert!(has_five);
    }

    #[test]
    fn test_constant_folding_comparison() {
        let pass = ConstantFolding::new();

        let result = pass.fold_binop(&BinOp::Lt, &IrConst::Int(1), &IrConst::Int(5));
        assert_eq!(result, Some(IrConst::Bool(true)));

        let result = pass.fold_binop(&BinOp::Eq, &IrConst::Int(5), &IrConst::Int(5));
        assert_eq!(result, Some(IrConst::Bool(true)));

        let result = pass.fold_binop(&BinOp::Gt, &IrConst::Int(3), &IrConst::Int(7));
        assert_eq!(result, Some(IrConst::Bool(false)));
    }

    #[test]
    fn test_constant_folding_boolean() {
        let pass = ConstantFolding::new();

        let result = pass.fold_binop(&BinOp::And, &IrConst::Bool(true), &IrConst::Bool(false));
        assert_eq!(result, Some(IrConst::Bool(false)));

        let result = pass.fold_binop(&BinOp::Or, &IrConst::Bool(true), &IrConst::Bool(false));
        assert_eq!(result, Some(IrConst::Bool(true)));
    }

    #[test]
    fn test_constant_folding_division_by_zero() {
        let pass = ConstantFolding::new();

        let result = pass.fold_binop(&BinOp::Div, &IrConst::Int(5), &IrConst::Int(0));
        assert_eq!(result, None); // Should not fold division by zero
    }

    // ─────────────────────────────────────────────────────────────────────
    // Dead Code Elimination tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_dce_removes_dead_code() {
        let mut builder = IrBuilder::new();
        builder.start_function("test", &[]);

        let a = builder.emit_const(IrConst::Int(42)); // Used
        let _b = builder.emit_const(IrConst::Int(100)); // Dead - not used
        builder.terminate(Terminator::Return { value: a });

        let mut func = builder
            .finish_function()
            .unwrap_or_else(|| IrFunction::new("", 0));

        let original_count = func.blocks[0].instructions.len();

        let pass = DeadCodeElimination::new();
        pass.run_function(&mut func);

        // Should have removed the dead instruction
        assert!(func.blocks[0].instructions.len() < original_count);
    }

    #[test]
    fn test_dce_keeps_used_code() {
        let mut builder = IrBuilder::new();
        builder.start_function("test", &[]);

        let a = builder.emit_const(IrConst::Int(1));
        let b = builder.emit_const(IrConst::Int(2));
        let c = builder.emit_binop(BinOp::Add, a, b);
        builder.terminate(Terminator::Return { value: c });

        let mut func = builder
            .finish_function()
            .unwrap_or_else(|| IrFunction::new("", 0));

        let original_count = func.blocks[0].instructions.len();

        let pass = DeadCodeElimination::new();
        pass.run_function(&mut func);

        // All instructions should be kept (all are used)
        assert_eq!(func.blocks[0].instructions.len(), original_count);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Copy Propagation tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_copy_propagation() {
        let pass = CopyPropagation::new();

        let mut copies = HashMap::new();
        copies.insert(Reg::new(2), Reg::new(1));
        copies.insert(Reg::new(3), Reg::new(2));

        let propagator = CopyPropagation { copies };

        // %3 should resolve to %1 through the chain
        assert_eq!(propagator.root(Reg::new(3)), Reg::new(1));
        assert_eq!(propagator.root(Reg::new(2)), Reg::new(1));
        assert_eq!(propagator.root(Reg::new(1)), Reg::new(1));
    }

    // ─────────────────────────────────────────────────────────────────────
    // Common Subexpression Elimination tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_cse_instruction_key() {
        let inst1 = Instruction::BinOp {
            dst: Reg::new(3),
            op: BinOp::Add,
            left: Reg::new(1),
            right: Reg::new(2),
        };
        let inst2 = Instruction::BinOp {
            dst: Reg::new(4),
            op: BinOp::Add,
            left: Reg::new(1),
            right: Reg::new(2),
        };

        let key1 = CommonSubexpressionElimination::instruction_key(&inst1);
        let key2 = CommonSubexpressionElimination::instruction_key(&inst2);

        // Same operation should have same key (ignoring dst)
        assert_eq!(key1, key2);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Unreachable Block Elimination tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_unreachable_block_elimination() {
        let mut func = IrFunction::new("test", 0);

        // Entry block returns immediately
        if let Some(entry) = func.block_mut(BlockId::ENTRY) {
            entry.terminate(Terminator::Return { value: Reg::VOID });
        }

        // Add an unreachable block
        let unreachable = func.new_block();
        if let Some(block) = func.block_mut(unreachable) {
            block.terminate(Terminator::Return { value: Reg::VOID });
        }

        assert_eq!(func.block_count(), 2);

        let pass = UnreachableBlockElimination::new();
        pass.run_function(&mut func);

        // Should have removed the unreachable block
        assert_eq!(func.block_count(), 1);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Optimizer tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_optimizer_levels() {
        let o0 = Optimizer::new(OptLevel::O0);
        let o1 = Optimizer::new(OptLevel::O1);
        let o2 = Optimizer::new(OptLevel::O2);
        let o3 = Optimizer::new(OptLevel::O3);

        assert_eq!(o0.pass_names().len(), 0);
        assert_eq!(o1.pass_names().len(), 2);
        assert_eq!(o2.pass_names().len(), 5);
        assert!(o3.pass_names().len() > 5); // Multiple iterations
    }

    #[test]
    fn test_optimizer_composition() {
        let pass = ConstantFolding::new();
        let comp = pass.composition();
        let unique = comp.unique();

        assert!(unique.contains(&LexPrimitiva::Frequency));
        assert!(unique.contains(&LexPrimitiva::Mapping));
    }

    #[test]
    fn test_optimizer_full_pipeline() {
        let mut builder = IrBuilder::new();
        builder.start_function("test", &[]);

        // Create code that can be optimized:
        // %1 = 2
        // %2 = 3
        // %3 = %1 + %2  → should fold to 5
        // %4 = copy %3
        // %5 = %4 + 0   → should simplify via copy prop
        // %6 = 999      → dead code

        let a = builder.emit_const(IrConst::Int(2));
        let b = builder.emit_const(IrConst::Int(3));
        let c = builder.emit_binop(BinOp::Add, a, b);
        builder.emit(Instruction::Copy {
            dst: Reg::new(100),
            src: c,
        });
        let _dead = builder.emit_const(IrConst::Int(999));
        builder.terminate(Terminator::Return { value: c });

        let mut func = builder
            .finish_function()
            .unwrap_or_else(|| IrFunction::new("", 0));

        let optimizer = Optimizer::new(OptLevel::O2);
        optimizer.optimize_function(&mut func);

        // After optimization, dead code should be removed
        // and constants should be folded
        let has_999 = func.blocks[0].instructions.iter().any(|inst| {
            matches!(
                inst,
                Instruction::LoadConst {
                    value: IrConst::Int(999),
                    ..
                }
            )
        });
        assert!(!has_999, "Dead code should be eliminated");
    }
}
