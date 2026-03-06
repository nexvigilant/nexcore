//! PubMed — pharmacovigilance literature config.
//!
//! PubMed indexes over 37 million biomedical citations including the primary
//! PV literature: signal detection methodology, disproportionality analyses,
//! drug-induced case reports, systematic reviews, and post-marketing surveillance
//! studies.
//!
//! Tools cover PV-optimised literature search, abstract extraction, case report
//! identification, signal detection paper retrieval, systematic review browsing,
//! MeSH-term navigation, and citation network exploration.

use serde_json::json;

use crate::config::{PvVertical, StationConfig};
use crate::StationBuilder;

/// Standard tags applied to every PubMed tool.
const TAGS: &[&str] = &[
    "pharmacovigilance",
    "nexvigilant",
    "patient-safety",
    "pubmed",
    "literature",
    "signal-detection",
    "evidence-synthesis",
];

/// Build the PubMed pharmacovigilance literature station config.
///
/// Returns a [`StationConfig`] with 7 tools covering PV-optimised literature
/// search, abstract extraction, case report retrieval, signal detection papers,
/// systematic review browsing, MeSH term navigation, and related-article
/// traversal.
pub fn config() -> StationConfig {
    StationBuilder::new(
        PvVertical::PubMed,
        "PubMed — Pharmacovigilance Literature",
    )
    .description(
        "Search and extract pharmacovigilance literature from PubMed, including \
         case reports, signal detection studies, disproportionality analyses, \
         systematic reviews of drug safety, and post-marketing surveillance \
         publications. Tools are tuned for PV evidence synthesis: \
         MeSH-aware searches, case report filters, methodology classifiers, \
         and citation network traversal. Covers 37+ million indexed citations.",
    )
    .tags(TAGS.iter().copied())
    // Tool 1 — PV-focused literature search
    .fill_tool(
        "search-pv-literature",
        "Search PubMed for pharmacovigilance literature using drug names, \
         adverse event terms, or MeSH headings. Supports article type filters \
         (case reports, systematic reviews, clinical trials) and date ranges. \
         Returns PMIDs, titles, journal, publication year, and article type.",
        "/",
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Free-text or MeSH query (e.g. 'atorvastatin[MeSH] AND rhabdomyolysis')"
                },
                "drug": {
                    "type": "string",
                    "description": "Drug or substance name (automatically combined with [Substance Name] MeSH tag)"
                },
                "adverseEvent": {
                    "type": "string",
                    "description": "Adverse event term (combined with MeSH Drug-Related Side Effects heading)"
                },
                "articleType": {
                    "type": "string",
                    "enum": [
                        "case-reports",
                        "systematic-review",
                        "meta-analysis",
                        "clinical-trial",
                        "observational-study",
                        "review"
                    ],
                    "description": "PubMed publication type filter"
                },
                "dateFrom": {
                    "type": "string",
                    "description": "Start date for publication range (YYYY or YYYY/MM/DD)"
                },
                "dateTo": {
                    "type": "string",
                    "description": "End date for publication range (YYYY or YYYY/MM/DD)"
                },
                "maxResults": {
                    "type": "integer",
                    "default": 20,
                    "description": "Number of results to return (default 20, max 200)"
                },
                "sortBy": {
                    "type": "string",
                    "enum": ["relevance", "date", "pub_date"],
                    "default": "relevance",
                    "description": "Sort order for results"
                }
            }
        }),
    )
    // Tool 2 — article abstract extraction
    .fill_tool(
        "get-abstract",
        "Extract the full abstract, authors, journal, DOI, and MeSH terms for \
         a PubMed article identified by PMID. Returns structured citation data \
         suitable for downstream NLP processing or evidence table construction.",
        "/{pmid}/",
        json!({
            "type": "object",
            "required": ["pmid"],
            "properties": {
                "pmid": {
                    "type": "string",
                    "description": "PubMed identifier (e.g. '37123456')"
                }
            }
        }),
    )
    // Tool 3 — case report search (specific to individual patient reports)
    .fill_tool(
        "search-case-reports",
        "Search PubMed specifically for case reports and case series describing \
         adverse drug reactions for a given substance-event pair. Returns ranked \
         results with PMID, title, and publication year. Optimised for \
         pharmacovigilance hypothesis generation and signal validation.",
        "/",
        json!({
            "type": "object",
            "required": ["drug"],
            "properties": {
                "drug": {
                    "type": "string",
                    "description": "Drug or active substance name (e.g. 'clozapine')"
                },
                "adverseEvent": {
                    "type": "string",
                    "description": "Adverse event or reaction term (e.g. 'agranulocytosis')"
                },
                "minYear": {
                    "type": "integer",
                    "description": "Earliest publication year to include"
                },
                "maxResults": {
                    "type": "integer",
                    "default": 30,
                    "description": "Maximum case reports to return"
                }
            }
        }),
    )
    // Tool 4 — signal detection methodology papers
    .fill_tool(
        "search-signal-detection-papers",
        "Retrieve PubMed publications on pharmacovigilance signal detection \
         methodology, including disproportionality analysis (PRR, ROR, IC, EBGM), \
         machine learning methods, natural language processing for ICSR mining, \
         and population-based pharmacoepidemiology. Returns papers filtered \
         by method class and application domain.",
        "/",
        json!({
            "type": "object",
            "properties": {
                "method": {
                    "type": "string",
                    "enum": [
                        "disproportionality",
                        "prr",
                        "ror",
                        "ic",
                        "ebgm",
                        "machine-learning",
                        "nlp",
                        "pharmacoepidemiology",
                        "bayesian"
                    ],
                    "description": "Signal detection method class to filter by"
                },
                "applicationDomain": {
                    "type": "string",
                    "description": "Application domain (e.g. 'vaccines', 'oncology', 'paediatric')"
                },
                "dateFrom": {
                    "type": "string",
                    "description": "Publication start year (YYYY)"
                },
                "maxResults": {
                    "type": "integer",
                    "default": 25,
                    "description": "Maximum results to return"
                }
            }
        }),
    )
    // Tool 5 — systematic reviews and meta-analyses of drug safety
    .fill_tool(
        "search-systematic-reviews",
        "Browse PubMed systematic reviews and meta-analyses focused on drug \
         safety outcomes. Returns structured safety evidence at the highest \
         level of the evidence hierarchy. Supports benefit-risk analysis, \
         PSUR preparation, and label update decisions.",
        "/",
        json!({
            "type": "object",
            "properties": {
                "drug": {
                    "type": "string",
                    "description": "Drug or drug class (e.g. 'SGLT2 inhibitors', 'warfarin')"
                },
                "safetyOutcome": {
                    "type": "string",
                    "description": "Safety outcome of interest (e.g. 'cardiovascular events', 'hepatotoxicity')"
                },
                "reviewType": {
                    "type": "string",
                    "enum": ["systematic-review", "meta-analysis", "both"],
                    "default": "both",
                    "description": "Type of synthesised evidence to retrieve"
                },
                "dateFrom": {
                    "type": "string",
                    "description": "Start year filter (YYYY)"
                },
                "maxResults": {
                    "type": "integer",
                    "default": 15,
                    "description": "Maximum results to return"
                }
            }
        }),
    )
    // Tool 6 — MeSH term browsing for PV ontology alignment
    .fill_tool(
        "browse-mesh-terms",
        "Navigate PubMed MeSH (Medical Subject Headings) to find standardised \
         drug and adverse event terminology. Returns the MeSH hierarchy, \
         preferred term, entry terms (synonyms), and scope note. \
         Use to align free-text queries to controlled vocabulary before \
         running signal detection searches.",
        "/mesh/",
        json!({
            "type": "object",
            "required": ["term"],
            "properties": {
                "term": {
                    "type": "string",
                    "description": "Drug name, adverse event, or partial term to look up in MeSH"
                }
            }
        }),
    )
    // Tool 7 — related articles (citation network traversal)
    .fill_tool(
        "get-related-articles",
        "Retrieve PubMed articles computationally similar to a seed PMID, \
         using PubMed's related articles algorithm (word adjacency + MeSH \
         co-occurrence). Returns up to 20 related PMIDs with titles and \
         similarity scores. Supports lateral citation network traversal \
         during evidence landscape mapping.",
        "/{pmid}/",
        json!({
            "type": "object",
            "required": ["pmid"],
            "properties": {
                "pmid": {
                    "type": "string",
                    "description": "Seed PubMed identifier (e.g. '37123456')"
                },
                "maxResults": {
                    "type": "integer",
                    "default": 20,
                    "description": "Number of related articles to return (max 50)"
                },
                "articleType": {
                    "type": "string",
                    "enum": [
                        "case-reports",
                        "systematic-review",
                        "clinical-trial",
                        "any"
                    ],
                    "default": "any",
                    "description": "Optional filter on related article type"
                }
            }
        }),
    )
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AccessTier, ExecutionType, PvVertical};

    #[test]
    fn pubmed_domain() {
        let cfg = config();
        assert_eq!(cfg.domain, "pubmed.ncbi.nlm.nih.gov");
        assert_eq!(cfg.vertical, PvVertical::PubMed);
    }

    #[test]
    fn pubmed_tool_count() {
        let cfg = config();
        assert_eq!(cfg.total_tools(), 7);
    }

    #[test]
    fn pubmed_all_public() {
        let cfg = config();
        assert_eq!(cfg.public_tool_count(), 7);
        assert_eq!(cfg.gated_tool_count(), 0);
        assert_eq!(cfg.premium_tool_count(), 0);
    }

    #[test]
    fn pubmed_all_tools_fill() {
        let cfg = config();
        for tool in &cfg.tools {
            assert_eq!(
                tool.execution_type,
                ExecutionType::Fill,
                "tool '{}' should be Fill",
                tool.name
            );
        }
    }

    #[test]
    fn pubmed_all_tools_have_input_schema() {
        let cfg = config();
        for tool in &cfg.tools {
            assert!(
                tool.input_schema.is_some(),
                "tool '{}' is missing input_schema",
                tool.name
            );
        }
    }

    #[test]
    fn pubmed_required_tags() {
        let cfg = config();
        let required = ["pharmacovigilance", "nexvigilant", "patient-safety"];
        for tag in required {
            assert!(
                cfg.tags.iter().any(|t| t == tag),
                "config missing required tag '{tag}'"
            );
        }
    }

    #[test]
    fn pubmed_has_disclaimer() {
        let cfg = config();
        assert!(cfg.description.contains("DISCLAIMER"));
        assert!(cfg.description.contains("NexVigilant"));
    }

    #[test]
    fn pubmed_contributor() {
        let cfg = config();
        assert_eq!(cfg.contributor, "MatthewCampCorp");
    }

    #[test]
    fn get_abstract_requires_pmid() {
        let cfg = config();
        let tool = cfg.tools.iter().find(|t| t.name == "get-abstract").unwrap();
        let schema = tool.input_schema.as_ref().unwrap();
        let required = schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "pmid"));
    }

    #[test]
    fn search_pv_literature_has_article_type_enum() {
        let cfg = config();
        let tool = cfg
            .tools
            .iter()
            .find(|t| t.name == "search-pv-literature")
            .unwrap();
        let schema = tool.input_schema.as_ref().unwrap();
        let article_type_enum = &schema["properties"]["articleType"]["enum"];
        assert!(article_type_enum.is_array());
        // case-reports, systematic-review, meta-analysis, clinical-trial, observational-study, review
        assert_eq!(article_type_enum.as_array().unwrap().len(), 6);
    }

    #[test]
    fn signal_detection_method_enum_includes_ebgm() {
        let cfg = config();
        let tool = cfg
            .tools
            .iter()
            .find(|t| t.name == "search-signal-detection-papers")
            .unwrap();
        let schema = tool.input_schema.as_ref().unwrap();
        let method_enum = &schema["properties"]["method"]["enum"];
        assert!(method_enum
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "ebgm"));
    }

    #[test]
    fn search_case_reports_requires_drug() {
        let cfg = config();
        let tool = cfg
            .tools
            .iter()
            .find(|t| t.name == "search-case-reports")
            .unwrap();
        let schema = tool.input_schema.as_ref().unwrap();
        let required = schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "drug"));
    }
}
