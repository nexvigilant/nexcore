//! Full-text search for KSB articles.

use rayon::prelude::*;
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};

use super::{KsbArticle, KsbDomain, KsbIndex};

/// Search result with relevance score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Article ID.
    pub id: String,
    /// Article title.
    pub title: String,
    /// Domain.
    pub domain: String,
    /// Description snippet.
    pub description: String,
    /// Relevance score (0-100).
    pub score: u32,
    /// Matched terms.
    pub matches: Vec<String>,
}

/// Search articles by query.
///
/// Searches title, description, content, and triggers.
/// Returns results sorted by relevance.
#[must_use]
pub fn search_articles(
    index: &KsbIndex,
    query: &str,
    domain_filter: Option<KsbDomain>,
    limit: usize,
) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

    if query_terms.is_empty() {
        return Vec::new();
    }

    // Build regex for each term
    let patterns: Vec<_> = query_terms
        .iter()
        .filter_map(|term| {
            RegexBuilder::new(&regex::escape(term))
                .case_insensitive(true)
                .build()
                .ok()
        })
        .collect();

    // Get articles to search
    let articles: Vec<&KsbArticle> = match domain_filter {
        Some(domain) => index.get_domain(domain),
        None => index.articles(),
    };

    // Score articles in parallel
    let mut results: Vec<SearchResult> = articles
        .par_iter()
        .filter_map(|article| score_article(article, &patterns, &query_terms))
        .collect();

    // Sort by score descending
    results.sort_by(|a, b| b.score.cmp(&a.score));

    // Limit results
    results.truncate(limit);

    results
}

fn score_article(
    article: &KsbArticle,
    patterns: &[regex::Regex],
    terms: &[&str],
) -> Option<SearchResult> {
    let mut score: u32 = 0;
    let mut matches = Vec::new();

    let title_lower = article.title.to_lowercase();
    let desc_lower = article.description.to_lowercase();
    let content_lower = article.content.to_lowercase();

    for (i, pattern) in patterns.iter().enumerate() {
        let term = terms.get(i).unwrap_or(&"");

        // Title match (highest weight)
        if pattern.is_match(&article.title) {
            score += 50;
            matches.push(format!("title:{}", term));
        }

        // Description match (high weight)
        if pattern.is_match(&article.description) {
            score += 30;
            if !matches.iter().any(|m| m.ends_with(term)) {
                matches.push(format!("description:{}", term));
            }
        }

        // Trigger match (medium weight)
        if article.triggers.iter().any(|t| pattern.is_match(t)) {
            score += 25;
            if !matches.iter().any(|m| m.ends_with(term)) {
                matches.push(format!("trigger:{}", term));
            }
        }

        // Content match (lower weight, count occurrences)
        let content_count = pattern.find_iter(&content_lower).count();
        if content_count > 0 {
            score += (content_count.min(10) * 2) as u32;
            if !matches.iter().any(|m| m.ends_with(term)) {
                matches.push(format!("content:{}", term));
            }
        }

        // ID match (medium weight)
        if pattern.is_match(&article.id) {
            score += 20;
        }
    }

    // Boost for matching all terms
    if matches.len() >= terms.len() {
        score += 20;
    }

    // Exact title match bonus
    if title_lower == terms.join(" ") {
        score += 50;
    }

    // Exact description phrase bonus
    if desc_lower.contains(&terms.join(" ")) {
        score += 15;
    }

    if score == 0 {
        return None;
    }

    // Cap score at 100
    score = score.min(100);

    Some(SearchResult {
        id: article.id.clone(),
        title: article.title.clone(),
        domain: article.domain.code().to_string(),
        description: truncate_description(&article.description, 150),
        score,
        matches,
    })
}

fn truncate_description(desc: &str, max_len: usize) -> String {
    if desc.len() <= max_len {
        desc.to_string()
    } else {
        format!("{}...", &desc[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate_description("short", 10), "short");
        assert_eq!(
            truncate_description("this is a longer string", 10),
            "this is..."
        );
    }
}
