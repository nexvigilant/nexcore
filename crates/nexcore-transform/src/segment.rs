//! Paragraph segmentation: raw text -> Vec<Paragraph>.
//!
//! Splits input text on double-newlines, trims whitespace,
//! filters empties, and assigns sequential indices with word counts.

use serde::{Deserialize, Serialize};

/// A single indexed paragraph from the source text.
///
/// Tier: T2-P | Dominant: sigma (Sequence)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Paragraph {
    /// Zero-based paragraph index.
    pub index: usize,
    /// The raw text content (trimmed).
    pub text: String,
    /// Word count.
    pub word_count: usize,
}

/// The segmented source text with metadata.
///
/// Tier: T2-C | Dominant: sigma (Sequence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceText {
    /// Title of the source document.
    pub title: String,
    /// Segmented paragraphs in order.
    pub paragraphs: Vec<Paragraph>,
    /// Total word count across all paragraphs.
    pub total_words: usize,
}

/// Segment raw text into paragraphs.
///
/// Splits on `\n\n` (double newline), trims each segment,
/// and filters out empty results.
pub fn segment(title: &str, text: &str) -> SourceText {
    let paragraphs: Vec<Paragraph> = text
        .split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .enumerate()
        .map(|(i, s)| {
            let word_count = s.split_whitespace().count();
            Paragraph {
                index: i,
                text: s.to_string(),
                word_count,
            }
        })
        .collect();

    let total_words = paragraphs.iter().map(|p| p.word_count).sum();

    SourceText {
        title: title.to_string(),
        paragraphs,
        total_words,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_segmentation() {
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let result = segment("Test", text);
        assert_eq!(result.paragraphs.len(), 3);
        assert_eq!(result.paragraphs[0].index, 0);
        assert_eq!(result.paragraphs[1].index, 1);
        assert_eq!(result.paragraphs[2].index, 2);
    }

    #[test]
    fn test_empty_text() {
        let result = segment("Empty", "");
        assert!(result.paragraphs.is_empty());
        assert_eq!(result.total_words, 0);
    }

    #[test]
    fn test_single_paragraph() {
        let result = segment("Single", "Just one paragraph here.");
        assert_eq!(result.paragraphs.len(), 1);
        assert_eq!(result.paragraphs[0].word_count, 4);
        assert_eq!(result.total_words, 4);
    }

    #[test]
    fn test_whitespace_trimming() {
        let text = "  First.  \n\n  Second.  \n\n\n\n  Third.  ";
        let result = segment("Trim", text);
        assert_eq!(result.paragraphs.len(), 3);
        assert_eq!(result.paragraphs[0].text, "First.");
        assert_eq!(result.paragraphs[2].text, "Third.");
    }

    #[test]
    fn test_empty_paragraphs_filtered() {
        let text = "Real.\n\n\n\n\n\nAlso real.";
        let result = segment("Filter", text);
        // Multiple \n\n produce empty segments that get filtered
        assert!(result.paragraphs.len() >= 2);
        assert!(result.paragraphs.iter().all(|p| !p.text.is_empty()));
    }

    #[test]
    fn test_word_count_accuracy() {
        let text = "One two three.\n\nFour five.";
        let result = segment("Count", text);
        assert_eq!(result.paragraphs[0].word_count, 3);
        assert_eq!(result.paragraphs[1].word_count, 2);
        assert_eq!(result.total_words, 5);
    }

    #[test]
    fn test_title_preserved() {
        let result = segment("My Title", "Content.");
        assert_eq!(result.title, "My Title");
    }

    #[test]
    fn test_multiline_paragraph_kept_together() {
        let text = "Line one\nLine two\nLine three\n\nSeparate paragraph.";
        let result = segment("Multi", text);
        assert_eq!(result.paragraphs.len(), 2);
        assert!(result.paragraphs[0].text.contains("Line one"));
        assert!(result.paragraphs[0].text.contains("Line three"));
    }
}
