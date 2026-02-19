//! Bioinformatics Utility: FASTA/FASTQ parsing and processing.
//!
//! Provides lightweight parsers for common genomic data formats.

use crate::error::{DnaError, Result};
use crate::types::{Nucleotide, Strand};
use std::io::{BufRead, BufReader};

/// A single genomic sequence record.
#[derive(Debug, Clone)]
pub struct BioRecord {
    pub id: String,
    pub description: Option<String>,
    pub sequence: Strand,
    pub quality: Option<Vec<u8>>, // For FASTQ
}

pub struct BioParser;

impl BioParser {
    /// Parse a FASTA file from a reader.
    pub fn parse_fasta<R: BufRead>(reader: R) -> Result<Vec<BioRecord>> {
        let mut records = Vec::new();
        let mut current_id = String::new();
        let mut current_desc = None;
        let mut current_seq_bases = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| DnaError::IoError(e.to_string()))?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with('>') {
                // Save previous record
                if !current_id.is_empty() {
                    records.push(BioRecord {
                        id: current_id.clone(),
                        description: current_desc.take(),
                        sequence: Strand::new(current_seq_bases.clone()),
                        quality: None,
                    });
                    current_seq_bases.clear();
                }

                // Parse header
                let header = &line[1..];
                let mut parts = header.splitn(2, |c: char| c.is_whitespace());
                current_id = parts.next().unwrap_or("").to_string();
                current_desc = parts.next().map(|s| s.to_string());
            } else {
                // Parse sequence
                for ch in line.chars() {
                    if let Ok(nuc) = Nucleotide::from_char(ch) {
                        current_seq_bases.push(nuc);
                    }
                }
            }
        }

        // Push final record
        if !current_id.is_empty() {
            records.push(BioRecord {
                id: current_id,
                description: current_desc,
                sequence: Strand::new(current_seq_bases),
                quality: None,
            });
        }

        Ok(records)
    }

    /// Parse FASTA from a string (convenience for EFetch responses).
    pub fn parse_fasta_str(input: &str) -> Result<Vec<BioRecord>> {
        Self::parse_fasta(std::io::Cursor::new(input))
    }

    /// Parse a FASTQ file from a reader.
    pub fn parse_fastq<R: BufRead>(reader: R) -> Result<Vec<BioRecord>> {
        let mut records = Vec::new();
        let mut lines = reader.lines();

        while let Some(line1) = lines.next() {
            let id_line = line1.map_err(|e| DnaError::IoError(e.to_string()))?;
            if !id_line.starts_with('@') {
                continue;
            }

            let id = id_line[1..]
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();

            let seq_line = lines
                .next()
                .ok_or(DnaError::InvalidBase('?'))?
                .map_err(|e| DnaError::IoError(e.to_string()))?;
            let mut bases = Vec::new();
            for ch in seq_line.trim().chars() {
                bases.push(Nucleotide::from_char(ch)?);
            }

            let _plus_line = lines
                .next()
                .ok_or(DnaError::InvalidBase('?'))?
                .map_err(|e| DnaError::IoError(e.to_string()))?;

            let qual_line = lines
                .next()
                .ok_or(DnaError::InvalidBase('?'))?
                .map_err(|e| DnaError::IoError(e.to_string()))?;
            let quality = qual_line.trim().as_bytes().to_vec();

            records.push(BioRecord {
                id,
                description: None,
                sequence: Strand::new(bases),
                quality: Some(quality),
            });
        }

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_fasta() {
        let fasta = ">seq1 First sequence\nATGC\n>seq2 Second sequence\nGCTA\n";
        let records = BioParser::parse_fasta(Cursor::new(fasta)).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, "seq1");
        assert_eq!(records[0].description, Some("First sequence".to_string()));
        assert_eq!(records[0].sequence.to_string_repr(), "ATGC");
    }
}
