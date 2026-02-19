//! Program format: data + code segments for structured DNA execution.
//!
//! A `Program` bundles a data segment (pre-loaded into VM memory) with
//! an executable code strand. This is the output of the assembler and the
//! input to the VM.
//!
//! Tier: T2-C (σ Sequence + ∂ Boundary + ς State + μ Mapping)

use crate::error::Result;
use crate::types::Strand;
use crate::vm::{CodonVM, VmConfig, VmResult};

// ---------------------------------------------------------------------------
// Program
// ---------------------------------------------------------------------------

/// A compiled DNA program with data and code segments.
///
/// Tier: T2-C (σ + ∂ + ς + μ)
///
/// The data segment is loaded into VM memory starting at address 0.
/// The code segment is loaded as the executable strand.
#[derive(Debug, Clone)]
pub struct Program {
    /// Values to pre-load into VM memory (index = memory address).
    pub data: Vec<i64>,
    /// The executable code as a DNA strand.
    pub code: Strand,
}

impl Program {
    /// Create a program from just code (no data segment).
    #[must_use]
    pub fn code_only(code: Strand) -> Self {
        Self {
            data: Vec::new(),
            code,
        }
    }

    /// Create a program with both data and code segments.
    #[must_use]
    pub fn new(data: Vec<i64>, code: Strand) -> Self {
        Self { data, code }
    }

    /// Execute this program on a new VM with default config.
    pub fn run(&self) -> Result<VmResult> {
        self.run_with(VmConfig::default())
    }

    /// Execute with custom VM config.
    pub fn run_with(&self, config: VmConfig) -> Result<VmResult> {
        let mut vm = CodonVM::with_config(config);
        vm.load(&self.code)?;

        // Pre-load data segment into memory
        for (addr, &value) in self.data.iter().enumerate() {
            vm.write_memory(addr, value)?;
        }

        vm.execute()
    }

    /// Get the raw DNA strand (code segment).
    #[must_use]
    pub fn strand(&self) -> &Strand {
        &self.code
    }

    /// Get total codon count in the code segment.
    pub fn codon_count(&self) -> Result<usize> {
        let codons = self.code.codons()?;
        Ok(codons.len())
    }

    /// Get the number of data values.
    #[must_use]
    pub fn data_count(&self) -> usize {
        self.data.len()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::isa::{self, Instruction};
    use crate::types::Nucleotide;
    use crate::vm::HaltReason;

    /// ISA-agnostic helper: instruction → nucleotide triple.
    fn nuc(instr: Instruction) -> (Nucleotide, Nucleotide, Nucleotide) {
        match isa::encode(&instr) {
            Some(c) => (c.0, c.1, c.2),
            None => (Nucleotide::A, Nucleotide::A, Nucleotide::A),
        }
    }

    /// Build a strand from nucleotide triples.
    fn strand_from(codons: &[(Nucleotide, Nucleotide, Nucleotide)]) -> Strand {
        let mut bases = Vec::with_capacity(codons.len() * 3);
        for &(a, b, c) in codons {
            bases.push(a);
            bases.push(b);
            bases.push(c);
        }
        Strand::new(bases)
    }

    #[test]
    fn code_only_program() {
        // entry → push1 → out → halt
        let code = strand_from(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Push1),
            nuc(Instruction::Output),
            nuc(Instruction::Halt),
        ]);
        let prog = Program::code_only(code);
        assert_eq!(prog.data_count(), 0);
        let result = prog.run();
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.output, vec![1]);
            assert_eq!(r.halt_reason, HaltReason::Normal);
        }
    }

    #[test]
    fn data_segment_loaded() {
        // Load data[0]=42, then load it and output
        let code = strand_from(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Push0),
            nuc(Instruction::Load),
            nuc(Instruction::Output),
            nuc(Instruction::Halt),
        ]);
        let prog = Program::new(vec![42], code);
        let result = prog.run();
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.output, vec![42]);
        }
    }

    #[test]
    fn multi_data_values() {
        // Load data[0]=10, data[1]=20, add them
        let code = strand_from(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Push0),
            nuc(Instruction::Load),
            nuc(Instruction::Push1),
            nuc(Instruction::Load),
            nuc(Instruction::Add),
            nuc(Instruction::Output),
            nuc(Instruction::Halt),
        ]);
        let prog = Program::new(vec![10, 20], code);
        let result = prog.run();
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.output, vec![30]);
        }
    }

    #[test]
    fn codon_count() {
        let code = strand_from(&[nuc(Instruction::Entry), nuc(Instruction::Halt)]);
        let prog = Program::code_only(code);
        let count = prog.codon_count();
        assert!(count.is_ok());
        if let Ok(n) = count {
            assert_eq!(n, 2);
        }
    }

    #[test]
    fn custom_config() {
        let code = strand_from(&[nuc(Instruction::Entry), nuc(Instruction::Halt)]);
        let prog = Program::code_only(code);
        let config = VmConfig {
            max_stack: 64,
            max_memory: 256,
            max_cycles: 1000,
            ..VmConfig::default()
        };
        let result = prog.run_with(config);
        assert!(result.is_ok());
    }

    #[test]
    fn data_exceeds_memory_fails() {
        let code = strand_from(&[nuc(Instruction::Entry), nuc(Instruction::Halt)]);
        // Try to load 2000 values into 1024-slot memory
        let data: Vec<i64> = (0..2000).collect();
        let prog = Program::new(data, code);
        let result = prog.run();
        assert!(result.is_err());
    }
}
