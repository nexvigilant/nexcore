//! Instruction Set Architecture: the single source of truth for all 64 VM instructions.
//!
//! Every codon maps to exactly one instruction. The ISA provides:
//! - `Instruction` enum (64 real + 1 pseudo-instruction `Lit`)
//! - Encode: `Instruction` → `Codon`
//! - Decode: `Codon` → `Instruction`
//! - Mnemonic lookup (bidirectional)
//! - Full catalog for introspection
//!
//! Tier: T2-C (μ Mapping + σ Sequence + ∂ Boundary + κ Comparison)

use crate::types::Codon;

// ---------------------------------------------------------------------------
// Instruction enum
// ---------------------------------------------------------------------------

/// All 64 VM instructions plus the `Lit` pseudo-instruction.
///
/// Tier: T2-P (μ Mapping + ∂ Boundary)
///
/// Each variant maps to exactly one codon index (0-63), except `Lit`
/// which the assembler expands to an ATG + length + digit codon sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    // --- Stop family (3) ---
    /// TAA (16) — normal termination
    Halt,
    /// TAG (18) — error termination
    HaltErr,
    /// TGA (24) — yield termination
    HaltYield,

    // --- Met (1) ---
    /// ATG (6) — program entry point
    Entry,

    // --- Gly: Stack (4) ---
    /// GGA (40) — no operation
    Nop,
    /// GGT (41) — duplicate top of stack
    Dup,
    /// GGG (42) — swap top two
    Swap,
    /// GGC (43) — discard top
    Pop,

    // --- Ala: Arithmetic (4) ---
    /// GCA (44) — addition
    Add,
    /// GCT (45) — subtraction
    Sub,
    /// GCG (46) — multiplication
    Mul,
    /// GCC (47) — division
    Div,

    // --- Val: Compare (4) ---
    /// GTA (36) — equal
    Eq,
    /// GTT (37) — less than
    Lt,
    /// GTG (38) — greater than
    Gt,
    /// GTC (39) — not equal
    Neq,

    // --- Leu: Control (6) ---
    /// TTA (20) — absolute jump
    Jmp,
    /// TTG (22) — conditional jump (nonzero)
    JmpIf,
    /// CTA (52) — relative backward jump
    JmpBack,
    /// CTT (53) — function call
    Call,
    /// CTG (54) — function return
    Ret,
    /// CTC (55) — conditional jump (zero)
    JmpIfZ,

    // --- Ser: Math (6) ---
    /// TCA (28) — modulo
    Mod,
    /// TCT (29) — absolute value
    Abs,
    /// TCG (30) — negate
    Neg,
    /// TCC (31) — increment
    Inc,
    /// AGT (9) — decrement
    Dec,
    /// AGC (11) — maximum
    Max,

    // --- Pro: Bitwise (4) ---
    /// CCA (60) — bitwise AND
    BitAnd,
    /// CCT (61) — bitwise OR
    BitOr,
    /// CCG (62) — bitwise XOR
    BitXor,
    /// CCC (63) — bitwise NOT
    BitNot,

    // --- Thr: Stack2 (4) ---
    /// ACA (12) — rotate top 3
    Rot,
    /// ACT (13) — copy second-from-top
    Over,
    /// ACG (14) — copy nth from top
    Pick,
    /// ACC (15) — push stack depth
    Depth,

    // --- Ile: Constants (3) ---
    /// ATA (4) — push 0
    Push0,
    /// ATT (5) — push 1
    Push1,
    /// ATC (7) — push -1
    PushNeg1,

    // --- Phe: Memory (2) ---
    /// TTT (21) — load from memory
    Load,
    /// TTC (23) — store to memory
    Store,

    // --- Trp: I/O (1) ---
    /// TGG (26) — output value
    Output,

    // --- Arg: Extended math (6) ---
    /// CGA (56) — minimum
    Min,
    /// CGT (57) — power
    Pow,
    /// CGG (58) — integer square root
    Sqrt,
    /// CGC (59) — integer log base 2
    Log2,
    /// AGA (8) — sign (-1, 0, 1)
    Sign,
    /// AGG (10) — clamp(val, min, max)
    Clamp,

    // --- Tyr (2) ---
    /// TAT (17) — duplicate top two
    Dup2,
    /// TAC (19) — three-way compare
    Cmp,

    // --- His (2) ---
    /// CAT (49) — push memory size
    MemSize,
    /// CAC (51) — clear all memory
    MemClear,

    // --- Cys (2) ---
    /// TGT (25) — conditional select
    IfElse,
    /// TGC (27) — assert nonzero
    Assert,

    // --- Asn (2) ---
    /// AAT (1) — shift left
    Shl,
    /// AAC (3) — shift right
    Shr,

    // --- Gln (2) ---
    /// CAA (48) — logical AND
    And,
    /// CAG (50) — logical OR
    Or,

    // --- Lys (2) ---
    /// AAA (0) — push accumulator
    PushAcc,
    /// AAG (2) — store to accumulator
    StoreAcc,

    // --- Asp (2) ---
    /// GAT (33) — peek at top (non-destructive dup)
    Peek,
    /// GAC (35) — push 1 if stack empty
    IsEmpty,

    // --- Glu (2) ---
    /// GAA (32) — increment counter
    CntInc,
    /// GAG (34) — read counter
    CntRead,

    // --- Pseudo-instruction ---
    /// Assembler-only: expands to ATG + length + digit codons.
    Lit(i64),
}

// ---------------------------------------------------------------------------
// Codon index → Instruction (decode)
// ---------------------------------------------------------------------------

/// Decode a raw index (0-63) into its instruction.
///
/// Values outside 0-63 map to `Nop` (safety fallback).
#[must_use]
pub fn decode_index(index: u8) -> Instruction {
    match index {
        // Family 0 — σ Sequence (Data Flow)
        0 => Instruction::Nop,
        1 => Instruction::Dup,
        2 => Instruction::Swap,
        3 => Instruction::Pop,
        4 => Instruction::Rot,
        5 => Instruction::Over,
        6 => Instruction::Pick,
        7 => Instruction::Depth,
        // Family 1 — μ Mapping (Transform)
        8 => Instruction::Add,
        9 => Instruction::Sub,
        10 => Instruction::Mul,
        11 => Instruction::Div,
        12 => Instruction::Mod,
        13 => Instruction::Neg,
        14 => Instruction::Abs,
        15 => Instruction::Inc,
        // Family 2 — ς State (Storage)
        16 => Instruction::Load,
        17 => Instruction::Store,
        18 => Instruction::Push0,
        19 => Instruction::Push1,
        20 => Instruction::PushNeg1,
        21 => Instruction::PushAcc,
        22 => Instruction::StoreAcc,
        23 => Instruction::Peek,
        // Family 3 — ρ Recursion (Iteration)
        24 => Instruction::Dec,
        25 => Instruction::Sign,
        26 => Instruction::Clamp,
        27 => Instruction::Min,
        28 => Instruction::Max,
        29 => Instruction::Pow,
        30 => Instruction::Sqrt,
        31 => Instruction::Log2,
        // Family 4 — ∂ Boundary (Lifecycle)
        32 => Instruction::Entry,
        33 => Instruction::Halt,
        34 => Instruction::HaltErr,
        35 => Instruction::HaltYield,
        36 => Instruction::Assert,
        37 => Instruction::Output,
        38 => Instruction::MemSize,
        39 => Instruction::MemClear,
        // Family 5 — → Causality (Control)
        40 => Instruction::Jmp,
        41 => Instruction::JmpIf,
        42 => Instruction::JmpIfZ,
        43 => Instruction::JmpBack,
        44 => Instruction::Call,
        45 => Instruction::Ret,
        46 => Instruction::IfElse,
        47 => Instruction::Cmp,
        // Family 6 — κ Comparison (Testing)
        48 => Instruction::Eq,
        49 => Instruction::Neq,
        50 => Instruction::Lt,
        51 => Instruction::Gt,
        52 => Instruction::And,
        53 => Instruction::Or,
        54 => Instruction::Dup2,
        55 => Instruction::IsEmpty,
        // Family 7 — N Quantity (Numeric)
        56 => Instruction::Shl,
        57 => Instruction::Shr,
        58 => Instruction::BitAnd,
        59 => Instruction::BitOr,
        60 => Instruction::BitXor,
        61 => Instruction::BitNot,
        62 => Instruction::CntInc,
        63 => Instruction::CntRead,
        _ => Instruction::Nop,
    }
}

/// Decode a codon into its instruction.
///
/// Every valid codon (0-63) maps to exactly one instruction.
#[must_use]
pub fn decode(codon: &Codon) -> Instruction {
    decode_index(codon.index())
}

// ---------------------------------------------------------------------------
// Instruction → Codon (encode)
// ---------------------------------------------------------------------------

/// Encode an instruction into its codon.
///
/// Returns `None` for `Lit(_)` which is a pseudo-instruction expanded by the assembler.
pub fn encode(instr: &Instruction) -> Option<Codon> {
    let idx = match instr {
        // Family 0 — σ Sequence (Data Flow)
        Instruction::Nop => 0,
        Instruction::Dup => 1,
        Instruction::Swap => 2,
        Instruction::Pop => 3,
        Instruction::Rot => 4,
        Instruction::Over => 5,
        Instruction::Pick => 6,
        Instruction::Depth => 7,
        // Family 1 — μ Mapping (Transform)
        Instruction::Add => 8,
        Instruction::Sub => 9,
        Instruction::Mul => 10,
        Instruction::Div => 11,
        Instruction::Mod => 12,
        Instruction::Neg => 13,
        Instruction::Abs => 14,
        Instruction::Inc => 15,
        // Family 2 — ς State (Storage)
        Instruction::Load => 16,
        Instruction::Store => 17,
        Instruction::Push0 => 18,
        Instruction::Push1 => 19,
        Instruction::PushNeg1 => 20,
        Instruction::PushAcc => 21,
        Instruction::StoreAcc => 22,
        Instruction::Peek => 23,
        // Family 3 — ρ Recursion (Iteration)
        Instruction::Dec => 24,
        Instruction::Sign => 25,
        Instruction::Clamp => 26,
        Instruction::Min => 27,
        Instruction::Max => 28,
        Instruction::Pow => 29,
        Instruction::Sqrt => 30,
        Instruction::Log2 => 31,
        // Family 4 — ∂ Boundary (Lifecycle)
        Instruction::Entry => 32,
        Instruction::Halt => 33,
        Instruction::HaltErr => 34,
        Instruction::HaltYield => 35,
        Instruction::Assert => 36,
        Instruction::Output => 37,
        Instruction::MemSize => 38,
        Instruction::MemClear => 39,
        // Family 5 — → Causality (Control)
        Instruction::Jmp => 40,
        Instruction::JmpIf => 41,
        Instruction::JmpIfZ => 42,
        Instruction::JmpBack => 43,
        Instruction::Call => 44,
        Instruction::Ret => 45,
        Instruction::IfElse => 46,
        Instruction::Cmp => 47,
        // Family 6 — κ Comparison (Testing)
        Instruction::Eq => 48,
        Instruction::Neq => 49,
        Instruction::Lt => 50,
        Instruction::Gt => 51,
        Instruction::And => 52,
        Instruction::Or => 53,
        Instruction::Dup2 => 54,
        Instruction::IsEmpty => 55,
        // Family 7 — N Quantity (Numeric)
        Instruction::Shl => 56,
        Instruction::Shr => 57,
        Instruction::BitAnd => 58,
        Instruction::BitOr => 59,
        Instruction::BitXor => 60,
        Instruction::BitNot => 61,
        Instruction::CntInc => 62,
        Instruction::CntRead => 63,
        Instruction::Lit(_) => return None,
    };
    Codon::from_index(idx).ok()
}

// ---------------------------------------------------------------------------
// Mnemonic ↔ Instruction
// ---------------------------------------------------------------------------

/// Convert a mnemonic string to an instruction.
///
/// Returns `None` for unrecognized mnemonics.
/// The `lit` mnemonic is handled separately by the assembler (it requires an argument).
#[must_use]
pub fn from_mnemonic(s: &str) -> Option<Instruction> {
    match s {
        "halt" => Some(Instruction::Halt),
        "halt.err" => Some(Instruction::HaltErr),
        "halt.yield" => Some(Instruction::HaltYield),
        "entry" => Some(Instruction::Entry),
        "nop" => Some(Instruction::Nop),
        "dup" => Some(Instruction::Dup),
        "swap" => Some(Instruction::Swap),
        "pop" => Some(Instruction::Pop),
        "add" => Some(Instruction::Add),
        "sub" => Some(Instruction::Sub),
        "mul" => Some(Instruction::Mul),
        "div" => Some(Instruction::Div),
        "eq" => Some(Instruction::Eq),
        "lt" => Some(Instruction::Lt),
        "gt" => Some(Instruction::Gt),
        "neq" => Some(Instruction::Neq),
        "jmp" => Some(Instruction::Jmp),
        "jmpif" => Some(Instruction::JmpIf),
        "jmpback" => Some(Instruction::JmpBack),
        "call" => Some(Instruction::Call),
        "ret" => Some(Instruction::Ret),
        "jmpifz" => Some(Instruction::JmpIfZ),
        "mod" => Some(Instruction::Mod),
        "abs" => Some(Instruction::Abs),
        "neg" => Some(Instruction::Neg),
        "inc" => Some(Instruction::Inc),
        "dec" => Some(Instruction::Dec),
        "max" => Some(Instruction::Max),
        "band" => Some(Instruction::BitAnd),
        "bor" => Some(Instruction::BitOr),
        "bxor" => Some(Instruction::BitXor),
        "bnot" => Some(Instruction::BitNot),
        "rot" => Some(Instruction::Rot),
        "over" => Some(Instruction::Over),
        "pick" => Some(Instruction::Pick),
        "depth" => Some(Instruction::Depth),
        "push0" => Some(Instruction::Push0),
        "push1" => Some(Instruction::Push1),
        "push-1" => Some(Instruction::PushNeg1),
        "load" => Some(Instruction::Load),
        "store" => Some(Instruction::Store),
        "out" => Some(Instruction::Output),
        "min" => Some(Instruction::Min),
        "pow" => Some(Instruction::Pow),
        "sqrt" => Some(Instruction::Sqrt),
        "log2" => Some(Instruction::Log2),
        "sign" => Some(Instruction::Sign),
        "clamp" => Some(Instruction::Clamp),
        "dup2" => Some(Instruction::Dup2),
        "cmp" => Some(Instruction::Cmp),
        "memsz" => Some(Instruction::MemSize),
        "memclr" => Some(Instruction::MemClear),
        "ifelse" => Some(Instruction::IfElse),
        "assert" => Some(Instruction::Assert),
        "shl" => Some(Instruction::Shl),
        "shr" => Some(Instruction::Shr),
        "and" => Some(Instruction::And),
        "or" => Some(Instruction::Or),
        "pushacc" => Some(Instruction::PushAcc),
        "storeacc" => Some(Instruction::StoreAcc),
        "peek" => Some(Instruction::Peek),
        "isempty" => Some(Instruction::IsEmpty),
        "cntinc" => Some(Instruction::CntInc),
        "cntrd" => Some(Instruction::CntRead),
        _ => None,
    }
}

/// Convert an instruction to its mnemonic string.
#[must_use]
pub fn to_mnemonic(instr: &Instruction) -> &'static str {
    match instr {
        Instruction::Halt => "halt",
        Instruction::HaltErr => "halt.err",
        Instruction::HaltYield => "halt.yield",
        Instruction::Entry => "entry",
        Instruction::Nop => "nop",
        Instruction::Dup => "dup",
        Instruction::Swap => "swap",
        Instruction::Pop => "pop",
        Instruction::Add => "add",
        Instruction::Sub => "sub",
        Instruction::Mul => "mul",
        Instruction::Div => "div",
        Instruction::Eq => "eq",
        Instruction::Lt => "lt",
        Instruction::Gt => "gt",
        Instruction::Neq => "neq",
        Instruction::Jmp => "jmp",
        Instruction::JmpIf => "jmpif",
        Instruction::JmpBack => "jmpback",
        Instruction::Call => "call",
        Instruction::Ret => "ret",
        Instruction::JmpIfZ => "jmpifz",
        Instruction::Mod => "mod",
        Instruction::Abs => "abs",
        Instruction::Neg => "neg",
        Instruction::Inc => "inc",
        Instruction::Dec => "dec",
        Instruction::Max => "max",
        Instruction::BitAnd => "band",
        Instruction::BitOr => "bor",
        Instruction::BitXor => "bxor",
        Instruction::BitNot => "bnot",
        Instruction::Rot => "rot",
        Instruction::Over => "over",
        Instruction::Pick => "pick",
        Instruction::Depth => "depth",
        Instruction::Push0 => "push0",
        Instruction::Push1 => "push1",
        Instruction::PushNeg1 => "push-1",
        Instruction::Load => "load",
        Instruction::Store => "store",
        Instruction::Output => "out",
        Instruction::Min => "min",
        Instruction::Pow => "pow",
        Instruction::Sqrt => "sqrt",
        Instruction::Log2 => "log2",
        Instruction::Sign => "sign",
        Instruction::Clamp => "clamp",
        Instruction::Dup2 => "dup2",
        Instruction::Cmp => "cmp",
        Instruction::MemSize => "memsz",
        Instruction::MemClear => "memclr",
        Instruction::IfElse => "ifelse",
        Instruction::Assert => "assert",
        Instruction::Shl => "shl",
        Instruction::Shr => "shr",
        Instruction::And => "and",
        Instruction::Or => "or",
        Instruction::PushAcc => "pushacc",
        Instruction::StoreAcc => "storeacc",
        Instruction::Peek => "peek",
        Instruction::IsEmpty => "isempty",
        Instruction::CntInc => "cntinc",
        Instruction::CntRead => "cntrd",
        Instruction::Lit(_) => "lit",
    }
}

// ---------------------------------------------------------------------------
// Catalog
// ---------------------------------------------------------------------------

/// A single catalog entry describing one instruction.
#[derive(Debug, Clone)]
pub struct CatalogEntry {
    /// Codon index (0-63).
    pub index: u8,
    /// Assembly mnemonic.
    pub mnemonic: &'static str,
    /// Codon nucleotide string (e.g. "ATG").
    pub codon_str: &'static str,
    /// Amino acid abbreviation.
    pub amino_acid: &'static str,
    /// Stack effect description.
    pub stack_effect: &'static str,
}

/// The complete instruction catalog (64 entries).
///
/// Sorted by codon index for deterministic output.
#[must_use]
pub fn catalog() -> Vec<CatalogEntry> {
    let entries: [(u8, &str, &str, &str, &str); 64] = [
        // Family 0 — σ Sequence (Data Flow)
        (0, "nop", "AAA", "Lys", "( -- )"),
        (1, "dup", "AAT", "Asn", "(a -- a a)"),
        (2, "swap", "AAG", "Lys", "(a b -- b a)"),
        (3, "pop", "AAC", "Asn", "(a -- )"),
        (4, "rot", "ATA", "Ile", "(a b c -- b c a)"),
        (5, "over", "ATT", "Ile", "(a b -- a b a)"),
        (6, "pick", "ATG", "Met", "(n -- nth)"),
        (7, "depth", "ATC", "Ile", "( -- depth)"),
        // Family 1 — μ Mapping (Transform)
        (8, "add", "AGA", "Arg", "(a b -- a+b)"),
        (9, "sub", "AGT", "Ser", "(a b -- a-b)"),
        (10, "mul", "AGG", "Arg", "(a b -- a*b)"),
        (11, "div", "AGC", "Ser", "(a b -- a/b)"),
        (12, "mod", "ACA", "Thr", "(a b -- a%b)"),
        (13, "neg", "ACT", "Thr", "(a -- -a)"),
        (14, "abs", "ACG", "Thr", "(a -- |a|)"),
        (15, "inc", "ACC", "Thr", "(a -- a+1)"),
        // Family 2 — ς State (Storage)
        (16, "load", "TAA", "Stp", "(addr -- val)"),
        (17, "store", "TAT", "Tyr", "(val addr -- )"),
        (18, "push0", "TAG", "Stp", "( -- 0)"),
        (19, "push1", "TAC", "Tyr", "( -- 1)"),
        (20, "push-1", "TTA", "Leu", "( -- -1)"),
        (21, "pushacc", "TTT", "Phe", "( -- acc)"),
        (22, "storeacc", "TTG", "Leu", "(val -- )"),
        (23, "peek", "TTC", "Phe", "( -- top)"),
        // Family 3 — ρ Recursion (Iteration)
        (24, "dec", "TGA", "Stp", "(a -- a-1)"),
        (25, "sign", "TGT", "Cys", "(a -- sign)"),
        (26, "clamp", "TGG", "Trp", "(v lo hi -- clamped)"),
        (27, "min", "TGC", "Cys", "(a b -- min)"),
        (28, "max", "TCA", "Ser", "(a b -- max)"),
        (29, "pow", "TCT", "Ser", "(a b -- a^b)"),
        (30, "sqrt", "TCG", "Ser", "(a -- sqrt)"),
        (31, "log2", "TCC", "Ser", "(a -- log2)"),
        // Family 4 — ∂ Boundary (Lifecycle)
        (32, "entry", "GAA", "Glu", "( -- )"),
        (33, "halt", "GAT", "Asp", "( -- )"),
        (34, "halt.err", "GAG", "Glu", "( -- )"),
        (35, "halt.yield", "GAC", "Asp", "( -- )"),
        (36, "assert", "GTA", "Val", "(val -- )"),
        (37, "out", "GTT", "Val", "(val -- )"),
        (38, "memsz", "GTG", "Val", "( -- size)"),
        (39, "memclr", "GTC", "Val", "( -- )"),
        // Family 5 — → Causality (Control)
        (40, "jmp", "GGA", "Gly", "(addr -- )"),
        (41, "jmpif", "GGT", "Gly", "(cond addr -- )"),
        (42, "jmpifz", "GGG", "Gly", "(cond addr -- )"),
        (43, "jmpback", "GGC", "Gly", "(off -- )"),
        (44, "call", "GCA", "Ala", "(addr -- )"),
        (45, "ret", "GCT", "Ala", "( -- )"),
        (46, "ifelse", "GCG", "Ala", "(c t e -- r)"),
        (47, "cmp", "GCC", "Ala", "(a b -- ord)"),
        // Family 6 — κ Comparison (Testing)
        (48, "eq", "CAA", "Gln", "(a b -- a==b)"),
        (49, "neq", "CAT", "His", "(a b -- a!=b)"),
        (50, "lt", "CAG", "Gln", "(a b -- a<b)"),
        (51, "gt", "CAC", "His", "(a b -- a>b)"),
        (52, "and", "CTA", "Leu", "(a b -- a&&b)"),
        (53, "or", "CTT", "Leu", "(a b -- a||b)"),
        (54, "dup2", "CTG", "Leu", "(a b -- a b a b)"),
        (55, "isempty", "CTC", "Leu", "( -- bool)"),
        // Family 7 — N Quantity (Numeric)
        (56, "shl", "CGA", "Arg", "(a n -- a<<n)"),
        (57, "shr", "CGT", "Arg", "(a n -- a>>n)"),
        (58, "band", "CGG", "Arg", "(a b -- a&b)"),
        (59, "bor", "CGC", "Arg", "(a b -- a|b)"),
        (60, "bxor", "CCA", "Pro", "(a b -- a^b)"),
        (61, "bnot", "CCT", "Pro", "(a -- ~a)"),
        (62, "cntinc", "CCG", "Pro", "( -- )"),
        (63, "cntrd", "CCC", "Pro", "( -- cnt)"),
    ];

    entries
        .iter()
        .map(
            |&(index, mnemonic, codon_str, amino_acid, stack_effect)| CatalogEntry {
                index,
                mnemonic,
                codon_str,
                amino_acid,
                stack_effect,
            },
        )
        .collect()
}

// ---------------------------------------------------------------------------
// Literal encoding helpers
// ---------------------------------------------------------------------------

/// Encode an i64 value as a sequence of codons for the PUSH_LIT protocol.
///
/// Returns the codons to emit AFTER the ATG marker:
/// `[len_codon, digit_codons...]`
///
/// The value is encoded in base-64 (big-endian) using codon indices as digits.
/// Negative values use two's complement within the digit width.
pub fn encode_literal(value: i64) -> Vec<Codon> {
    if value == 0 {
        // len=0 means push 0
        if let Ok(c) = Codon::from_index(0) {
            return vec![c];
        }
    }

    // Determine how many base-64 digits we need
    let abs_val = if value < 0 {
        // For negatives, we need enough digits then two's complement
        (value.wrapping_neg()) as u64
    } else {
        value as u64
    };

    let mut digits = Vec::new();
    let mut remaining = abs_val;

    if remaining == 0 {
        digits.push(0u8);
    } else {
        while remaining > 0 {
            digits.push((remaining % 64) as u8);
            remaining /= 64;
        }
        digits.reverse();
    }

    // For negative values: compute two's complement in the digit width
    let codons = if value < 0 {
        let n = digits.len();
        // Max value representable in n base-64 digits
        let max_val = 64u64.pow(n as u32);
        let twos_comp = max_val.wrapping_sub(abs_val);

        let mut neg_digits = Vec::new();
        let mut rem = twos_comp;
        for _ in 0..n {
            neg_digits.push((rem % 64) as u8);
            rem /= 64;
        }
        neg_digits.reverse();

        // Set high bit of first digit to signal negative
        // If the high bit is already clear, we're fine with two's complement
        // But we need a way to distinguish: use an extra digit if MSB is clear
        if neg_digits.first().copied().unwrap_or(0) < 32 {
            // Need one more digit to ensure MSB is set
            neg_digits.insert(0, 63); // 0b111111 — all bits set
        }

        neg_digits
    } else {
        // Positive: ensure MSB of first digit is clear (< 32)
        if digits.first().copied().unwrap_or(0) >= 32 {
            digits.insert(0, 0);
        }
        digits
    };

    // Build codon sequence: len + digits
    let len = codons.len();
    let mut result = Vec::with_capacity(len + 1);

    // Len codon (index = number of digit codons)
    if let Ok(c) = Codon::from_index(len as u8) {
        result.push(c);
    }

    for &d in &codons {
        if let Ok(c) = Codon::from_index(d) {
            result.push(c);
        }
    }

    result
}

/// Decode a literal value from digit codons (after reading the len codon).
///
/// `digits` are the codon indices read after the length codon.
/// Uses two's complement: if MSB of first digit is set, value is negative.
#[must_use]
pub fn decode_literal(digits: &[u8]) -> i64 {
    if digits.is_empty() {
        return 0;
    }

    // Build unsigned value
    let mut value: u64 = 0;
    for &d in digits {
        value = value.wrapping_mul(64).wrapping_add(d as u64);
    }

    // Check sign: MSB of first digit (bit 5)
    let is_negative = digits[0] >= 32;

    if is_negative {
        // Two's complement: value represents max - |actual|
        let n = digits.len();
        let max_val = 64u64.pow(n as u32);
        let magnitude = max_val.wrapping_sub(value);
        -(magnitude as i64)
    } else {
        value as i64
    }
}

// ---------------------------------------------------------------------------
// Display helper for codon triplet string
// ---------------------------------------------------------------------------

/// Get the three-character nucleotide string for a codon.
#[must_use]
pub fn codon_str(codon: &Codon) -> String {
    format!(
        "{}{}{}",
        codon.0.as_char(),
        codon.1.as_char(),
        codon.2.as_char()
    )
}

/// Get the amino acid abbreviation for a codon.
#[must_use]
pub fn codon_amino(codon: &Codon) -> &'static str {
    let table = crate::codon_table::CodonTable::standard();
    table.translate(codon).abbrev()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Nucleotide;

    #[test]
    fn all_64_encode_decode_roundtrip() {
        for idx in 0u8..64 {
            let codon = Codon::from_index(idx);
            assert!(codon.is_ok(), "failed to create codon for index {idx}");
            if let Ok(c) = codon {
                let instr = decode(&c);
                // Lit is never returned by decode
                assert!(!matches!(instr, Instruction::Lit(_)));
                let encoded = encode(&instr);
                assert!(encoded.is_some(), "failed to encode instruction {instr:?}");
                if let Some(enc) = encoded {
                    assert_eq!(
                        enc.index(),
                        idx,
                        "roundtrip failed for index {idx}: {:?} → {:?} → index {}",
                        c,
                        instr,
                        enc.index()
                    );
                }
            }
        }
    }

    #[test]
    fn all_64_mnemonics_roundtrip() {
        for idx in 0u8..64 {
            if let Ok(c) = Codon::from_index(idx) {
                let instr = decode(&c);
                let mnem = to_mnemonic(&instr);
                let back = from_mnemonic(mnem);
                assert!(
                    back.is_some(),
                    "mnemonic '{mnem}' not recognized (index {idx})"
                );
                if let Some(back_instr) = back {
                    assert_eq!(
                        instr, back_instr,
                        "mnemonic roundtrip failed for index {idx}: {mnem}"
                    );
                }
            }
        }
    }

    #[test]
    fn catalog_has_64_entries() {
        let cat = catalog();
        assert_eq!(cat.len(), 64);
    }

    #[test]
    fn catalog_indices_are_0_to_63() {
        let cat = catalog();
        for (i, entry) in cat.iter().enumerate() {
            assert_eq!(entry.index as usize, i);
        }
    }

    #[test]
    fn entry_is_gaa() {
        // v3 ISA: Entry is ∂σ(32) = GAA (Glu)
        let codon = encode(&Instruction::Entry);
        assert!(codon.is_some());
        if let Some(c) = codon {
            assert_eq!(c.index(), 32);
            assert_eq!(c.0, Nucleotide::G);
            assert_eq!(c.1, Nucleotide::A);
            assert_eq!(c.2, Nucleotide::A);
        }
    }

    #[test]
    fn halt_is_gat() {
        // v3 ISA: Halt is ∂μ(33) = GAT (Asp)
        let codon = encode(&Instruction::Halt);
        assert!(codon.is_some());
        if let Some(c) = codon {
            assert_eq!(c.index(), 33);
        }
    }

    #[test]
    fn family_alignment() {
        // Verify glyph families are index-aligned: family = index / 8
        // σ=0, μ=1, ς=2, ρ=3, ∂=4, →=5, κ=6, N=7
        assert_eq!(encode(&Instruction::Nop).map(|c| c.index() / 8), Some(0)); // σ
        assert_eq!(encode(&Instruction::Add).map(|c| c.index() / 8), Some(1)); // μ
        assert_eq!(encode(&Instruction::Load).map(|c| c.index() / 8), Some(2)); // ς
        assert_eq!(encode(&Instruction::Dec).map(|c| c.index() / 8), Some(3)); // ρ
        assert_eq!(encode(&Instruction::Entry).map(|c| c.index() / 8), Some(4)); // ∂
        assert_eq!(encode(&Instruction::Jmp).map(|c| c.index() / 8), Some(5)); // →
        assert_eq!(encode(&Instruction::Eq).map(|c| c.index() / 8), Some(6)); // κ
        assert_eq!(encode(&Instruction::Shl).map(|c| c.index() / 8), Some(7)); // N
    }

    #[test]
    fn lit_returns_none() {
        assert!(encode(&Instruction::Lit(42)).is_none());
    }

    #[test]
    fn literal_encode_zero() {
        let codons = encode_literal(0);
        assert_eq!(codons.len(), 1); // just the len=0 codon
        assert_eq!(codons[0].index(), 0);
    }

    #[test]
    fn literal_encode_small_positive() {
        // Value 5: needs 1 digit (index 5), MSB clear
        let codons = encode_literal(5);
        // Should be: len_codon + 1 digit codon
        assert!(codons.len() >= 2);
        let len = codons[0].index() as usize;
        let mut digits = Vec::new();
        for c in &codons[1..] {
            digits.push(c.index());
        }
        assert_eq!(digits.len(), len);
        let decoded = decode_literal(&digits);
        assert_eq!(decoded, 5);
    }

    #[test]
    fn literal_encode_42() {
        let codons = encode_literal(42);
        let len = codons[0].index() as usize;
        let digits: Vec<u8> = codons[1..].iter().map(|c| c.index()).collect();
        assert_eq!(digits.len(), len);
        assert_eq!(decode_literal(&digits), 42);
    }

    #[test]
    fn literal_encode_1000() {
        let codons = encode_literal(1000);
        let len = codons[0].index() as usize;
        let digits: Vec<u8> = codons[1..].iter().map(|c| c.index()).collect();
        assert_eq!(digits.len(), len);
        assert_eq!(decode_literal(&digits), 1000);
    }

    #[test]
    fn literal_encode_negative() {
        let codons = encode_literal(-42);
        let len = codons[0].index() as usize;
        let digits: Vec<u8> = codons[1..].iter().map(|c| c.index()).collect();
        assert_eq!(digits.len(), len);
        assert_eq!(decode_literal(&digits), -42);
    }

    #[test]
    fn literal_encode_negative_1000() {
        let codons = encode_literal(-1000);
        let len = codons[0].index() as usize;
        let digits: Vec<u8> = codons[1..].iter().map(|c| c.index()).collect();
        assert_eq!(digits.len(), len);
        assert_eq!(decode_literal(&digits), -1000);
    }

    #[test]
    fn literal_roundtrip_various() {
        let values = [0, 1, -1, 42, -42, 63, 64, 100, 1000, -1000, 4095, -4095];
        for &v in &values {
            let codons = encode_literal(v);
            let len = codons[0].index() as usize;
            let digits: Vec<u8> = codons[1..].iter().map(|c| c.index()).collect();
            assert_eq!(digits.len(), len, "len mismatch for value {v}");
            let decoded = decode_literal(&digits);
            assert_eq!(decoded, v, "roundtrip failed for value {v}");
        }
    }

    #[test]
    fn specific_mnemonics() {
        assert_eq!(from_mnemonic("add"), Some(Instruction::Add));
        assert_eq!(from_mnemonic("halt"), Some(Instruction::Halt));
        assert_eq!(from_mnemonic("lit"), None); // lit is not a bare mnemonic
        assert_eq!(from_mnemonic("unknown"), None);
        assert_eq!(to_mnemonic(&Instruction::Output), "out");
        assert_eq!(to_mnemonic(&Instruction::PushNeg1), "push-1");
    }

    #[test]
    fn decode_literal_empty() {
        assert_eq!(decode_literal(&[]), 0);
    }
}
