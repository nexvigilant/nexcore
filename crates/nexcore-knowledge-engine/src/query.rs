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
    pub fragment_id: String,
}

/// Query response with results and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub results: Vec<QueryResult>,
    pub total_matches: usize,
    pub pack_name: String,
    pub pack_version: u32,
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
    ) -> Result<Vec<QueryResponse>> {
        let packs = if let Some(name) = pack_name {
            vec![self.store.load_latest(name)?]
        } else {
            let indices = self.store.list_packs()?;
            let mut packs = Vec::new();
            for idx in &indices {
                match self.store.load_latest(&idx.name) {
                    Ok(p) => packs.push(p),
                    Err(_) => continue,
                }
            }
            packs
        };

        if packs.is_empty() {
            return Err(KnowledgeEngineError::PackNotFound(
                pack_name.unwrap_or("any").to_string(),
            ));
        }

        let mut responses = Vec::new();
        for pack in &packs {
            let response = self.query_pack(pack, query, &mode, domain_filter, limit);
            if !response.results.is_empty() {
                responses.push(response);
            }
        }

        Ok(responses)
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
    use chrono::Utc;

    fn setup() -> (KnowledgeStore, QueryEngine) {
        let store = KnowledgeStore::temp().unwrap();
        let compiler = KnowledgeCompiler::new(store.clone());
        let options = CompileOptions {
            name: "test-query".to_string(),
            include_distillations: false,
            include_artifacts: false,
            include_implicit: false,
            sources: vec![
                RawKnowledge {
                    text: "Signal detection uses PRR for pharmacovigilance safety.".to_string(),
                    source: KnowledgeSource::FreeText,
                    domain: Some("pv".to_string()),
                    timestamp: Utc::now(),
                },
                RawKnowledge {
                    text: "Rust traits enable polymorphism through generics and dynamic dispatch."
                        .to_string(),
                    source: KnowledgeSource::FreeText,
                    domain: Some("rust".to_string()),
                    timestamp: Utc::now(),
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
        let responses = engine
            .query(
                "signal detection",
                Some("test-query"),
                QueryMode::Keyword,
                None,
                10,
            )
            .unwrap();
        assert!(!responses.is_empty());
        assert!(responses[0].results[0].relevance > 0.0);
    }

    #[test]
    fn domain_filter() {
        let (_store, engine) = setup();
        let responses = engine
            .query(
                "traits",
                Some("test-query"),
                QueryMode::Keyword,
                Some("rust"),
                10,
            )
            .unwrap();
        assert!(!responses.is_empty());
        for result in &responses[0].results {
            assert_eq!(result.domain, "rust");
        }
    }
}
