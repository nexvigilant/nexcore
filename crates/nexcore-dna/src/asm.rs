//! Two-pass assembler: text → Program.
//!
//! Translates human-readable assembly source into executable DNA programs.
//!
//! ## Syntax
//!
//! ```text
//! ; comment
//! .data              ; data segment
//!     42             ; mem[0] = 42
//! .code              ; code segment
//!     entry          ; ATG entry point
//!     lit 5          ; push literal 5
//! loop:              ; label declaration
//!     dec
//!     lit @loop      ; label reference (resolves to codon offset)
//!     jmpif
//!     halt
//! ```
//!
//! Tier: T3 (σ + μ + ∂ + ς + → + ∃)

use crate::error::{DnaError, Result};
use crate::isa;
use crate::program::Program;
use crate::types::{Codon, Strand};

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Token
// ---------------------------------------------------------------------------

/// A parsed token from assembly source.
///
/// Tier: T2-P (∂ Boundary + ∃ Existence)
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// An instruction mnemonic (e.g., "add", "halt").
    Mnemonic(String),
    /// A label declaration (e.g., "loop:").
    Label(String),
    /// A label reference (e.g., "@loop").
    LabelRef(String),
    /// A numeric literal value.
    Literal(i64),
    /// A section directive.
    Section(Section),
}

/// Assembly section type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Data,
    Code,
}

// ---------------------------------------------------------------------------
// Assembler
// ---------------------------------------------------------------------------

/// Assemble source text into a Program.
///
/// Two-pass assembly:
/// 1. Scan for labels, count codon offsets, parse data values
/// 2. Generate codons with resolved label addresses
pub fn assemble(source: &str) -> Result<Program> {
    let lines = parse_lines(source)?;
    let (data, instructions) = pass1(&lines)?;
    let labels = resolve_labels(&instructions)?;
    let code = pass2(&instructions, &labels)?;
    Ok(Program::new(data, code))
}

/// Assemble source and also return resolved label positions (codon offsets).
///
/// Used by the compiler to annotate gene boundaries in compiled genomes.
/// Returns `(Program, HashMap<label_name, codon_offset>)`.
pub fn assemble_with_labels(source: &str) -> Result<(Program, HashMap<String, usize>)> {
    let lines = parse_lines(source)?;
    let (data, instructions) = pass1(&lines)?;
    let labels = resolve_labels(&instructions)?;
    let code = pass2(&instructions, &labels)?;
    Ok((Program::new(data, code), labels))
}

// ---------------------------------------------------------------------------
// Line parsing
// ---------------------------------------------------------------------------

/// A parsed source line with its original line number.
#[derive(Debug)]
struct SourceLine {
    line_num: usize,
    tokens: Vec<Token>,
}

/// Parse all non-empty, non-comment lines into token sequences.
fn parse_lines(source: &str) -> Result<Vec<SourceLine>> {
    let mut result = Vec::new();

    for (i, line) in source.lines().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();

        // Skip empty lines and pure comments
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }

        // Strip inline comments
        let code_part = if let Some(pos) = trimmed.find(';') {
            trimmed[..pos].trim()
        } else {
            trimmed
        };

        if code_part.is_empty() {
            continue;
        }

        let tokens = tokenize(code_part, line_num)?;
        if !tokens.is_empty() {
            result.push(SourceLine { line_num, tokens });
        }
    }

    Ok(result)
}

/// Tokenize a single line (comments already stripped).
fn tokenize(line: &str, line_num: usize) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let parts: Vec<&str> = line.split_whitespace().collect();

    let mut i = 0;
    while i < parts.len() {
        let part = parts[i];

        if part == ".data" {
            tokens.push(Token::Section(Section::Data));
        } else if part == ".code" {
            tokens.push(Token::Section(Section::Code));
        } else if let Some(name) = part.strip_suffix(':') {
            // Label declaration
            if name.is_empty() {
                return Err(DnaError::SyntaxError(line_num, "empty label name".into()));
            }
            tokens.push(Token::Label(name.to_lowercase()));
        } else if let Some(label) = part.strip_prefix('@') {
            // Label reference
            if label.is_empty() {
                return Err(DnaError::SyntaxError(
                    line_num,
                    "empty label reference".into(),
                ));
            }
            tokens.push(Token::LabelRef(label.to_lowercase()));
        } else if part == "lit" {
            // Lit requires an argument
            i += 1;
            if i >= parts.len() {
                return Err(DnaError::SyntaxError(
                    line_num,
                    "'lit' requires a value or @label argument".into(),
                ));
            }
            let arg = parts[i];
            if let Some(label) = arg.strip_prefix('@') {
                // lit @label — push label address
                tokens.push(Token::Mnemonic("lit".into()));
                tokens.push(Token::LabelRef(label.to_lowercase()));
            } else {
                // lit N — push numeric literal
                let val: i64 = arg
                    .parse()
                    .map_err(|_| DnaError::InvalidLiteral(arg.into()))?;
                tokens.push(Token::Mnemonic("lit".into()));
                tokens.push(Token::Literal(val));
            }
        } else {
            // Check if it's a number (for data segment)
            let lower = part.to_lowercase();
            if let Ok(val) = lower.parse::<i64>() {
                tokens.push(Token::Literal(val));
            } else {
                // Must be a mnemonic
                tokens.push(Token::Mnemonic(lower));
            }
        }

        i += 1;
    }

    Ok(tokens)
}

// ---------------------------------------------------------------------------
// Pass 1: scan labels + count offsets
// ---------------------------------------------------------------------------

/// An intermediate instruction before label resolution.
#[derive(Debug, Clone)]
enum AsmInstr {
    /// A single codon instruction.
    Simple(isa::Instruction),
    /// A literal with known value.
    LitValue(i64),
    /// A literal with a label reference (resolved iteratively).
    LitLabel(String),
    /// A label marker (zero-width, records label position in instruction stream).
    LabelMarker(String),
}

impl AsmInstr {
    /// How many codons this instruction occupies in the code strand.
    fn codon_count(&self) -> usize {
        match self {
            AsmInstr::Simple(_) => 1,
            AsmInstr::LitValue(v) => literal_total_codons(*v),
            AsmInstr::LitLabel(_) => 4, // initial estimate, refined by resolve_labels
            AsmInstr::LabelMarker(_) => 0,
        }
    }
}

/// Exact codon count for a literal value (including ATG marker).
fn literal_total_codons(value: i64) -> usize {
    match value {
        -1..=1 => 1,                               // optimized to push0/push1/push-1
        _ => 1 + isa::encode_literal(value).len(), // ATG + (len + digits)
    }
}

/// Pass 1 output: (data_values, instructions).
/// Labels are embedded as LabelMarker instructions and resolved iteratively.
type Pass1Result = (Vec<i64>, Vec<AsmInstr>);

/// Pass 1: parse data segment, build instruction list with label markers.
fn pass1(lines: &[SourceLine]) -> Result<Pass1Result> {
    let mut data = Vec::new();
    let mut instructions: Vec<AsmInstr> = Vec::new();
    let mut seen_labels: HashMap<String, usize> = HashMap::new();
    let mut section = Section::Code; // default to code if no section directive

    for line in lines {
        let tokens = &line.tokens;
        let mut ti = 0;

        while ti < tokens.len() {
            match &tokens[ti] {
                Token::Section(s) => {
                    section = *s;
                }
                Token::Label(name) => {
                    if seen_labels.contains_key(name) {
                        return Err(DnaError::DuplicateLabel(name.clone()));
                    }
                    seen_labels.insert(name.clone(), instructions.len());
                    instructions.push(AsmInstr::LabelMarker(name.clone()));
                }
                Token::Literal(val) => match section {
                    Section::Data => {
                        data.push(*val);
                    }
                    Section::Code => {
                        return Err(DnaError::SyntaxError(
                            line.line_num,
                            format!("unexpected literal {val} in code section (use 'lit {val}')"),
                        ));
                    }
                },
                Token::Mnemonic(m) => {
                    if m == "lit" {
                        // Consume the next token as the lit argument
                        ti += 1;
                        if ti >= tokens.len() {
                            return Err(DnaError::SyntaxError(
                                line.line_num,
                                "'lit' missing argument".into(),
                            ));
                        }
                        match &tokens[ti] {
                            Token::Literal(val) => {
                                instructions.push(AsmInstr::LitValue(*val));
                            }
                            Token::LabelRef(name) => {
                                instructions.push(AsmInstr::LitLabel(name.clone()));
                            }
                            _ => {
                                return Err(DnaError::SyntaxError(
                                    line.line_num,
                                    "'lit' requires a number or @label".into(),
                                ));
                            }
                        }
                    } else if let Some(instr) = isa::from_mnemonic(m) {
                        instructions.push(AsmInstr::Simple(instr));
                    } else {
                        return Err(DnaError::UnknownMnemonic(m.clone()));
                    }
                }
                Token::LabelRef(name) => {
                    // Bare label ref without preceding lit — treat as lit @label
                    instructions.push(AsmInstr::LitLabel(name.clone()));
                }
            }
            ti += 1;
        }
    }

    Ok((data, instructions))
}

// ---------------------------------------------------------------------------
// Label resolution (iterative fixpoint)
// ---------------------------------------------------------------------------

/// Resolve label addresses iteratively until stable.
///
/// LitLabel codons have variable size depending on the address value.
/// We iterate: compute offsets → resolve labels → recompute sizes → repeat.
/// Converges in 1-3 iterations for typical programs.
fn resolve_labels(instructions: &[AsmInstr]) -> Result<HashMap<String, usize>> {
    let mut labels: HashMap<String, usize> = HashMap::new();

    // Initial pass: compute offsets with estimated LitLabel sizes
    let mut offset = 0;
    for instr in instructions {
        match instr {
            AsmInstr::LabelMarker(name) => {
                labels.insert(name.clone(), offset);
            }
            _ => {
                offset += instr.codon_count();
            }
        }
    }

    // Iterative refinement: recompute with actual literal sizes
    for _ in 0..10 {
        let mut new_labels: HashMap<String, usize> = HashMap::new();
        let mut new_offset = 0;
        let mut changed = false;

        for instr in instructions {
            match instr {
                AsmInstr::LabelMarker(name) => {
                    if labels.get(name) != Some(&new_offset) {
                        changed = true;
                    }
                    new_labels.insert(name.clone(), new_offset);
                }
                AsmInstr::LitLabel(name) => {
                    let addr = labels.get(name).copied().unwrap_or(0) as i64;
                    new_offset += literal_total_codons(addr);
                }
                _ => {
                    new_offset += instr.codon_count();
                }
            }
        }

        labels = new_labels;
        if !changed {
            break;
        }
    }

    Ok(labels)
}

// ---------------------------------------------------------------------------
// Pass 2: generate codons
// ---------------------------------------------------------------------------

/// Pass 2: generate the code strand from instructions with resolved labels.
fn pass2(instructions: &[AsmInstr], labels: &HashMap<String, usize>) -> Result<Strand> {
    let mut codons: Vec<Codon> = Vec::new();

    for instr in instructions {
        match instr {
            AsmInstr::Simple(instruction) => {
                if let Some(codon) = isa::encode(instruction) {
                    codons.push(codon);
                }
            }
            AsmInstr::LitValue(val) => {
                emit_literal(*val, &mut codons)?;
            }
            AsmInstr::LitLabel(name) => {
                let addr = labels
                    .get(name)
                    .copied()
                    .ok_or_else(|| DnaError::UndefinedLabel(name.clone()))?;
                emit_literal(addr as i64, &mut codons)?;
            }
            AsmInstr::LabelMarker(_) => {
                // Zero-width marker — no codons emitted
            }
        }
    }

    // Convert codons to nucleotide bases
    let mut bases = Vec::with_capacity(codons.len() * 3);
    for c in &codons {
        bases.push(c.0);
        bases.push(c.1);
        bases.push(c.2);
    }

    Ok(Strand::new(bases))
}

/// Emit codons for a literal value, optimizing small constants.
fn emit_literal(value: i64, codons: &mut Vec<Codon>) -> Result<()> {
    // Optimize: use single-codon push for 0, 1, -1
    match value {
        0 => {
            if let Some(c) = isa::encode(&isa::Instruction::Push0) {
                codons.push(c);
            }
        }
        1 => {
            if let Some(c) = isa::encode(&isa::Instruction::Push1) {
                codons.push(c);
            }
        }
        -1 => {
            if let Some(c) = isa::encode(&isa::Instruction::PushNeg1) {
                codons.push(c);
            }
        }
        _ => {
            // ATG marker for PUSH_LIT
            if let Some(atg) = isa::encode(&isa::Instruction::Entry) {
                codons.push(atg);
            }
            // Encoded digits (includes len codon)
            let encoded = isa::encode_literal(value);
            codons.extend_from_slice(&encoded);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::HaltReason;

    #[test]
    fn assemble_minimal() {
        let source = "
.code
    entry
    halt
";
        let prog = assemble(source);
        assert!(prog.is_ok());
        if let Ok(p) = prog {
            assert_eq!(p.data_count(), 0);
            let result = p.run();
            assert!(result.is_ok());
            if let Ok(r) = result {
                assert_eq!(r.halt_reason, HaltReason::Normal);
            }
        }
    }

    #[test]
    fn assemble_push_and_output() {
        let source = "
.code
    entry
    push1
    out
    halt
";
        let prog = assemble(source);
        assert!(prog.is_ok());
        if let Ok(p) = prog {
            let result = p.run();
            assert!(result.is_ok());
            if let Ok(r) = result {
                assert_eq!(r.output, vec![1]);
            }
        }
    }

    #[test]
    fn assemble_literal() {
        let source = "
.code
    entry
    lit 42
    out
    halt
";
        let prog = assemble(source);
        assert!(prog.is_ok());
        if let Ok(p) = prog {
            let result = p.run();
            assert!(result.is_ok());
            if let Ok(r) = result {
                assert_eq!(r.output, vec![42]);
            }
        }
    }

    #[test]
    fn assemble_literal_zero_optimized() {
        let source = "
.code
    entry
    lit 0
    out
    halt
";
        let prog = assemble(source);
        assert!(prog.is_ok());
        if let Ok(p) = prog {
            let result = p.run();
            assert!(result.is_ok());
            if let Ok(r) = result {
                assert_eq!(r.output, vec![0]);
            }
        }
    }

    #[test]
    fn assemble_literal_negative() {
        let source = "
.code
    entry
    lit -42
    out
    halt
";
        let prog = assemble(source);
        assert!(prog.is_ok());
        if let Ok(p) = prog {
            let result = p.run();
            assert!(result.is_ok());
            if let Ok(r) = result {
                assert_eq!(r.output, vec![-42]);
            }
        }
    }

    #[test]
    fn assemble_data_segment() {
        let source = "
.data
    42
    100
.code
    entry
    push0
    load
    out
    halt
";
        let prog = assemble(source);
        assert!(prog.is_ok());
        if let Ok(p) = prog {
            assert_eq!(p.data.len(), 2);
            let result = p.run();
            assert!(result.is_ok());
            if let Ok(r) = result {
                assert_eq!(r.output, vec![42]);
            }
        }
    }

    #[test]
    fn assemble_arithmetic() {
        let source = "
.code
    entry
    lit 5
    lit 3
    add
    out
    halt
";
        let prog = assemble(source);
        assert!(prog.is_ok());
        if let Ok(p) = prog {
            let result = p.run();
            assert!(result.is_ok());
            if let Ok(r) = result {
                assert_eq!(r.output, vec![8]);
            }
        }
    }

    #[test]
    fn assemble_comments() {
        let source = "
; This is a full-line comment
.code       ; section directive
    entry   ; entry point
    push1   ; push 1
    out     ; output it
    halt    ; done
";
        let prog = assemble(source);
        assert!(prog.is_ok());
        if let Ok(p) = prog {
            let result = p.run();
            assert!(result.is_ok());
            if let Ok(r) = result {
                assert_eq!(r.output, vec![1]);
            }
        }
    }

    #[test]
    fn assemble_unknown_mnemonic() {
        let source = "
.code
    entry
    bogus
    halt
";
        let result = assemble(source);
        assert!(result.is_err());
    }

    #[test]
    fn assemble_duplicate_label() {
        let source = "
.code
    entry
loop:
    nop
loop:
    halt
";
        let result = assemble(source);
        assert!(result.is_err());
    }

    #[test]
    fn assemble_undefined_label() {
        let source = "
.code
    entry
    lit @missing
    jmp
    halt
";
        let result = assemble(source);
        assert!(result.is_err());
    }

    #[test]
    fn assemble_lit_missing_arg() {
        let source = "
.code
    entry
    lit
";
        let result = assemble(source);
        assert!(result.is_err());
    }

    #[test]
    fn assemble_empty_source() {
        // No entry point → VM will error
        let result = assemble("");
        assert!(result.is_ok());
        if let Ok(p) = result {
            // Running should fail with NoEntryPoint
            let run_result = p.run();
            assert!(run_result.is_err());
        }
    }

    #[test]
    fn assemble_case_insensitive() {
        let source = "
.code
    ENTRY
    PUSH1
    OUT
    HALT
";
        let prog = assemble(source);
        assert!(prog.is_ok());
        if let Ok(p) = prog {
            let result = p.run();
            assert!(result.is_ok());
            if let Ok(r) = result {
                assert_eq!(r.output, vec![1]);
            }
        }
    }

    #[test]
    fn tokenize_basic() {
        let tokens = tokenize("entry", 1);
        assert!(tokens.is_ok());
        if let Ok(t) = tokens {
            assert_eq!(t.len(), 1);
            assert_eq!(t[0], Token::Mnemonic("entry".into()));
        }
    }

    #[test]
    fn tokenize_label() {
        let tokens = tokenize("loop:", 1);
        assert!(tokens.is_ok());
        if let Ok(t) = tokens {
            assert_eq!(t.len(), 1);
            assert_eq!(t[0], Token::Label("loop".into()));
        }
    }

    #[test]
    fn tokenize_section() {
        let tokens = tokenize(".data", 1);
        assert!(tokens.is_ok());
        if let Ok(t) = tokens {
            assert_eq!(t.len(), 1);
            assert_eq!(t[0], Token::Section(Section::Data));
        }
    }

    #[test]
    fn tokenize_lit_number() {
        let tokens = tokenize("lit 42", 1);
        assert!(tokens.is_ok());
        if let Ok(t) = tokens {
            assert_eq!(t.len(), 2);
            assert_eq!(t[0], Token::Mnemonic("lit".into()));
            assert_eq!(t[1], Token::Literal(42));
        }
    }

    #[test]
    fn tokenize_lit_label() {
        let tokens = tokenize("lit @loop", 1);
        assert!(tokens.is_ok());
        if let Ok(t) = tokens {
            assert_eq!(t.len(), 2);
            assert_eq!(t[0], Token::Mnemonic("lit".into()));
            assert_eq!(t[1], Token::LabelRef("loop".into()));
        }
    }
}
