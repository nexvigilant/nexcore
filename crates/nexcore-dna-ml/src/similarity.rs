//! DNA-domain similarity features.
//!
//! After encoding PV feature vectors as DNA strands, compute biological
//! similarity metrics that become additional ML features. These capture
//! structural relationships invisible to raw numeric comparison.

use nexcore_dna::types::Strand;

/// DNA similarity metrics between two encoded feature strands.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DnaSimilarity {
    /// Hamming distance (mismatched bases / total bases).
    pub hamming_distance: f64,
    /// GC content ratio of strand A.
    pub gc_content_a: f64,
    /// GC content ratio of strand B.
    pub gc_content_b: f64,
    /// GC content divergence |gc_a - gc_b|.
    pub gc_divergence: f64,
    /// Longest common subsequence length ratio.
    pub lcs_ratio: f64,
}

/// Compute DNA similarity between two encoded strands.
pub fn compute_similarity(a: &Strand, b: &Strand) -> DnaSimilarity {
    let len = a.bases.len().min(b.bases.len());
    let hamming = if len > 0 {
        let mismatches = a
            .bases
            .iter()
            .zip(b.bases.iter())
            .filter(|(x, y)| x != y)
            .count();
        mismatches as f64 / len as f64
    } else {
        1.0
    };

    let gc_a = gc_content(&a.bases);
    let gc_b = gc_content(&b.bases);

    DnaSimilarity {
        hamming_distance: hamming,
        gc_content_a: gc_a,
        gc_content_b: gc_b,
        gc_divergence: (gc_a - gc_b).abs(),
        lcs_ratio: lcs_ratio(&a.bases, &b.bases),
    }
}

/// Compute GC content (fraction of G+C bases).
fn gc_content(bases: &[nexcore_dna::types::Nucleotide]) -> f64 {
    if bases.is_empty() {
        return 0.0;
    }
    let gc = bases
        .iter()
        .filter(|b| {
            matches!(
                b,
                nexcore_dna::types::Nucleotide::G | nexcore_dna::types::Nucleotide::C
            )
        })
        .count();
    gc as f64 / bases.len() as f64
}

/// Longest common subsequence ratio (LCS length / max strand length).
///
/// Uses O(min(m,n)) space DP.
fn lcs_ratio(a: &[nexcore_dna::types::Nucleotide], b: &[nexcore_dna::types::Nucleotide]) -> f64 {
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return 1.0;
    }

    // Truncate to prevent excessive computation on large strands
    let limit = 512;
    let a = if a.len() > limit { &a[..limit] } else { a };
    let b = if b.len() > limit { &b[..limit] } else { b };

    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let mut prev = vec![0u32; short.len() + 1];
    let mut curr = vec![0u32; short.len() + 1];

    for i in 1..=long.len() {
        for j in 1..=short.len() {
            if long[i - 1] == short[j - 1] {
                curr[j] = prev[j - 1] + 1;
            } else {
                curr[j] = prev[j].max(curr[j - 1]);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
        curr.iter_mut().for_each(|x| *x = 0);
    }

    let lcs_len = *prev.iter().max().unwrap_or(&0);
    lcs_len as f64 / max_len as f64
}

/// Compute pairwise similarity between a sample strand and a set of reference strands.
///
/// Returns the mean similarity metrics across all references — used as
/// additional features for the ML model.
pub fn mean_similarity_features(sample: &Strand, references: &[Strand]) -> Vec<f64> {
    if references.is_empty() {
        return vec![0.0; 5];
    }

    let sims: Vec<DnaSimilarity> = references
        .iter()
        .map(|r| compute_similarity(sample, r))
        .collect();

    let n = sims.len() as f64;
    vec![
        sims.iter().map(|s| s.hamming_distance).sum::<f64>() / n,
        sims.iter().map(|s| s.gc_content_a).sum::<f64>() / n,
        sims.iter().map(|s| s.gc_divergence).sum::<f64>() / n,
        sims.iter().map(|s| s.lcs_ratio).sum::<f64>() / n,
        // Strand length as normalized feature
        sample.bases.len() as f64 / 256.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_dna::storage;

    #[test]
    fn identical_strands_zero_hamming() {
        let data = b"test data";
        let a = storage::encode(data);
        let b = storage::encode(data);
        let sim = compute_similarity(&a, &b);
        assert!((sim.hamming_distance).abs() < f64::EPSILON);
        assert!((sim.gc_divergence).abs() < f64::EPSILON);
    }

    #[test]
    fn different_strands_nonzero_hamming() {
        let a = storage::encode(b"AAAA");
        let b = storage::encode(b"ZZZZ");
        let sim = compute_similarity(&a, &b);
        assert!(sim.hamming_distance > 0.0);
    }

    #[test]
    fn mean_features_correct_length() {
        let s = storage::encode(b"sample");
        let refs = vec![storage::encode(b"ref1"), storage::encode(b"ref2")];
        let feats = mean_similarity_features(&s, &refs);
        assert_eq!(feats.len(), 5);
    }
}
