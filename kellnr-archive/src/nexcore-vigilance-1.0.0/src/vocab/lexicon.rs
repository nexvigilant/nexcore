//! Lexicon management for domain vocabularies.
//!
//! Provides CRUD operations for vocabulary entries, search functionality,
//! and export capabilities for lexicon documentation.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::domain::VocabDomain;
use super::error::{VocabError, VocabResult};
use super::tier::VocabTier;

/// A vocabulary entry in a lexicon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexiconEntry {
    /// The term (lowercase).
    pub term: String,
    /// Definition/meaning.
    pub definition: String,
    /// Vocabulary tier.
    pub tier: VocabTier,
    /// Domain (if Tier 3).
    pub domain: Option<VocabDomain>,
    /// Example usage (optional).
    pub example: Option<String>,
    /// Related terms.
    pub related: Vec<String>,
}

impl LexiconEntry {
    /// Create a new lexicon entry.
    #[must_use]
    pub fn new(term: impl Into<String>, definition: impl Into<String>, tier: VocabTier) -> Self {
        Self {
            term: term.into().to_lowercase(),
            definition: definition.into(),
            tier,
            domain: None,
            example: None,
            related: Vec::new(),
        }
    }

    /// Set the domain.
    #[must_use]
    pub fn with_domain(mut self, domain: VocabDomain) -> Self {
        self.domain = Some(domain);
        self.tier = VocabTier::DomainSpecific;
        self
    }

    /// Set an example.
    #[must_use]
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.example = Some(example.into());
        self
    }

    /// Add related terms.
    #[must_use]
    pub fn with_related(mut self, terms: Vec<String>) -> Self {
        self.related = terms;
        self
    }
}

/// A domain lexicon containing vocabulary entries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Lexicon {
    /// Domain this lexicon covers (None = general).
    pub domain: Option<VocabDomain>,
    /// Entries indexed by term.
    entries: HashMap<String, LexiconEntry>,
}

impl Lexicon {
    /// Create a new empty lexicon. O(1)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a lexicon for a specific domain. O(1)
    #[must_use]
    pub fn for_domain(domain: VocabDomain) -> Self {
        Self {
            domain: Some(domain),
            entries: HashMap::new(),
        }
    }

    /// Number of entries. O(1)
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty. O(1)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Add an entry. O(1)
    pub fn add(&mut self, entry: LexiconEntry) -> VocabResult<()> {
        if entry.term.is_empty() {
            return Err(VocabError::EmptyTerm);
        }

        if self.entries.contains_key(&entry.term) {
            return Err(VocabError::EntryExists(entry.term));
        }

        self.entries.insert(entry.term.clone(), entry);
        Ok(())
    }

    /// Get an entry by term. O(1)
    #[must_use]
    pub fn get(&self, term: &str) -> Option<&LexiconEntry> {
        self.entries.get(&term.to_lowercase())
    }

    /// Remove an entry. O(1)
    pub fn remove(&mut self, term: &str) -> VocabResult<LexiconEntry> {
        let lower = term.to_lowercase();
        self.entries
            .remove(&lower)
            .ok_or_else(|| VocabError::EntryNotFound(term.to_string()))
    }

    /// Update an existing entry. O(1)
    pub fn update(&mut self, entry: LexiconEntry) -> VocabResult<()> {
        if !self.entries.contains_key(&entry.term) {
            return Err(VocabError::EntryNotFound(entry.term));
        }

        self.entries.insert(entry.term.clone(), entry);
        Ok(())
    }

    /// Search entries by partial match. O(n)
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<&LexiconEntry> {
        let lower = query.to_lowercase();
        self.entries
            .values()
            .filter(|e| e.term.contains(&lower) || e.definition.to_lowercase().contains(&lower))
            .collect()
    }

    /// Get all entries for a tier. O(n)
    #[must_use]
    pub fn by_tier(&self, tier: VocabTier) -> Vec<&LexiconEntry> {
        self.entries.values().filter(|e| e.tier == tier).collect()
    }

    /// Get all entries sorted by term. O(n log n)
    #[must_use]
    pub fn all_sorted(&self) -> Vec<&LexiconEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by(|a, b| a.term.cmp(&b.term));
        entries
    }

    /// Export to markdown format. O(n)
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        let title = match &self.domain {
            Some(d) => format!("# {} Lexicon\n\n", d),
            None => "# Vocabulary Lexicon\n\n".to_string(),
        };
        md.push_str(&title);

        for entry in self.all_sorted() {
            md.push_str(&format!("## {}\n\n", entry.term));
            md.push_str(&format!("**Tier:** {}\n\n", entry.tier));
            md.push_str(&format!("{}\n\n", entry.definition));

            if let Some(ex) = &entry.example {
                md.push_str(&format!("**Example:** {}\n\n", ex));
            }

            if !entry.related.is_empty() {
                md.push_str(&format!("**Related:** {}\n\n", entry.related.join(", ")));
            }

            md.push_str("---\n\n");
        }

        md
    }

    /// Export to JSON format.
    pub fn to_json(&self) -> VocabResult<String> {
        serde_json::to_string_pretty(self).map_err(VocabError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry() -> LexiconEntry {
        LexiconEntry::new(
            "validate",
            "To check for correctness",
            VocabTier::CrossDomain,
        )
    }

    #[test]
    fn test_entry_creation() {
        let entry = sample_entry();
        assert_eq!(entry.term, "validate");
        assert_eq!(entry.tier, VocabTier::CrossDomain);
    }

    #[test]
    fn test_entry_with_domain() {
        let entry = LexiconEntry::new(
            "faers",
            "FDA Adverse Event Reporting System",
            VocabTier::Basic,
        )
        .with_domain(VocabDomain::Pharmacovigilance);
        assert_eq!(entry.tier, VocabTier::DomainSpecific);
        assert_eq!(entry.domain, Some(VocabDomain::Pharmacovigilance));
    }

    #[test]
    fn test_lexicon_add_get() {
        let mut lexicon = Lexicon::new();
        let entry = sample_entry();

        assert!(lexicon.add(entry).is_ok());
        assert_eq!(lexicon.len(), 1);

        let found = lexicon.get("validate");
        assert!(found.is_some());
    }

    #[test]
    fn test_lexicon_duplicate_error() {
        let mut lexicon = Lexicon::new();
        let entry1 = sample_entry();
        let entry2 = sample_entry();

        assert!(lexicon.add(entry1).is_ok());
        let result = lexicon.add(entry2);
        assert!(matches!(result, Err(VocabError::EntryExists(_))));
    }

    #[test]
    fn test_lexicon_search() {
        let mut lexicon = Lexicon::new();
        lexicon
            .add(LexiconEntry::new(
                "validate",
                "Check correctness",
                VocabTier::CrossDomain,
            ))
            .ok();
        lexicon
            .add(LexiconEntry::new(
                "validator",
                "One who validates",
                VocabTier::CrossDomain,
            ))
            .ok();
        lexicon
            .add(LexiconEntry::new("check", "To examine", VocabTier::Basic))
            .ok();

        let results = lexicon.search("valid");
        assert_eq!(results.len(), 2);

        let results = lexicon.search("correctness");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_lexicon_by_tier() {
        let mut lexicon = Lexicon::new();
        lexicon
            .add(LexiconEntry::new("the", "Article", VocabTier::Basic))
            .ok();
        lexicon
            .add(LexiconEntry::new(
                "validate",
                "Check",
                VocabTier::CrossDomain,
            ))
            .ok();

        let basic = lexicon.by_tier(VocabTier::Basic);
        assert_eq!(basic.len(), 1);
    }

    #[test]
    fn test_lexicon_markdown_export() {
        let mut lexicon = Lexicon::for_domain(VocabDomain::Pharmacovigilance);
        lexicon
            .add(LexiconEntry::new(
                "icsr",
                "Individual Case Safety Report",
                VocabTier::DomainSpecific,
            ))
            .ok();

        let md = lexicon.to_markdown();
        assert!(md.contains("# Pharmacovigilance Lexicon"));
        assert!(md.contains("## icsr"));
    }
}
