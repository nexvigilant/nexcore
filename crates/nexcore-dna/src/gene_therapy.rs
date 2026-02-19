//! Gene Replacement Therapy Research Toolkit.
//!
//! Provides computational tools for CRISPR/Cas9 guide design, off-target
//! analysis, HDR donor template construction, codon optimization, and
//! delivery vector payload feasibility checking.
//!
//! Tier: T2-C (σ + μ + κ + ∂ + → + N)
//!
//! **Research use only.** Not for clinical decision-making.

use crate::alignment::SequenceAligner;
use crate::codon_table::CodonTable;
use crate::error::{DnaError, Result};
use crate::ops;
use crate::types::{AminoAcid, Nucleotide, Strand};

use core::fmt;

// ---------------------------------------------------------------------------
// PAM Sites & Guide RNA Design
// ---------------------------------------------------------------------------

/// A CRISPR PAM (Protospacer Adjacent Motif) type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PamType {
    /// SpCas9: NGG (most common)
    Ngg,
    /// SaCas9: NNGRRT
    Nngrrt,
    /// Cas12a/Cpf1: TTTV (upstream PAM)
    Tttv,
}

impl PamType {
    /// PAM sequence pattern as human-readable string.
    #[must_use]
    pub fn pattern(&self) -> &'static str {
        match self {
            Self::Ngg => "NGG",
            Self::Nngrrt => "NNGRRT",
            Self::Tttv => "TTTV",
        }
    }

    /// Length of the PAM sequence.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Ngg => 3,
            Self::Nngrrt => 6,
            Self::Tttv => 4,
        }
    }

    /// Whether the PAM is upstream (5') or downstream (3') of the protospacer.
    #[must_use]
    pub fn is_upstream(&self) -> bool {
        matches!(self, Self::Tttv)
    }

    /// Default guide length for this PAM type.
    #[must_use]
    pub fn guide_length(&self) -> usize {
        match self {
            Self::Ngg => 20,
            Self::Nngrrt => 21,
            Self::Tttv => 23,
        }
    }

    /// Check if bases at a position match this PAM.
    fn matches_at(&self, bases: &[Nucleotide], pos: usize) -> bool {
        match self {
            Self::Ngg => {
                pos + 3 <= bases.len()
                    // N = any, G, G
                    && bases[pos + 1] == Nucleotide::G
                    && bases[pos + 2] == Nucleotide::G
            }
            Self::Nngrrt => {
                pos + 6 <= bases.len()
                    // N, N, G, R(A|G), R(A|G), T
                    && bases[pos + 2] == Nucleotide::G
                    && matches!(bases[pos + 3], Nucleotide::A | Nucleotide::G)
                    && matches!(bases[pos + 4], Nucleotide::A | Nucleotide::G)
                    && bases[pos + 5] == Nucleotide::T
            }
            Self::Tttv => {
                pos + 4 <= bases.len()
                    // T, T, T, V(A|G|C, not T)
                    && bases[pos] == Nucleotide::T
                    && bases[pos + 1] == Nucleotide::T
                    && bases[pos + 2] == Nucleotide::T
                    && bases[pos + 3] != Nucleotide::T
            }
        }
    }
}

impl fmt::Display for PamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pattern())
    }
}

/// A designed CRISPR guide RNA with scoring.
#[derive(Debug, Clone)]
pub struct GuideRna {
    /// The protospacer sequence (complementary to target).
    pub protospacer: Strand,
    /// Position on the target strand where the guide binds.
    pub position: usize,
    /// Which strand the guide targets (true = sense/forward).
    pub sense_strand: bool,
    /// PAM type used.
    pub pam: PamType,
    /// GC content of the guide (optimal: 40-70%).
    pub gc_content: f64,
    /// Composite on-target score (0.0-1.0).
    pub on_target_score: f64,
    /// Self-complementarity score (lower is better, max 1.0).
    pub self_comp: f64,
    /// Cut site position (relative to target sequence).
    pub cut_site: usize,
}

impl GuideRna {
    /// Overall quality score combining multiple factors.
    #[must_use]
    pub fn quality_score(&self) -> f64 {
        let gc_penalty = if (0.4..=0.7).contains(&self.gc_content) {
            1.0
        } else if (0.3..=0.8).contains(&self.gc_content) {
            0.7
        } else {
            0.3
        };
        let self_comp_penalty = 1.0 - self.self_comp;
        self.on_target_score * gc_penalty * self_comp_penalty
    }
}

impl fmt::Display for GuideRna {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let strand_label = if self.sense_strand { "+" } else { "-" };
        write!(
            f,
            "Guide@{}({}) PAM={} GC={:.1}% score={:.3} cut={}  {}",
            self.position,
            strand_label,
            self.pam,
            self.gc_content * 100.0,
            self.quality_score(),
            self.cut_site,
            self.protospacer.to_string_repr(),
        )
    }
}

/// Find all CRISPR guide RNA candidates in a target sequence.
///
/// Scans both sense and antisense strands for PAM sites, extracts
/// protospacer sequences, and scores each candidate.
pub fn design_guides(target: &Strand, pam: PamType) -> Vec<GuideRna> {
    let guide_len = pam.guide_length();
    let mut guides = Vec::new();

    // Scan sense strand
    scan_strand_for_guides(target, pam, guide_len, true, &mut guides);

    // Scan antisense strand
    let antisense = ops::complement(target);
    scan_strand_for_guides(&antisense, pam, guide_len, false, &mut guides);

    // Sort by quality score descending
    guides.sort_by(|a, b| {
        b.quality_score()
            .partial_cmp(&a.quality_score())
            .unwrap_or(core::cmp::Ordering::Equal)
    });

    guides
}

fn scan_strand_for_guides(
    strand: &Strand,
    pam: PamType,
    guide_len: usize,
    is_sense: bool,
    guides: &mut Vec<GuideRna>,
) {
    let bases = &strand.bases;

    if pam.is_upstream() {
        // PAM is 5' of protospacer (Cas12a/Cpf1)
        for i in 0..bases.len().saturating_sub(pam.len() + guide_len) {
            if pam.matches_at(bases, i) {
                let proto_start = i + pam.len();
                let proto_end = proto_start + guide_len;
                if proto_end <= bases.len() {
                    let protospacer = Strand::new(bases[proto_start..proto_end].to_vec());
                    let gc = ops::gc_content(&protospacer);
                    let on_score = score_guide_on_target(&protospacer);
                    let sc = self_complementarity(&protospacer);
                    let cut = if is_sense {
                        proto_start + 18 // Cas12a cuts ~18-23bp from PAM
                    } else {
                        strand.len() - proto_start - 18
                    };
                    guides.push(GuideRna {
                        protospacer,
                        position: if is_sense {
                            proto_start
                        } else {
                            strand.len() - proto_end
                        },
                        sense_strand: is_sense,
                        pam,
                        gc_content: gc,
                        on_target_score: on_score,
                        self_comp: sc,
                        cut_site: cut,
                    });
                }
            }
        }
    } else {
        // PAM is 3' of protospacer (SpCas9, SaCas9)
        for i in guide_len..bases.len().saturating_sub(pam.len()) {
            let pam_pos = i;
            if pam.matches_at(bases, pam_pos) {
                let proto_start = pam_pos - guide_len;
                let protospacer = Strand::new(bases[proto_start..pam_pos].to_vec());
                let gc = ops::gc_content(&protospacer);
                let on_score = score_guide_on_target(&protospacer);
                let sc = self_complementarity(&protospacer);
                // SpCas9 cuts 3bp upstream of PAM
                let cut = if is_sense {
                    pam_pos - 3
                } else {
                    strand.len() - pam_pos + 3
                };
                guides.push(GuideRna {
                    protospacer,
                    position: if is_sense {
                        proto_start
                    } else {
                        strand.len() - pam_pos
                    },
                    sense_strand: is_sense,
                    pam,
                    gc_content: gc,
                    on_target_score: on_score,
                    self_comp: sc,
                    cut_site: cut,
                });
            }
        }
    }
}

/// Heuristic on-target activity score based on positional nucleotide preferences.
/// Based on Doench et al. 2016 Rule Set 2 simplified heuristics.
fn score_guide_on_target(guide: &Strand) -> f64 {
    let bases = &guide.bases;
    let len = bases.len();
    if len < 20 {
        return 0.5;
    }

    let mut score: f64 = 0.5;

    // Position 20 (PAM-proximal): G preferred
    if bases[len - 1] == Nucleotide::G {
        score += 0.1;
    }
    // Position 16: C preferred (seed region)
    if len > 4 && bases[len - 5] == Nucleotide::C {
        score += 0.05;
    }
    // Positions 1-4 (PAM-distal): avoid T-rich
    let distal_t: usize = bases[..4.min(len)]
        .iter()
        .filter(|&&b| b == Nucleotide::T)
        .count();
    if distal_t >= 3 {
        score -= 0.15;
    }
    // Avoid polyN runs (>= 4 identical bases in a row)
    let mut max_run = 1usize;
    let mut current_run = 1usize;
    for i in 1..len {
        if bases[i] == bases[i - 1] {
            current_run += 1;
            if current_run > max_run {
                max_run = current_run;
            }
        } else {
            current_run = 1;
        }
    }
    if max_run >= 4 {
        score -= 0.2;
    }

    // GC in seed region (last 12 bases) — prefer 40-70%
    let seed_start = len.saturating_sub(12);
    let seed = Strand::new(bases[seed_start..].to_vec());
    let seed_gc = ops::gc_content(&seed);
    if (0.4..=0.7).contains(&seed_gc) {
        score += 0.1;
    }

    score.clamp(0.0_f64, 1.0_f64)
}

/// Self-complementarity: fraction of bases that can form internal hairpins.
fn self_complementarity(guide: &Strand) -> f64 {
    let bases = &guide.bases;
    let len = bases.len();
    if len < 8 {
        return 0.0;
    }
    let mut matches = 0usize;
    // Check for reverse-complement matches between first half and second half
    let half = len / 2;
    for i in 0..half {
        if bases[i].complement() == bases[len - 1 - i] {
            matches += 1;
        }
    }
    matches as f64 / half as f64
}

// ---------------------------------------------------------------------------
// Off-Target Analysis
// ---------------------------------------------------------------------------

/// An off-target hit from scanning a reference sequence.
#[derive(Debug, Clone)]
pub struct OffTargetHit {
    /// Position in the reference.
    pub position: usize,
    /// Number of mismatches.
    pub mismatches: usize,
    /// Whether the PAM is present at this off-target site.
    pub pam_present: bool,
    /// The off-target sequence.
    pub sequence: Strand,
    /// Off-target score (lower = more likely to cut, higher = safer).
    pub score: f64,
}

impl fmt::Display for OffTargetHit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OffTarget@{} mm={} pam={} score={:.3} {}",
            self.position,
            self.mismatches,
            if self.pam_present { "Y" } else { "N" },
            self.score,
            self.sequence.to_string_repr(),
        )
    }
}

/// Scan a reference sequence for potential off-target sites.
///
/// Returns all positions with <= `max_mismatches` differences from the guide.
/// Includes mismatch-position-weighted scoring (seed region mismatches reduce risk more).
pub fn off_target_scan(
    guide: &GuideRna,
    reference: &Strand,
    max_mismatches: usize,
) -> Vec<OffTargetHit> {
    let guide_len = guide.protospacer.len();
    let ref_bases = &reference.bases;
    let guide_bases = &guide.protospacer.bases;

    let mut hits = Vec::new();

    if ref_bases.len() < guide_len + guide.pam.len() {
        return hits;
    }

    for i in 0..=ref_bases.len() - guide_len - guide.pam.len() {
        let mut mismatches = 0usize;
        let mut mismatch_positions = Vec::new();

        for j in 0..guide_len {
            if ref_bases[i + j] != guide_bases[j] {
                mismatches += 1;
                mismatch_positions.push(j);
                if mismatches > max_mismatches {
                    break;
                }
            }
        }

        if mismatches <= max_mismatches {
            // Check PAM presence downstream
            let pam_pos = i + guide_len;
            let pam_present = guide.pam.matches_at(ref_bases, pam_pos);

            // MIT-style off-target scoring: mismatches in seed region (last 12 bases)
            // are weighted more heavily
            let ot_score = if pam_present {
                compute_off_target_score(guide_len, &mismatch_positions)
            } else {
                1.0 // No PAM = very unlikely to cut
            };

            let sequence = Strand::new(ref_bases[i..i + guide_len].to_vec());

            hits.push(OffTargetHit {
                position: i,
                mismatches,
                pam_present,
                sequence,
                score: ot_score,
            });
        }
    }

    // Sort by score ascending (most dangerous first)
    hits.sort_by(|a, b| {
        a.score
            .partial_cmp(&b.score)
            .unwrap_or(core::cmp::Ordering::Equal)
    });

    hits
}

/// MIT off-target scoring: position-dependent mismatch penalties.
/// Lower score = higher off-target risk.
fn compute_off_target_score(guide_len: usize, mismatch_positions: &[usize]) -> f64 {
    if mismatch_positions.is_empty() {
        return 0.0; // Perfect match = maximum risk
    }

    // Position weights: PAM-proximal (seed) mismatches reduce cutting more
    // Weight increases toward the PAM end (position guide_len-1)
    let mut product = 1.0f64;
    for &pos in mismatch_positions {
        let distance_from_pam = guide_len.saturating_sub(pos + 1);
        // Seed region (last 12 bases) has higher weight
        let weight = if distance_from_pam < 12 {
            0.2 + 0.05 * distance_from_pam as f64 // 0.2-0.75 for seed
        } else {
            0.8 + 0.02 * (distance_from_pam - 12) as f64 // 0.8-1.0 for distal
        };
        product *= 1.0 - weight;
    }

    // Mean pairwise distance penalty
    let mean_dist = if mismatch_positions.len() > 1 {
        let mut total_dist = 0usize;
        let mut pairs = 0usize;
        for i in 0..mismatch_positions.len() {
            for j in (i + 1)..mismatch_positions.len() {
                total_dist += mismatch_positions[j] - mismatch_positions[i];
                pairs += 1;
            }
        }
        if pairs > 0 {
            total_dist as f64 / pairs as f64 / guide_len as f64
        } else {
            0.0
        }
    } else {
        0.0
    };

    // Combine: more mismatches + wider spread + seed mismatches = safer
    let count_penalty = 1.0 / (1.0 + mismatch_positions.len() as f64);
    (1.0 - product) * (1.0 - count_penalty) + mean_dist * 0.1
}

// ---------------------------------------------------------------------------
// HDR Donor Template Design
// ---------------------------------------------------------------------------

/// An HDR (Homology-Directed Repair) donor template for gene correction.
#[derive(Debug, Clone)]
pub struct DonorTemplate {
    /// Left homology arm.
    pub left_arm: Strand,
    /// The corrected sequence (replaces mutant).
    pub correction: Strand,
    /// Right homology arm.
    pub right_arm: Strand,
    /// Total template length.
    pub total_length: usize,
    /// Mutation type being corrected.
    pub mutation_type: MutationType,
}

/// Type of mutation being corrected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationType {
    /// Single nucleotide variant.
    Snv,
    /// Small insertion.
    Insertion,
    /// Small deletion.
    Deletion,
    /// Multiple base substitution.
    Complex,
}

impl fmt::Display for MutationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snv => write!(f, "SNV"),
            Self::Insertion => write!(f, "insertion"),
            Self::Deletion => write!(f, "deletion"),
            Self::Complex => write!(f, "complex"),
        }
    }
}

impl DonorTemplate {
    /// Full template sequence (left_arm + correction + right_arm).
    #[must_use]
    pub fn full_sequence(&self) -> Strand {
        let mut bases = Vec::with_capacity(self.total_length);
        bases.extend_from_slice(&self.left_arm.bases);
        bases.extend_from_slice(&self.correction.bases);
        bases.extend_from_slice(&self.right_arm.bases);
        Strand::new(bases)
    }
}

impl fmt::Display for DonorTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DonorTemplate: [{}bp arm]—[{}bp {}]—[{}bp arm] total={}bp",
            self.left_arm.len(),
            self.correction.len(),
            self.mutation_type,
            self.right_arm.len(),
            self.total_length,
        )
    }
}

/// Design an HDR donor template to correct a mutation.
///
/// Takes wild-type and mutant sequences, identifies the mutation region,
/// and constructs a donor template with homology arms of the specified length.
pub fn design_donor_template(
    wild_type: &Strand,
    mutant: &Strand,
    cut_site: usize,
    arm_length: usize,
) -> Result<DonorTemplate> {
    if wild_type.is_empty() || mutant.is_empty() {
        return Err(DnaError::SyntaxError(0, "empty sequence".into()));
    }
    if cut_site >= wild_type.len() {
        return Err(DnaError::IndexOutOfBounds(cut_site, wild_type.len()));
    }

    // Find mutation boundaries by comparing wild-type and mutant
    let (mut_start, mut_end_wt, mut_end_mut) = find_mutation_region(wild_type, mutant);

    // Classify mutation type
    let wt_region_len = mut_end_wt - mut_start;
    let mut_region_len = mut_end_mut - mut_start;
    let mutation_type = if wt_region_len == 1 && mut_region_len == 1 {
        MutationType::Snv
    } else if wt_region_len > mut_region_len {
        MutationType::Insertion // WT has more bases → mutant has a deletion → we insert
    } else if wt_region_len < mut_region_len {
        MutationType::Deletion // WT has fewer bases → mutant has an insertion → we delete
    } else {
        MutationType::Complex
    };

    // Build homology arms from wild-type sequence around the cut site
    let left_start = cut_site.saturating_sub(arm_length);
    let left_end = mut_start.min(cut_site);
    let right_start = mut_end_wt.max(cut_site);
    let right_end = (cut_site + arm_length).min(wild_type.len());

    let left_arm = Strand::new(wild_type.bases[left_start..left_end].to_vec());
    let correction = Strand::new(wild_type.bases[mut_start..mut_end_wt].to_vec());
    let right_arm = Strand::new(wild_type.bases[right_start..right_end].to_vec());

    let total_length = left_arm.len() + correction.len() + right_arm.len();

    Ok(DonorTemplate {
        left_arm,
        correction,
        right_arm,
        total_length,
        mutation_type,
    })
}

/// Find the region where wild-type and mutant differ.
/// Returns (start, end_in_wt, end_in_mutant).
fn find_mutation_region(wt: &Strand, mut_seq: &Strand) -> (usize, usize, usize) {
    let wt_len = wt.len();
    let mut_len = mut_seq.len();
    let min_len = wt_len.min(mut_len);

    // Find first mismatch from left
    let mut start = 0;
    while start < min_len && wt.bases[start] == mut_seq.bases[start] {
        start += 1;
    }

    // Find first mismatch from right
    let mut end_wt = wt_len;
    let mut end_mut = mut_len;
    while end_wt > start && end_mut > start && wt.bases[end_wt - 1] == mut_seq.bases[end_mut - 1] {
        end_wt -= 1;
        end_mut -= 1;
    }

    // If sequences are identical, return a zero-width region at start
    if start >= end_wt.min(end_mut) {
        (start, start, start)
    } else {
        (start, end_wt, end_mut)
    }
}

// ---------------------------------------------------------------------------
// Codon Optimization
// ---------------------------------------------------------------------------

/// Human codon usage frequencies (fraction of usage per amino acid).
/// Based on GenBank CDS data for Homo sapiens.
struct HumanCodonUsage;

impl HumanCodonUsage {
    /// Get the frequency score for a codon in human cells (0.0-1.0).
    /// Higher = more frequently used = better expression.
    fn frequency(codon: &crate::types::Codon) -> f64 {
        use Nucleotide::{A, C, G, T};
        // Simplified human codon preference table (most-preferred codon per AA = 1.0)
        match (codon.0, codon.1, codon.2) {
            // Phe: TTC preferred over TTT
            (T, T, C) => 1.0,
            (T, T, T) => 0.46,
            // Leu: CTG strongly preferred
            (C, T, G) => 1.0,
            (C, T, C) => 0.60,
            (T, T, G) => 0.39,
            (C, T, A) => 0.22,
            (C, T, T) => 0.40,
            (T, T, A) => 0.23,
            // Ile: ATC preferred
            (A, T, C) => 1.0,
            (A, T, T) => 0.52,
            (A, T, A) => 0.25,
            // Val: GTG preferred
            (G, T, G) => 1.0,
            (G, T, C) => 0.66,
            (G, T, T) => 0.40,
            (G, T, A) => 0.28,
            // Ser: AGC preferred
            (A, G, C) => 1.0,
            (T, C, C) => 0.90,
            (T, C, T) => 0.76,
            (A, G, T) => 0.61,
            (T, C, A) => 0.61,
            (T, C, G) => 0.28,
            // Pro: CCC preferred
            (C, C, C) => 1.0,
            (C, C, A) => 0.85,
            (C, C, T) => 0.89,
            (C, C, G) => 0.44,
            // Thr: ACC preferred
            (A, C, C) => 1.0,
            (A, C, A) => 0.76,
            (A, C, T) => 0.66,
            (A, C, G) => 0.36,
            // Ala: GCC preferred
            (G, C, C) => 1.0,
            (G, C, T) => 0.72,
            (G, C, A) => 0.65,
            (G, C, G) => 0.37,
            // Tyr: TAC preferred
            (T, A, C) => 1.0,
            (T, A, T) => 0.56,
            // His: CAC preferred
            (C, A, C) => 1.0,
            (C, A, T) => 0.58,
            // Gln: CAG preferred
            (C, A, G) => 1.0,
            (C, A, A) => 0.35,
            // Asn: AAC preferred
            (A, A, C) => 1.0,
            (A, A, T) => 0.47,
            // Lys: AAG preferred
            (A, A, G) => 1.0,
            (A, A, A) => 0.43,
            // Asp: GAC preferred
            (G, A, C) => 1.0,
            (G, A, T) => 0.56,
            // Glu: GAG preferred
            (G, A, G) => 1.0,
            (G, A, A) => 0.42,
            // Cys: TGC preferred
            (T, G, C) => 1.0,
            (T, G, T) => 0.55,
            // Trp: TGG only codon
            (T, G, G) => 1.0,
            // Arg: CGG slightly preferred, but complex usage
            (A, G, G) => 1.0,
            (C, G, C) => 0.90,
            (C, G, G) => 0.82,
            (A, G, A) => 0.76,
            (C, G, T) => 0.38,
            (C, G, A) => 0.50,
            // Gly: GGC preferred
            (G, G, C) => 1.0,
            (G, G, A) => 0.74,
            (G, G, G) => 0.59,
            (G, G, T) => 0.48,
            // Met: ATG only codon
            (A, T, G) => 1.0,
            // Stop codons
            (T, A, A) | (T, A, G) | (T, G, A) => 0.0,
        }
    }
}

/// Codon Adaptation Index (CAI) for a coding sequence in human cells.
///
/// Returns a score from 0.0 to 1.0 where 1.0 means every codon is the
/// most-preferred human codon.
pub fn codon_adaptation_index(coding_seq: &Strand) -> Result<f64> {
    if coding_seq.len() < 3 {
        return Err(DnaError::IncompleteCodon(coding_seq.len()));
    }

    let codons = coding_seq.codons()?;
    if codons.is_empty() {
        return Ok(0.0);
    }

    // Geometric mean of relative adaptiveness
    let mut log_sum = 0.0f64;
    let mut count = 0usize;

    for codon in &codons {
        let freq = HumanCodonUsage::frequency(codon);
        if freq > 0.0 {
            log_sum += freq.ln();
            count += 1;
        }
    }

    if count == 0 {
        return Ok(0.0);
    }

    Ok((log_sum / count as f64).exp())
}

/// Optimize a coding sequence for human expression by replacing codons
/// with the most-preferred human synonymous codons.
///
/// Preserves the amino acid sequence exactly.
pub fn codon_optimize(coding_seq: &Strand) -> Result<Strand> {
    let table = CodonTable::standard();
    let codons = coding_seq.codons()?;
    let mut optimized_bases = Vec::with_capacity(coding_seq.len());

    for codon in &codons {
        let aa = table.translate(codon);
        let synonymous = table.codons_for(aa);

        // Pick the codon with highest human usage frequency
        let best = synonymous
            .iter()
            .max_by(|a, b| {
                HumanCodonUsage::frequency(a)
                    .partial_cmp(&HumanCodonUsage::frequency(b))
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
            .unwrap_or(codon);

        optimized_bases.push(best.0);
        optimized_bases.push(best.1);
        optimized_bases.push(best.2);
    }

    Ok(Strand::new(optimized_bases))
}

/// Result of codon optimization comparison.
#[derive(Debug)]
pub struct OptimizationResult {
    /// Original CAI score.
    pub original_cai: f64,
    /// Optimized CAI score.
    pub optimized_cai: f64,
    /// Improvement factor.
    pub improvement: f64,
    /// Number of codons changed.
    pub codons_changed: usize,
    /// Total codons.
    pub total_codons: usize,
    /// The optimized sequence.
    pub optimized: Strand,
}

impl fmt::Display for OptimizationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CAI: {:.3} → {:.3} ({:+.1}%) | {}/{} codons changed",
            self.original_cai,
            self.optimized_cai,
            (self.improvement - 1.0) * 100.0,
            self.codons_changed,
            self.total_codons,
        )
    }
}

/// Optimize and compare: returns both original and optimized CAI scores.
pub fn optimize_and_compare(coding_seq: &Strand) -> Result<OptimizationResult> {
    let original_cai = codon_adaptation_index(coding_seq)?;
    let optimized = codon_optimize(coding_seq)?;
    let optimized_cai = codon_adaptation_index(&optimized)?;

    let original_codons = coding_seq.codons()?;
    let optimized_codons = optimized.codons()?;
    let codons_changed = original_codons
        .iter()
        .zip(optimized_codons.iter())
        .filter(|(a, b)| a != b)
        .count();

    let improvement = if original_cai > 0.0 {
        optimized_cai / original_cai
    } else {
        1.0
    };

    Ok(OptimizationResult {
        original_cai,
        optimized_cai,
        improvement,
        codons_changed,
        total_codons: original_codons.len(),
        optimized,
    })
}

// ---------------------------------------------------------------------------
// Delivery Vector Payload Check
// ---------------------------------------------------------------------------

/// Delivery vector type with packaging constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryVector {
    /// Adeno-Associated Virus (~4.7 kb payload)
    Aav,
    /// Adenovirus (~36 kb payload)
    Adenovirus,
    /// Lentivirus (~8 kb payload)
    Lentivirus,
    /// Lipid Nanoparticle (mRNA, ~15 kb practical limit)
    Lnp,
}

impl DeliveryVector {
    /// Maximum payload in base pairs.
    #[must_use]
    pub fn max_payload_bp(&self) -> usize {
        match self {
            Self::Aav => 4_700,
            Self::Adenovirus => 36_000,
            Self::Lentivirus => 8_000,
            Self::Lnp => 15_000,
        }
    }

    /// Name of the vector.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Aav => "AAV",
            Self::Adenovirus => "Adenovirus",
            Self::Lentivirus => "Lentivirus",
            Self::Lnp => "LNP",
        }
    }
}

impl fmt::Display for DeliveryVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (max {}bp)", self.name(), self.max_payload_bp())
    }
}

/// Result of payload feasibility check.
#[derive(Debug)]
pub struct PayloadCheck {
    /// The delivery vector assessed.
    pub vector: DeliveryVector,
    /// Cargo size in base pairs.
    pub cargo_size: usize,
    /// Maximum payload capacity.
    pub max_capacity: usize,
    /// Remaining capacity (negative = won't fit).
    pub remaining: i64,
    /// Whether the cargo fits.
    pub fits: bool,
    /// Utilization percentage.
    pub utilization: f64,
}

impl fmt::Display for PayloadCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.fits { "FITS" } else { "EXCEEDS" };
        write!(
            f,
            "{}: {}bp / {}bp ({:.1}% utilization) [{}] remaining={}bp",
            self.vector.name(),
            self.cargo_size,
            self.max_capacity,
            self.utilization * 100.0,
            status,
            self.remaining,
        )
    }
}

/// Check whether a cargo sequence fits in a delivery vector.
///
/// Accounts for the cargo sequence plus typical regulatory elements
/// (promoter ~500bp, polyA ~250bp, ITRs ~290bp for AAV).
pub fn payload_check(cargo: &Strand, vector: DeliveryVector) -> PayloadCheck {
    let overhead = match vector {
        DeliveryVector::Aav => 290 + 500 + 250, // ITRs + promoter + polyA
        DeliveryVector::Adenovirus => 500 + 250,
        DeliveryVector::Lentivirus => 600 + 500 + 250, // LTRs + promoter + polyA
        DeliveryVector::Lnp => 100 + 250,              // 5'cap/UTR + polyA
    };

    let total_cargo = cargo.len() + overhead;
    let max_capacity = vector.max_payload_bp();
    let remaining = max_capacity as i64 - total_cargo as i64;

    PayloadCheck {
        vector,
        cargo_size: total_cargo,
        max_capacity,
        remaining,
        fits: remaining >= 0,
        utilization: total_cargo as f64 / max_capacity as f64,
    }
}

/// Check cargo against all available vectors, return sorted by fit.
pub fn payload_check_all(cargo: &Strand) -> Vec<PayloadCheck> {
    let vectors = [
        DeliveryVector::Aav,
        DeliveryVector::Lentivirus,
        DeliveryVector::Lnp,
        DeliveryVector::Adenovirus,
    ];

    let mut results: Vec<PayloadCheck> = vectors.iter().map(|v| payload_check(cargo, *v)).collect();

    // Sort: fitting vectors first (by utilization), then non-fitting
    results.sort_by(|a, b| match (a.fits, b.fits) {
        (true, false) => core::cmp::Ordering::Less,
        (false, true) => core::cmp::Ordering::Greater,
        _ => a
            .utilization
            .partial_cmp(&b.utilization)
            .unwrap_or(core::cmp::Ordering::Equal),
    });

    results
}

// ---------------------------------------------------------------------------
// Variant Impact Prediction
// ---------------------------------------------------------------------------

/// Predicted impact of a mutation on protein function.
#[derive(Debug, Clone)]
pub struct VariantImpact {
    /// Position in the coding sequence.
    pub position: usize,
    /// Original codon.
    pub original_codon: String,
    /// Mutant codon.
    pub mutant_codon: String,
    /// Original amino acid.
    pub original_aa: AminoAcid,
    /// Mutant amino acid.
    pub mutant_aa: AminoAcid,
    /// Whether the mutation is synonymous (no AA change).
    pub synonymous: bool,
    /// Impact classification.
    pub impact: ImpactLevel,
    /// Property change (if non-synonymous).
    pub property_change: Option<String>,
}

/// Impact severity classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactLevel {
    /// No amino acid change.
    Synonymous,
    /// Conservative: similar chemical properties.
    Conservative,
    /// Non-conservative: different chemical properties.
    NonConservative,
    /// Nonsense: introduces a premature stop codon.
    Nonsense,
    /// Frameshift: changes reading frame (indel).
    Frameshift,
}

impl fmt::Display for ImpactLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Synonymous => write!(f, "synonymous"),
            Self::Conservative => write!(f, "conservative"),
            Self::NonConservative => write!(f, "non-conservative"),
            Self::Nonsense => write!(f, "nonsense"),
            Self::Frameshift => write!(f, "frameshift"),
        }
    }
}

impl fmt::Display for VariantImpact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pos={} {} ({})→{} ({}) [{}]",
            self.position,
            self.original_codon,
            self.original_aa.abbrev(),
            self.mutant_codon,
            self.mutant_aa.abbrev(),
            self.impact,
        )?;
        if let Some(change) = &self.property_change {
            write!(f, " property: {change}")?;
        }
        Ok(())
    }
}

/// Predict the impact of a point mutation at a given position in a coding sequence.
pub fn predict_variant_impact(
    coding_seq: &Strand,
    position: usize,
    new_base: Nucleotide,
) -> Result<VariantImpact> {
    if position >= coding_seq.len() {
        return Err(DnaError::IndexOutOfBounds(position, coding_seq.len()));
    }

    let table = CodonTable::standard();

    // Find which codon this position belongs to
    let codon_idx = position / 3;
    let codon_start = codon_idx * 3;
    if codon_start + 3 > coding_seq.len() {
        return Err(DnaError::IncompleteCodon(coding_seq.len()));
    }

    let original_codon = crate::types::Codon(
        coding_seq.bases[codon_start],
        coding_seq.bases[codon_start + 1],
        coding_seq.bases[codon_start + 2],
    );

    let mut mutant_bases = [
        coding_seq.bases[codon_start],
        coding_seq.bases[codon_start + 1],
        coding_seq.bases[codon_start + 2],
    ];
    mutant_bases[position - codon_start] = new_base;
    let mutant_codon = crate::types::Codon(mutant_bases[0], mutant_bases[1], mutant_bases[2]);

    let original_aa = table.translate(&original_codon);
    let mutant_aa = table.translate(&mutant_codon);

    let synonymous = original_aa == mutant_aa;
    let impact = if synonymous {
        ImpactLevel::Synonymous
    } else if mutant_aa == AminoAcid::Stop {
        ImpactLevel::Nonsense
    } else if original_aa.property() == mutant_aa.property() {
        ImpactLevel::Conservative
    } else {
        ImpactLevel::NonConservative
    };

    let property_change = if !synonymous && mutant_aa != AminoAcid::Stop {
        Some(format!(
            "{} → {}",
            original_aa.property(),
            mutant_aa.property()
        ))
    } else {
        None
    };

    let orig_str = format!(
        "{}{}{}",
        original_codon.0.as_char(),
        original_codon.1.as_char(),
        original_codon.2.as_char()
    );
    let mut_str = format!(
        "{}{}{}",
        mutant_codon.0.as_char(),
        mutant_codon.1.as_char(),
        mutant_codon.2.as_char()
    );

    Ok(VariantImpact {
        position,
        original_codon: orig_str,
        mutant_codon: mut_str,
        original_aa,
        mutant_aa,
        synonymous,
        impact,
        property_change,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pam_ngg_detection() {
        // 20 A's then TGG = 23 bases. PAM at position 20 (T=N, G, G).
        let seq = Strand::parse("AAAAAAAAAAAAAAAAAAAATGG").unwrap();
        assert_eq!(seq.len(), 23);
        assert!(PamType::Ngg.matches_at(&seq.bases, 20));
    }

    #[test]
    fn guide_design_finds_candidates() {
        // 30bp with NGG PAM at position 20
        let target = Strand::parse("ATGCGATCGATCGATCGATCAGGCGATCGA").unwrap();
        let guides = design_guides(&target, PamType::Ngg);
        // Should find at least one guide
        assert!(!guides.is_empty());
    }

    #[test]
    fn guide_gc_content_reasonable() {
        let target = Strand::parse("ATGCGATCGATCGATCGATCAGGCGATCGATCGATCGATC").unwrap();
        let guides = design_guides(&target, PamType::Ngg);
        for g in &guides {
            assert!((0.0..=1.0).contains(&g.gc_content));
            assert!((0.0..=1.0).contains(&g.quality_score()));
        }
    }

    #[test]
    fn off_target_finds_exact_match() {
        let guide_seq = Strand::parse("ATGCGATCGATCGATCGATC").unwrap();
        let guide = GuideRna {
            protospacer: guide_seq,
            position: 0,
            sense_strand: true,
            pam: PamType::Ngg,
            gc_content: 0.5,
            on_target_score: 0.8,
            self_comp: 0.2,
            cut_site: 17,
        };
        // Reference contains the guide + PAM
        let reference = Strand::parse("ATGCGATCGATCGATCGATCTGG").unwrap();
        let hits = off_target_scan(&guide, &reference, 3);
        assert!(!hits.is_empty());
        assert_eq!(hits[0].mismatches, 0);
    }

    #[test]
    fn donor_template_snv() {
        let wt = Strand::parse("ATGCGATCGATCGATCGA").unwrap();
        let mut mutant = wt.clone();
        mutant.bases[9] = Nucleotide::T; // point mutation at pos 9: A→T
        let template = design_donor_template(&wt, &mutant, 9, 5).unwrap();
        assert_eq!(template.mutation_type, MutationType::Snv);
        assert!(template.total_length > 0);
    }

    #[test]
    fn codon_optimize_preserves_protein() {
        let seq = Strand::parse("ATGTTTATCGAA").unwrap(); // Met-Phe-Ile-Glu
        let table = CodonTable::standard();
        let original_protein = ops::translate(&seq, &table).unwrap();

        let optimized = codon_optimize(&seq).unwrap();
        let optimized_protein = ops::translate(&optimized, &table).unwrap();

        assert_eq!(original_protein, optimized_protein);
    }

    #[test]
    fn cai_score_in_range() {
        let seq = Strand::parse("ATGTTCATCGAG").unwrap(); // human-preferred codons
        let cai = codon_adaptation_index(&seq).unwrap();
        assert!((0.0..=1.0).contains(&cai));
    }

    #[test]
    fn payload_aav_small_gene() {
        let cargo = Strand::new(vec![Nucleotide::A; 1500]); // 1.5kb gene
        let check = payload_check(&cargo, DeliveryVector::Aav);
        assert!(check.fits);
    }

    #[test]
    fn payload_aav_large_gene() {
        let cargo = Strand::new(vec![Nucleotide::A; 5000]); // 5kb gene
        let check = payload_check(&cargo, DeliveryVector::Aav);
        assert!(!check.fits); // Exceeds AAV capacity
    }

    #[test]
    fn variant_impact_synonymous() {
        // AAA (Lys) → AAG (Lys) = synonymous
        let seq = Strand::parse("AAA").unwrap();
        let impact = predict_variant_impact(&seq, 2, Nucleotide::G).unwrap();
        assert!(impact.synonymous);
        assert_eq!(impact.impact, ImpactLevel::Synonymous);
    }

    #[test]
    fn variant_impact_nonsense() {
        // AAA (Lys) → TAA (Stop) = nonsense
        let seq = Strand::parse("AAA").unwrap();
        let impact = predict_variant_impact(&seq, 0, Nucleotide::T).unwrap();
        assert!(!impact.synonymous);
        assert_eq!(impact.impact, ImpactLevel::Nonsense);
    }

    #[test]
    fn variant_impact_nonconservative() {
        // GAA (Glu, acidic) → AAA (Lys, basic)
        let seq = Strand::parse("GAA").unwrap();
        let impact = predict_variant_impact(&seq, 0, Nucleotide::A).unwrap();
        assert_eq!(impact.impact, ImpactLevel::NonConservative);
        assert!(impact.property_change.is_some());
    }

    #[test]
    fn find_mutation_region_snv() {
        let wt = Strand::parse("ATGCGA").unwrap();
        let mut mt = wt.clone();
        mt.bases[3] = Nucleotide::A; // C→A at pos 3
        let (start, end_wt, end_mt) = find_mutation_region(&wt, &mt);
        assert_eq!(start, 3);
        assert_eq!(end_wt, 4);
        assert_eq!(end_mt, 4);
    }

    #[test]
    fn optimize_and_compare_improves_cai() {
        // Suboptimal human codons
        let seq = Strand::parse("ATGTTTATAGATGAA").unwrap();
        let result = optimize_and_compare(&seq).unwrap();
        assert!(result.optimized_cai >= result.original_cai);
        assert!(result.codons_changed > 0);
    }
}
