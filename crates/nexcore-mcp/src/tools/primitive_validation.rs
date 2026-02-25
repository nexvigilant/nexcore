//! Corpus-Backed Primitive Validation Tools
//!
//! Validates regulatory primitives against external literature sources with
//! professional citations in Vancouver format. Addresses the "Expert Generation Mode"
//! limitation by requiring corpus backing for full confidence.
//!
//! ## Validation Tiers
//!
//! | Tier | Source | Confidence | Example |
//! |------|--------|------------|---------|
//! | 1 | Authoritative (ICH, FDA, EMA) | 1.0 | ICH E2A definition |
//! | 2 | Peer-reviewed (PubMed) | 0.9 | PMID:12345678 |
//! | 3 | Validated web (domain-filtered) | 0.8 | WHO, FDA.gov |
//! | 4 | Expert generation (model knowledge) | 0.6 | No external citation |

use crate::params::{PrimitiveCiteParams, PrimitiveValidateBatchParams, PrimitiveValidateParams};
use nexcore_chrono::DateTime;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ============================================================================
// Types
// ============================================================================

/// Validation tier with confidence multiplier
///
/// Tier: T2-P (Cross-domain evidence tier)
/// Grounds to: T1 primitive `u8` via explicit discriminants
/// Ord: Implemented (tier ordering)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ValidationTier {
    /// Tier 1: Authoritative sources (ICH, FDA, EMA) - confidence 1.0
    Authoritative = 1,
    /// Tier 2: Peer-reviewed literature (PubMed, Semantic Scholar) - confidence 0.9
    PeerReviewed = 2,
    /// Tier 3: Validated web sources (domain-filtered) - confidence 0.8
    ValidatedWeb = 3,
    /// Tier 4: Expert generation (model knowledge) - confidence 0.6
    ExpertGeneration = 4,
}

/// Quantified code for ValidationTier.
///
/// Tier: T2-P (Cross-domain primitive code)
/// Grounds to: T1 primitive `u8`
/// Ord: Implemented (numeric code ordering)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ValidationTierCode(pub u8);

impl From<ValidationTier> for ValidationTierCode {
    fn from(value: ValidationTier) -> Self {
        ValidationTierCode(value as u8)
    }
}

impl ValidationTier {
    fn confidence(&self) -> f64 {
        match self {
            ValidationTier::Authoritative => 1.0,
            ValidationTier::PeerReviewed => 0.9,
            ValidationTier::ValidatedWeb => 0.8,
            ValidationTier::ExpertGeneration => 0.6,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            ValidationTier::Authoritative => "Tier 1: Authoritative",
            ValidationTier::PeerReviewed => "Tier 2: Peer-Reviewed",
            ValidationTier::ValidatedWeb => "Tier 3: Validated Web",
            ValidationTier::ExpertGeneration => "Tier 4: Expert Generation",
        }
    }

    fn from_number(n: u8) -> Self {
        match n {
            1 => ValidationTier::Authoritative,
            2 => ValidationTier::PeerReviewed,
            3 => ValidationTier::ValidatedWeb,
            _ => ValidationTier::ExpertGeneration,
        }
    }
}

/// A citation in Vancouver format
///
/// Tier: T3 (Domain-specific citation)
/// Grounds to T1 Concepts via String/Option and DateTime
/// Ord: N/A (composite record)
#[derive(Debug, Clone, Serialize)]
pub struct Citation {
    /// Vancouver-formatted citation string
    pub vancouver: String,
    /// PubMed ID if available
    pub pmid: Option<String>,
    /// Digital Object Identifier if available
    pub doi: Option<String>,
    /// Validation tier
    pub tier: ValidationTier,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Validation timestamp
    pub validated_at: DateTime,
    /// Source URL
    pub url: Option<String>,
}

/// Result of primitive validation
///
/// Tier: T3 (Domain-specific validation result)
/// Grounds to T1 Concepts via String/bool/f64 and nested structs
/// Ord: N/A (composite record)
#[derive(Debug, Serialize)]
pub struct ValidationResult {
    /// The primitive term validated
    pub term: String,
    /// Whether the term was validated (found in corpus)
    pub validated: bool,
    /// Highest confidence tier achieved
    pub tier: ValidationTier,
    /// Overall confidence (tier confidence × evidence strength)
    pub confidence: f64,
    /// Definition found in corpus (if any)
    pub definition: Option<String>,
    /// Citations supporting the validation
    pub citations: Vec<Citation>,
    /// Domain context
    pub domain: String,
    /// Validation timestamp
    pub validated_at: DateTime,
}

// ============================================================================
// BioOntology API Integration (Tier 1 - Authoritative Ontologies)
// ============================================================================

/// BioOntology search result
///
/// Tier: T3 (Domain-specific API response)
/// Grounds to T1 Concepts via Option/Vec and nested records
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
struct BioOntologyResult {
    #[serde(rename = "@id")]
    _id: Option<String>,
    #[serde(rename = "prefLabel")]
    pref_label: Option<String>,
    definition: Option<Vec<String>>,
    links: Option<BioOntologyLinks>,
}

/// BioOntology link record
///
/// Tier: T3 (Domain-specific API response)
/// Grounds to T1 Concepts via Option<String>
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
struct BioOntologyLinks {
    ontology: Option<String>,
}

/// BioOntology search response
///
/// Tier: T3 (Domain-specific API response)
/// Grounds to T1 Concepts via Option/Vec/u32
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
struct BioOntologySearchResponse {
    collection: Option<Vec<BioOntologyResult>>,
    #[serde(rename = "totalCount")]
    _total_count: Option<u32>,
}

/// Search BioOntology for pharmacovigilance-relevant ontologies (MedDRA, SNOMED, etc.)
async fn search_bioontology(term: &str, max_results: usize) -> Vec<(String, String, String)> {
    // BioOntology API key (non-sensitive, public for development)
    let api_key = "443f11f6-1067-4044-bcbd-72cacd506efb";

    let encoded_term = urlencoding::encode(term);

    // Search across PV-relevant ontologies: MedDRA, SNOMED-CT, NCI Thesaurus
    let search_url = format!(
        "https://data.bioontology.org/search?q={}&ontologies=MEDDRA,SNOMEDCT,NCIT&pagesize={}&apikey={}",
        encoded_term, max_results, api_key
    );

    let response = match reqwest::get(&search_url).await {
        Ok(resp) => resp,
        Err(_) => return vec![],
    };

    let data: BioOntologySearchResponse = match response.json().await {
        Ok(data) => data,
        Err(_) => return vec![],
    };

    let mut results = Vec::new();
    if let Some(collection) = data.collection {
        for item in collection.into_iter().take(max_results) {
            let label = item.pref_label.unwrap_or_default();
            let definition = item
                .definition
                .and_then(|d| d.first().cloned())
                .unwrap_or_else(|| "No definition available".to_string());
            let ontology = item
                .links
                .and_then(|l| l.ontology)
                .unwrap_or_else(|| "Unknown".to_string());

            // Extract ontology name from URL
            let ont_name = ontology.rsplit('/').next().unwrap_or(&ontology).to_string();

            results.push((label, definition, ont_name));
        }
    }

    results
}

// ============================================================================
// PubMed E-utilities Integration
// ============================================================================

/// PubMed article summary from E-utilities API
///
/// Tier: T3 (Domain-specific PubMed summary)
/// Grounds to T1 Concepts via String/Option fields
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
struct PubMedArticle {
    uid: String,
    title: Option<String>,
    sortfirstauthor: Option<String>,
    source: Option<String>,
    pubdate: Option<String>,
    #[serde(rename = "elocationid")]
    doi: Option<String>,
}

/// PubMed E-utilities result wrapper
///
/// Tier: T3 (Domain-specific PubMed response)
/// Grounds to T1 Concepts via Option and nested records
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
struct PubMedResult {
    result: Option<PubMedResultInner>,
}

/// PubMed result inner container
///
/// Tier: T3 (Domain-specific PubMed response)
/// Grounds to T1 Concepts via Option/Vec/HashMap
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
struct PubMedResultInner {
    uids: Option<Vec<String>>,
    #[serde(flatten)]
    articles: std::collections::HashMap<String, PubMedArticle>,
}

/// Search PubMed for articles matching a term
async fn search_pubmed(term: &str, domain: &str, max_results: usize) -> Vec<PubMedArticle> {
    // Build search query with domain context
    let query = match domain.to_lowercase().as_str() {
        "pv" | "pharmacovigilance" => format!(
            "({}) AND (pharmacovigilance OR adverse event OR drug safety)",
            term
        ),
        "regulatory" => format!("({}) AND (regulatory OR FDA OR EMA OR ICH)", term),
        "medical" => format!("({}) AND (medical OR clinical)", term),
        _ => term.to_string(),
    };

    let encoded_query = urlencoding::encode(&query);

    // Step 1: Search for PMIDs
    let search_url = format!(
        "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&term={}&retmax={}&retmode=json",
        encoded_query, max_results
    );

    let search_response = match reqwest::get(&search_url).await {
        Ok(resp) => resp,
        Err(_) => return vec![],
    };

    /// PubMed search result wrapper
    ///
    /// Tier: T3 (Domain-specific PubMed response)
    /// Grounds to T1 Concepts via Option and nested record
    /// Ord: N/A (composite record)
    #[derive(Deserialize)]
    struct SearchResult {
        esearchresult: Option<SearchResultInner>,
    }
    /// PubMed search result inner container
    ///
    /// Tier: T3 (Domain-specific PubMed response)
    /// Grounds to T1 Concepts via Option and Vec
    /// Ord: N/A (composite record)
    #[derive(Deserialize)]
    struct SearchResultInner {
        idlist: Option<Vec<String>>,
    }

    let search_data: SearchResult = match search_response.json().await {
        Ok(data) => data,
        Err(_) => return vec![],
    };

    let pmids = match search_data.esearchresult.and_then(|r| r.idlist) {
        Some(ids) if !ids.is_empty() => ids,
        _ => return vec![],
    };

    // Step 2: Fetch summaries for PMIDs
    let ids_str = pmids.join(",");
    let summary_url = format!(
        "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi?db=pubmed&id={}&retmode=json",
        ids_str
    );

    let summary_response = match reqwest::get(&summary_url).await {
        Ok(resp) => resp,
        Err(_) => return vec![],
    };

    let summary_data: PubMedResult = match summary_response.json().await {
        Ok(data) => data,
        Err(_) => return vec![],
    };

    // Extract articles
    let mut articles = Vec::new();
    if let Some(result) = summary_data.result {
        if let Some(uids) = result.uids {
            for uid in uids {
                if let Some(article) = result.articles.get(&uid) {
                    articles.push(PubMedArticle {
                        uid: article.uid.clone(),
                        title: article.title.clone(),
                        sortfirstauthor: article.sortfirstauthor.clone(),
                        source: article.source.clone(),
                        pubdate: article.pubdate.clone(),
                        doi: article.doi.clone(),
                    });
                }
            }
        }
    }

    articles
}

/// Format a PubMed article as Vancouver citation
fn format_vancouver(article: &PubMedArticle) -> String {
    let author = article.sortfirstauthor.as_deref().unwrap_or("Unknown");
    let title = article.title.as_deref().unwrap_or("Untitled");
    let journal = article.source.as_deref().unwrap_or("Unknown Journal");
    let date = article.pubdate.as_deref().unwrap_or("n.d.");

    // Extract year from date
    let year = date.split_whitespace().next().unwrap_or(date);

    // Vancouver format: Author et al. Title. Journal. Year;volume(issue):pages.
    // Since we don't have full details, use simplified format
    format!(
        "{} et al. {}. {}. {};PMID:{}.",
        author.split_whitespace().next().unwrap_or(author),
        title.trim_end_matches('.'),
        journal,
        year,
        article.uid
    )
}

// ============================================================================
// ICH Glossary Integration (Tier 1 - Authoritative)
// ============================================================================

/// Check if term exists in ICH glossary (Tier 1 validation)
fn check_ich_glossary(term: &str) -> Option<(String, Citation)> {
    use nexcore_vigilance::pv::regulatory::ich_glossary::{lookup_term, search_terms};

    // Try exact lookup first
    if let Some(ich_term) = lookup_term(term) {
        let citation = Citation {
            vancouver: format!(
                "International Council for Harmonisation. {}. {} {}. Geneva: ICH; {}.",
                ich_term.name,
                ich_term.source.guideline_id,
                ich_term.source.guideline_title,
                ich_term.source.status
            ),
            pmid: None,
            doi: Some("https://doi.org/10.56759/eftb6868".to_string()),
            tier: ValidationTier::Authoritative,
            confidence: 1.0,
            validated_at: DateTime::now(),
            url: Some(format!(
                "https://www.ich.org/page/{}",
                ich_term.source.guideline_id.to_lowercase()
            )),
        };
        return Some((ich_term.definition.to_string(), citation));
    }

    // Try fuzzy search
    let results = search_terms(term);
    if let Some(result) = results.first() {
        if result.score >= 0.8 {
            let citation = Citation {
                vancouver: format!(
                    "International Council for Harmonisation. {}. {} {}. Geneva: ICH.",
                    result.term.name,
                    result.term.source.guideline_id,
                    result.term.source.guideline_title,
                ),
                pmid: None,
                doi: Some("https://doi.org/10.56759/eftb6868".to_string()),
                tier: ValidationTier::Authoritative,
                confidence: result.score,
                validated_at: DateTime::now(),
                url: None,
            };
            return Some((result.term.definition.to_string(), citation));
        }
    }

    None
}

// ============================================================================
// Tool Implementations
// ============================================================================

/// Validate a primitive term against external corpus
pub async fn validate(params: PrimitiveValidateParams) -> Result<CallToolResult, McpError> {
    let mut citations = Vec::new();
    let mut definition: Option<String> = None;
    let mut highest_tier = ValidationTier::ExpertGeneration;
    let _min_tier = ValidationTier::from_number(params.min_tier);

    // Tier 1a: Check ICH glossary (authoritative regulatory)
    if let Some((def, citation)) = check_ich_glossary(&params.term) {
        definition = Some(def);
        highest_tier = ValidationTier::Authoritative;
        citations.push(citation);
    }

    // Tier 1b: Check BioOntology (authoritative ontologies: MedDRA, SNOMED, NCI)
    let onto_results = search_bioontology(&params.term, 3).await;
    for (label, def, ontology) in onto_results {
        let citation = Citation {
            vancouver: format!(
                "{}. {}. BioPortal: National Center for Biomedical Ontology. Available from: https://bioportal.bioontology.org/ontologies/{}",
                label, ontology, ontology
            ),
            pmid: None,
            doi: None,
            tier: ValidationTier::Authoritative,
            confidence: 0.95, // High confidence for standard ontologies
            validated_at: DateTime::now(),
            url: Some(format!(
                "https://bioportal.bioontology.org/ontologies/{}",
                ontology
            )),
        };
        citations.push(citation);

        if definition.is_none() && !def.is_empty() && def != "No definition available" {
            definition = Some(def);
        }
        if highest_tier != ValidationTier::Authoritative {
            highest_tier = ValidationTier::Authoritative;
        }
    }

    // Tier 2: Search PubMed (peer-reviewed literature)
    if params.min_tier >= 2 || highest_tier == ValidationTier::ExpertGeneration {
        let articles = search_pubmed(&params.term, &params.domain, params.max_citations).await;
        for article in articles {
            let citation = Citation {
                vancouver: format_vancouver(&article),
                pmid: Some(article.uid.clone()),
                doi: article.doi.clone(),
                tier: ValidationTier::PeerReviewed,
                confidence: 0.9,
                validated_at: DateTime::now(),
                url: Some(format!("https://pubmed.ncbi.nlm.nih.gov/{}/", article.uid)),
            };
            citations.push(citation);

            if highest_tier == ValidationTier::ExpertGeneration {
                highest_tier = ValidationTier::PeerReviewed;
            }
        }
    }

    // Build result
    let validated = !citations.is_empty();
    let confidence = if validated {
        highest_tier.confidence()
    } else {
        ValidationTier::ExpertGeneration.confidence()
    };

    let result = ValidationResult {
        term: params.term.clone(),
        validated,
        tier: highest_tier,
        confidence,
        definition,
        citations: citations.into_iter().take(params.max_citations).collect(),
        domain: params.domain,
        validated_at: DateTime::now(),
    };

    // Format output
    let mut output = String::new();
    output.push_str(&format!(
        "╭─────────────────────────────────────────────────────────────╮\n"
    ));
    output.push_str(&format!(
        "│ Primitive Validation: {:<37} │\n",
        truncate(&result.term, 37)
    ));
    output.push_str(&format!(
        "╰─────────────────────────────────────────────────────────────╯\n\n"
    ));

    output.push_str(&format!(
        "Status: {} {}\n",
        if result.validated { "✅" } else { "⚠️" },
        if result.validated {
            "VALIDATED"
        } else {
            "UNVALIDATED (Expert Generation Mode)"
        }
    ));
    output.push_str(&format!("Tier: {}\n", result.tier.as_str()));
    output.push_str(&format!("Confidence: {:.0}%\n", result.confidence * 100.0));
    output.push_str(&format!("Domain: {}\n\n", result.domain));

    if let Some(def) = &result.definition {
        output.push_str("Definition:\n");
        output.push_str(&format!("  {}\n\n", def));
    }

    if !result.citations.is_empty() {
        output.push_str(&format!("Citations ({}):\n", result.citations.len()));
        for (i, citation) in result.citations.iter().enumerate() {
            output.push_str(&format!("\n{}. {}\n", i + 1, citation.vancouver));
            if let Some(pmid) = &citation.pmid {
                output.push_str(&format!("   PMID: {}\n", pmid));
            }
            if let Some(doi) = &citation.doi {
                output.push_str(&format!("   DOI: {}\n", doi));
            }
            if let Some(url) = &citation.url {
                output.push_str(&format!("   URL: {}\n", url));
            }
            output.push_str(&format!(
                "   Tier: {} (confidence: {:.0}%)\n",
                citation.tier.as_str(),
                citation.confidence * 100.0
            ));
        }
    } else {
        output.push_str("⚠️  No external citations found.\n");
        output.push_str("   This primitive is using Expert Generation Mode (confidence: 60%).\n");
        output.push_str(
            "   Consider validating against authoritative sources for regulatory compliance.\n",
        );
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

/// Generate a professional citation for a PubMed ID or DOI
pub async fn cite(params: PrimitiveCiteParams) -> Result<CallToolResult, McpError> {
    let identifier = params.identifier.trim();

    // Determine if PMID or DOI
    let is_pmid = identifier.chars().all(|c| c.is_ascii_digit());

    if is_pmid {
        // Fetch from PubMed
        let summary_url = format!(
            "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi?db=pubmed&id={}&retmode=json",
            identifier
        );

        let response = reqwest::get(&summary_url)
            .await
            .map_err(|e| McpError::internal_error(format!("PubMed request failed: {}", e), None))?;

        let data: PubMedResult = response
            .json()
            .await
            .map_err(|e| McpError::internal_error(format!("PubMed parse failed: {}", e), None))?;

        if let Some(result) = data.result {
            if let Some(article) = result.articles.get(identifier) {
                let vancouver = format_vancouver(article);

                let output = json!({
                    "format": params.format,
                    "citation": vancouver,
                    "pmid": identifier,
                    "doi": article.doi,
                    "title": article.title,
                    "journal": article.source,
                    "year": article.pubdate,
                    "url": format!("https://pubmed.ncbi.nlm.nih.gov/{}/", identifier),
                });

                return Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&output).unwrap_or_default(),
                )]));
            }
        }

        return Ok(CallToolResult::success(vec![Content::text(format!(
            "PMID {} not found in PubMed",
            identifier
        ))]));
    }

    // For DOI, provide template
    Ok(CallToolResult::success(vec![Content::text(format!(
        "DOI citation generation not yet implemented.\n\
         Use PMID for automatic citation, or format manually:\n\n\
         Vancouver format:\n\
         Author(s). Title. Journal. Year;Volume(Issue):Pages. doi:{}",
        identifier
    ))]))
}

/// Validate multiple primitives in batch
pub async fn validate_batch(
    params: PrimitiveValidateBatchParams,
) -> Result<CallToolResult, McpError> {
    let mut results = Vec::new();
    let mut validated_count = 0;
    let mut tier_counts = std::collections::HashMap::new();

    for term in &params.terms {
        let validation = validate(PrimitiveValidateParams {
            term: term.clone(),
            domain: params.domain.clone(),
            min_tier: params.min_tier,
            max_citations: 2, // Fewer citations per term in batch mode
        })
        .await;

        if let Ok(result) = validation {
            // Parse result to track statistics
            let text = result
                .content
                .first()
                .and_then(|c| c.as_text())
                .map(|t| t.text.as_str())
                .unwrap_or("");

            let is_validated = text.contains("VALIDATED") && !text.contains("UNVALIDATED");
            if is_validated {
                validated_count += 1;
            }

            // Extract tier
            if text.contains("Tier 1") {
                *tier_counts.entry("Tier 1: Authoritative").or_insert(0) += 1;
            } else if text.contains("Tier 2") {
                *tier_counts.entry("Tier 2: Peer-Reviewed").or_insert(0) += 1;
            } else if text.contains("Tier 3") {
                *tier_counts.entry("Tier 3: Validated Web").or_insert(0) += 1;
            } else {
                *tier_counts.entry("Tier 4: Expert Generation").or_insert(0) += 1;
            }

            results.push(json!({
                "term": term,
                "validated": is_validated,
            }));
        }
    }

    let summary = json!({
        "total": params.terms.len(),
        "validated": validated_count,
        "unvalidated": params.terms.len() - validated_count,
        "validation_rate": format!("{:.0}%", (validated_count as f64 / params.terms.len() as f64) * 100.0),
        "tier_distribution": tier_counts,
        "domain": params.domain,
        "results": results,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&summary).unwrap_or_default(),
    )]))
}

/// List validation tiers and their confidence levels
pub fn validation_tiers() -> Result<CallToolResult, McpError> {
    let output = r#"╭─────────────────────────────────────────────────────────────╮
│ Corpus-Backed Primitive Validation Tiers                    │
╰─────────────────────────────────────────────────────────────╯

TIER 1: AUTHORITATIVE (Confidence: 100%)
  Sources: ICH Guidelines, FDA CFR, EMA GVP, CIOMS
  Example: ICH E2A "Clinical Safety Data Management"
  Citation: Regulatory document with DOI

TIER 2: PEER-REVIEWED (Confidence: 90%)
  Sources: PubMed, Semantic Scholar indexed journals
  Example: PMID:12345678 in Drug Safety journal
  Citation: Vancouver format with PMID/DOI

TIER 3: VALIDATED WEB (Confidence: 80%)
  Sources: WHO, FDA.gov, EMA.europa.eu (domain-filtered)
  Example: WHO Pharmacovigilance guidance
  Citation: URL with access date

TIER 4: EXPERT GENERATION (Confidence: 60%)
  Sources: Model knowledge (no external validation)
  Example: Derived definition without citation
  Citation: None - NOT SUITABLE FOR REGULATORY

─────────────────────────────────────────────────────────────
REGULATORY COMPLIANCE NOTE:
For regulatory submissions (ICSRs, PBRERs, DSURs, RMPs),
primitives MUST be validated at Tier 1 or Tier 2.
Expert Generation Mode (Tier 4) is NOT acceptable for
regulatory documentation requiring citations.
─────────────────────────────────────────────────────────────"#;

    Ok(CallToolResult::success(vec![Content::text(
        output.to_string(),
    )]))
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
