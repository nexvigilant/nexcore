//! Book metadata following Dublin Core and EPUB/Kindle standards.

use serde::{Deserialize, Serialize};

/// Complete book metadata for EPUB and Kindle publishing.
///
/// Based on Dublin Core Metadata Element Set (DCMES) plus
/// EPUB-specific and KDP-specific extensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookMetadata {
    /// Book title (dc:title). Required.
    pub title: String,
    /// Subtitle. Optional.
    pub subtitle: Option<String>,
    /// Author name(s) (dc:creator). At least one required.
    pub authors: Vec<Author>,
    /// Publisher name (dc:publisher).
    pub publisher: Option<String>,
    /// Language code (dc:language). BCP 47 format, e.g. "en", "en-US".
    pub language: String,
    /// ISBN-13 (with or without hyphens).
    pub isbn: Option<String>,
    /// Book description / blurb (dc:description).
    pub description: Option<String>,
    /// Subject keywords (dc:subject).
    pub subjects: Vec<String>,
    /// Publication date (dc:date). ISO 8601, e.g. "2026-03-28".
    pub date: Option<String>,
    /// Rights statement (dc:rights).
    pub rights: Option<String>,
    /// BISAC category codes for retail classification.
    pub bisac_codes: Vec<String>,
    /// Kindle-specific: series name.
    pub series: Option<String>,
    /// Kindle-specific: series number.
    pub series_number: Option<u32>,
    /// EPUB unique identifier (auto-generated if not provided).
    pub identifier: Option<String>,
}

/// Author with role classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// Display name (e.g. "Matthew Campion").
    pub name: String,
    /// File-as form for sorting (e.g. "Campion, Matthew").
    pub file_as: Option<String>,
    /// MARC relator role. Defaults to "aut" (author).
    pub role: AuthorRole,
}

/// MARC relator codes for contributor roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorRole {
    /// Author (aut)
    Author,
    /// Editor (edt)
    Editor,
    /// Translator (trl)
    Translator,
    /// Illustrator (ill)
    Illustrator,
    /// Foreword author (aui)
    ForewordAuthor,
    /// Contributor (ctb)
    Contributor,
}

impl AuthorRole {
    /// Returns the MARC relator code.
    pub fn marc_code(&self) -> &'static str {
        match self {
            Self::Author => "aut",
            Self::Editor => "edt",
            Self::Translator => "trl",
            Self::Illustrator => "ill",
            Self::ForewordAuthor => "aui",
            Self::Contributor => "ctb",
        }
    }
}

impl Default for AuthorRole {
    fn default() -> Self {
        Self::Author
    }
}

impl BookMetadata {
    /// Create minimal metadata with just title, author, and language.
    pub fn new(
        title: impl Into<String>,
        author_name: impl Into<String>,
        language: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            authors: vec![Author {
                name: author_name.into(),
                file_as: None,
                role: AuthorRole::Author,
            }],
            publisher: None,
            language: language.into(),
            isbn: None,
            description: None,
            subjects: Vec::new(),
            date: None,
            rights: None,
            bisac_codes: Vec::new(),
            series: None,
            series_number: None,
            identifier: None,
        }
    }

    /// Get or generate the unique identifier for the EPUB package.
    pub fn unique_id(&self) -> String {
        if let Some(ref id) = self.identifier {
            return id.clone();
        }
        if let Some(ref isbn) = self.isbn {
            return format!("urn:isbn:{}", isbn.replace('-', ""));
        }
        // Deterministic fallback from title + author
        let seed = format!("{}:{}", self.title, self.primary_author());
        format!("urn:nexvigilant:{:x}", simple_hash(&seed))
    }

    /// Returns the primary author name (first author).
    pub fn primary_author(&self) -> &str {
        self.authors
            .first()
            .map(|a| a.name.as_str())
            .unwrap_or("Unknown Author")
    }

    /// Validate metadata completeness for EPUB generation.
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if self.title.trim().is_empty() {
            issues.push("Title is required".into());
        }
        if self.authors.is_empty() {
            issues.push("At least one author is required".into());
        }
        if self.language.trim().is_empty() {
            issues.push("Language code is required".into());
        }
        if let Some(ref isbn) = self.isbn {
            if !validate_isbn13(isbn) {
                issues.push(format!("Invalid ISBN-13: {isbn}"));
            }
        }
        issues
    }
}

/// Simple deterministic hash for identifier generation.
fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(u64::from(byte));
    }
    hash
}

/// Validate an ISBN-13 checksum.
fn validate_isbn13(isbn: &str) -> bool {
    let digits: Vec<u32> = isbn
        .chars()
        .filter(|c| c.is_ascii_digit())
        .filter_map(|c| c.to_digit(10))
        .collect();
    if digits.len() != 13 {
        return false;
    }
    let sum: u32 = digits
        .iter()
        .enumerate()
        .map(|(i, &d)| if i % 2 == 0 { d } else { d * 3 })
        .sum();
    sum % 10 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_metadata() {
        let meta = BookMetadata::new("Test Book", "Test Author", "en");
        assert_eq!(meta.title, "Test Book");
        assert_eq!(meta.primary_author(), "Test Author");
        assert_eq!(meta.language, "en");
        assert!(meta.validate().is_empty());
    }

    #[test]
    fn test_isbn_validation() {
        // Valid ISBN-13 for "The Art of Computer Programming"
        assert!(validate_isbn13("978-0-201-89683-1"));
        // Invalid checksum
        assert!(!validate_isbn13("978-0-201-89683-0"));
        // Too short
        assert!(!validate_isbn13("978-0-201"));
    }

    #[test]
    fn test_unique_id_from_isbn() {
        let mut meta = BookMetadata::new("Test", "Author", "en");
        meta.isbn = Some("978-0-201-89683-1".into());
        assert_eq!(meta.unique_id(), "urn:isbn:9780201896831");
    }

    #[test]
    fn test_unique_id_deterministic() {
        let meta = BookMetadata::new("My Book", "John Doe", "en");
        let id1 = meta.unique_id();
        let id2 = meta.unique_id();
        assert_eq!(id1, id2);
        assert!(id1.starts_with("urn:nexvigilant:"));
    }

    #[test]
    fn test_validation_catches_empty_title() {
        let meta = BookMetadata::new("", "Author", "en");
        let issues = meta.validate();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("Title"));
    }

    #[test]
    fn test_marc_codes() {
        assert_eq!(AuthorRole::Author.marc_code(), "aut");
        assert_eq!(AuthorRole::Editor.marc_code(), "edt");
        assert_eq!(AuthorRole::Translator.marc_code(), "trl");
    }
}
