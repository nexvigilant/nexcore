//! Gene type system: biological DNA structures as programming constructs.
//!
//! In molecular biology:
//! - **Gene** = a unit of heredity encoding one protein (function)
//! - **Operon** = a cluster of co-regulated genes
//! - **Genome** = the complete genetic material of an organism (program)
//! - **Plasmid** = a small, portable, self-replicating DNA molecule
//!
//! In nexcore-dna, these map directly:
//! - **Gene** = a compiled function with its DNA sequence
//! - **Operon** = a module grouping related functions
//! - **Genome** = a complete compiled program with gene annotations
//! - **Plasmid** = a portable code snippet that can be inserted into any genome
//!
//! ## Biological Operations
//!
//! | Operation | Biology | nexcore-dna |
//! |-----------|---------|-------------|
//! | Expression | Gene → protein | Run a single function |
//! | Mutation | Point/insertion/deletion | Alter a gene's codons |
//! | Crossover | Homologous recombination | Swap segments between genes |
//! | Splicing | Insert transposon | Add a gene to a genome |
//! | Excision | Remove gene | Remove a gene from a genome |
//! | Transcription | DNA → mRNA | Extract gene's coding sequence |
//!
//! Tier: T3 (σ + μ + ∂ + ς + ρ + → + ∃ + Σ)

use crate::error::{DnaError, Result};
use crate::types::{Codon, Nucleotide, Strand};
use crate::vm::{CodonVM, VmConfig, VmResult};

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Gene — the fundamental unit of hereditary computation
// ---------------------------------------------------------------------------

/// A Gene: a named, parameterized function extracted from compiled DNA.
///
/// ## Biological Analogy
///
/// - **Promoter**: The start boundary (ATG-adjacent region)
/// - **Coding region**: The instruction codons (function body)
/// - **Terminator**: The return/halt instruction at the end
///
/// Tier: T3 (σ Sequence + μ Mapping + ∂ Boundary + ρ Recursion + → Causality)
#[derive(Debug, Clone)]
pub struct Gene {
    /// Gene name (function identifier).
    pub name: String,
    /// Parameter names (formal arguments).
    pub params: Vec<String>,
    /// Start codon index in the parent genome strand.
    pub start_codon: usize,
    /// End codon index (inclusive) in the parent genome strand.
    pub end_codon: usize,
    /// The gene's coding sequence (extracted sub-strand).
    pub sequence: Strand,
}

impl Gene {
    /// Number of codons in the coding region.
    #[must_use]
    pub fn codon_count(&self) -> usize {
        self.sequence.len() / 3
    }

    /// Number of nucleotides in the coding region.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sequence.len()
    }

    /// Whether the gene has zero nucleotides.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sequence.is_empty()
    }

    /// Number of formal parameters.
    #[must_use]
    pub fn arity(&self) -> usize {
        self.params.len()
    }

    /// Get a specific codon from the gene's coding sequence.
    pub fn codon_at(&self, offset: usize) -> Result<Codon> {
        let codons = self.sequence.codons()?;
        if offset >= codons.len() {
            return Err(DnaError::IndexOutOfBounds(offset, codons.len()));
        }
        Ok(codons[offset])
    }

    /// Transcribe: extract the coding sequence as a new strand.
    ///
    /// In biology, transcription copies DNA → mRNA.
    /// Here, it returns a standalone copy of the gene's DNA.
    #[must_use]
    pub fn transcribe(&self) -> Strand {
        self.sequence.clone()
    }

    /// Create a point mutation: change one codon at the given offset.
    ///
    /// Point mutations are the simplest form of genetic variation.
    /// They change a single codon without affecting the reading frame.
    pub fn point_mutate(&self, offset: usize, new_codon: Codon) -> Result<Gene> {
        let mut bases = self.sequence.bases.clone();
        let base_idx = offset * 3;
        if base_idx + 2 >= bases.len() {
            return Err(DnaError::IndexOutOfBounds(offset, bases.len() / 3));
        }
        bases[base_idx] = new_codon.0;
        bases[base_idx + 1] = new_codon.1;
        bases[base_idx + 2] = new_codon.2;
        Ok(Gene {
            name: self.name.clone(),
            params: self.params.clone(),
            start_codon: self.start_codon,
            end_codon: self.end_codon,
            sequence: Strand::new(bases),
        })
    }

    /// Deletion mutation: remove one codon (causes frameshift).
    ///
    /// Deletions remove genetic material, potentially altering
    /// all downstream instructions (frameshift).
    pub fn deletion(&self, offset: usize) -> Result<Gene> {
        let mut bases = self.sequence.bases.clone();
        let base_idx = offset * 3;
        if base_idx + 2 >= bases.len() {
            return Err(DnaError::IndexOutOfBounds(offset, bases.len() / 3));
        }
        // Remove 3 bases (one codon)
        bases.drain(base_idx..base_idx + 3);
        Ok(Gene {
            name: self.name.clone(),
            params: self.params.clone(),
            start_codon: self.start_codon,
            end_codon: if self.end_codon > 0 {
                self.end_codon - 1
            } else {
                0
            },
            sequence: Strand::new(bases),
        })
    }

    /// Insertion mutation: add a codon at the given offset (causes frameshift).
    ///
    /// Insertions add genetic material, pushing downstream instructions.
    pub fn insertion(&self, offset: usize, codon: Codon) -> Result<Gene> {
        let mut bases = self.sequence.bases.clone();
        let base_idx = offset * 3;
        if base_idx > bases.len() {
            return Err(DnaError::IndexOutOfBounds(offset, bases.len() / 3));
        }
        bases.insert(base_idx, codon.0);
        bases.insert(base_idx + 1, codon.1);
        bases.insert(base_idx + 2, codon.2);
        Ok(Gene {
            name: self.name.clone(),
            params: self.params.clone(),
            start_codon: self.start_codon,
            end_codon: self.end_codon + 1,
            sequence: Strand::new(bases),
        })
    }

    /// Display the gene's DNA sequence as a codon-delimited string.
    #[must_use]
    pub fn dna_display(&self) -> String {
        let codons = self.sequence.codons().unwrap_or_default();
        codons
            .iter()
            .map(|c| format!("{c}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl core::fmt::Display for Gene {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Gene({}, arity={}, codons={})",
            self.name,
            self.arity(),
            self.codon_count()
        )
    }
}

// ---------------------------------------------------------------------------
// Crossover — homologous recombination between genes
// ---------------------------------------------------------------------------

/// Single-point crossover: swap segments between two genes.
///
/// Given genes A and B and a crossover point (codon offset),
/// produces two offspring:
/// - Offspring 1: A[0..point] + B[point..]
/// - Offspring 2: B[0..point] + A[point..]
///
/// This models homologous recombination in biology.
pub fn crossover(gene_a: &Gene, gene_b: &Gene, point: usize) -> Result<(Gene, Gene)> {
    let codons_a = gene_a.sequence.codons()?;
    let codons_b = gene_b.sequence.codons()?;

    if point > codons_a.len() || point > codons_b.len() {
        let max_len = codons_a.len().max(codons_b.len());
        return Err(DnaError::IndexOutOfBounds(point, max_len));
    }

    // Build offspring strands
    let offspring_1 = build_strand_from_codons(&codons_a[..point], &codons_b[point..]);
    let offspring_2 = build_strand_from_codons(&codons_b[..point], &codons_a[point..]);

    Ok((
        Gene {
            name: format!("{}_x_{}", gene_a.name, gene_b.name),
            params: gene_a.params.clone(),
            start_codon: 0,
            end_codon: offspring_1.len() / 3,
            sequence: offspring_1,
        },
        Gene {
            name: format!("{}_x_{}", gene_b.name, gene_a.name),
            params: gene_b.params.clone(),
            start_codon: 0,
            end_codon: offspring_2.len() / 3,
            sequence: offspring_2,
        },
    ))
}

/// Build a strand from two slices of codons.
fn build_strand_from_codons(head: &[Codon], tail: &[Codon]) -> Strand {
    let mut bases = Vec::with_capacity((head.len() + tail.len()) * 3);
    for c in head.iter().chain(tail.iter()) {
        bases.push(c.0);
        bases.push(c.1);
        bases.push(c.2);
    }
    Strand::new(bases)
}

// ---------------------------------------------------------------------------
// GeneAnnotation — metadata from compilation
// ---------------------------------------------------------------------------

/// Annotation for a gene produced during compilation.
///
/// The compiler records where each function's code lands in the
/// assembled strand, enabling gene extraction from the genome.
///
/// Tier: T2-P (μ Mapping + ∂ Boundary)
#[derive(Debug, Clone)]
pub struct GeneAnnotation {
    /// Function name.
    pub name: String,
    /// Formal parameter names.
    pub params: Vec<String>,
    /// Codon index where the function starts (label position).
    pub start_codon: usize,
}

// ---------------------------------------------------------------------------
// Genome — the complete compiled program with gene catalog
// ---------------------------------------------------------------------------

/// A Genome: the complete compiled program as an annotated DNA strand.
///
/// Contains the full DNA sequence plus a catalog of gene locations,
/// enabling individual gene expression, mutation, and recombination.
///
/// Tier: T3 (σ + μ + ∂ + ∃ + Σ + ς)
#[derive(Debug, Clone)]
pub struct Genome {
    /// The complete DNA strand (entire program).
    pub dna: Strand,
    /// The gene catalog: all functions with their locations.
    pub genes: Vec<Gene>,
    /// Main code region start (codon index, after entry ATG).
    pub main_start: usize,
    /// Main code region end (codon index, the halt instruction).
    pub main_end: usize,
    /// Data segment values (pre-loaded into VM memory).
    pub data: Vec<i64>,
}

impl Genome {
    /// Build a Genome from a compiled program and gene annotations.
    ///
    /// Extracts gene sequences from the program's DNA strand using
    /// the annotation positions provided by the compiler.
    pub fn from_program(
        program: &crate::program::Program,
        annotations: Vec<GeneAnnotation>,
        labels: &HashMap<String, usize>,
    ) -> Result<Self> {
        let codons = program.code.codons()?;
        let total_codons = codons.len();

        // Find main region: entry (first ATG) to halt
        let main_start = codons.iter().position(|c| c.is_start()).unwrap_or(0) + 1; // skip ATG itself

        // Main ends at first halt (Stop codon) after entry
        let main_end = codons
            .iter()
            .skip(main_start)
            .position(|c| c.is_stop())
            .map(|p| p + main_start)
            .unwrap_or(total_codons.saturating_sub(1));

        // Extract genes from annotations
        let mut genes = Vec::with_capacity(annotations.len());
        for (i, ann) in annotations.iter().enumerate() {
            let start = ann.start_codon;
            // End is the codon before the next function, or end of strand
            let end = if i + 1 < annotations.len() {
                annotations[i + 1].start_codon.saturating_sub(1)
            } else {
                total_codons.saturating_sub(1)
            };

            // Find the ret instruction to get precise end
            let precise_end = find_ret_codon(&codons, start, end);

            // Extract sub-strand
            let gene_codons = &codons[start..=precise_end];
            let mut bases = Vec::with_capacity(gene_codons.len() * 3);
            for c in gene_codons {
                bases.push(c.0);
                bases.push(c.1);
                bases.push(c.2);
            }

            genes.push(Gene {
                name: ann.name.clone(),
                params: ann.params.clone(),
                start_codon: start,
                end_codon: precise_end,
                sequence: Strand::new(bases),
            });
        }

        // Sort genes by label position for deterministic ordering
        let _ = labels; // labels used for future features (splicing)

        Ok(Genome {
            dna: program.code.clone(),
            genes,
            main_start,
            main_end,
            data: program.data.clone(),
        })
    }

    /// Find a gene by name.
    #[must_use]
    pub fn find_gene(&self, name: &str) -> Option<&Gene> {
        self.genes.iter().find(|g| g.name == name)
    }

    /// Express (execute) a specific gene with the given arguments.
    ///
    /// In biology, gene expression produces a protein.
    /// Here, it runs the function and returns the result.
    ///
    /// The full genome DNA is loaded into the VM, arguments are pushed
    /// onto the stack, and execution jumps directly to the gene's entry.
    pub fn express(&self, gene_name: &str, args: &[i64]) -> Result<VmResult> {
        self.express_with(gene_name, args, VmConfig::default())
    }

    /// Express a gene with custom VM config.
    pub fn express_with(
        &self,
        gene_name: &str,
        args: &[i64],
        config: VmConfig,
    ) -> Result<VmResult> {
        let gene = self
            .find_gene(gene_name)
            .ok_or_else(|| DnaError::GeneNotFound(gene_name.to_string()))?;

        let mut vm = CodonVM::with_config(config);
        vm.load(&self.dna)?;

        // Pre-load data segment
        for (addr, &value) in self.data.iter().enumerate() {
            vm.write_memory(addr, value)?;
        }

        // Push arguments (left-to-right, callee pops in reverse)
        for &arg in args {
            vm.push_value(arg)?;
        }

        // Execute from gene start
        vm.execute_from(gene.start_codon)
    }

    /// Run the main program (equivalent to Program::run).
    pub fn run(&self) -> Result<VmResult> {
        let program = crate::program::Program::new(self.data.clone(), self.dna.clone());
        program.run()
    }

    /// Total codon count in the genome.
    #[must_use]
    pub fn codon_count(&self) -> usize {
        self.dna.len() / 3
    }

    /// Number of annotated genes.
    #[must_use]
    pub fn gene_count(&self) -> usize {
        self.genes.len()
    }

    /// Get a catalog of all genes: (name, arity, codon_count).
    #[must_use]
    pub fn catalog(&self) -> Vec<(&str, usize, usize)> {
        self.genes
            .iter()
            .map(|g| (g.name.as_str(), g.arity(), g.codon_count()))
            .collect()
    }

    /// Get the GC content ratio of the genome (G+C / total).
    ///
    /// GC content is a fundamental property of DNA, affecting melting
    /// temperature and structural stability.
    #[must_use]
    pub fn gc_content(&self) -> f64 {
        if self.dna.is_empty() {
            return 0.0;
        }
        let gc = self
            .dna
            .bases
            .iter()
            .filter(|n| matches!(n, Nucleotide::G | Nucleotide::C))
            .count();
        gc as f64 / self.dna.len() as f64
    }
}

impl core::fmt::Display for Genome {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Genome(codons={}, genes={}, data={})",
            self.codon_count(),
            self.gene_count(),
            self.data.len()
        )
    }
}

// ---------------------------------------------------------------------------
// Plasmid — portable code snippet
// ---------------------------------------------------------------------------

/// A Plasmid: a portable, self-contained DNA snippet carrying genes.
///
/// In biology, plasmids are small circular DNA molecules that can be
/// transferred between organisms, carrying useful genes (e.g., antibiotic
/// resistance).
///
/// In nexcore-dna, a Plasmid packages one or more genes as a standalone
/// DNA fragment that can be inserted into any genome.
///
/// Tier: T2-C (σ Sequence + ∂ Boundary + μ Mapping)
#[derive(Debug, Clone)]
pub struct Plasmid {
    /// Plasmid name.
    pub name: String,
    /// Genes carried by this plasmid.
    pub genes: Vec<Gene>,
    /// The plasmid's DNA sequence.
    pub dna: Strand,
}

impl Plasmid {
    /// Create a plasmid from a gene.
    ///
    /// Wraps a single gene in a plasmid container.
    #[must_use]
    pub fn from_gene(gene: Gene) -> Self {
        let dna = gene.sequence.clone();
        let name = gene.name.clone();
        Self {
            name,
            genes: vec![gene],
            dna,
        }
    }

    /// Create a plasmid from source code.
    ///
    /// Compiles the source, extracts the first function as a gene,
    /// and wraps it in a plasmid.
    pub fn from_source(name: &str, source: &str) -> Result<Self> {
        let genome = crate::lang::compiler::compile_genome(source)?;
        if genome.genes.is_empty() {
            return Err(DnaError::GeneNotFound(
                "no functions found in source".to_string(),
            ));
        }
        Ok(Self {
            name: name.to_string(),
            genes: genome.genes,
            dna: genome.dna,
        })
    }

    /// Number of genes in this plasmid.
    #[must_use]
    pub fn gene_count(&self) -> usize {
        self.genes.len()
    }

    /// Total codon count.
    #[must_use]
    pub fn codon_count(&self) -> usize {
        self.dna.len() / 3
    }
}

impl core::fmt::Display for Plasmid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Plasmid({}, genes={}, codons={})",
            self.name,
            self.gene_count(),
            self.codon_count()
        )
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find the last `ret` (CTG, codon index 54) instruction in a range.
///
/// In biology, the terminator marks where transcription ends.
/// Here, `ret` marks where a function's coding region ends.
fn find_ret_codon(codons: &[Codon], start: usize, max_end: usize) -> usize {
    // CTG = Leu = ret instruction, codon index 54
    let ret_codon = Codon(Nucleotide::C, Nucleotide::T, Nucleotide::G);
    let end = max_end.min(codons.len().saturating_sub(1));

    // Scan backwards from end to find ret
    for i in (start..=end).rev() {
        if codons[i] == ret_codon {
            return i;
        }
    }
    // If no ret found, return max_end (function might end with halt)
    end
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile source to a genome for testing.
    fn genome(source: &str) -> Genome {
        crate::lang::compiler::compile_genome(source).unwrap_or_else(|e| {
            panic!("genome compilation failed: {e}");
        })
    }

    // --- Gene basics ---

    #[test]
    fn genome_has_gene() {
        let g = genome("fn double(x) do\n  return x * 2\nend\ndouble(5)");
        assert_eq!(g.gene_count(), 1);
        let gene = g.find_gene("double");
        assert!(gene.is_some());
        if let Some(gene) = gene {
            assert_eq!(gene.name, "double");
            assert_eq!(gene.arity(), 1);
            assert!(gene.codon_count() > 0);
        }
    }

    #[test]
    fn genome_multiple_genes() {
        let g = genome(
            "fn add(a, b) do\n  return a + b\nend\n\
             fn mul(a, b) do\n  return a * b\nend\n\
             add(2, 3)",
        );
        assert_eq!(g.gene_count(), 2);
        assert!(g.find_gene("add").is_some());
        assert!(g.find_gene("mul").is_some());
    }

    #[test]
    fn genome_catalog() {
        let g = genome(
            "fn inc(x) do\n  return x + 1\nend\n\
             fn dec(x) do\n  return x - 1\nend\n\
             inc(0)",
        );
        let cat = g.catalog();
        assert_eq!(cat.len(), 2);
        assert_eq!(cat[0].0, "inc"); // name
        assert_eq!(cat[0].1, 1); // arity
        assert_eq!(cat[1].0, "dec");
    }

    // --- Gene expression ---

    #[test]
    fn express_gene() {
        let g = genome("fn double(x) do\n  return x * 2\nend\ndouble(1)");
        let result = g.express("double", &[21]);
        assert!(result.is_ok());
        if let Ok(r) = result {
            // The function returns via ret, which pops call stack.
            // Since we jump directly, ret will halt (empty call stack → Normal halt).
            assert!(!r.stack.is_empty());
            assert_eq!(r.stack[r.stack.len() - 1], 42);
        }
    }

    #[test]
    fn express_nonexistent_gene() {
        let g = genome("fn f() do\n  return 1\nend\nf()");
        let result = g.express("nonexistent", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn express_no_params() {
        let g = genome("fn answer() do\n  return 42\nend\nanswer()");
        let result = g.express("answer", &[]);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert!(!r.stack.is_empty());
            assert_eq!(r.stack[r.stack.len() - 1], 42);
        }
    }

    // --- Main program execution ---

    #[test]
    fn genome_run_main() {
        let g = genome("fn triple(x) do\n  return x * 3\nend\ntriple(7)");
        let result = g.run();
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.output, vec![21]);
        }
    }

    // --- Transcription ---

    #[test]
    fn gene_transcribe() {
        let g = genome("fn f(x) do\n  return x + 1\nend\nf(0)");
        if let Some(gene) = g.find_gene("f") {
            let mrna = gene.transcribe();
            assert!(!mrna.is_empty());
            assert_eq!(mrna.len(), gene.len());
        }
    }

    // --- Mutations ---

    #[test]
    fn gene_point_mutation() {
        let g = genome("fn f(x) do\n  return x + 1\nend\nf(0)");
        if let Some(gene) = g.find_gene("f") {
            let nop_codon = Codon(Nucleotide::G, Nucleotide::G, Nucleotide::A); // NOP
            let mutant = gene.point_mutate(0, nop_codon);
            assert!(mutant.is_ok());
            if let Ok(m) = mutant {
                let c = m.codon_at(0);
                assert!(c.is_ok());
                if let Ok(codon) = c {
                    assert_eq!(codon, nop_codon);
                }
            }
        }
    }

    #[test]
    fn gene_deletion_mutation() {
        let g = genome("fn f(x) do\n  return x + 1\nend\nf(0)");
        if let Some(gene) = g.find_gene("f") {
            let original_count = gene.codon_count();
            let mutant = gene.deletion(0);
            assert!(mutant.is_ok());
            if let Ok(m) = mutant {
                assert_eq!(m.codon_count(), original_count - 1);
            }
        }
    }

    #[test]
    fn gene_insertion_mutation() {
        let g = genome("fn f(x) do\n  return x + 1\nend\nf(0)");
        if let Some(gene) = g.find_gene("f") {
            let original_count = gene.codon_count();
            let nop_codon = Codon(Nucleotide::G, Nucleotide::G, Nucleotide::A);
            let mutant = gene.insertion(0, nop_codon);
            assert!(mutant.is_ok());
            if let Ok(m) = mutant {
                assert_eq!(m.codon_count(), original_count + 1);
            }
        }
    }

    #[test]
    fn gene_mutation_out_of_bounds() {
        let g = genome("fn f() do\n  return 1\nend\nf()");
        if let Some(gene) = g.find_gene("f") {
            let nop = Codon(Nucleotide::G, Nucleotide::G, Nucleotide::A);
            let result = gene.point_mutate(9999, nop);
            assert!(result.is_err());
        }
    }

    // --- Crossover ---

    #[test]
    fn crossover_genes() {
        let g = genome(
            "fn add1(x) do\n  return x + 1\nend\n\
             fn mul2(x) do\n  return x * 2\nend\n\
             add1(0)",
        );
        let gene_a = g.find_gene("add1");
        let gene_b = g.find_gene("mul2");
        if let (Some(a), Some(b)) = (gene_a, gene_b) {
            let min_len = a.codon_count().min(b.codon_count());
            if min_len > 1 {
                let result = crossover(a, b, 1);
                assert!(result.is_ok());
                if let Ok((o1, o2)) = result {
                    assert!(o1.codon_count() > 0);
                    assert!(o2.codon_count() > 0);
                }
            }
        }
    }

    #[test]
    fn crossover_at_zero() {
        let g = genome(
            "fn a(x) do\n  return x\nend\n\
             fn b(x) do\n  return x\nend\n\
             a(0)",
        );
        if let (Some(ga), Some(gb)) = (g.find_gene("a"), g.find_gene("b")) {
            let result = crossover(ga, gb, 0);
            assert!(result.is_ok());
        }
    }

    // --- Plasmid ---

    #[test]
    fn plasmid_from_source() {
        let p = Plasmid::from_source("math", "fn add(a, b) do\n  return a + b\nend\nadd(1, 2)");
        assert!(p.is_ok());
        if let Ok(plasmid) = p {
            assert_eq!(plasmid.name, "math");
            assert!(plasmid.gene_count() > 0);
            assert!(plasmid.codon_count() > 0);
        }
    }

    #[test]
    fn plasmid_no_functions() {
        let p = Plasmid::from_source("empty", "42");
        assert!(p.is_err());
    }

    #[test]
    fn plasmid_from_gene() {
        let g = genome("fn f(x) do\n  return x\nend\nf(0)");
        if let Some(gene) = g.find_gene("f") {
            let p = Plasmid::from_gene(gene.clone());
            assert_eq!(p.name, "f");
            assert_eq!(p.gene_count(), 1);
        }
    }

    // --- Genome properties ---

    #[test]
    fn genome_gc_content() {
        let g = genome("42");
        let gc = g.gc_content();
        // GC content should be between 0 and 1
        assert!(gc >= 0.0);
        assert!(gc <= 1.0);
    }

    #[test]
    fn genome_display() {
        let g = genome("fn f(x) do\n  return x\nend\nf(0)");
        let s = format!("{g}");
        assert!(s.contains("Genome"));
        assert!(s.contains("genes=1"));
    }

    #[test]
    fn gene_display() {
        let g = genome("fn double(x) do\n  return x * 2\nend\ndouble(1)");
        if let Some(gene) = g.find_gene("double") {
            let s = format!("{gene}");
            assert!(s.contains("Gene"));
            assert!(s.contains("double"));
            assert!(s.contains("arity=1"));
        }
    }

    #[test]
    fn gene_dna_display() {
        let g = genome("fn f() do\n  return 1\nend\nf()");
        if let Some(gene) = g.find_gene("f") {
            let dna = gene.dna_display();
            assert!(!dna.is_empty());
            // Should contain three-letter codon groups
            assert!(dna.contains(' '));
        }
    }

    // --- Wave 5 gene integration tests ---

    #[test]
    fn genome_express_two_params() {
        let g = genome("fn add(a, b) do\n  return a + b\nend\nadd(1, 2)");
        let result = g.express("add", &[17, 25]);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert!(!r.stack.is_empty());
            assert_eq!(r.stack[r.stack.len() - 1], 42);
        }
    }

    #[test]
    fn genome_express_recursive() {
        let source = "
fn fact(n) do
  if n <= 1 do
    return 1
  end
  return n * fact(n - 1)
end
fact(1)
";
        let g = genome(source);
        // Express factorial(5) = 120
        let result = g.express("fact", &[5]);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert!(!r.stack.is_empty());
            assert_eq!(r.stack[r.stack.len() - 1], 120);
        }
    }

    #[test]
    fn crossover_display() {
        let g = genome(
            "fn a(x) do\n  return x + 1\nend\n\
             fn b(x) do\n  return x * 2\nend\n\
             a(0)",
        );
        if let (Some(ga), Some(gb)) = (g.find_gene("a"), g.find_gene("b")) {
            let min_len = ga.codon_count().min(gb.codon_count());
            if min_len > 1 {
                let result = crossover(ga, gb, 1);
                assert!(result.is_ok());
                if let Ok((o1, o2)) = result {
                    // Offspring should have meaningful names
                    assert!(o1.name.contains("_x_"));
                    assert!(o2.name.contains("_x_"));
                    // Display should work
                    let s1 = format!("{o1}");
                    let s2 = format!("{o2}");
                    assert!(s1.contains("Gene"));
                    assert!(s2.contains("Gene"));
                }
            }
        }
    }

    #[test]
    fn plasmid_compile_and_express() {
        let source = "fn square(x) do\n  return x * x\nend\nsquare(1)";
        let p = Plasmid::from_source("math_utils", source);
        assert!(p.is_ok());
        if let Ok(plasmid) = p {
            assert_eq!(plasmid.name, "math_utils");
            assert!(plasmid.gene_count() >= 1);
            // Verify the gene was captured
            let has_square = plasmid.genes.iter().any(|g| g.name == "square");
            assert!(has_square);
            // Display should work
            let s = format!("{plasmid}");
            assert!(s.contains("Plasmid"));
            assert!(s.contains("math_utils"));
        }
    }
}
