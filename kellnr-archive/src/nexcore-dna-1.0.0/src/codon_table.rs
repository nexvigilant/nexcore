//! Standard genetic code: 64 codons → 21 amino acids (20 + Stop).
//!
//! The codon table maps each nucleotide triplet to an amino acid using the
//! universal genetic code. This is a pure μ Mapping.

use crate::types::{AminoAcid, Codon};

/// The standard genetic code mapping 64 codons to amino acids.
///
/// Tier: T2-P (μ Mapping + ∂ Boundary)
///
/// Layout: indexed by `Codon::index()` (first*16 + second*4 + third).
/// A=0, T=1, G=2, C=3.
#[derive(Debug, Clone)]
pub struct CodonTable {
    table: [AminoAcid; 64],
}

impl Default for CodonTable {
    fn default() -> Self {
        Self::standard()
    }
}

impl CodonTable {
    /// The standard (universal) genetic code.
    ///
    /// Ordered by codon index: AAA=0, AAT=1, AAG=2, AAC=3,
    /// ATA=4, ATT=5, ATG=6, ATC=7, ... CCC=63.
    #[must_use]
    pub fn standard() -> Self {
        use AminoAcid::*;

        // Index: first*16 + second*4 + third
        // First base: A=0, T=1, G=2, C=3
        // Second base: A=0, T=1, G=2, C=3
        // Third base: A=0, T=1, G=2, C=3
        let table: [AminoAcid; 64] = [
            // A__ (first = A = 0)
            // AA_ (second = A = 0)
            Lys, // AAA (0)  - Lysine
            Asn, // AAT (1)  - Asparagine
            Lys, // AAG (2)  - Lysine
            Asn, // AAC (3)  - Asparagine
            // AT_ (second = T = 1)
            Ile, // ATA (4)  - Isoleucine
            Ile, // ATT (5)  - Isoleucine
            Met, // ATG (6)  - Methionine (START)
            Ile, // ATC (7)  - Isoleucine
            // AG_ (second = G = 2)
            Arg, // AGA (8)  - Arginine
            Ser, // AGT (9)  - Serine
            Arg, // AGG (10) - Arginine
            Ser, // AGC (11) - Serine
            // AC_ (second = C = 3)
            Thr, // ACA (12) - Threonine
            Thr, // ACT (13) - Threonine
            Thr, // ACG (14) - Threonine
            Thr, // ACC (15) - Threonine
            // T__ (first = T = 1)
            // TA_ (second = A = 0)
            Stop, // TAA (16) - Stop (ochre)
            Tyr,  // TAT (17) - Tyrosine
            Stop, // TAG (18) - Stop (amber)
            Tyr,  // TAC (19) - Tyrosine
            // TT_ (second = T = 1)
            Leu, // TTA (20) - Leucine
            Phe, // TTT (21) - Phenylalanine
            Leu, // TTG (22) - Leucine
            Phe, // TTC (23) - Phenylalanine
            // TG_ (second = G = 2)
            Stop, // TGA (24) - Stop (opal)
            Cys,  // TGT (25) - Cysteine
            Trp,  // TGG (26) - Tryptophan
            Cys,  // TGC (27) - Cysteine
            // TC_ (second = C = 3)
            Ser, // TCA (28) - Serine
            Ser, // TCT (29) - Serine
            Ser, // TCA (30) - Serine
            Ser, // TCC (31) - Serine
            // G__ (first = G = 2)
            // GA_ (second = A = 0)
            Glu, // GAA (32) - Glutamic acid
            Asp, // GAT (33) - Aspartic acid
            Glu, // GAG (34) - Glutamic acid
            Asp, // GAC (35) - Aspartic acid
            // GT_ (second = T = 1)
            Val, // GTA (36) - Valine
            Val, // GTT (37) - Valine
            Val, // GTG (38) - Valine
            Val, // GTC (39) - Valine
            // GG_ (second = G = 2)
            Gly, // GGA (40) - Glycine
            Gly, // GGT (41) - Glycine
            Gly, // GGG (42) - Glycine
            Gly, // GGC (43) - Glycine
            // GC_ (second = C = 3)
            Ala, // GCA (44) - Alanine
            Ala, // GCT (45) - Alanine
            Ala, // GCG (46) - Alanine
            Ala, // GCC (47) - Alanine
            // C__ (first = C = 3)
            // CA_ (second = A = 0)
            Gln, // CAA (48) - Glutamine
            His, // CAT (49) - Histidine
            Gln, // CAG (50) - Glutamine
            His, // CAC (51) - Histidine
            // CT_ (second = T = 1)
            Leu, // CTA (52) - Leucine
            Leu, // CTT (53) - Leucine
            Leu, // CTG (54) - Leucine
            Leu, // CTC (55) - Leucine
            // CG_ (second = G = 2)
            Arg, // CGA (56) - Arginine
            Arg, // CGT (57) - Arginine
            Arg, // CGG (58) - Arginine
            Arg, // CGC (59) - Arginine
            // CC_ (second = C = 3)
            Pro, // CCA (60) - Proline
            Pro, // CCT (61) - Proline
            Pro, // CCG (62) - Proline
            Pro, // CCC (63) - Proline
        ];

        Self { table }
    }

    /// Translate a single codon to its amino acid.
    #[must_use]
    pub fn translate(&self, codon: &Codon) -> AminoAcid {
        self.table[codon.index() as usize]
    }

    /// Find all codons that encode a given amino acid.
    #[must_use]
    pub fn codons_for(&self, aa: AminoAcid) -> Vec<Codon> {
        (0u8..64)
            .filter(|&i| self.table[i as usize] == aa)
            .filter_map(|i| Codon::from_index(i).ok())
            .collect()
    }

    /// Degeneracy: how many codons encode each amino acid.
    #[must_use]
    pub fn degeneracy(&self, aa: AminoAcid) -> usize {
        self.table.iter().filter(|&&a| a == aa).count()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Nucleotide;

    #[test]
    fn all_64_codons_mapped() {
        let table = CodonTable::standard();
        for i in 0u8..64 {
            let codon = Codon::from_index(i);
            assert!(codon.is_ok());
            if let Some(c) = codon.ok() {
                let _aa = table.translate(&c);
                // Every codon produces a valid amino acid (including Stop)
            }
        }
    }

    #[test]
    fn met_has_one_codon() {
        let table = CodonTable::standard();
        assert_eq!(table.degeneracy(AminoAcid::Met), 1);
        let codons = table.codons_for(AminoAcid::Met);
        assert_eq!(codons.len(), 1);
        // ATG
        assert_eq!(codons[0].0, Nucleotide::A);
        assert_eq!(codons[0].1, Nucleotide::T);
        assert_eq!(codons[0].2, Nucleotide::G);
    }

    #[test]
    fn trp_has_one_codon() {
        let table = CodonTable::standard();
        assert_eq!(table.degeneracy(AminoAcid::Trp), 1);
    }

    #[test]
    fn leu_has_six_codons() {
        let table = CodonTable::standard();
        assert_eq!(table.degeneracy(AminoAcid::Leu), 6);
    }

    #[test]
    fn ser_has_six_codons() {
        let table = CodonTable::standard();
        assert_eq!(table.degeneracy(AminoAcid::Ser), 6);
    }

    #[test]
    fn arg_has_six_codons() {
        let table = CodonTable::standard();
        assert_eq!(table.degeneracy(AminoAcid::Arg), 6);
    }

    #[test]
    fn stop_has_three_codons() {
        let table = CodonTable::standard();
        assert_eq!(table.degeneracy(AminoAcid::Stop), 3);
    }

    #[test]
    fn total_codons_is_64() {
        let table = CodonTable::standard();
        let total: usize = [
            AminoAcid::Ala,
            AminoAcid::Arg,
            AminoAcid::Asn,
            AminoAcid::Asp,
            AminoAcid::Cys,
            AminoAcid::Gln,
            AminoAcid::Glu,
            AminoAcid::Gly,
            AminoAcid::His,
            AminoAcid::Ile,
            AminoAcid::Leu,
            AminoAcid::Lys,
            AminoAcid::Met,
            AminoAcid::Phe,
            AminoAcid::Pro,
            AminoAcid::Ser,
            AminoAcid::Thr,
            AminoAcid::Trp,
            AminoAcid::Tyr,
            AminoAcid::Val,
            AminoAcid::Stop,
        ]
        .iter()
        .map(|aa| table.degeneracy(*aa))
        .sum();
        assert_eq!(total, 64);
    }

    #[test]
    fn atg_translates_to_met() {
        let table = CodonTable::standard();
        let atg = Codon(Nucleotide::A, Nucleotide::T, Nucleotide::G);
        assert_eq!(table.translate(&atg), AminoAcid::Met);
    }

    #[test]
    fn stop_codons_translate_to_stop() {
        let table = CodonTable::standard();
        let taa = Codon(Nucleotide::T, Nucleotide::A, Nucleotide::A);
        let tag = Codon(Nucleotide::T, Nucleotide::A, Nucleotide::G);
        let tga = Codon(Nucleotide::T, Nucleotide::G, Nucleotide::A);
        assert_eq!(table.translate(&taa), AminoAcid::Stop);
        assert_eq!(table.translate(&tag), AminoAcid::Stop);
        assert_eq!(table.translate(&tga), AminoAcid::Stop);
    }
}
