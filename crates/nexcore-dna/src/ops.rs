//! Biological operations on DNA/RNA strands.
//!
//! Implements the central dogma (replication, transcription, translation)
//! plus mutations and analysis.

use crate::codon_table::CodonTable;
use crate::error::{DnaError, Result};
use crate::types::{AminoAcid, DoubleHelix, Nucleotide, Strand};

/// Compute the reverse complement of a strand.
///
/// A↔T, G↔C, then reversed (5'→3' to 3'→5').
///
/// Tier: T2-P (σ Sequence + μ Mapping)
#[must_use]
pub fn complement(strand: &Strand) -> Strand {
    let bases: Vec<Nucleotide> = strand.bases.iter().rev().map(|n| n.complement()).collect();
    Strand::new(bases)
}

/// Semiconservative replication: produce a double helix from a single strand.
///
/// Tier: T2-P (σ Sequence + → Causality)
#[must_use]
pub fn replicate(strand: &Strand) -> DoubleHelix {
    DoubleHelix::from_sense(strand.clone())
}

/// Transcribe DNA to mRNA (T → U in representation).
///
/// The input must be a DNA strand (not already RNA).
///
/// Tier: T2-P (σ Sequence + μ Mapping)
pub fn transcribe(strand: &Strand) -> Result<Strand> {
    if strand.is_rna {
        return Err(DnaError::AlreadyRna);
    }
    // mRNA is a copy of the sense strand with T→U
    Ok(Strand::new_rna(strand.bases.clone()))
}

/// Translate an mRNA strand to a protein (amino acid sequence).
///
/// Scans for the first AUG (start codon), then translates triplets until
/// a stop codon or end of strand.
///
/// Tier: T2-C (σ + μ + → + ∂)
pub fn translate(strand: &Strand, table: &CodonTable) -> Result<Vec<AminoAcid>> {
    // Find start codon (AUG = ATG in internal repr)
    let start_pos = strand
        .bases
        .windows(3)
        .position(|w| w[0] == Nucleotide::A && w[1] == Nucleotide::T && w[2] == Nucleotide::G)
        .ok_or(DnaError::NoStartCodon)?;

    let mut protein = Vec::new();
    let mut pos = start_pos;

    while pos + 3 <= strand.bases.len() {
        let codon = crate::types::Codon(
            strand.bases[pos],
            strand.bases[pos + 1],
            strand.bases[pos + 2],
        );
        let aa = table.translate(&codon);

        if aa == AminoAcid::Stop {
            break;
        }

        protein.push(aa);
        pos += 3;
    }

    Ok(protein)
}

/// Point mutation: replace the nucleotide at a given position.
///
/// Tier: T2-P (ς State + ∂ Boundary)
pub fn mutate(strand: &mut Strand, position: usize, replacement: Nucleotide) -> Result<()> {
    if position >= strand.bases.len() {
        return Err(DnaError::IndexOutOfBounds(position, strand.bases.len()));
    }
    strand.bases[position] = replacement;
    Ok(())
}

/// Insertion mutation: add a nucleotide at a position (causes frameshift).
///
/// Tier: T2-P (σ Sequence + ∂ Boundary)
pub fn insert(strand: &mut Strand, position: usize, nucleotide: Nucleotide) -> Result<()> {
    if position > strand.bases.len() {
        return Err(DnaError::IndexOutOfBounds(position, strand.bases.len()));
    }
    strand.bases.insert(position, nucleotide);
    Ok(())
}

/// Deletion mutation: remove the nucleotide at a position (causes frameshift).
///
/// Tier: T2-P (σ Sequence + ∂ Boundary)
pub fn delete(strand: &mut Strand, position: usize) -> Result<()> {
    if position >= strand.bases.len() {
        return Err(DnaError::IndexOutOfBounds(position, strand.bases.len()));
    }
    strand.bases.remove(position);
    Ok(())
}

/// GC content: fraction of G and C bases in the strand.
///
/// Returns 0.0 for empty strands.
///
/// Tier: T2-P (N Quantity)
#[must_use]
pub fn gc_content(strand: &Strand) -> f64 {
    if strand.is_empty() {
        return 0.0;
    }
    let gc_count = strand
        .bases
        .iter()
        .filter(|&&n| n == Nucleotide::G || n == Nucleotide::C)
        .count();
    gc_count as f64 / strand.bases.len() as f64
}

/// Hamming distance: number of positions where two strands differ.
///
/// Strands must be the same length.
///
/// Tier: T2-P (κ Comparison + N Quantity)
pub fn hamming_distance(a: &Strand, b: &Strand) -> Result<usize> {
    if a.len() != b.len() {
        return Err(DnaError::LengthMismatch(a.len(), b.len()));
    }
    Ok(a.bases
        .iter()
        .zip(b.bases.iter())
        .filter(|(x, y)| x != y)
        .count())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complement_reverses_and_pairs() {
        let strand = Strand::parse("ATGC");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let comp = complement(&s);
            // ATGC → complement each: TACG → reverse: GCAT
            assert_eq!(comp.to_string_repr(), "GCAT");
        }
    }

    #[test]
    fn complement_involution() {
        let strand = Strand::parse("ATGCGATCGA");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let double_comp = complement(&complement(&s));
            assert_eq!(s.bases, double_comp.bases);
        }
    }

    #[test]
    fn replicate_produces_valid_helix() {
        let strand = Strand::parse("ATGCGA");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let helix = replicate(&s);
            assert!(helix.is_valid());
            assert_eq!(helix.len(), 6);
        }
    }

    #[test]
    fn transcribe_dna_to_rna() {
        let strand = Strand::parse("ATGCGA");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let rna = transcribe(&s);
            assert!(rna.is_ok());
            if let Some(r) = rna.ok() {
                assert!(r.is_rna);
                assert_eq!(r.to_string_repr(), "AUGCGA");
            }
        }
    }

    #[test]
    fn transcribe_rna_fails() {
        let rna = Strand::new_rna(vec![Nucleotide::A, Nucleotide::T, Nucleotide::G]);
        assert!(transcribe(&rna).is_err());
    }

    #[test]
    fn translate_simple_protein() {
        // ATG GAA TAA = Met Glu Stop
        let strand = Strand::parse("ATGGAATAA");
        let table = CodonTable::standard();
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let protein = translate(&s, &table);
            assert!(protein.is_ok());
            if let Some(p) = protein.ok() {
                assert_eq!(p.len(), 2);
                assert_eq!(p[0], AminoAcid::Met);
                assert_eq!(p[1], AminoAcid::Glu);
            }
        }
    }

    #[test]
    fn translate_no_start_codon() {
        let strand = Strand::parse("GAATAA");
        let table = CodonTable::standard();
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            assert!(translate(&s, &table).is_err());
        }
    }

    #[test]
    fn translate_with_upstream_bases() {
        // GGG ATG GCT TAA = skip Gly, then Met Ala Stop
        let strand = Strand::parse("GGGATGGCTTAA");
        let table = CodonTable::standard();
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let protein = translate(&s, &table);
            assert!(protein.is_ok());
            if let Some(p) = protein.ok() {
                assert_eq!(p.len(), 2);
                assert_eq!(p[0], AminoAcid::Met);
                assert_eq!(p[1], AminoAcid::Ala);
            }
        }
    }

    #[test]
    fn mutate_valid_position() {
        let strand = Strand::parse("AAAA");
        assert!(strand.is_ok());
        if let Some(mut s) = strand.ok() {
            assert!(mutate(&mut s, 2, Nucleotide::G).is_ok());
            assert_eq!(s.bases[2], Nucleotide::G);
        }
    }

    #[test]
    fn mutate_out_of_bounds() {
        let strand = Strand::parse("AAAA");
        assert!(strand.is_ok());
        if let Some(mut s) = strand.ok() {
            assert!(mutate(&mut s, 10, Nucleotide::G).is_err());
        }
    }

    #[test]
    fn insert_shifts_frame() {
        let strand = Strand::parse("ATG");
        assert!(strand.is_ok());
        if let Some(mut s) = strand.ok() {
            assert!(insert(&mut s, 1, Nucleotide::C).is_ok());
            assert_eq!(s.len(), 4);
            assert_eq!(s.bases[1], Nucleotide::C);
        }
    }

    #[test]
    fn delete_shifts_frame() {
        let strand = Strand::parse("ATGC");
        assert!(strand.is_ok());
        if let Some(mut s) = strand.ok() {
            assert!(delete(&mut s, 1).is_ok());
            assert_eq!(s.len(), 3);
            assert_eq!(s.bases[1], Nucleotide::G);
        }
    }

    #[test]
    fn gc_content_calculation() {
        let strand = Strand::parse("ATGC");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let gc = gc_content(&s);
            assert!((gc - 0.5).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn gc_content_empty() {
        let strand = Strand::new(vec![]);
        assert!((gc_content(&strand) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gc_content_all_gc() {
        let strand = Strand::parse("GGCC");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            assert!((gc_content(&s) - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn hamming_distance_identical() {
        let a = Strand::parse("ATGC");
        let b = Strand::parse("ATGC");
        if let (Some(a), Some(b)) = (a.ok(), b.ok()) {
            let dist = hamming_distance(&a, &b);
            assert!(dist.is_ok());
            if let Some(d) = dist.ok() {
                assert_eq!(d, 0);
            }
        }
    }

    #[test]
    fn hamming_distance_all_different() {
        let a = Strand::parse("AAAA");
        let b = Strand::parse("TTTT");
        if let (Some(a), Some(b)) = (a.ok(), b.ok()) {
            let dist = hamming_distance(&a, &b);
            assert!(dist.is_ok());
            if let Some(d) = dist.ok() {
                assert_eq!(d, 4);
            }
        }
    }

    #[test]
    fn hamming_distance_length_mismatch() {
        let a = Strand::parse("AAA");
        let b = Strand::parse("AAAA");
        if let (Some(a), Some(b)) = (a.ok(), b.ok()) {
            assert!(hamming_distance(&a, &b).is_err());
        }
    }
}
