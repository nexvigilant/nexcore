//! Python Bindings Module - PyO3 bridge for Python integration
//!
//! This module provides Python bindings for ich-glossary functions via PyO3.
//! It's only compiled when the `python` feature is enabled.
//!
//! ## Building the Python Extension
//!
//! ```bash
//! # Build with maturin (recommended)
//! pip install maturin
//! cd ~/.claude/rust/ich-glossary
//! maturin build --release --features python
//!
//! # Or build manually
//! cargo build --release --features python
//! ```
//!
//! ## Usage from Python
//!
//! ```python
//! import ich_glossary
//!
//! # O(1) term lookup
//! term = ich_glossary.lookup_term("adverse drug reaction")
//! if term:
//!     print(f"Definition: {term['definition']}")
//!     print(f"Source: {term['source']['guideline_id']}")
//!     print(f"Abbreviation: {term['abbreviation']}")
//!
//! # Search across all terms
//! results = ich_glossary.search_terms("signal detection")
//! for result in results[:5]:
//!     print(f"{result['term']['name']}: {result['score']}")
//!
//! # Get all terms by category
//! safety_terms = ich_glossary.terms_by_category("Safety")
//! print(f"Found {len(safety_terms)} safety terms")
//!
//! # Get terms by guideline
//! e2a_terms = ich_glossary.terms_by_guideline("E2A")
//! for term in e2a_terms:
//!     print(f"- {term['name']}")
//!
//! # Autocomplete suggestions
//! suggestions = ich_glossary.autocomplete("adv", 5)
//! print(f"Suggestions: {suggestions}")
//!
//! # Get related terms
//! related = ich_glossary.get_related_terms("adverse event")
//! for term in related:
//!     print(f"Related: {term['name']}")
//!
//! # Get glossary metadata
//! meta = ich_glossary.glossary_metadata()
//! print(f"Version: {meta['version']}, Terms: {meta['total_terms']}")
//!
//! # Get statistics
//! stats = ich_glossary.get_search_stats()
//! print(f"Total terms: {stats['total_terms']}")
//! ```

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyDict;

#[cfg(feature = "python")]
use crate::regulatory::ich_glossary::search::{
    autocomplete, get_related_terms, get_search_stats, search_by_guideline_series,
    search_terms, search_with_query,
};
#[cfg(feature = "python")]
use crate::regulatory::ich_glossary::terms::{
    all_terms, lookup_by_abbreviation, lookup_term, new_terms, terms_by_category,
    terms_by_guideline, terms_with_abbreviations, terms_with_alternatives,
};
#[cfg(feature = "python")]
use crate::regulatory::ich_glossary::types::{IchCategory, TermQuery};
#[cfg(feature = "python")]
use crate::{glossary_metadata, TOTAL_GUIDELINE_COUNT, TOTAL_TERM_COUNT};

// ============================================================================
// Term Lookup Operations
// ============================================================================

/// O(1) lookup of term by name (case-insensitive)
///
/// Args:
///     name: Term name to look up (e.g., "adverse drug reaction", "ADR")
///
/// Returns:
///     dict with term data or None if not found
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "lookup_term")]
fn py_lookup_term(py: Python<'_>, name: &str) -> PyResult<Option<PyObject>> {
    match lookup_term(name) {
        Some(term) => {
            let json_str = serde_json::to_string(term)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
            let json_module = py.import("json")?;
            let py_dict = json_module.call_method1("loads", (json_str,))?;
            Ok(Some(py_dict.into()))
        }
        None => Ok(None),
    }
}

/// Lookup term by abbreviation (e.g., "ADR", "GCP", "SAE")
///
/// Args:
///     abbrev: Abbreviation to look up
///
/// Returns:
///     dict with term data or None if not found
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "lookup_by_abbreviation")]
fn py_lookup_by_abbreviation(py: Python<'_>, abbrev: &str) -> PyResult<Option<PyObject>> {
    match lookup_by_abbreviation(abbrev) {
        Some(term) => {
            let json_str = serde_json::to_string(term)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
            let json_module = py.import("json")?;
            let py_dict = json_module.call_method1("loads", (json_str,))?;
            Ok(Some(py_dict.into()))
        }
        None => Ok(None),
    }
}

/// Get all terms in the glossary
///
/// Returns:
///     list of term dicts
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "all_terms")]
fn py_all_terms(py: Python<'_>) -> PyResult<PyObject> {
    let terms = all_terms();
    let json_str = serde_json::to_string(&terms)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let json_module = py.import("json")?;
    let py_list = json_module.call_method1("loads", (json_str,))?;
    Ok(py_list.into())
}

/// Get terms by ICH category
///
/// Args:
///     category: One of "Quality", "Safety", "Efficacy", "Multidisciplinary"
///
/// Returns:
///     list of term dicts in that category
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "terms_by_category")]
fn py_terms_by_category(py: Python<'_>, category: &str) -> PyResult<PyObject> {
    let cat = match category.to_lowercase().as_str() {
        "quality" | "q" => IchCategory::Quality,
        "safety" | "s" => IchCategory::Safety,
        "efficacy" | "e" => IchCategory::Efficacy,
        "multidisciplinary" | "m" => IchCategory::Multidisciplinary,
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Unknown category: {}. Use one of: Quality, Safety, Efficacy, Multidisciplinary",
                category
            )))
        }
    };

    let terms = terms_by_category(cat);
    let json_str = serde_json::to_string(&terms)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let json_module = py.import("json")?;
    let py_list = json_module.call_method1("loads", (json_str,))?;
    Ok(py_list.into())
}

/// Get terms by guideline ID (e.g., "E2A", "Q9(R1)")
///
/// Args:
///     guideline_id: Guideline identifier (partial match supported)
///
/// Returns:
///     list of term dicts from that guideline
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "terms_by_guideline")]
fn py_terms_by_guideline(py: Python<'_>, guideline_id: &str) -> PyResult<PyObject> {
    let terms = terms_by_guideline(guideline_id);
    let json_str = serde_json::to_string(&terms)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let json_module = py.import("json")?;
    let py_list = json_module.call_method1("loads", (json_str,))?;
    Ok(py_list.into())
}

/// Get new terms (added in current glossary version)
///
/// Returns:
///     list of term dicts marked as new
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "new_terms")]
fn py_new_terms(py: Python<'_>) -> PyResult<PyObject> {
    let terms = new_terms();
    let json_str = serde_json::to_string(&terms)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let json_module = py.import("json")?;
    let py_list = json_module.call_method1("loads", (json_str,))?;
    Ok(py_list.into())
}

/// Get terms with multiple definitions (from different guidelines)
///
/// Returns:
///     list of term dicts that have alternative_definitions
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "terms_with_alternatives")]
fn py_terms_with_alternatives(py: Python<'_>) -> PyResult<PyObject> {
    let terms = terms_with_alternatives();
    let json_str = serde_json::to_string(&terms)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let json_module = py.import("json")?;
    let py_list = json_module.call_method1("loads", (json_str,))?;
    Ok(py_list.into())
}

/// Get terms with abbreviations
///
/// Returns:
///     list of term dicts that have abbreviations
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "terms_with_abbreviations")]
fn py_terms_with_abbreviations(py: Python<'_>) -> PyResult<PyObject> {
    let terms = terms_with_abbreviations();
    let json_str = serde_json::to_string(&terms)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let json_module = py.import("json")?;
    let py_list = json_module.call_method1("loads", (json_str,))?;
    Ok(py_list.into())
}

// ============================================================================
// Search Operations
// ============================================================================

/// Search terms by query string
///
/// Args:
///     query: Search text
///
/// Returns:
///     list of search result dicts with term, score, and match_type
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "search_terms")]
fn py_search_terms(py: Python<'_>, query: &str) -> PyResult<PyObject> {
    let results = search_terms(query);
    let json_str = serde_json::to_string(&results)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let json_module = py.import("json")?;
    let py_list = json_module.call_method1("loads", (json_str,))?;
    Ok(py_list.into())
}

/// Advanced search with filters
///
/// Args:
///     query: Search text
///     category: Optional category filter ("Quality", "Safety", etc.)
///     guideline: Optional guideline filter
///     search_definitions: Whether to search in definitions (default: False)
///     limit: Maximum results (default: None = unlimited)
///     min_score: Minimum relevance score (default: None)
///
/// Returns:
///     list of search result dicts
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(
    name = "search",
    signature = (query, category=None, guideline=None, search_definitions=false, limit=None, min_score=None)
)]
fn py_search(
    py: Python<'_>,
    query: &str,
    category: Option<&str>,
    guideline: Option<&str>,
    search_definitions: bool,
    limit: Option<usize>,
    min_score: Option<f64>,
) -> PyResult<PyObject> {
    let mut term_query = TermQuery::new(query);

    if let Some(cat_str) = category {
        let cat = match cat_str.to_lowercase().as_str() {
            "quality" | "q" => IchCategory::Quality,
            "safety" | "s" => IchCategory::Safety,
            "efficacy" | "e" => IchCategory::Efficacy,
            "multidisciplinary" | "m" => IchCategory::Multidisciplinary,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unknown category: {}",
                    cat_str
                )))
            }
        };
        term_query = term_query.with_category(cat);
    }

    if let Some(gl) = guideline {
        term_query = term_query.with_guideline(gl);
    }

    if search_definitions {
        term_query = term_query.search_definitions();
    }

    if let Some(lim) = limit {
        term_query = term_query.limit(lim);
    }

    term_query.min_score = min_score;

    let results = search_with_query(term_query);
    let json_str = serde_json::to_string(&results)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let json_module = py.import("json")?;
    let py_list = json_module.call_method1("loads", (json_str,))?;
    Ok(py_list.into())
}

/// Get autocomplete suggestions for partial input
///
/// Args:
///     partial: Partial term text
///     limit: Maximum suggestions (default: 10)
///
/// Returns:
///     list of term name strings
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "autocomplete", signature = (partial, limit=10))]
fn py_autocomplete(_py: Python<'_>, partial: &str, limit: usize) -> PyResult<Vec<String>> {
    let suggestions = autocomplete(partial, limit);
    Ok(suggestions.into_iter().map(|s| s.to_string()).collect())
}

/// Get related terms via "see also" references
///
/// Args:
///     term_key: Term name or key to find related terms for
///
/// Returns:
///     list of related term dicts
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "get_related_terms")]
fn py_get_related_terms(py: Python<'_>, term_key: &str) -> PyResult<PyObject> {
    let related = get_related_terms(term_key);
    let json_str = serde_json::to_string(&related)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let json_module = py.import("json")?;
    let py_list = json_module.call_method1("loads", (json_str,))?;
    Ok(py_list.into())
}

/// Search terms by guideline series prefix (e.g., "E2" for all E2x)
///
/// Args:
///     prefix: Guideline prefix (e.g., "E2", "Q9")
///
/// Returns:
///     list of term dicts
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "search_by_guideline_series")]
fn py_search_by_guideline_series(py: Python<'_>, prefix: &str) -> PyResult<PyObject> {
    let terms = search_by_guideline_series(prefix);
    let json_str = serde_json::to_string(&terms)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let json_module = py.import("json")?;
    let py_list = json_module.call_method1("loads", (json_str,))?;
    Ok(py_list.into())
}

// ============================================================================
// Metadata & Statistics
// ============================================================================

/// Get glossary metadata
///
/// Returns:
///     dict with version, release_date, source, source_url, total_terms, total_guidelines
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "glossary_metadata")]
fn py_glossary_metadata(py: Python<'_>) -> PyResult<PyObject> {
    let meta = glossary_metadata();
    let json_str = serde_json::to_string(&meta)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let json_module = py.import("json")?;
    let py_dict = json_module.call_method1("loads", (json_str,))?;
    Ok(py_dict.into())
}

/// Get search statistics
///
/// Returns:
///     dict with total_terms, terms_with_abbreviations, terms_with_alternatives,
///     terms_by_category, new_terms
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "get_search_stats")]
fn py_get_search_stats(py: Python<'_>) -> PyResult<PyObject> {
    let stats = get_search_stats();
    let json_str = serde_json::to_string(&stats)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    let json_module = py.import("json")?;
    let py_dict = json_module.call_method1("loads", (json_str,))?;
    Ok(py_dict.into())
}

/// Get ICH category information
///
/// Args:
///     category: Category name ("Quality", "Safety", "Efficacy", "Multidisciplinary")
///
/// Returns:
///     dict with prefix, name, description
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "get_category_info")]
fn py_get_category_info(py: Python<'_>, category: &str) -> PyResult<PyObject> {
    let cat = match category.to_lowercase().as_str() {
        "quality" | "q" => IchCategory::Quality,
        "safety" | "s" => IchCategory::Safety,
        "efficacy" | "e" => IchCategory::Efficacy,
        "multidisciplinary" | "m" => IchCategory::Multidisciplinary,
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Unknown category: {}",
                category
            )))
        }
    };

    let dict = PyDict::new(py);
    dict.set_item("prefix", cat.prefix())?;
    dict.set_item("name", cat.name())?;
    dict.set_item("description", cat.description())?;
    Ok(dict.into())
}

/// List all ICH categories
///
/// Returns:
///     list of category dicts with prefix, name, description
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "list_categories")]
fn py_list_categories(py: Python<'_>) -> PyResult<PyObject> {
    let categories = vec![
        IchCategory::Quality,
        IchCategory::Safety,
        IchCategory::Efficacy,
        IchCategory::Multidisciplinary,
    ];

    let mut result = Vec::new();
    for cat in categories {
        let dict = PyDict::new(py);
        dict.set_item("prefix", cat.prefix())?;
        dict.set_item("name", cat.name())?;
        dict.set_item("description", cat.description())?;
        result.push(dict);
    }

    Ok(result.into_pyobject(py)?.into())
}

// ============================================================================
// Module Registration
// ============================================================================

/// Python module definition
#[cfg(feature = "python")]
#[pymodule]
pub fn ich_glossary(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Term lookup operations
    m.add_function(wrap_pyfunction!(py_lookup_term, m)?)?;
    m.add_function(wrap_pyfunction!(py_lookup_by_abbreviation, m)?)?;
    m.add_function(wrap_pyfunction!(py_all_terms, m)?)?;
    m.add_function(wrap_pyfunction!(py_terms_by_category, m)?)?;
    m.add_function(wrap_pyfunction!(py_terms_by_guideline, m)?)?;
    m.add_function(wrap_pyfunction!(py_new_terms, m)?)?;
    m.add_function(wrap_pyfunction!(py_terms_with_alternatives, m)?)?;
    m.add_function(wrap_pyfunction!(py_terms_with_abbreviations, m)?)?;

    // Search operations
    m.add_function(wrap_pyfunction!(py_search_terms, m)?)?;
    m.add_function(wrap_pyfunction!(py_search, m)?)?;
    m.add_function(wrap_pyfunction!(py_autocomplete, m)?)?;
    m.add_function(wrap_pyfunction!(py_get_related_terms, m)?)?;
    m.add_function(wrap_pyfunction!(py_search_by_guideline_series, m)?)?;

    // Metadata & statistics
    m.add_function(wrap_pyfunction!(py_glossary_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(py_get_search_stats, m)?)?;
    m.add_function(wrap_pyfunction!(py_get_category_info, m)?)?;
    m.add_function(wrap_pyfunction!(py_list_categories, m)?)?;

    // Module info
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__doc__", "ICH Glossary - High-performance pharmacovigilance terminology lookup")?;
    m.add("TOTAL_TERMS", TOTAL_TERM_COUNT)?;
    m.add("TOTAL_GUIDELINES", TOTAL_GUIDELINE_COUNT)?;

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(all(test, not(feature = "python")))]
mod tests {
    #[test]
    fn test_module_compiles_without_python_feature() {
        // This test just verifies the module compiles without the python feature
        assert!(true);
    }
}
