//! Inline text utilities for PVDSL.
//!
//! Provides Levenshtein edit distance without external dependencies.

/// Levenshtein edit distance result.
pub struct LevenshteinResult {
    /// Raw edit distance (number of operations).
    pub distance: usize,
    /// Similarity ratio [0.0, 1.0] where 1.0 = identical.
    pub similarity: f64,
}

/// Compute Levenshtein distance with full result including similarity ratio.
#[must_use]
pub fn levenshtein(source: &str, target: &str) -> LevenshteinResult {
    let source_chars: Vec<char> = source.chars().collect();
    let target_chars: Vec<char> = target.chars().collect();
    let m = source_chars.len();
    let n = target_chars.len();

    if m == 0 {
        return LevenshteinResult {
            distance: n,
            similarity: if n == 0 { 1.0 } else { 0.0 },
        };
    }
    if n == 0 {
        return LevenshteinResult {
            distance: m,
            similarity: 0.0,
        };
    }

    // Optimize: iterate over shorter dimension
    let (short, long, short_len, long_len) = if m <= n {
        (&source_chars, &target_chars, m, n)
    } else {
        (&target_chars, &source_chars, n, m)
    };

    let mut prev_row: Vec<usize> = (0..=short_len).collect();
    let mut curr_row = vec![0usize; short_len + 1];

    for j in 1..=long_len {
        curr_row[0] = j;
        for i in 1..=short_len {
            let cost = if short[i - 1] == long[j - 1] { 0 } else { 1 };
            curr_row[i] = (prev_row[i] + 1)
                .min(curr_row[i - 1] + 1)
                .min(prev_row[i - 1] + cost);
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    let distance = prev_row[short_len];
    let max_len = m.max(n);
    let similarity = if max_len == 0 {
        1.0
    } else {
        1.0 - distance as f64 / max_len as f64
    };

    LevenshteinResult {
        distance,
        similarity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_basic() {
        let result = levenshtein("kitten", "sitting");
        assert_eq!(result.distance, 3);
    }

    #[test]
    fn test_levenshtein_identical() {
        let result = levenshtein("abc", "abc");
        assert_eq!(result.distance, 0);
        assert!((result.similarity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_levenshtein_empty() {
        let result = levenshtein("", "abc");
        assert_eq!(result.distance, 3);
        assert!((result.similarity - 0.0).abs() < f64::EPSILON);
    }
}
