//! Disassembler: Strand → human-readable listing.
//!
//! Converts a DNA strand (compiled program) back to a readable instruction
//! listing with codon offsets, raw nucleotides, amino acid names, and
//! instruction mnemonics.
//!
//! Tier: T2-C (μ Mapping + σ Sequence + κ Comparison + ∃ Existence)

use crate::error::Result;
use crate::isa;
use crate::types::{Codon, Strand};

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/// Disassembler display options.
///
/// Tier: T2-P (∂ Boundary + κ Comparison)
#[derive(Debug, Clone)]
pub struct DisasmOptions {
    /// Show raw codon nucleotides (e.g., "ATG").
    pub show_codons: bool,
    /// Show amino acid column (e.g., "Met").
    pub show_amino: bool,
    /// Show codon offset numbers.
    pub show_offsets: bool,
}

impl Default for DisasmOptions {
    fn default() -> Self {
        Self {
            show_codons: true,
            show_amino: true,
            show_offsets: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Disassembler
// ---------------------------------------------------------------------------

/// Disassemble a strand into a human-readable listing.
pub fn disassemble(strand: &Strand) -> Result<String> {
    disassemble_with(strand, &DisasmOptions::default())
}

/// Disassemble with custom options.
pub fn disassemble_with(strand: &Strand, opts: &DisasmOptions) -> Result<String> {
    let codons = strand.codons()?;
    let mut output = String::new();
    let table = crate::codon_table::CodonTable::standard();

    // Header
    output.push_str("; nexcore-dna disassembly\n");
    output.push_str(&format!(
        "; {} codons, {} nucleotides\n\n",
        codons.len(),
        strand.len()
    ));

    let mut i = 0;
    let mut is_first_atg = true;

    while i < codons.len() {
        let codon = &codons[i];
        let instr = isa::decode(codon);
        let aa = table.translate(codon);

        // Check if this ATG is the entry point or a PUSH_LIT
        if matches!(instr, isa::Instruction::Entry) {
            if is_first_atg {
                // Entry point
                is_first_atg = false;
                format_line(&mut output, i, codon, &aa, "entry", opts);
                i += 1;
            } else {
                // PUSH_LIT sequence: ATG + len + digits
                let lit_result = decode_push_lit(&codons, i);
                match lit_result {
                    Some((value, consumed)) => {
                        let annotation = format!("lit {value}");
                        format_line(&mut output, i, codon, &aa, &annotation, opts);
                        i += consumed;
                    }
                    None => {
                        // Malformed — show as raw entry
                        format_line(&mut output, i, codon, &aa, "entry", opts);
                        i += 1;
                    }
                }
            }
        } else {
            let mnemonic = isa::to_mnemonic(&instr);
            format_line(&mut output, i, codon, &aa, mnemonic, opts);
            i += 1;
        }
    }

    Ok(output)
}

/// Format a single disassembly line.
fn format_line(
    output: &mut String,
    offset: usize,
    codon: &Codon,
    aa: &crate::types::AminoAcid,
    mnemonic: &str,
    opts: &DisasmOptions,
) {
    if opts.show_offsets {
        output.push_str(&format!("{offset:4}  "));
    }
    if opts.show_codons {
        output.push_str(&format!(
            "{}{}{}  ",
            codon.0.as_char(),
            codon.1.as_char(),
            codon.2.as_char()
        ));
    }
    if opts.show_amino {
        output.push_str(&format!("{:<4} ", aa.abbrev()));
    }
    output.push_str(mnemonic);
    output.push('\n');
}

/// Decode a PUSH_LIT sequence starting at position `start` (the ATG codon).
/// Returns `(value, total_codons_consumed)` or None if malformed.
fn decode_push_lit(codons: &[Codon], start: usize) -> Option<(i64, usize)> {
    // start points to ATG, next should be the length codon
    let len_pos = start + 1;
    if len_pos >= codons.len() {
        return None;
    }

    let len = codons[len_pos].index() as usize;

    if len == 0 {
        // Literal 0
        return Some((0, 2)); // ATG + len
    }

    // Read digit codons
    let mut digits = Vec::with_capacity(len);
    for j in 0..len {
        let pos = len_pos + 1 + j;
        if pos >= codons.len() {
            return None;
        }
        digits.push(codons[pos].index());
    }

    let value = isa::decode_literal(&digits);
    let consumed = 1 + 1 + len; // ATG + len + digits
    Some((value, consumed))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::isa::{self, Instruction};
    use crate::types::Nucleotide;

    /// ISA-agnostic helper: instruction → nucleotide triple.
    fn nuc(instr: Instruction) -> (Nucleotide, Nucleotide, Nucleotide) {
        match isa::encode(&instr) {
            Some(c) => (c.0, c.1, c.2),
            None => (Nucleotide::A, Nucleotide::A, Nucleotide::A),
        }
    }

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
    fn disassemble_simple() {
        // entry → push1 → out → halt
        let strand = strand_from(&[
            nuc(Instruction::Entry),
            nuc(Instruction::Push1),
            nuc(Instruction::Output),
            nuc(Instruction::Halt),
        ]);
        let result = disassemble(&strand);
        assert!(result.is_ok());
        if let Ok(text) = result {
            assert!(text.contains("entry"));
            assert!(text.contains("push1"));
            assert!(text.contains("out"));
            assert!(text.contains("halt"));
        }
    }

    #[test]
    fn disassemble_header() {
        let strand = strand_from(&[nuc(Instruction::Entry), nuc(Instruction::Halt)]);
        let result = disassemble(&strand);
        assert!(result.is_ok());
        if let Ok(text) = result {
            assert!(text.contains("; nexcore-dna disassembly"));
            assert!(text.contains("2 codons"));
            assert!(text.contains("6 nucleotides"));
        }
    }

    #[test]
    fn disassemble_options_no_codons() {
        let strand = strand_from(&[nuc(Instruction::Entry), nuc(Instruction::Halt)]);
        let opts = DisasmOptions {
            show_codons: false,
            show_amino: true,
            show_offsets: true,
        };
        let result = disassemble_with(&strand, &opts);
        assert!(result.is_ok());
        if let Ok(text) = result {
            // With show_codons=false, no nucleotide triplets should appear
            let data_lines: Vec<&str> = text
                .lines()
                .filter(|l| !l.starts_with(';') && !l.is_empty())
                .collect();
            for line in data_lines {
                // No 3-letter nucleotide codes in data lines
                assert!(!line.contains("GAA"));
                assert!(!line.contains("GAT"));
            }
        }
    }

    #[test]
    fn disassemble_options_no_amino() {
        let strand = strand_from(&[nuc(Instruction::Entry), nuc(Instruction::Halt)]);
        let opts = DisasmOptions {
            show_codons: true,
            show_amino: false,
            show_offsets: true,
        };
        let result = disassemble_with(&strand, &opts);
        assert!(result.is_ok());
        if let Ok(text) = result {
            // Data lines should not contain amino acid abbreviations
            let data_lines: Vec<&str> = text
                .lines()
                .filter(|l| !l.starts_with(';') && !l.is_empty())
                .collect();
            for line in data_lines {
                assert!(!line.contains("Glu "));
                assert!(!line.contains("Asp "));
            }
        }
    }

    #[test]
    fn disassemble_with_pushlit() {
        // Assemble then disassemble to verify PUSH_LIT annotation
        let source = "
.code
    entry
    lit 42
    out
    halt
";
        let prog = crate::asm::assemble(source);
        assert!(prog.is_ok());
        if let Ok(p) = prog {
            let result = disassemble(p.strand());
            assert!(result.is_ok());
            if let Ok(text) = result {
                assert!(text.contains("lit 42"));
                assert!(text.contains("entry"));
                assert!(text.contains("out"));
                assert!(text.contains("halt"));
            }
        }
    }

    #[test]
    fn disassemble_roundtrip_mnemonics() {
        // Build a program, disassemble it, check key mnemonics appear
        let source = "
.code
    entry
    push1
    push1
    add
    out
    halt
";
        let prog = crate::asm::assemble(source);
        assert!(prog.is_ok());
        if let Ok(p) = prog {
            let result = disassemble(p.strand());
            assert!(result.is_ok());
            if let Ok(text) = result {
                assert!(text.contains("push1"));
                assert!(text.contains("add"));
            }
        }
    }

    #[test]
    fn decode_push_lit_value() {
        // Entry marker codon + len(1) + digit(5)
        let entry_codon = match isa::encode(&Instruction::Entry) {
            Some(c) => c,
            None => return, // Entry must encode
        };
        let len_codon = match Codon::from_index(1) {
            Ok(c) => c,
            Err(_) => return, // index 1 must be valid
        };
        let digit_codon = match Codon::from_index(5) {
            Ok(c) => c,
            Err(_) => return, // index 5 must be valid
        };
        let codons = vec![entry_codon, len_codon, digit_codon];
        let result = decode_push_lit(&codons, 0);
        assert!(result.is_some());
        if let Some((val, consumed)) = result {
            assert_eq!(val, 5);
            assert_eq!(consumed, 3);
        }
    }
}
