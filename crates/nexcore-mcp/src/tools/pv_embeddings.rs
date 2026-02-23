//! Domain-Specific PV Embeddings — TF-IDF + graph similarity for pharmacovigilance.
//!
//! Inspired by AI Engineering Bible Section 4 (Domain Embeddings):
//! instead of generic embeddings that lose PV nuance, builds domain-specific
//! vectors from the 904 ICH glossary terms using their definitions,
//! cross-references, and regulatory source metadata.
//!
//! # Embedding Strategy (No External API)
//!
//! 1. **TF-IDF Definition Vectors**: Each ICH term gets a sparse vector
//!    computed from its definition text. Terms with similar definitions
//!    have high cosine similarity.
//!
//! 2. **Graph Similarity**: `see_also` cross-references form a semantic
//!    graph. Terms connected by short paths are semantically related.
//!
//! 3. **Composite Score**: `0.6 × tfidf_cosine + 0.3 × graph_proximity + 0.1 × category_match`
//!
//! # T1 Grounding: μ(Mapping) + κ(Comparison) + N(Quantity) + σ(Sequence)
//! - μ: Term → vector mapping
//! - κ: Cosine similarity comparison
//! - N: TF-IDF numeric weights
//! - σ: Graph traversal sequence

use crate::params::{PvEmbeddingGetParams, PvEmbeddingSimilarityParams, PvEmbeddingStatsParams};
use nexcore_vigilance::pv::regulatory::ich_glossary::{Term, all_terms, lookup_term, search_terms};
use parking_lot::RwLock;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::LazyLock;

// ============================================================================
// TF-IDF Index (computed once, cached)
// ============================================================================

/// A term's sparse TF-IDF vector: maps vocabulary_token → weight.
type SparseVector = HashMap<String, f64>;

/// The full TF-IDF index over all ICH terms.
struct TfIdfIndex {
    /// Term key → sparse vector
    vectors: HashMap<String, SparseVector>,
    /// Document frequency: token → how many term definitions contain it
    df: HashMap<String, usize>,
    /// Total number of terms indexed
    doc_count: usize,
    /// Vocabulary size
    vocab_size: usize,
    /// See-also adjacency list: term_key → set of related term_keys
    graph: HashMap<String, HashSet<String>>,
    /// Term key → ICH category set
    categories: HashMap<String, Vec<String>>,
}

/// Build the TF-IDF index from all ICH terms.
fn build_index() -> TfIdfIndex {
    let terms = all_terms();
    let doc_count = terms.len();

    // 1. Tokenize all definitions and compute document frequency
    let mut term_tokens: HashMap<String, Vec<String>> = HashMap::new();
    let mut df: HashMap<String, usize> = HashMap::new();
    let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
    let mut categories: HashMap<String, Vec<String>> = HashMap::new();

    for term in &terms {
        let key = term.key.to_string();

        // Tokenize definition + name + abbreviation
        let mut text = format!("{} {}", term.name, term.definition);
        if let Some(abbr) = term.abbreviation {
            text.push(' ');
            text.push_str(abbr);
        }
        if let Some(clar) = term.clarification {
            text.push(' ');
            text.push_str(clar);
        }

        let tokens = tokenize(&text);

        // Document frequency: count unique tokens per doc
        let unique: HashSet<&String> = tokens.iter().collect();
        for token in &unique {
            *df.entry((*token).clone()).or_insert(0) += 1;
        }

        term_tokens.insert(key.clone(), tokens);

        // Build graph from see_also
        let mut neighbors = HashSet::new();
        for related in term.see_also {
            let related_key = normalize_key(related);
            neighbors.insert(related_key);
        }
        graph.insert(key.clone(), neighbors);

        // Categories
        let cats: Vec<String> = term.categories().iter().map(|c| format!("{c:?}")).collect();
        categories.insert(key, cats);
    }

    let vocab_size = df.len();

    // 2. Compute TF-IDF vectors
    let mut vectors: HashMap<String, SparseVector> = HashMap::new();
    let ln_doc_count = (doc_count as f64).ln_1p();

    for (key, tokens) in &term_tokens {
        if tokens.is_empty() {
            continue;
        }

        // Term frequency
        let mut tf: HashMap<String, usize> = HashMap::new();
        for token in tokens {
            *tf.entry(token.clone()).or_insert(0) += 1;
        }

        let doc_len = tokens.len() as f64;
        let mut vec = SparseVector::new();

        for (token, count) in &tf {
            let tf_norm = *count as f64 / doc_len;
            let term_df = df.get(token.as_str()).copied().unwrap_or(1) as f64;
            let idf = (doc_count as f64 / term_df).ln_1p();
            let weight = tf_norm * idf;
            if weight > 0.001 {
                vec.insert(token.clone(), weight);
            }
        }

        // L2 normalize
        let magnitude: f64 = vec.values().map(|v| v * v).sum::<f64>().sqrt();
        if magnitude > 0.0 {
            for val in vec.values_mut() {
                *val /= magnitude;
            }
        }

        vectors.insert(key.clone(), vec);
    }

    TfIdfIndex {
        vectors,
        df,
        doc_count,
        vocab_size,
        graph,
        categories,
    }
}

static INDEX: LazyLock<RwLock<TfIdfIndex>> = LazyLock::new(|| RwLock::new(build_index()));

// ============================================================================
// Tokenization & Similarity
// ============================================================================

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|s| s.len() >= 2)
        .map(|s| s.to_lowercase())
        .collect()
}

fn normalize_key(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Cosine similarity between two sparse vectors.
fn cosine_similarity(a: &SparseVector, b: &SparseVector) -> f64 {
    // Iterate over the smaller vector for efficiency
    let (small, large) = if a.len() <= b.len() { (a, b) } else { (b, a) };

    let dot: f64 = small
        .iter()
        .filter_map(|(k, v)| large.get(k).map(|w| v * w))
        .sum();

    // Vectors are already L2-normalized, so dot product IS cosine similarity
    dot
}

/// Graph proximity: 1.0 if direct neighbor, 0.5 if 2 hops, 0.25 if 3 hops, 0.0 otherwise.
fn graph_proximity(index: &TfIdfIndex, key_a: &str, key_b: &str) -> f64 {
    if key_a == key_b {
        return 1.0;
    }

    // BFS up to depth 3
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((key_a.to_string(), 0_u8));
    visited.insert(key_a.to_string());

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= 3 {
            continue;
        }

        if let Some(neighbors) = index.graph.get(&current) {
            for neighbor in neighbors {
                if neighbor == key_b {
                    return match depth + 1 {
                        1 => 1.0,
                        2 => 0.5,
                        3 => 0.25,
                        _ => 0.0,
                    };
                }
                if visited.insert(neighbor.clone()) {
                    queue.push_back((neighbor.clone(), depth + 1));
                }
            }
        }
    }

    0.0
}

/// Category overlap: 1.0 if same category, 0.5 if partial overlap, 0.0 if none.
fn category_overlap(index: &TfIdfIndex, key_a: &str, key_b: &str) -> f64 {
    let cats_a = index.categories.get(key_a);
    let cats_b = index.categories.get(key_b);

    match (cats_a, cats_b) {
        (Some(a), Some(b)) if !a.is_empty() && !b.is_empty() => {
            let set_a: HashSet<&String> = a.iter().collect();
            let set_b: HashSet<&String> = b.iter().collect();
            let intersection = set_a.intersection(&set_b).count();
            let union = set_a.union(&set_b).count();
            if union > 0 {
                intersection as f64 / union as f64
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

// ============================================================================
// Tool Implementations
// ============================================================================

/// `pv_embedding_similarity` — Find semantically similar PV terms.
///
/// Computes composite similarity: 0.6 × TF-IDF cosine + 0.3 × graph proximity + 0.1 × category overlap.
/// Works for both ICH terms and free-text queries.
pub fn pv_embedding_similarity(
    params: PvEmbeddingSimilarityParams,
) -> Result<CallToolResult, McpError> {
    let index = INDEX.read();
    let limit = params.limit.unwrap_or(10);
    let min_sim = params.min_similarity.unwrap_or(0.1);
    let include_vectors = params.include_vectors.unwrap_or(false);

    // Build query vector
    let query_tokens = tokenize(&params.query);
    if query_tokens.is_empty() {
        return Err(McpError::invalid_params(
            "Query must contain at least one meaningful term".to_string(),
            None,
        ));
    }

    // Check if query matches an existing term
    let query_key = normalize_key(&params.query);
    let query_is_term = index.vectors.contains_key(&query_key);

    // Compute TF-IDF vector for query
    let doc_len = query_tokens.len() as f64;
    let mut tf: HashMap<String, usize> = HashMap::new();
    for token in &query_tokens {
        *tf.entry(token.clone()).or_insert(0) += 1;
    }

    let mut query_vec = SparseVector::new();
    for (token, count) in &tf {
        let tf_norm = *count as f64 / doc_len;
        let term_df = index.df.get(token.as_str()).copied().unwrap_or(1) as f64;
        let idf = (index.doc_count as f64 / term_df).ln_1p();
        let weight = tf_norm * idf;
        if weight > 0.001 {
            query_vec.insert(token.clone(), weight);
        }
    }

    // L2 normalize query vector
    let magnitude: f64 = query_vec.values().map(|v| v * v).sum::<f64>().sqrt();
    if magnitude > 0.0 {
        for val in query_vec.values_mut() {
            *val /= magnitude;
        }
    }

    // Score all terms
    let mut scored: Vec<(String, f64, f64, f64, f64)> = Vec::new(); // (key, composite, tfidf, graph, category)

    for (term_key, term_vec) in &index.vectors {
        if query_is_term && *term_key == query_key {
            continue; // Skip self-match
        }

        let tfidf_sim = cosine_similarity(&query_vec, term_vec);
        let graph_sim = if query_is_term {
            graph_proximity(&index, &query_key, term_key)
        } else {
            0.0
        };
        let cat_sim = if query_is_term {
            category_overlap(&index, &query_key, term_key)
        } else {
            0.0
        };

        let composite = 0.6 * tfidf_sim + 0.3 * graph_sim + 0.1 * cat_sim;

        if composite >= min_sim {
            scored.push((term_key.clone(), composite, tfidf_sim, graph_sim, cat_sim));
        }
    }

    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    scored.truncate(limit);

    // Build results
    let results: Vec<serde_json::Value> = scored
        .iter()
        .map(|(key, composite, tfidf, graph, cat)| {
            let term = lookup_term(key);
            let term_name = term.map(|t| t.name).unwrap_or(key.as_str());
            let definition = term.map(|t| t.definition).unwrap_or("");
            let abbreviation = term.and_then(|t| t.abbreviation);
            let see_also: Vec<&str> = term.map(|t| t.see_also.to_vec()).unwrap_or_default();

            let mut entry = json!({
                "term": term_name,
                "key": key,
                "composite_similarity": (*composite * 1000.0).round() / 1000.0,
                "tfidf_similarity": (*tfidf * 1000.0).round() / 1000.0,
                "graph_proximity": (*graph * 1000.0).round() / 1000.0,
                "category_overlap": (*cat * 1000.0).round() / 1000.0,
                "abbreviation": abbreviation,
                "definition_preview": definition.chars().take(150).collect::<String>(),
                "see_also": see_also,
            });

            if include_vectors {
                if let Some(vec) = index.vectors.get(key.as_str()) {
                    // Top 10 vector dimensions by weight
                    let mut dims: Vec<_> = vec.iter().collect();
                    dims.sort_by(|a, b| b.1.total_cmp(a.1));
                    dims.truncate(10);
                    let top_dims: Vec<serde_json::Value> = dims
                        .iter()
                        .map(|(k, v)| json!({"token": k, "weight": (*v * 1000.0).round() / 1000.0}))
                        .collect();
                    entry
                        .as_object_mut()
                        .map(|o| o.insert("top_vector_dims".to_string(), json!(top_dims)));
                }
            }

            entry
        })
        .collect();

    let result = json!({
        "status": "success",
        "query": params.query,
        "query_is_ich_term": query_is_term,
        "scoring": {
            "formula": "0.6 × tfidf_cosine + 0.3 × graph_proximity + 0.1 × category_overlap",
            "min_similarity": min_sim,
        },
        "result_count": results.len(),
        "results": results,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `pv_embedding_get` — Get the full embedding for a specific ICH term.
///
/// Returns the sparse TF-IDF vector, graph neighbors, and metadata.
pub fn pv_embedding_get(params: PvEmbeddingGetParams) -> Result<CallToolResult, McpError> {
    let index = INDEX.read();

    // Try exact key first, then search
    let key = normalize_key(&params.term);
    let term = lookup_term(&key).or_else(|| lookup_term(&params.term));

    let term = term.ok_or_else(|| {
        // Try fuzzy match
        let suggestions = search_terms(&params.term);
        let suggestion_names: Vec<&str> = suggestions.iter().take(5).map(|s| s.term.name).collect();
        McpError::invalid_params(
            format!(
                "Term '{}' not found. Did you mean: {:?}",
                params.term, suggestion_names
            ),
            None,
        )
    })?;

    let term_key = term.key.to_string();

    // Get vector
    let vector = index.vectors.get(&term_key);
    let vector_info = vector.map(|v| {
        let mut dims: Vec<_> = v.iter().collect();
        dims.sort_by(|a, b| b.1.total_cmp(a.1));
        let top_dims: Vec<serde_json::Value> = dims
            .iter()
            .take(20)
            .map(|(k, w)| json!({"token": k, "weight": (*w * 1000.0).round() / 1000.0}))
            .collect();
        json!({
            "dimensions": v.len(),
            "top_20_dimensions": top_dims,
            "l2_norm": 1.0, // Already normalized
        })
    });

    // Get graph neighbors
    let neighbors = index.graph.get(&term_key);
    let neighbor_names: Vec<&str> = neighbors
        .map(|n| {
            n.iter()
                .filter_map(|k| lookup_term(k).map(|t| t.name))
                .collect()
        })
        .unwrap_or_default();

    // Categories
    let categories = index.categories.get(&term_key).cloned().unwrap_or_default();

    let result = json!({
        "term": term.name,
        "key": term_key,
        "definition": term.definition,
        "abbreviation": term.abbreviation,
        "source": {
            "guideline": term.source.guideline_id,
            "section": term.source.section,
        },
        "embedding": {
            "method": "tfidf_sparse",
            "vector": vector_info,
        },
        "graph": {
            "see_also": term.see_also,
            "resolved_neighbors": neighbor_names,
            "neighbor_count": neighbor_names.len(),
        },
        "categories": categories,
        "is_new": term.is_new,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `pv_embedding_stats` — Index statistics for the PV embedding system.
pub fn pv_embedding_stats(_params: PvEmbeddingStatsParams) -> Result<CallToolResult, McpError> {
    let index = INDEX.read();

    // Graph stats
    let total_edges: usize = index.graph.values().map(|v| v.len()).sum();
    let max_neighbors = index.graph.values().map(|v| v.len()).max().unwrap_or(0);
    let terms_with_neighbors = index.graph.values().filter(|v| !v.is_empty()).count();

    // Vector stats
    let avg_dims = if index.vectors.is_empty() {
        0.0
    } else {
        index.vectors.values().map(|v| v.len()).sum::<usize>() as f64 / index.vectors.len() as f64
    };
    let max_dims = index.vectors.values().map(|v| v.len()).max().unwrap_or(0);

    // Category distribution
    let mut cat_counts: HashMap<String, usize> = HashMap::new();
    for cats in index.categories.values() {
        for cat in cats {
            *cat_counts.entry(cat.clone()).or_insert(0) += 1;
        }
    }

    // Top IDF terms (most discriminating)
    let mut idf_terms: Vec<_> = index
        .df
        .iter()
        .map(|(term, df_count)| {
            let idf = (index.doc_count as f64 / *df_count as f64).ln_1p();
            (term.clone(), idf, *df_count)
        })
        .collect();
    idf_terms.sort_by(|a, b| b.1.total_cmp(&a.1));
    let top_discriminating: Vec<serde_json::Value> = idf_terms
        .iter()
        .take(15)
        .map(|(term, idf, df)| {
            json!({"token": term, "idf": (*idf * 100.0).round() / 100.0, "doc_frequency": df})
        })
        .collect();

    let result = json!({
        "index": {
            "total_terms": index.doc_count,
            "total_vectors": index.vectors.len(),
            "vocabulary_size": index.vocab_size,
            "embedding_method": "tfidf_sparse_l2_normalized",
        },
        "vectors": {
            "avg_dimensions": (avg_dims * 10.0).round() / 10.0,
            "max_dimensions": max_dims,
        },
        "graph": {
            "total_edges": total_edges,
            "terms_with_neighbors": terms_with_neighbors,
            "max_neighbors": max_neighbors,
            "avg_neighbors": if terms_with_neighbors > 0 {
                (total_edges as f64 / terms_with_neighbors as f64 * 10.0).round() / 10.0
            } else {
                0.0
            },
        },
        "categories": cat_counts,
        "top_discriminating_tokens": top_discriminating,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
