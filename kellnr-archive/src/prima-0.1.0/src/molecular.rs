// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Molecular Biology Mapping
//!
//! Maps Prima compilation to the Central Dogma of Molecular Biology.
//!
//! ```text
//! DNA → Transcription → RNA → Translation → Protein
//!  ↓         ↓           ↓         ↓           ↓
//! Source → Lexer/Parser → IR → Interpreter → Value
//! ```
//!
//! ## Tier: T2-C (μ + σ + → + ρ)

use std::collections::HashMap;

// ============================================================================
// Nucleotide (4 bases = biology's binary)
// ============================================================================

/// The 4 nucleotide bases — `c[A,T,G,C]` = 2² symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nucleotide {
    A,
    T,
    G,
    C,
}

impl Nucleotide {
    /// Transcription: T → U in RNA.
    #[must_use]
    pub const fn to_rna(self) -> char {
        match self {
            Self::A => 'A',
            Self::T => 'U',
            Self::G => 'G',
            Self::C => 'C',
        }
    }

    /// Binary encoding (2 bits).
    #[must_use]
    pub const fn to_bits(self) -> u8 {
        match self {
            Self::A => 0b00,
            Self::T => 0b01,
            Self::G => 0b10,
            Self::C => 0b11,
        }
    }
}

// ============================================================================
// Codon (3 nucleotides → 1 amino acid)
// ============================================================================

/// Codon = `c[N₁,N₂,N₃]` → primitive triplet.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Codon(pub Nucleotide, pub Nucleotide, pub Nucleotide);

impl Codon {
    /// Format as RNA string.
    #[must_use]
    pub fn to_rna(&self) -> String {
        format!("{}{}{}", self.0.to_rna(), self.1.to_rna(), self.2.to_rna())
    }
}

// ============================================================================
// Amino Acids (20 building blocks)
// ============================================================================

/// The 20 amino acids + Stop signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AminoAcid {
    Gly,
    Ala,
    Val,
    Leu,
    Ile,
    Pro,
    Met, // Nonpolar
    Ser,
    Thr,
    Cys,
    Asn,
    Gln, // Polar
    Phe,
    Tyr,
    Trp, // Aromatic
    Lys,
    Arg,
    His, // Basic
    Asp,
    Glu,  // Acidic
    Stop, // Termination
}

impl AminoAcid {
    /// Single-letter IUPAC code.
    #[must_use]
    pub const fn code(&self) -> char {
        match self {
            Self::Gly => 'G',
            Self::Ala => 'A',
            Self::Val => 'V',
            Self::Leu => 'L',
            Self::Ile => 'I',
            Self::Pro => 'P',
            Self::Met => 'M',
            Self::Ser => 'S',
            Self::Thr => 'T',
            Self::Cys => 'C',
            Self::Asn => 'N',
            Self::Gln => 'Q',
            Self::Phe => 'F',
            Self::Tyr => 'Y',
            Self::Trp => 'W',
            Self::Lys => 'K',
            Self::Arg => 'R',
            Self::His => 'H',
            Self::Asp => 'D',
            Self::Glu => 'E',
            Self::Stop => '*',
        }
    }
}

// ============================================================================
// Genetic Code (tRNA = Symbol Table)
// ============================================================================

/// μ[Codon → AminoAcid] — biology's symbol table.
pub struct GeneticCode {
    table: HashMap<String, AminoAcid>,
}

impl Default for GeneticCode {
    fn default() -> Self {
        Self::standard()
    }
}

impl GeneticCode {
    /// Standard genetic code (SGC).
    #[must_use]
    pub fn standard() -> Self {
        let mut table = HashMap::new();
        load_nonpolar(&mut table);
        load_polar(&mut table);
        load_aromatic(&mut table);
        load_charged(&mut table);
        load_stop(&mut table);
        Self { table }
    }

    /// Translate codon → amino acid.
    #[must_use]
    pub fn translate(&self, codon: &str) -> Option<AminoAcid> {
        self.table.get(codon).copied()
    }

    /// Translate mRNA → protein.
    #[must_use]
    pub fn translate_mrna(&self, mrna: &str) -> Vec<AminoAcid> {
        mrna.as_bytes()
            .chunks(3)
            .filter_map(|c| {
                if c.len() == 3 {
                    self.translate(&String::from_utf8_lossy(c))
                } else {
                    None
                }
            })
            .take_while(|aa| *aa != AminoAcid::Stop)
            .collect()
    }
}

// Helper: load nonpolar amino acids
fn load_nonpolar(t: &mut HashMap<String, AminoAcid>) {
    // Phe (F)
    t.insert("UUU".into(), AminoAcid::Phe);
    t.insert("UUC".into(), AminoAcid::Phe);
    // Leu (L)
    for c in ["UUA", "UUG", "CUU", "CUC", "CUA", "CUG"] {
        t.insert(c.into(), AminoAcid::Leu);
    }
    // Ile (I)
    for c in ["AUU", "AUC", "AUA"] {
        t.insert(c.into(), AminoAcid::Ile);
    }
    // Met (M) - START
    t.insert("AUG".into(), AminoAcid::Met);
    // Val (V)
    for c in ["GUU", "GUC", "GUA", "GUG"] {
        t.insert(c.into(), AminoAcid::Val);
    }
    // Pro (P)
    for c in ["CCU", "CCC", "CCA", "CCG"] {
        t.insert(c.into(), AminoAcid::Pro);
    }
    // Ala (A)
    for c in ["GCU", "GCC", "GCA", "GCG"] {
        t.insert(c.into(), AminoAcid::Ala);
    }
    // Gly (G)
    for c in ["GGU", "GGC", "GGA", "GGG"] {
        t.insert(c.into(), AminoAcid::Gly);
    }
}

// Helper: load polar amino acids
fn load_polar(t: &mut HashMap<String, AminoAcid>) {
    // Ser (S)
    for c in ["UCU", "UCC", "UCA", "UCG", "AGU", "AGC"] {
        t.insert(c.into(), AminoAcid::Ser);
    }
    // Thr (T)
    for c in ["ACU", "ACC", "ACA", "ACG"] {
        t.insert(c.into(), AminoAcid::Thr);
    }
    // Cys (C)
    t.insert("UGU".into(), AminoAcid::Cys);
    t.insert("UGC".into(), AminoAcid::Cys);
    // Asn (N)
    t.insert("AAU".into(), AminoAcid::Asn);
    t.insert("AAC".into(), AminoAcid::Asn);
    // Gln (Q)
    t.insert("CAA".into(), AminoAcid::Gln);
    t.insert("CAG".into(), AminoAcid::Gln);
}

// Helper: load aromatic amino acids
fn load_aromatic(t: &mut HashMap<String, AminoAcid>) {
    // Tyr (Y)
    t.insert("UAU".into(), AminoAcid::Tyr);
    t.insert("UAC".into(), AminoAcid::Tyr);
    // Trp (W)
    t.insert("UGG".into(), AminoAcid::Trp);
}

// Helper: load charged amino acids
fn load_charged(t: &mut HashMap<String, AminoAcid>) {
    // Lys (K) - basic
    t.insert("AAA".into(), AminoAcid::Lys);
    t.insert("AAG".into(), AminoAcid::Lys);
    // Arg (R) - basic
    for c in ["CGU", "CGC", "CGA", "CGG", "AGA", "AGG"] {
        t.insert(c.into(), AminoAcid::Arg);
    }
    // His (H) - basic
    t.insert("CAU".into(), AminoAcid::His);
    t.insert("CAC".into(), AminoAcid::His);
    // Asp (D) - acidic
    t.insert("GAU".into(), AminoAcid::Asp);
    t.insert("GAC".into(), AminoAcid::Asp);
    // Glu (E) - acidic
    t.insert("GAA".into(), AminoAcid::Glu);
    t.insert("GAG".into(), AminoAcid::Glu);
}

// Helper: load stop codons
fn load_stop(t: &mut HashMap<String, AminoAcid>) {
    t.insert("UAA".into(), AminoAcid::Stop);
    t.insert("UAG".into(), AminoAcid::Stop);
    t.insert("UGA".into(), AminoAcid::Stop);
}

// ============================================================================
// Central Dogma Mapping
// ============================================================================

/// Biology → Prima stage mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CentralDogma {
    Transcription, // DNA → mRNA = source → AST
    Translation,   // mRNA → Protein = AST → Value
    Folding,       // Structure optimization
    Replication,   // ρ — self-copy
    Proofreading,  // κ:gate — error check
}

impl CentralDogma {
    /// Prima equivalent.
    #[must_use]
    pub const fn prima(&self) -> &'static str {
        match self {
            Self::Transcription => "μ[source→AST]",
            Self::Translation => "μ[AST→Value]",
            Self::Folding => "μ[Type→Type]",
            Self::Replication => "ρ[self→self]",
            Self::Proofreading => "κ[T₁,T₂]→Bool",
        }
    }
}

// ============================================================================
// ADME (Pharmacokinetics)
// ============================================================================

/// Drug lifecycle = computation lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Adme {
    Absorption,   // Input
    Distribution, // Memory
    Metabolism,   // Compute
    Elimination,  // GC/return
}

impl Adme {
    /// Symbol.
    #[must_use]
    pub const fn symbol(&self) -> char {
        match self {
            Self::Absorption => 'A',
            Self::Distribution => 'D',
            Self::Metabolism => 'M',
            Self::Elimination => 'E',
        }
    }

    /// Prima equivalent.
    #[must_use]
    pub const fn prima(&self) -> &'static str {
        match self {
            Self::Absorption => "π:read",
            Self::Distribution => "ς:alloc",
            Self::Metabolism => "→:compute",
            Self::Elimination => "∅:free",
        }
    }
}

// ============================================================================
// Ribosome (VM)
// ============================================================================

/// Biological VM — translates mRNA → protein.
#[derive(Default)]
pub struct Ribosome(GeneticCode);

impl Ribosome {
    /// Translate mRNA to protein string.
    #[must_use]
    pub fn translate(&self, mrna: &str) -> String {
        self.0
            .translate_mrna(mrna)
            .iter()
            .map(|a| a.code())
            .collect()
    }

    /// Find start codon (AUG).
    #[must_use]
    pub fn find_start(mrna: &str) -> Option<usize> {
        mrna.find("AUG")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nucleotide_transcription() {
        assert_eq!(Nucleotide::T.to_rna(), 'U');
    }

    #[test]
    fn test_codon_to_rna() {
        let c = Codon(Nucleotide::A, Nucleotide::T, Nucleotide::G);
        assert_eq!(c.to_rna(), "AUG");
    }

    #[test]
    fn test_start_codon() {
        let gc = GeneticCode::standard();
        assert_eq!(gc.translate("AUG"), Some(AminoAcid::Met));
    }

    #[test]
    fn test_stop_codons() {
        let gc = GeneticCode::standard();
        assert_eq!(gc.translate("UAA"), Some(AminoAcid::Stop));
        assert_eq!(gc.translate("UAG"), Some(AminoAcid::Stop));
        assert_eq!(gc.translate("UGA"), Some(AminoAcid::Stop));
    }

    #[test]
    fn test_translate_mrna() {
        let gc = GeneticCode::standard();
        let protein = gc.translate_mrna("AUGUUUUAA");
        assert_eq!(protein, vec![AminoAcid::Met, AminoAcid::Phe]);
    }

    #[test]
    fn test_ribosome() {
        let r = Ribosome::default();
        assert_eq!(r.translate("AUGUUUUAA"), "MF");
    }

    #[test]
    fn test_find_start() {
        assert_eq!(Ribosome::find_start("GCUAUGCCC"), Some(3));
    }

    #[test]
    fn test_codon_redundancy() {
        let gc = GeneticCode::standard();
        // Error tolerance: multiple codons → same amino acid
        assert_eq!(gc.translate("UUU"), gc.translate("UUC"));
    }

    #[test]
    fn test_central_dogma_prima() {
        assert!(CentralDogma::Transcription.prima().contains("AST"));
    }

    #[test]
    fn test_adme_symbols() {
        assert_eq!(
            [
                Adme::Absorption,
                Adme::Distribution,
                Adme::Metabolism,
                Adme::Elimination
            ]
            .map(|a| a.symbol()),
            ['A', 'D', 'M', 'E']
        );
    }

    #[test]
    fn test_amino_acid_codes() {
        assert_eq!(AminoAcid::Met.code(), 'M');
        assert_eq!(AminoAcid::Stop.code(), '*');
    }
}
