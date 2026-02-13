//! NexCore Concept Encoding — Signal Detection Programs & Type Registry.
//!
//! Encodes NexCore signal detection algorithms (PRR, ROR, IC, EBGM) as DNA
//! programs, and the GroundsTo type registry as DnaRecords.
//!
//! This module demonstrates the full DNA → computation → domain pipeline:
//! NexCore concepts → DNA sequences → executable programs → results.
//!
//! Tier: T3 (→ + κ + N + ∂ + ς + ν + σ + μ)

use crate::data::{DnaRecord, DnaValue};
use crate::error::Result;
use crate::grounding::GroundsTo;
use crate::lang::compiler;
use crate::program::Program;

// ============================================================================
// Signal Detection Algorithms as DNA Programs
// ============================================================================

/// The PRR (Proportional Reporting Ratio) source code.
///
/// Formula: PRR = (a / (a + b)) / (c / (c + d))
/// Outputs numerator and denominator separately (integer VM).
/// Where: a=drug+event, b=drug+no_event, c=no_drug+event, d=no_drug+no_event
pub const PRR_SOURCE: &str = "\
let a = 15
let b = 100
let c = 20
let d = 10000
let num = a * (c + d)
let den = c * (a + b)
print(num, den)";

/// The ROR (Reporting Odds Ratio) source code.
///
/// Formula: ROR = (a * d) / (b * c)
pub const ROR_SOURCE: &str = "\
let a = 15
let b = 100
let c = 20
let d = 10000
let num = a * d
let den = b * c
print(num, den)";

/// The IC (Information Component) approximation source.
///
/// Computes the ratio components for IC = log2(a*N / ((a+b)*(a+c)))
/// The log2 step is external -- DNA VM does integer arithmetic.
pub const IC_SOURCE: &str = "\
let a = 15
let b = 100
let c = 20
let d = 10000
let n = a + b + c + d
let observed = a * n
let expected = (a + b) * (a + c)
print(observed, expected)";

/// The EBGM (Empirical Bayesian Geometric Mean) approximation source.
///
/// Simplified: computes observed/expected ratio components.
pub const EBGM_SOURCE: &str = "\
let a = 15
let b = 100
let c = 20
let d = 10000
let n = a + b + c + d
let observed = a
let expected = (a + b) * (a + c) / n
print(observed, expected)";

/// A compiled signal detection algorithm as a DNA program.
pub struct SignalDetectionProgram {
    /// Algorithm name (PRR, ROR, IC, EBGM).
    pub name: &'static str,
    /// Human-readable formula.
    pub formula: &'static str,
    /// The compiled DNA program.
    pub program: Program,
    /// The DNA strand as a string.
    pub strand: String,
}

/// Build all 4 signal detection programs.
///
/// Each algorithm is compiled to a DNA program using reference data
/// (a=15, b=100, c=20, d=10000) and outputs ratio components via `print()`.
pub fn signal_detection_programs() -> Result<Vec<SignalDetectionProgram>> {
    let algorithms: [(&str, &str, &str); 4] = [
        ("PRR", "(a/(a+b)) / (c/(c+d))", PRR_SOURCE),
        ("ROR", "(a×d) / (b×c)", ROR_SOURCE),
        ("IC", "log2(a×N / ((a+b)×(a+c)))", IC_SOURCE),
        ("EBGM", "observed / expected (Bayesian)", EBGM_SOURCE),
    ];

    let mut programs = Vec::with_capacity(4);
    for (name, formula, source) in algorithms {
        let program = compiler::compile(source)?;
        let strand = program.code.to_string_repr();
        programs.push(SignalDetectionProgram {
            name,
            formula,
            program,
            strand,
        });
    }
    Ok(programs)
}

/// Encode all 4 signal detection algorithms as a combined DNA strand.
///
/// End-to-end pipeline: NexCore algorithms → DNA → single strand string.
pub fn nexcore_genome_strand() -> Result<String> {
    let programs = signal_detection_programs()?;
    let mut combined = String::new();
    for p in &programs {
        combined.push_str(&p.strand);
    }
    Ok(combined)
}

/// Source code for the full NexCore signal genome as functions.
///
/// Four signal detection algorithms encoded as DNA functions, compilable
/// via `compile_genome()` to produce a `Genome` with individual `Gene` objects.
pub const NEXCORE_SIGNAL_GENOME_SOURCE: &str = "\
fn prr(a, b, c, d) do
  let num = a * (c + d)
  let den = c * (a + b)
  return num
end

fn ror(a, b, c, d) do
  let num = a * d
  let den = b * c
  return num
end

fn ic(a, b, c, d) do
  let n = a + b + c + d
  let observed = a * n
  let expected = (a + b) * (a + c)
  return observed
end

fn ebgm(a, b, c, d) do
  let n = a + b + c + d
  let observed = a
  let expected = (a + b) * (a + c) / n
  return observed
end

prr(15, 100, 20, 10000)";

/// Compile the NexCore signal detection genome.
///
/// Returns a `Genome` containing 4 genes (prr, ror, ic, ebgm) as individually
/// addressable, mutable DNA segments.
pub fn nexcore_signal_genome() -> Result<crate::gene::Genome> {
    compiler::compile_genome(NEXCORE_SIGNAL_GENOME_SOURCE)
}

// ============================================================================
// Type Registry as DnaRecords
// ============================================================================

/// Encode a GroundsTo type as a DnaRecord.
///
/// Captures name, tier, dominant primitive, confidence, and primitive count.
pub fn encode_type_record<T: GroundsTo>(name: &str) -> DnaRecord {
    let comp = T::primitive_composition();
    let tier = crate::grounding::Tier::classify(&comp);

    let mut record = DnaRecord::new();
    record.set("name".to_string(), DnaValue::text(name));
    record.set("tier".to_string(), DnaValue::text(tier.code()));
    if let Some(dom) = comp.dominant {
        record.set("dominant".to_string(), DnaValue::text(dom.symbol()));
    }
    record.set("confidence".to_string(), DnaValue::float(comp.confidence));
    record.set(
        "primitive_count".to_string(),
        DnaValue::int(comp.primitives.len() as i64),
    );
    record
}

/// Encode key NexCore DNA types as a DnaRecord registry.
///
/// Returns a Vec of (name, DnaRecord) pairs encoding GroundsTo metadata
/// as DNA-encoded fields.
pub fn type_registry() -> Vec<(String, DnaRecord)> {
    use crate::types::*;

    vec![
        (
            "Nucleotide".to_string(),
            encode_type_record::<Nucleotide>("Nucleotide"),
        ),
        (
            "AminoAcid".to_string(),
            encode_type_record::<AminoAcid>("AminoAcid"),
        ),
        ("Codon".to_string(), encode_type_record::<Codon>("Codon")),
        ("Strand".to_string(), encode_type_record::<Strand>("Strand")),
        (
            "DoubleHelix".to_string(),
            encode_type_record::<DoubleHelix>("DoubleHelix"),
        ),
    ]
}

/// Encode Guardian thresholds as a DnaRecord.
///
/// Stores the 5 severity thresholds that define Guardian risk zones.
pub fn guardian_thresholds_record() -> DnaRecord {
    let mut record = DnaRecord::new();
    record.set("risk_threshold".to_string(), DnaValue::float(50.0));
    record.set("safe_severity_max".to_string(), DnaValue::int(25));
    record.set("concern_severity_max".to_string(), DnaValue::int(50));
    record.set("action_severity_max".to_string(), DnaValue::int(75));
    record.set("critical_severity_max".to_string(), DnaValue::int(100));
    record
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_detection_programs_build() {
        let programs = signal_detection_programs();
        assert!(programs.is_ok());
        if let Ok(progs) = programs {
            assert_eq!(progs.len(), 4);
            assert_eq!(progs[0].name, "PRR");
            assert_eq!(progs[1].name, "ROR");
            assert_eq!(progs[2].name, "IC");
            assert_eq!(progs[3].name, "EBGM");
            // Each strand must be valid DNA
            for p in &progs {
                assert!(!p.strand.is_empty());
                assert!(p.strand.chars().all(|c| matches!(c, 'A' | 'T' | 'G' | 'C')));
            }
        }
    }

    #[test]
    fn prr_compiles_and_runs() {
        let result = compiler::eval(PRR_SOURCE);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert!(!r.output.is_empty());
        }
    }

    #[test]
    fn ror_compiles_and_runs() {
        let result = compiler::eval(ROR_SOURCE);
        assert!(result.is_ok());
    }

    #[test]
    fn ic_compiles_and_runs() {
        let result = compiler::eval(IC_SOURCE);
        assert!(result.is_ok());
    }

    #[test]
    fn ebgm_compiles_and_runs() {
        let result = compiler::eval(EBGM_SOURCE);
        assert!(result.is_ok());
    }

    #[test]
    fn type_registry_encodes() {
        let registry = type_registry();
        assert_eq!(registry.len(), 5);
        let (name, record) = &registry[0];
        assert_eq!(name, "Nucleotide");
        assert!(!record.fields.is_empty());
    }

    #[test]
    fn guardian_thresholds_encode() {
        let record = guardian_thresholds_record();
        assert_eq!(record.fields.len(), 5);
    }

    #[test]
    fn nexcore_genome_strand_valid_dna() {
        let strand = nexcore_genome_strand();
        assert!(strand.is_ok());
        if let Ok(s) = strand {
            assert!(!s.is_empty());
            assert!(s.chars().all(|c| matches!(c, 'A' | 'T' | 'G' | 'C')));
        }
    }

    #[test]
    fn encode_type_record_nucleotide() {
        let record = encode_type_record::<crate::types::Nucleotide>("Nucleotide");
        assert_eq!(record.fields.len(), 5); // name, tier, dominant, confidence, primitive_count
    }

    #[test]
    fn encode_type_record_strand() {
        let record = encode_type_record::<crate::types::Strand>("Strand");
        assert!(record.fields.len() >= 4);
    }

    #[test]
    fn guardian_record_roundtrip() {
        let record = guardian_thresholds_record();
        let display = crate::data::transcribe_record(&record);
        assert!(display.contains("risk_threshold"));
        assert!(display.contains("safe_severity_max"));
    }

    #[test]
    fn nexcore_signal_genome_compiles() {
        let genome = nexcore_signal_genome();
        assert!(genome.is_ok());
        if let Ok(g) = genome {
            assert_eq!(g.gene_count(), 4);
            assert!(g.codon_count() > 0);
        }
    }

    #[test]
    fn measure_strand_sizes() {
        let programs = signal_detection_programs();
        assert!(programs.is_ok());
        if let Ok(progs) = programs {
            eprintln!("\n┌──────────────────────────────────────────────────────┐");
            eprintln!("│ SIGNAL DETECTION: Source vs DNA Strand Sizes         │");
            eprintln!("├──────────┬─────────────┬──────────────┬──────────────┤");
            eprintln!("│ Algorithm│ Source bytes │ Strand bases │ Compression  │");
            eprintln!("├──────────┼─────────────┼──────────────┼──────────────┤");
            let sources = [PRR_SOURCE, ROR_SOURCE, IC_SOURCE, EBGM_SOURCE];
            let mut total_source = 0usize;
            let mut total_strand = 0usize;
            for (i, p) in progs.iter().enumerate() {
                let src_bytes = sources[i].len();
                let strand_bases = p.strand.len();
                total_source += src_bytes;
                total_strand += strand_bases;
                eprintln!(
                    "│ {:<8} │ {:>11} │ {:>12} │ {:>10.1}×  │",
                    p.name,
                    src_bytes,
                    strand_bases,
                    src_bytes as f64 / strand_bases as f64
                );
            }
            eprintln!("├──────────┼─────────────┼──────────────┼──────────────┤");
            eprintln!(
                "│ TOTAL    │ {:>11} │ {:>12} │ {:>10.1}×  │",
                total_source,
                total_strand,
                total_source as f64 / total_strand as f64
            );
            eprintln!("└──────────┴─────────────┴──────────────┴──────────────┘");

            // Genome strand
            let genome = nexcore_genome_strand();
            if let Ok(s) = genome {
                eprintln!(
                    "Combined genome strand: {} nucleotides ({} bytes at 2 bits/base)",
                    s.len(),
                    s.len() / 4
                );
            }

            // Execution cost
            for (i, src) in sources.iter().enumerate() {
                let result = compiler::eval(src);
                if let Ok(r) = result {
                    eprintln!(
                        "{}: {} cycles, output={:?}",
                        progs[i].name, r.cycles, r.output
                    );
                }
            }
        }
    }

    #[test]
    fn prr_output_values_correct() {
        // a=15, b=100, c=20, d=10000
        // num = 15 * (20 + 10000) = 15 * 10020 = 150300
        // den = 20 * (15 + 100)   = 20 * 115   = 2300
        let result = compiler::eval(PRR_SOURCE);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.output.len(), 2);
            assert_eq!(r.output[0], 150300);
            assert_eq!(r.output[1], 2300);
        }
    }
}
