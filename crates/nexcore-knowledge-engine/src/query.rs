//! Query engine over compiled knowledge packs.

use serde::{Deserialize, Serialize};

use crate::compression::token_similarity;
use crate::error::{KnowledgeEngineError, Result};
use crate::knowledge_pack::KnowledgePack;
use crate::store::KnowledgeStore;

/// Query mode.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QueryMode {
    #[default]
    Keyword,
    Concept,
    Domain,
}

/// A single query result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub content: String,
    pub concepts: Vec<String>,
    pub domain: String,
    pub relevance: f64,
    pub fragment_id: crate::KnowledgeId,
}

/// Query response with results and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub results: Vec<QueryResult>,
    pub total_matches: usize,
    pub pack_name: String,
    pub pack_version: u32,
}

/// Result of a query operation, including metadata about how many packs were searched.
#[derive(Debug, Clone)]
pub struct QueryOutcome {
    /// Responses from packs that had matching results.
    pub responses: Vec<QueryResponse>,
    /// Total number of packs that were loaded and searched (regardless of matches).
    pub packs_loaded: usize,
}

/// Query engine.
pub struct QueryEngine {
    store: KnowledgeStore,
}

impl QueryEngine {
    pub fn new(store: KnowledgeStore) -> Self {
        Self { store }
    }

    /// Query a specific pack or all packs.
    pub fn query(
        &self,
        query: &str,
        pack_name: Option<&str>,
        mode: QueryMode,
        domain_filter: Option<&str>,
        limit: usize,
    ) -> Result<QueryOutcome> {
        let packs = if let Some(name) = pack_name {
            vec![self.store.load_latest(name)?]
        } else {
            let indices = self.store.list_packs()?;
            let mut packs = Vec::new();
            for idx in &indices {
                if let Ok(p) = self.store.load_latest(&idx.name) {
                    packs.push(p);
                }
            }
            packs
        };

        if packs.is_empty() {
            return if pack_name.is_some() {
                Err(KnowledgeEngineError::PackNotFound(
                    pack_name.unwrap_or("").to_string(),
                ))
            } else {
                Err(KnowledgeEngineError::EmptyStore)
            };
        }

        let packs_loaded = packs.len();
        let mut responses = Vec::new();
        for pack in &packs {
            let response = self.query_pack(pack, query, &mode, domain_filter, limit);
            if !response.results.is_empty() {
                responses.push(response);
            }
        }

        Ok(QueryOutcome {
            responses,
            packs_loaded,
        })
    }

    /// Query all packs and return a single flat list sorted by global relevance.
    ///
    /// Unlike `query()` which returns per-pack responses, this merges all results
    /// into one ranked list — useful when the caller doesn't care which pack a
    /// fragment came from.
    pub fn query_merged(
        &self,
        query: &str,
        mode: QueryMode,
        domain_filter: Option<&str>,
        limit: usize,
    ) -> Result<Vec<QueryResult>> {
        let indices = self.store.list_packs()?;
        if indices.is_empty() {
            return Err(KnowledgeEngineError::EmptyStore);
        }

        let mut all_results: Vec<QueryResult> = Vec::new();
        for idx in &indices {
            if let Ok(pack) = self.store.load_latest(&idx.name) {
                let response = self.query_pack(&pack, query, &mode, domain_filter, limit);
                all_results.extend(response.results);
            }
        }

        // Global sort by relevance descending
        all_results.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all_results.truncate(limit);

        Ok(all_results)
    }

    fn query_pack(
        &self,
        pack: &KnowledgePack,
        query: &str,
        mode: &QueryMode,
        domain_filter: Option<&str>,
        limit: usize,
    ) -> QueryResponse {
        let query_lower = query.to_lowercase();
        let mut results: Vec<QueryResult> = Vec::new();

        for frag in &pack.fragments {
            // Domain filter
            if let Some(domain) = domain_filter {
                if frag.domain != domain {
                    continue;
                }
            }

            let relevance = match mode {
                QueryMode::Keyword => {
                    // Token Jaccard similarity between query and fragment text
                    let text_sim = token_similarity(query, &frag.text);
                    // Boost if query terms appear literally
                    let literal_boost = if frag.text.to_lowercase().contains(&query_lower) {
                        0.3
                    } else {
                        0.0
                    };
                    (text_sim + literal_boost).min(1.0)
                }
                QueryMode::Concept => {
                    // Match against concept terms
                    let concept_terms: Vec<&str> =
                        frag.concepts.iter().map(|c| c.term.as_str()).collect();
                    let max_sim = concept_terms
                        .iter()
                        .map(|t| token_similarity(query, t))
                        .fold(0.0_f64, f64::max);
                    max_sim
                }
                QueryMode::Domain => {
                    // Match against domain
                    if frag.domain.to_lowercase().contains(&query_lower) {
                        1.0
                    } else {
                        0.0
                    }
                }
            };

            if relevance > 0.05 {
                results.push(QueryResult {
                    content: frag.text.clone(),
                    concepts: frag.concepts.iter().map(|c| c.term.clone()).collect(),
                    domain: frag.domain.clone(),
                    relevance,
                    fragment_id: frag.id.clone(),
                });
            }
        }

        // Sort by relevance descending
        results.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let total_matches = results.len();
        results.truncate(limit);

        QueryResponse {
            results,
            total_matches,
            pack_name: pack.name.clone(),
            pack_version: pack.version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{CompileOptions, KnowledgeCompiler};
    use crate::ingest::{KnowledgeSource, RawKnowledge};
    use nexcore_chrono::DateTime;

    fn setup() -> (KnowledgeStore, QueryEngine) {
        let store = KnowledgeStore::temp().unwrap();
        let compiler = KnowledgeCompiler::new(store.clone());
        let options = CompileOptions {
            name: "test-query".to_string(),
            include_distillations: false,
            include_artifacts: false,
            include_implicit: false,
            include_staged: false,
            sources: vec![
                RawKnowledge {
                    text: "Signal detection uses PRR for pharmacovigilance safety.".to_string(),
                    source: KnowledgeSource::FreeText,
                    domain: Some("pv".to_string()),
                    timestamp: DateTime::now(),
                },
                RawKnowledge {
                    text: "Rust traits enable polymorphism through generics and dynamic dispatch."
                        .to_string(),
                    source: KnowledgeSource::FreeText,
                    domain: Some("rust".to_string()),
                    timestamp: DateTime::now(),
                },
            ],
        };
        compiler.compile(options).unwrap();
        let engine = QueryEngine::new(store.clone());
        (store, engine)
    }

    #[test]
    fn keyword_query() {
        let (_store, engine) = setup();
        let outcome = engine
            .query(
                "signal detection",
                Some("test-query"),
                QueryMode::Keyword,
                None,
                10,
            )
            .unwrap();
        assert_eq!(outcome.packs_loaded, 1);
        assert!(!outcome.responses.is_empty());
        assert!(outcome.responses[0].results[0].relevance > 0.0);
    }

    #[test]
    fn merged_query_across_packs() {
        let store = KnowledgeStore::temp().unwrap();
        // Compile two separate packs
        let compiler = KnowledgeCompiler::new(store.clone());
        compiler
            .compile(CompileOptions {
                name: "pack-a".to_string(),
                include_distillations: false,
                include_artifacts: false,
                include_implicit: false,
                include_staged: false,
                sources: vec![RawKnowledge {
                    text: "Signal detection uses PRR for pharmacovigilance safety.".to_string(),
                    source: KnowledgeSource::FreeText,
                    domain: Some("pv".to_string()),
                    timestamp: DateTime::now(),
                }],
            })
            .unwrap();
        compiler
            .compile(CompileOptions {
                name: "pack-b".to_string(),
                include_distillations: false,
                include_artifacts: false,
                include_implicit: false,
                include_staged: false,
                sources: vec![RawKnowledge {
                    text: "Signal processing and detection algorithms in safety monitoring."
                        .to_string(),
                    source: KnowledgeSource::FreeText,
                    domain: Some("pv".to_string()),
                    timestamp: DateTime::now(),
                }],
            })
            .unwrap();

        let engine = QueryEngine::new(store);
        let results = engine
            .query_merged("signal detection", QueryMode::Keyword, None, 10)
            .unwrap();
        // Both packs have signal detection — merged results should have entries from both
        assert!(
            results.len() >= 2,
            "expected merged results from 2 packs, got {}",
            results.len()
        );
        // Results must be sorted by relevance descending
        for pair in results.windows(2) {
            assert!(
                pair[0].relevance >= pair[1].relevance,
                "results not sorted: {} < {}",
                pair[0].relevance,
                pair[1].relevance
            );
        }
    }

    #[test]
    fn domain_filter() {
        let (_store, engine) = setup();
        let outcome = engine
            .query(
                "traits",
                Some("test-query"),
                QueryMode::Keyword,
                Some("rust"),
                10,
            )
            .unwrap();
        assert!(!outcome.responses.is_empty());
        for result in &outcome.responses[0].results {
            assert_eq!(result.domain, "rust");
        }
    }
}
