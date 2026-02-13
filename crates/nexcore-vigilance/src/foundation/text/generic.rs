//! # Generic Text Processing
//!
//! General-purpose text manipulation utilities.

use once_cell::sync::Lazy;
use regex::Regex;

/// Compiled whitespace normalization regex (avoids per-call compilation).
static WHITESPACE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\s+").unwrap_or_else(|_| unreachable!()));

/// Compiled code block extraction regex.
static CODE_BLOCK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"```(\w*)\n([\s\S]*?)```").unwrap_or_else(|_| unreachable!()));

/// Normalize whitespace in a string
///
/// Replaces multiple consecutive whitespace characters with a single space.
#[must_use]
pub fn normalize_whitespace(input: &str) -> String {
    WHITESPACE_RE.replace_all(input.trim(), " ").to_string()
}

/// Count words in a string
#[must_use]
pub fn word_count(input: &str) -> usize {
    input.split_whitespace().count()
}

/// Extract fenced code blocks from markdown
#[must_use]
pub fn extract_code_blocks(content: &str) -> Vec<CodeBlock> {
    CODE_BLOCK_RE
        .captures_iter(content)
        .map(|cap| CodeBlock {
            language: cap.get(1).map_or(String::new(), |m| m.as_str().to_string()),
            content: cap.get(2).map_or(String::new(), |m| m.as_str().to_string()),
        })
        .collect()
}

/// A fenced code block from markdown
#[derive(Debug, Clone)]
pub struct CodeBlock {
    /// Language identifier (e.g., "rust", "python")
    pub language: String,
    /// Content of the code block
    pub content: String,
}

/// Extract YAML frontmatter from markdown content
#[must_use]
pub fn extract_frontmatter(content: &str) -> Option<String> {
    if !content.starts_with("---") {
        return None;
    }

    let rest = &content[3..];
    let end_idx = rest.find("\n---")?;
    Some(rest[..end_idx].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("  hello   world  "), "hello world");
        assert_eq!(normalize_whitespace("a\n\nb"), "a b");
    }

    #[test]
    fn test_word_count() {
        assert_eq!(word_count("hello world"), 2);
        assert_eq!(word_count("one"), 1);
        assert_eq!(word_count(""), 0);
    }

    #[test]
    fn test_extract_code_blocks() {
        let content = "text\n```rust\nfn main() {}\n```\nmore";
        let blocks = extract_code_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, "rust");
        assert!(blocks[0].content.contains("fn main()"));
    }

    #[test]
    fn test_extract_frontmatter() {
        let content = "---\nname: test\n---\n# Content";
        let fm = extract_frontmatter(content).unwrap();
        assert!(fm.contains("name: test"));
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# Just a heading";
        assert!(extract_frontmatter(content).is_none());
    }
}
