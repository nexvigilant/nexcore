//! # Hamming Distance
//!
//! High-performance implementation of Hamming distance for safety-critical string comparison.
//! Hamming distance counts the number of positions at which the corresponding characters are different.
//!
//! Note: Hamming distance is only defined for strings of equal length.

use serde::{Deserialize, Serialize};

/// Result of a Hamming distance calculation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HammingResult {
    /// The number of character mismatches
    pub distance: usize,
    /// Normalized similarity (1.0 - distance / length)
    pub similarity: f64,
}

/// Compute Hamming distance between two strings of equal length.
///
/// Returns `None` if the strings have different lengths.
#[must_use]
pub fn hamming_distance(a: &str, b: &str) -> Option<usize> {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    if a_chars.len() != b_chars.len() {
        return None;
    }

    let distance = a_chars
        .iter()
        .zip(b_chars.iter())
        .filter(|(c1, c2)| c1 != c2)
        .count();

    Some(distance)
}

/// Compute Hamming distance with similarity score.
///
/// Returns `None` if the strings have different lengths.
#[must_use]
pub fn hamming(a: &str, b: &str) -> Option<HammingResult> {
    let dist = hamming_distance(a, b)?;
    let len = a.chars().count();

    let similarity = if len == 0 {
        1.0
    } else {
        1.0 - (dist as f64 / len as f64)
    };

    Some(HammingResult {
        distance: dist,
        similarity: (similarity * 10000.0).round() / 10000.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hamming_identical() {
        assert_eq!(hamming_distance("karolin", "karolin"), Some(0));
    }

    #[test]
    fn test_hamming_different() {
        assert_eq!(hamming_distance("karolin", "kathrin"), Some(3));
        assert_eq!(hamming_distance("karolin", "kerstin"), Some(3));
        assert_eq!(hamming_distance("1011101", "1001001"), Some(2));
    }

    #[test]
    fn test_hamming_different_lengths() {
        assert_eq!(hamming_distance("hello", "world!"), None);
    }

    #[test]
    fn test_hamming_unicode() {
        assert_eq!(hamming_distance("こんにちは", "こんばんわ"), Some(3));
    }
}
