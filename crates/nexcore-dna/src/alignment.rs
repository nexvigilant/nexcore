//! Sequence Alignment: Smith-Waterman (local) and Needleman-Wunsch (global).
//!
//! Implements fundamental bioinformatics algorithms for DNA sequence comparison.

use crate::types::{Nucleotide, Strand};

#[derive(Debug, Clone)]
pub struct AlignmentResult {
    pub score: i32,
    pub aligned_query: String,
    pub aligned_target: String,
    pub identity: f64,
}

pub struct SequenceAligner {
    pub match_score: i32,
    pub mismatch_score: i32,
    pub gap_penalty: i32,
}

impl Default for SequenceAligner {
    fn default() -> Self {
        Self {
            match_score: 2,
            mismatch_score: -1,
            gap_penalty: -2,
        }
    }
}

impl SequenceAligner {
    /// Local alignment using Smith-Waterman algorithm.
    pub fn smith_waterman(&self, query: &Strand, target: &Strand) -> AlignmentResult {
        let m = query.len();
        let n = target.len();

        // Scoring matrix
        let mut h = vec![vec![0i32; n + 1]; m + 1];
        let mut max_score = 0;
        let mut max_pos = (0, 0);

        for i in 1..=m {
            for j in 1..=n {
                let score_match = if query.bases[i - 1] == target.bases[j - 1] {
                    self.match_score
                } else {
                    self.mismatch_score
                };

                let score = (h[i - 1][j - 1] + score_match)
                    .max(h[i - 1][j] + self.gap_penalty)
                    .max(h[i][j - 1] + self.gap_penalty)
                    .max(0);

                h[i][j] = score;

                if score > max_score {
                    max_score = score;
                    max_pos = (i, j);
                }
            }
        }

        // Traceback
        let mut aligned_query = String::new();
        let mut aligned_target = String::new();
        let (mut curr_i, mut curr_j) = max_pos;
        let mut matches = 0;
        let mut total_len = 0;

        while curr_i > 0 && curr_j > 0 && h[curr_i][curr_j] > 0 {
            let score_match = if query.bases[curr_i - 1] == target.bases[curr_j - 1] {
                self.match_score
            } else {
                self.mismatch_score
            };

            if h[curr_i][curr_j] == h[curr_i - 1][curr_j - 1] + score_match {
                aligned_query.push(query.bases[curr_i - 1].as_char());
                aligned_target.push(target.bases[curr_j - 1].as_char());
                if query.bases[curr_i - 1] == target.bases[curr_j - 1] {
                    matches += 1;
                }
                curr_i -= 1;
                curr_j -= 1;
            } else if h[curr_i][curr_j] == h[curr_i - 1][curr_j] + self.gap_penalty {
                aligned_query.push(query.bases[curr_i - 1].as_char());
                aligned_target.push('-');
                curr_i -= 1;
            } else {
                aligned_query.push('-');
                aligned_target.push(target.bases[curr_j - 1].as_char());
                curr_j -= 1;
            }
            total_len += 1;
        }

        AlignmentResult {
            score: max_score,
            aligned_query: aligned_query.chars().rev().collect(),
            aligned_target: aligned_target.chars().rev().collect(),
            identity: if total_len > 0 {
                matches as f64 / total_len as f64
            } else {
                0.0
            },
        }
    }

    /// Global alignment using Needleman-Wunsch algorithm.
    pub fn needleman_wunsch(&self, query: &Strand, target: &Strand) -> AlignmentResult {
        let m = query.len();
        let n = target.len();

        let mut h = vec![vec![0i32; n + 1]; m + 1];

        for i in 0..=m {
            h[i][0] = i as i32 * self.gap_penalty;
        }
        for j in 0..=n {
            h[0][j] = j as i32 * self.gap_penalty;
        }

        for i in 1..=m {
            for j in 1..=n {
                let score_match = if query.bases[i - 1] == target.bases[j - 1] {
                    self.match_score
                } else {
                    self.mismatch_score
                };

                h[i][j] = (h[i - 1][j - 1] + score_match)
                    .max(h[i - 1][j] + self.gap_penalty)
                    .max(h[i][j - 1] + self.gap_penalty);
            }
        }

        // Traceback from bottom-right
        let mut aligned_query = String::new();
        let mut aligned_target = String::new();
        let (mut curr_i, mut curr_j) = (m, n);
        let mut matches = 0;
        let mut total_len = 0;

        while curr_i > 0 || curr_j > 0 {
            if curr_i > 0 && curr_j > 0 {
                let score_match = if query.bases[curr_i - 1] == target.bases[curr_j - 1] {
                    self.match_score
                } else {
                    self.mismatch_score
                };

                if h[curr_i][curr_j] == h[curr_i - 1][curr_j - 1] + score_match {
                    aligned_query.push(query.bases[curr_i - 1].as_char());
                    aligned_target.push(target.bases[curr_j - 1].as_char());
                    if query.bases[curr_i - 1] == target.bases[curr_j - 1] {
                        matches += 1;
                    }
                    curr_i -= 1;
                    curr_j -= 1;
                    total_len += 1;
                    continue;
                }
            }

            if curr_i > 0 && h[curr_i][curr_j] == h[curr_i - 1][curr_j] + self.gap_penalty {
                aligned_query.push(query.bases[curr_i - 1].as_char());
                aligned_target.push('-');
                curr_i -= 1;
            } else {
                aligned_query.push('-');
                aligned_target.push(target.bases[curr_j - 1].as_char());
                curr_j -= 1;
            }
            total_len += 1;
        }

        AlignmentResult {
            score: h[m][n],
            aligned_query: aligned_query.chars().rev().collect(),
            aligned_target: aligned_target.chars().rev().collect(),
            identity: if total_len > 0 {
                matches as f64 / total_len as f64
            } else {
                0.0
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Strand;

    #[test]
    fn test_smith_waterman() {
        let aligner = SequenceAligner::default();
        let query = Strand::parse("GATTACA").unwrap();
        let target = Strand::parse("GCATGCU").unwrap();

        let result = aligner.smith_waterman(&query, &target);
        assert!(result.score > 0);
        println!("SW Aligned Query:  {}", result.aligned_query);
        println!("SW Aligned Target: {}", result.aligned_target);
    }

    #[test]
    fn test_needleman_wunsch() {
        let aligner = SequenceAligner::default();
        let query = Strand::parse("GATTACA").unwrap();
        let target = Strand::parse("GCATGCU").unwrap();

        let result = aligner.needleman_wunsch(&query, &target);
        assert_eq!(result.aligned_query.len(), result.aligned_target.len());
        println!("NW Aligned Query:  {}", result.aligned_query);
        println!("NW Aligned Target: {}", result.aligned_target);
    }
}
