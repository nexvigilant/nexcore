//! KSB Index - domain-based article indexing.

use std::collections::HashMap;
use std::path::Path;

use nexcore_fs::walk::WalkDir;
use rayon::prelude::*;

use super::{KsbArticle, KsbDomain};
use crate::KnowledgeError;

/// KSB Index for fast article lookup.
#[derive(Debug, Default)]
pub struct KsbIndex {
    /// Articles by ID.
    by_id: HashMap<String, KsbArticle>,
    /// Article IDs by domain.
    by_domain: HashMap<KsbDomain, Vec<String>>,
    /// Total article count.
    count: usize,
}

impl KsbIndex {
    /// Create empty index.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Scan directory and build index.
    ///
    /// # Errors
    ///
    /// Returns error if directory cannot be read.
    pub fn scan(path: &Path) -> Result<Self, KnowledgeError> {
        let mut index = Self::new();

        // Collect all SKILL.md paths
        let skill_paths: Vec<_> = WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_name() == "SKILL.md")
            .map(|e| e.path().to_path_buf())
            .collect();

        // Parse articles in parallel
        let articles: Vec<_> = skill_paths
            .par_iter()
            .filter_map(|p| KsbArticle::from_path(p).ok())
            .collect();

        // Build index
        for article in articles {
            let id = article.id.clone();
            let domain = article.domain;

            index.by_id.insert(id.clone(), article);
            index.by_domain.entry(domain).or_default().push(id);
            index.count += 1;
        }

        Ok(index)
    }

    /// Get article by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&KsbArticle> {
        self.by_id.get(id)
    }

    /// Get all articles in a domain.
    #[must_use]
    pub fn get_domain(&self, domain: KsbDomain) -> Vec<&KsbArticle> {
        self.by_domain
            .get(&domain)
            .map(|ids| ids.iter().filter_map(|id| self.by_id.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get article count per domain.
    #[must_use]
    pub fn domain_counts(&self) -> HashMap<KsbDomain, usize> {
        self.by_domain
            .iter()
            .map(|(d, ids)| (*d, ids.len()))
            .collect()
    }

    /// Total article count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if index is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get all article IDs.
    #[must_use]
    pub fn ids(&self) -> Vec<&str> {
        self.by_id.keys().map(String::as_str).collect()
    }

    /// Get all articles.
    #[must_use]
    pub fn articles(&self) -> Vec<&KsbArticle> {
        self.by_id.values().collect()
    }
}

/// Get default KSB path.
#[must_use]
pub fn default_ksb_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(std::path::PathBuf::from(format!(
        "{}/nexcore/knowledge/pv-ksb",
        home
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_index() {
        let index = KsbIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }
}
