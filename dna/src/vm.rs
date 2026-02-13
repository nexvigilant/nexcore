//! Codon VM: a 64-instruction stack machine where codons ARE opcodes.
//!
//! v3 ISA: 8 glyph families × 8 instructions = 64 opcodes.
//! Family 4 (∂ Boundary) contains lifecycle: Entry(32), Halt(33), HaltErr(34), HaltYield(35).
//! Each of the 64 codons maps to exactly one instruction.

use crate::error::{DnaError, Result};
use crate::isa::{self, Instruction};
#[cfg(test)]
use crate::types::Nucleotide;
use crate::types::{Codon, Strand};

/// VM configuration parameters.
///
/// Tier: T2-P (∂ Boundary)
#[derive(Debug, Clone)]
pub struct VmConfig {
    /// Maximum stack depth.
    pub max_stack: usize,
    /// Maximum memory slots.
    pub max_memory: usize,
    /// Maximum execution cycles before halt.
    pub max_cycles: u64,
    /// Enable parity checking on 16-codon blocks (SPEC-v3 §8.3).
    ///
    /// When enabled, every 16th codon is treated as a parity check:
    /// XOR of the preceding 15 codon indices must equal the 16th.
    /// A mismatch halts with `HaltReason::ParityError`.
    pub parity_check: bool,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            max_stack: 256,
            max_memory: 1024,
            max_cycles: 100_000,
            parity_check: false,
        }
    }
}

/// Result of VM execution.
///
/// Tier: T2-C (σ + ς + N)
#[derive(Debug, Clone)]
pub struct VmResult {
    /// Values in the output buffer.
    pub output: Vec<i64>,
    /// Final stack state.
    pub stack: Vec<i64>,
    /// Total cycles executed.
    pub cycles: u64,
    /// How the program terminated.
    pub halt_reason: HaltReason,
}

/// How the VM halted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HaltReason {
    /// Normal halt (v3 ISA index 33).
    Normal,
    /// Error halt (v3 ISA index 34).
    Error,
    /// Yield halt (v3 ISA index 35).
    Yield,
    /// Reached end of program without explicit halt.
    EndOfProgram,
    /// Parity check failed at the given block index (SPEC-v3 §8.3).
    ParityError(usize),
}

/// The 64-instruction Codon Virtual Machine.
///
/// Tier: T3 (σ + μ + ς + ∂ + N + →)
///
/// Two-phase operation: `load(&Strand)` then `execute()`.
/// Entry point: first Entry codon (v3 ISA index 32) in the program.
#[derive(Debug)]
pub struct CodonVM {
    stack: Vec<i64>,
    program: Vec<Codon>,
    pc: usize,
    memory: Vec<i64>,
    accumulator: i64,
    counter: u64,
    output: Vec<i64>,
    call_stack: Vec<usize>,
    config: VmConfig,
}

impl CodonVM {
    /// Create a new VM with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(VmConfig::default())
    }

    /// Create a new VM with custom configuration.
    #[must_use]
    pub fn with_config(config: VmConfig) -> Self {
        let memory_size = config.max_memory;
        Self {
            stack: Vec::new(),
            program: Vec::new(),
            pc: 0,
            memory: vec![0i64; memory_size],
            accumulator: 0,
            counter: 0,
            output: Vec::new(),
            call_stack: Vec::new(),
            config,
        }
    }

    /// Load a program from a DNA/RNA strand.
    ///
    /// The strand length must be divisible by 3.
    /// If `parity_check` is enabled, verifies all 16-codon blocks.
    pub fn load(&mut self, strand: &Strand) -> Result<()> {
        self.program = strand.codons()?;
        self.pc = 0;
        self.stack.clear();
        self.output.clear();
        self.accumulator = 0;
        self.counter = 0;
        self.call_stack.clear();
        // Reset memory
        for slot in &mut self.memory {
            *slot = 0;
        }

        // Parity verification (SPEC-v3 §8.3)
        if self.config.parity_check {
            self.verify_parity()?;
        }

        Ok(())
    }

    /// Verify parity of all complete 16-codon blocks.
    ///
    /// Each block: 15 data codons + 1 parity codon.
    /// Parity = XOR of all 15 codon indices (masked to 6 bits).
    fn verify_parity(&self) -> Result<()> {
        let block_size = 16;
        let num_blocks = self.program.len() / block_size;

        for block_idx in 0..num_blocks {
            let base = block_idx * block_size;
            let mut xor: u8 = 0;
            for i in 0..15 {
                xor ^= self.program[base + i].index() & 0x3F;
            }
            let check = self.program[base + 15].index() & 0x3F;
            if xor != check {
                return Err(DnaError::ParityError(block_idx));
            }
        }

        Ok(())
    }

    /// Execute the loaded program.
    ///
    /// Finds the first Entry codon and begins execution from there.
    pub fn execute(&mut self) -> Result<VmResult> {
        // Find entry point (first Entry codon)
        let entry = self
            .program
            .iter()
            .position(|c| c.is_start())
            .ok_or(DnaError::NoEntryPoint)?;

        // Start AFTER the entry marker — it's not an instruction to dispatch.
        self.run_from(entry + 1)
    }

    /// Execute from a specific codon address.
    ///
    /// Used for gene expression: jump directly to a gene's start codon
    /// without searching for the Entry marker. The caller is responsible
    /// for setting up the stack (e.g. pushing function arguments).
    pub fn execute_from(&mut self, addr: usize) -> Result<VmResult> {
        if addr >= self.program.len() {
            return Err(DnaError::InvalidAddress(addr));
        }
        self.run_from(addr)
    }

    /// Internal run loop starting at the given PC.
    fn run_from(&mut self, start: usize) -> Result<VmResult> {
        self.pc = start;
        let mut cycles: u64 = 0;

        loop {
            if cycles >= self.config.max_cycles {
                return Err(DnaError::ExecutionLimit(cycles));
            }

            if self.pc >= self.program.len() {
                return Ok(VmResult {
                    output: self.output.clone(),
                    stack: self.stack.clone(),
                    cycles,
                    halt_reason: HaltReason::EndOfProgram,
                });
            }

            let codon = self.program[self.pc];
            cycles += 1;

            match self.dispatch(&codon)? {
                Flow::Continue => {
                    self.pc += 1;
                }
                Flow::Jump(addr) => {
                    self.pc = addr;
                }
                Flow::Halt(reason) => {
                    return Ok(VmResult {
                        output: self.output.clone(),
                        stack: self.stack.clone(),
                        cycles,
                        halt_reason: reason,
                    });
                }
            }
        }
    }

    /// Push a value onto the stack (for test setup).
    pub fn push_value(&mut self, value: i64) -> Result<()> {
        if self.stack.len() >= self.config.max_stack {
            return Err(DnaError::StackOverflow(self.pc, self.stack.len()));
        }
        self.stack.push(value);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // ISA-based dispatch: decode codon → match on Instruction
    // -----------------------------------------------------------------------

    fn dispatch(&mut self, codon: &Codon) -> Result<Flow> {
        match isa::decode(codon) {
            // Family 0 — σ Sequence (Data Flow)
            Instruction::Nop => Ok(Flow::Continue),
            Instruction::Dup => {
                let val = self.peek()?;
                self.push(val)?;
                Ok(Flow::Continue)
            }
            Instruction::Swap => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.push(a)?;
                self.push(b)?;
                Ok(Flow::Continue)
            }
            Instruction::Pop => {
                self.pop()?;
                Ok(Flow::Continue)
            }
            Instruction::Rot => {
                let c = self.pop()?;
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(b)?;
                self.push(c)?;
                self.push(a)?;
                Ok(Flow::Continue)
            }
            Instruction::Over => {
                if self.stack.len() < 2 {
                    return Err(DnaError::StackUnderflow(self.pc));
                }
                let val = self.stack[self.stack.len() - 2];
                self.push(val)?;
                Ok(Flow::Continue)
            }
            Instruction::Pick => {
                let n = self.pop()? as usize;
                if n >= self.stack.len() {
                    return Err(DnaError::StackUnderflow(self.pc));
                }
                let val = self.stack[self.stack.len() - 1 - n];
                self.push(val)?;
                Ok(Flow::Continue)
            }
            Instruction::Depth => {
                let depth = self.stack.len() as i64;
                self.push(depth)?;
                Ok(Flow::Continue)
            }

            // Family 1 — μ Mapping (Transform)
            Instruction::Add => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a.wrapping_add(b))?;
                Ok(Flow::Continue)
            }
            Instruction::Sub => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a.wrapping_sub(b))?;
                Ok(Flow::Continue)
            }
            Instruction::Mul => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a.wrapping_mul(b))?;
                Ok(Flow::Continue)
            }
            Instruction::Div => {
                let b = self.pop()?;
                if b == 0 {
                    return Err(DnaError::DivisionByZero(self.pc));
                }
                let a = self.pop()?;
                self.push(a / b)?;
                Ok(Flow::Continue)
            }
            Instruction::Mod => {
                let b = self.pop()?;
                if b == 0 {
                    return Err(DnaError::DivisionByZero(self.pc));
                }
                let a = self.pop()?;
                self.push(a % b)?;
                Ok(Flow::Continue)
            }
            Instruction::Neg => {
                let a = self.pop()?;
                self.push(a.wrapping_neg())?;
                Ok(Flow::Continue)
            }
            Instruction::Abs => {
                let a = self.pop()?;
                self.push(a.wrapping_abs())?;
                Ok(Flow::Continue)
            }
            Instruction::Inc => {
                let a = self.pop()?;
                self.push(a.wrapping_add(1))?;
                Ok(Flow::Continue)
            }

            // Family 2 — ς State (Storage)
            Instruction::Load => {
                let addr = self.pop()? as usize;
                if addr >= self.memory.len() {
                    return Err(DnaError::InvalidAddress(addr));
                }
                self.push(self.memory[addr])?;
                Ok(Flow::Continue)
            }
            Instruction::Store => {
                let addr = self.pop()? as usize;
                let val = self.pop()?;
                if addr >= self.memory.len() {
                    return Err(DnaError::InvalidAddress(addr));
                }
                self.memory[addr] = val;
                Ok(Flow::Continue)
            }
            Instruction::Push0 => {
                self.push(0)?;
                Ok(Flow::Continue)
            }
            Instruction::Push1 => {
                self.push(1)?;
                Ok(Flow::Continue)
            }
            Instruction::PushNeg1 => {
                self.push(-1)?;
                Ok(Flow::Continue)
            }
            Instruction::PushAcc => {
                self.push(self.accumulator)?;
                Ok(Flow::Continue)
            }
            Instruction::StoreAcc => {
                self.accumulator = self.pop()?;
                Ok(Flow::Continue)
            }
            Instruction::Peek => {
                let val = self.peek()?;
                self.push(val)?;
                Ok(Flow::Continue)
            }

            // Family 3 — ρ Recursion (Iteration)
            Instruction::Dec => {
                let a = self.pop()?;
                self.push(a.wrapping_sub(1))?;
                Ok(Flow::Continue)
            }
            Instruction::Sign => {
                let a = self.pop()?;
                self.push(a.signum())?;
                Ok(Flow::Continue)
            }
            Instruction::Clamp => {
                let max = self.pop()?;
                let min = self.pop()?;
                let val = self.pop()?;
                self.push(val.clamp(min, max))?;
                Ok(Flow::Continue)
            }
            Instruction::Min => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a.min(b))?;
                Ok(Flow::Continue)
            }
            Instruction::Max => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a.max(b))?;
                Ok(Flow::Continue)
            }
            Instruction::Pow => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = if b < 0 {
                    0
                } else if b > 62 {
                    if a == 0 {
                        0
                    } else if a == 1 {
                        1
                    } else if a == -1 {
                        if b % 2 == 0 { 1 } else { -1 }
                    } else {
                        i64::MAX
                    }
                } else {
                    a.wrapping_pow(b as u32)
                };
                self.push(result)?;
                Ok(Flow::Continue)
            }
            Instruction::Sqrt => {
                let a = self.pop()?;
                let result = if a < 0 { 0 } else { isqrt(a) };
                self.push(result)?;
                Ok(Flow::Continue)
            }
            Instruction::Log2 => {
                let a = self.pop()?;
                let result = if a <= 0 {
                    0
                } else {
                    63 - a.leading_zeros() as i64
                };
                self.push(result)?;
                Ok(Flow::Continue)
            }

            // Family 4 — ∂ Boundary (Lifecycle)
            Instruction::Entry => self.dispatch_push_lit(),
            Instruction::Halt => Ok(Flow::Halt(HaltReason::Normal)),
            Instruction::HaltErr => Ok(Flow::Halt(HaltReason::Error)),
            Instruction::HaltYield => Ok(Flow::Halt(HaltReason::Yield)),
            Instruction::Assert => {
                let val = self.pop()?;
                if val == 0 {
                    Ok(Flow::Halt(HaltReason::Error))
                } else {
                    Ok(Flow::Continue)
                }
            }
            Instruction::Output => {
                let val = self.pop()?;
                self.output.push(val);
                Ok(Flow::Continue)
            }
            Instruction::MemSize => {
                self.push(self.memory.len() as i64)?;
                Ok(Flow::Continue)
            }
            Instruction::MemClear => {
                for slot in &mut self.memory {
                    *slot = 0;
                }
                Ok(Flow::Continue)
            }

            // Family 5 — → Causality (Control)
            Instruction::Jmp => {
                let addr = self.pop()? as usize;
                if addr >= self.program.len() {
                    return Err(DnaError::InvalidAddress(addr));
                }
                Ok(Flow::Jump(addr))
            }
            Instruction::JmpIf => {
                let addr = self.pop()? as usize;
                let cond = self.pop()?;
                if cond != 0 {
                    if addr >= self.program.len() {
                        return Err(DnaError::InvalidAddress(addr));
                    }
                    Ok(Flow::Jump(addr))
                } else {
                    Ok(Flow::Continue)
                }
            }
            Instruction::JmpIfZ => {
                let addr = self.pop()? as usize;
                let cond = self.pop()?;
                if cond == 0 {
                    if addr >= self.program.len() {
                        return Err(DnaError::InvalidAddress(addr));
                    }
                    Ok(Flow::Jump(addr))
                } else {
                    Ok(Flow::Continue)
                }
            }
            Instruction::JmpBack => {
                let offset = self.pop()? as usize;
                let addr = self.pc.saturating_sub(offset);
                Ok(Flow::Jump(addr))
            }
            Instruction::Call => {
                let addr = self.pop()? as usize;
                if addr >= self.program.len() {
                    return Err(DnaError::InvalidAddress(addr));
                }
                self.call_stack.push(self.pc + 1);
                Ok(Flow::Jump(addr))
            }
            Instruction::Ret => {
                if let Some(ret_addr) = self.call_stack.pop() {
                    Ok(Flow::Jump(ret_addr))
                } else {
                    Ok(Flow::Halt(HaltReason::Normal))
                }
            }
            Instruction::IfElse => {
                let else_val = self.pop()?;
                let then_val = self.pop()?;
                let cond = self.pop()?;
                self.push(if cond != 0 { then_val } else { else_val })?;
                Ok(Flow::Continue)
            }
            Instruction::Cmp => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a.cmp(&b) as i64)?;
                Ok(Flow::Continue)
            }

            // Family 6 — κ Comparison (Testing)
            Instruction::Eq => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(i64::from(a == b))?;
                Ok(Flow::Continue)
            }
            Instruction::Neq => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(i64::from(a != b))?;
                Ok(Flow::Continue)
            }
            Instruction::Lt => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(i64::from(a < b))?;
                Ok(Flow::Continue)
            }
            Instruction::Gt => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(i64::from(a > b))?;
                Ok(Flow::Continue)
            }
            Instruction::And => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(i64::from(a != 0 && b != 0))?;
                Ok(Flow::Continue)
            }
            Instruction::Or => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(i64::from(a != 0 || b != 0))?;
                Ok(Flow::Continue)
            }
            Instruction::Dup2 => {
                if self.stack.len() < 2 {
                    return Err(DnaError::StackUnderflow(self.pc));
                }
                let a = self.stack[self.stack.len() - 2];
                let b = self.stack[self.stack.len() - 1];
                self.push(a)?;
                self.push(b)?;
                Ok(Flow::Continue)
            }
            Instruction::IsEmpty => {
                self.push(i64::from(self.stack.is_empty()))?;
                Ok(Flow::Continue)
            }

            // Family 7 — N Quantity (Numeric)
            Instruction::Shl => {
                let n = self.pop()?;
                let a = self.pop()?;
                let shift = (n & 63) as u32;
                self.push(a.wrapping_shl(shift))?;
                Ok(Flow::Continue)
            }
            Instruction::Shr => {
                let n = self.pop()?;
                let a = self.pop()?;
                let shift = (n & 63) as u32;
                self.push(a.wrapping_shr(shift))?;
                Ok(Flow::Continue)
            }
            Instruction::BitAnd => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a & b)?;
                Ok(Flow::Continue)
            }
            Instruction::BitOr => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a | b)?;
                Ok(Flow::Continue)
            }
            Instruction::BitXor => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(a ^ b)?;
                Ok(Flow::Continue)
            }
            Instruction::BitNot => {
                let a = self.pop()?;
                self.push(!a)?;
                Ok(Flow::Continue)
            }
            Instruction::CntInc => {
                self.counter = self.counter.wrapping_add(1);
                Ok(Flow::Continue)
            }
            Instruction::CntRead => {
                self.push(self.counter as i64)?;
                Ok(Flow::Continue)
            }

            // Pseudo-instruction — should not appear at runtime
            Instruction::Lit(_) => Ok(Flow::Continue),
        }
    }

    // --- PUSH_LIT: multi-codon literal encoding ---

    fn dispatch_push_lit(&mut self) -> Result<Flow> {
        // Read length codon
        self.pc += 1;
        if self.pc >= self.program.len() {
            return Err(DnaError::InvalidAddress(self.pc));
        }
        let len = self.program[self.pc].index() as usize;

        if len == 0 {
            self.push(0)?;
            return Ok(Flow::Continue);
        }

        // Read digit codons
        let mut digits = Vec::with_capacity(len);
        for _ in 0..len {
            self.pc += 1;
            if self.pc >= self.program.len() {
                return Err(DnaError::InvalidAddress(self.pc));
            }
            digits.push(self.program[self.pc].index());
        }

        let value = isa::decode_literal(&digits);
        self.push(value)?;
        Ok(Flow::Continue)
    }

    // --- Public memory access ---

    /// Write a value to VM memory at the given address.
    ///
    /// Used by `Program` to pre-load data segments.
    pub fn write_memory(&mut self, addr: usize, value: i64) -> Result<()> {
        if addr >= self.memory.len() {
            return Err(DnaError::InvalidAddress(addr));
        }
        self.memory[addr] = value;
        Ok(())
    }

    /// Read a value from VM memory at the given address.
    pub fn read_memory(&self, addr: usize) -> Result<i64> {
        if addr >= self.memory.len() {
            return Err(DnaError::InvalidAddress(addr));
        }
        Ok(self.memory[addr])
    }

    // --- Stack helpers ---

    fn push(&mut self, val: i64) -> Result<()> {
        if self.stack.len() >= self.config.max_stack {
            return Err(DnaError::StackOverflow(self.pc, self.stack.len()));
        }
        self.stack.push(val);
        Ok(())
    }

    fn pop(&mut self) -> Result<i64> {
        self.stack.pop().ok_or(DnaError::StackUnderflow(self.pc))
    }

    fn peek(&self) -> Result<i64> {
        self.stack
            .last()
            .copied()
            .ok_or(DnaError::StackUnderflow(self.pc))
    }
}

impl Default for CodonVM {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal control flow signal.
enum Flow {
    Continue,
    Jump(usize),
    Halt(HaltReason),
}

/// Integer square root (floor).
fn isqrt(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

// ---------------------------------------------------------------------------
// Helper: build strand from codon sequence
// ---------------------------------------------------------------------------

/// Build a strand from a slice of (first, second, third) nucleotide tuples.
#[cfg(test)]
#[must_use]
fn codons_to_strand(codons: &[(Nucleotide, Nucleotide, Nucleotide)]) -> Strand {
    let mut bases = Vec::with_capacity(codons.len() * 3);
    for &(a, b, c) in codons {
        bases.push(a);
        bases.push(b);
        bases.push(c);
    }
    Strand::new(bases)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// ISA-agnostic helper: instruction → nucleotide triple.
    /// Survives any future ISA reclassification.
    fn nuc(instr: Instruction) -> (Nucleotide, Nucleotide, Nucleotide) {
        match isa::encode(&instr) {
            Some(c) => (c.0, c.1, c.2),
            None => (Nucleotide::A, Nucleotide::A, Nucleotide::A),
        }
    }

    /// Helper to build a program strand from nucleotide triples.
    fn program(codons: &[(Nucleotide, Nucleotide, Nucleotide)]) -> Strand {
        codons_to_strand(codons)
    }

    #[test]
    fn halt_normal() {
        let prog = program(&[nuc(Instruction::Entry), nuc(Instruction::Halt)]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.halt_reason, HaltReason::Normal);
        }
    }

    #[test]
    fn halt_error() {
        let prog = program(&[nuc(Instruction::Entry), nuc(Instruction::HaltErr)]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.halt_reason, HaltReason::Error);
        }
    }

    #[test]
    fn halt_yield() {
        let prog = program(&[nuc(Instruction::Entry), nuc(Instruction::HaltYield)]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.halt_reason, HaltReason::Yield);
        }
    }

    #[test]
    fn push_constants() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Push1),
            nuc(Instruction::Push0),
            nuc(Instruction::PushNeg1),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.stack, vec![1, 0, -1]);
        }
    }

    #[test]
    fn arithmetic_add() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Push1),
            nuc(Instruction::Push1),
            nuc(Instruction::Add),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.stack, vec![2]);
        }
    }

    #[test]
    fn arithmetic_sub() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Sub),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        assert!(vm.push_value(5).is_ok());
        assert!(vm.push_value(3).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.stack, vec![2]);
        }
    }

    #[test]
    fn arithmetic_mul() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Mul),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        assert!(vm.push_value(6).is_ok());
        assert!(vm.push_value(7).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.stack, vec![42]);
        }
    }

    #[test]
    fn arithmetic_div() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Div),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        assert!(vm.push_value(10).is_ok());
        assert!(vm.push_value(3).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.stack, vec![3]);
        }
    }

    #[test]
    fn division_by_zero() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Div),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        assert!(vm.push_value(10).is_ok());
        assert!(vm.push_value(0).is_ok());
        let result = vm.execute();
        assert!(result.is_err());
    }

    #[test]
    fn stack_dup_swap_pop() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Dup),
            nuc(Instruction::Swap),
            nuc(Instruction::Pop),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        assert!(vm.push_value(42).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            // 42 → dup → [42,42] → swap → [42,42] → pop → [42]
            assert_eq!(r.stack, vec![42]);
        }
    }

    #[test]
    fn output_instruction() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Output),
            nuc(Instruction::Output),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        assert!(vm.push_value(100).is_ok());
        assert!(vm.push_value(200).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.output, vec![200, 100]);
        }
    }

    #[test]
    fn comparison_eq() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Eq),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        assert!(vm.push_value(5).is_ok());
        assert!(vm.push_value(5).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.stack, vec![1]);
        }
    }

    #[test]
    fn comparison_neq() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Neq),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        assert!(vm.push_value(5).is_ok());
        assert!(vm.push_value(3).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.stack, vec![1]);
        }
    }

    #[test]
    fn memory_load_store() {
        // STORE pops: addr first, then value. LOAD pops: addr.
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Store),
            nuc(Instruction::Load),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        assert!(vm.push_value(0).is_ok()); // load addr (consumed third)
        assert!(vm.push_value(42).is_ok()); // store value (consumed second)
        assert!(vm.push_value(0).is_ok()); // store addr (consumed first)
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.stack, vec![42]);
        }
    }

    #[test]
    fn stack_overflow() {
        let config = VmConfig {
            max_stack: 4,
            ..VmConfig::default()
        };
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Push1),
            nuc(Instruction::Push1),
            nuc(Instruction::Push1),
            nuc(Instruction::Push1),
            nuc(Instruction::Push1), // overflow
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::with_config(config);
        assert!(vm.load(&prog).is_ok());
        let result = vm.execute();
        assert!(result.is_err());
    }

    #[test]
    fn stack_underflow() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Add), // needs 2 operands, stack empty
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        let result = vm.execute();
        assert!(result.is_err());
    }

    #[test]
    fn execution_limit() {
        let config = VmConfig {
            max_cycles: 5,
            ..VmConfig::default()
        };
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Nop),
            nuc(Instruction::Nop),
            nuc(Instruction::Nop),
            nuc(Instruction::Nop),
            nuc(Instruction::Nop),
            nuc(Instruction::Nop),
        ]);
        let mut vm = CodonVM::with_config(config);
        assert!(vm.load(&prog).is_ok());
        let result = vm.execute();
        assert!(result.is_err());
    }

    #[test]
    fn no_entry_point() {
        // Program with no Entry instruction
        let prog = program(&[nuc(Instruction::Nop), nuc(Instruction::Halt)]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        let result = vm.execute();
        assert!(result.is_err());
    }

    #[test]
    fn accumulator_store_and_push() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::StoreAcc),
            nuc(Instruction::PushAcc),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        assert!(vm.push_value(42).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.stack, vec![42]);
        }
    }

    #[test]
    fn counter_inc_and_read() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::CntInc),
            nuc(Instruction::CntInc),
            nuc(Instruction::CntInc),
            nuc(Instruction::CntRead),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.stack, vec![3]);
        }
    }

    #[test]
    fn bitwise_and_or_xor() {
        // AND: 0b1100 & 0b1010 = 0b1000 = 8
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::BitAnd),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        assert!(vm.push_value(0b1100).is_ok());
        assert!(vm.push_value(0b1010).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.stack, vec![8]);
        }
    }

    #[test]
    fn if_else_true() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::IfElse),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        assert!(vm.push_value(1).is_ok()); // cond = true
        assert!(vm.push_value(10).is_ok()); // then
        assert!(vm.push_value(20).is_ok()); // else
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.stack, vec![10]);
        }
    }

    #[test]
    fn if_else_false() {
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::IfElse),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        assert!(vm.push_value(0).is_ok()); // cond = false
        assert!(vm.push_value(10).is_ok()); // then
        assert!(vm.push_value(20).is_ok()); // else
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.stack, vec![20]);
        }
    }

    #[test]
    fn isqrt_correctness() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(4), 2);
        assert_eq!(isqrt(9), 3);
        assert_eq!(isqrt(10), 3);
        assert_eq!(isqrt(100), 10);
    }

    #[test]
    fn factorial_program() {
        // Compute 5! = 120: pre-push values and multiply
        let prog = program(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Mul),
            nuc(Instruction::Mul),
            nuc(Instruction::Mul),
            nuc(Instruction::Mul),
            nuc(Instruction::Halt),
        ]);
        let mut vm = CodonVM::new();
        assert!(vm.load(&prog).is_ok());
        // Push 1, 2, 3, 4, 5 → mul chain: 1*2=2, 2*3=6, 6*4=24, 24*5=120
        assert!(vm.push_value(1).is_ok());
        assert!(vm.push_value(2).is_ok());
        assert!(vm.push_value(3).is_ok());
        assert!(vm.push_value(4).is_ok());
        assert!(vm.push_value(5).is_ok());
        let result = vm.execute();
        assert!(result.is_ok());
        if let Some(r) = result.ok() {
            assert_eq!(r.stack, vec![120]);
        }
    }

    // --- Parity check tests ---

    #[test]
    fn parity_check_disabled_by_default() {
        // Default config has parity_check = false
        let config = VmConfig::default();
        assert!(!config.parity_check);
    }

    #[test]
    fn parity_check_valid_block() {
        // Build a 16-codon program: 15 instructions + 1 parity codon
        // The parity codon = XOR of the first 15 codon indices (masked to 6 bits)
        let instrs = [
            Instruction::Entry,
            Instruction::Push1,
            Instruction::Output,
            Instruction::Halt,
            Instruction::Nop,
            Instruction::Nop,
            Instruction::Nop,
            Instruction::Nop,
            Instruction::Nop,
            Instruction::Nop,
            Instruction::Nop,
            Instruction::Nop,
            Instruction::Nop,
            Instruction::Nop,
            Instruction::Nop,
        ];

        // Compute parity XOR over 15 codon indices
        let mut xor: u8 = 0;
        let mut codon_triples: Vec<(Nucleotide, Nucleotide, Nucleotide)> = Vec::with_capacity(16);
        for instr in &instrs {
            let triple = nuc(*instr);
            // Recover index from the codon
            if let Some(c) = isa::encode(instr) {
                xor ^= c.index() & 0x3F;
            }
            codon_triples.push(triple);
        }

        // The 16th codon must have index == xor (masked to 6 bits)
        // Use decode_index to find what instruction has this index, then encode it
        let parity_instr = isa::decode_index(xor & 0x3F);
        codon_triples.push(nuc(parity_instr));

        let strand = codons_to_strand(&codon_triples);
        let config = VmConfig {
            parity_check: true,
            ..VmConfig::default()
        };
        let mut vm = CodonVM::with_config(config);
        // Should load successfully — parity is valid
        assert!(vm.load(&strand).is_ok());
    }

    #[test]
    fn parity_check_invalid_block() {
        // Build a 16-codon program with wrong parity
        let mut codon_triples: Vec<(Nucleotide, Nucleotide, Nucleotide)> = Vec::with_capacity(16);
        for _ in 0..15 {
            codon_triples.push(nuc(Instruction::Nop));
        }
        // Wrong parity: Nop has some index, XOR of 15 identical values
        // won't match a different instruction's index
        codon_triples.push(nuc(Instruction::Halt)); // almost certainly wrong parity

        let strand = codons_to_strand(&codon_triples);
        let config = VmConfig {
            parity_check: true,
            ..VmConfig::default()
        };
        let mut vm = CodonVM::with_config(config);
        let result = vm.load(&strand);
        assert!(result.is_err());
    }

    #[test]
    fn parity_check_short_program_ok() {
        // Programs shorter than 16 codons have no complete blocks — parity passes
        let strand = codons_to_strand(&[nuc(Instruction::Entry), nuc(Instruction::Halt)]);
        let config = VmConfig {
            parity_check: true,
            ..VmConfig::default()
        };
        let mut vm = CodonVM::with_config(config);
        assert!(vm.load(&strand).is_ok());
    }
}
