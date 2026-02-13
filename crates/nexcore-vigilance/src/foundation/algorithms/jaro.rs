//! # Jaro and Jaro-Winkler Similarity
//!
//! High-performance implementation of Jaro and Jaro-Winkler string similarity algorithms.
//! achieving significant speedups over pure Python implementations.

use std::cmp::{max, min};

/// Compute Jaro similarity between two strings.
///
/// Similarity is a float between 0.0 (no match) and 1.0 (exact match).
#[must_use]
pub fn jaro_similarity(a: &str, b: &str) -> f64 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 && b_len == 0 {
        return 1.0;
    }
    if a_len == 0 || b_len == 0 {
        return 0.0;
    }

    let match_distance = (max(a_len, b_len) / 2).saturating_sub(1);

    let mut a_matches = vec![false; a_len];
    let mut b_matches = vec![false; b_len];

    let mut matches = 0;
    for i in 0..a_len {
        let start = i.saturating_sub(match_distance);
        let end = min(i + match_distance + 1, b_len);

        for j in start..end {
            if !b_matches[j] && a_chars[i] == b_chars[j] {
                a_matches[i] = true;
                b_matches[j] = true;
                matches += 1;
                break;
            }
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut transpositions = 0;
    let mut k = 0;
    for i in 0..a_len {
        if a_matches[i] {
            while !b_matches[k] {
                k += 1;
            }
            if a_chars[i] != b_chars[k] {
                transpositions += 1;
            }
            k += 1;
        }
    }

    let m = matches as f64;
    (m / a_len as f64 + m / b_len as f64 + (m - (transpositions as f64 / 2.0)) / m) / 3.0
}

/// Compute Jaro-Winkler similarity between two strings.
///
/// Winkler's modification increases the similarity for strings with a common prefix.
#[must_use]
pub fn jaro_winkler_similarity(a: &str, b: &str) -> f64 {
    let jaro = jaro_similarity(a, b);

    if jaro < 0.7 {
        return jaro;
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let prefix_len = a_chars
        .iter()
        .zip(b_chars.iter())
        .take(4) // Jaro-Winkler uses a max prefix of 4
        .take_while(|(c1, c2)| c1 == c2)
        .count();

    // Constant scaling factor for prefix matching (standard is 0.1)
    let p = 0.1;
    jaro + (prefix_len as f64 * p * (1.0 - jaro))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaro_basic() {
        // Standard examples
        assert!((jaro_similarity("MARTHA", "MARHTA") - 0.944).abs() < 0.001);
        assert!((jaro_similarity("DIXON", "DICKSONX") - 0.767).abs() < 0.001);
        assert!((jaro_similarity("JELLYFISH", "SMELLYFISH") - 0.896).abs() < 0.001);
    }

    #[test]
    fn test_jaro_winkler_basic() {
        assert!((jaro_winkler_similarity("MARTHA", "MARHTA") - 0.961).abs() < 0.001);
        assert!((jaro_winkler_similarity("DIXON", "DICKSONX") - 0.813).abs() < 0.001);
    }

    #[test]
    fn test_identical() {
        assert_eq!(jaro_similarity("hello", "hello"), 1.0);
        assert_eq!(jaro_winkler_similarity("hello", "hello"), 1.0);
    }

    #[test]
    fn test_completely_different() {
        assert_eq!(jaro_similarity("abc", "def"), 0.0);
        assert_eq!(jaro_winkler_similarity("abc", "def"), 0.0);
    }
}
