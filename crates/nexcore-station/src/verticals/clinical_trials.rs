//! ClinicalTrials.gov — clinical trial safety data config.
//!
//! ClinicalTrials.gov is the world's largest registry of clinical studies,
//! hosting results including adverse event tables, serious adverse event (SAE)
//! summaries, and primary/secondary outcome data from over 480,000 studies.
//!
//! Tools cover study search with safety-oriented filters, SAE table extraction,
//! adverse event frequency browsing by arm, protocol safety section retrieval,
//! phase and status filtering, and study-level results comparison. Together
//! they enable agents to perform cross-trial safety evidence synthesis.

use serde_json::json;

use crate::config::{PvVertical, StationConfig};
use crate::StationBuilder;

/// Standard tags applied to every ClinicalTrials tool.
const TAGS: &[&str] = &[
    "pharmacovigilance",
    "nexvigilant",
    "patient-safety",
    "clinical-trials",
    "sae",
    "adverse-events",
    "rct",
    "study-results",
];

/// Build the ClinicalTrials.gov station config.
///
/// Returns a [`StationConfig`] with 8 tools covering study search, SAE
/// extraction, AE arm comparison, protocol safety sections, DSM board
/// composition, phase/status filtering, results download, and multi-study
/// safety comparison.
pub fn config() -> StationConfig {
    StationBuilder::new(
        PvVertical::ClinicalTrials,
        "ClinicalTrials.gov — Clinical Trial Safety Data",
    )
    .description(
        "Access structured safety data from ClinicalTrials.gov, the world's \
         largest clinical study registry. Tools retrieve serious adverse event \
         (SAE) tables, adverse event frequency by treatment arm, protocol safety \
         management sections, Data Safety Monitoring Board (DSMB) information, \
         and study results filtered by phase or therapeutic area. Supports \
         clinical trial safety evidence synthesis and benefit-risk analysis.",
    )
    .tags(TAGS.iter().copied())
    // Tool 1 — study search with PV-relevant filters
    .fill_tool(
        "search-trials",
        "Search ClinicalTrials.gov for studies matching a drug, condition, or \
         intervention. Supports filtering by phase, recruitment status, age group, \
         and sponsor. Returns NCT IDs, titles, status, and phase for matched studies.",
        "/search",
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Drug name, condition, or free-text query (e.g. 'atorvastatin myopathy')"
                },
                "intervention": {
                    "type": "string",
                    "description": "Drug or device intervention name"
                },
                "condition": {
                    "type": "string",
                    "description": "Disease or condition (MeSH term or free text)"
                },
                "phase": {
                    "type": "string",
                    "enum": ["PHASE1", "PHASE2", "PHASE3", "PHASE4", "NA"],
                    "description": "Trial phase filter"
                },
                "status": {
                    "type": "string",
                    "enum": [
                        "RECRUITING",
                        "ACTIVE_NOT_RECRUITING",
                        "COMPLETED",
                        "TERMINATED",
                        "WITHDRAWN",
                        "SUSPENDED"
                    ],
                    "description": "Recruitment status filter"
                },
                "ageGroup": {
                    "type": "string",
                    "enum": ["CHILD", "ADULT", "OLDER_ADULT"],
                    "description": "Participant age group filter"
                },
                "maxResults": {
                    "type": "integer",
                    "default": 25,
                    "description": "Maximum results to return (default 25, max 100)"
                }
            }
        }),
    )
    // Tool 2 — SAE table for a specific study
    .fill_tool(
        "get-sae-table",
        "Extract the serious adverse event (SAE) table from a ClinicalTrials.gov \
         study results page. Returns SAE category, affected participants per arm, \
         and total counts. Essential for cross-trial SAE frequency comparison.",
        "/study/{nctId}/results",
        json!({
            "type": "object",
            "required": ["nctId"],
            "properties": {
                "nctId": {
                    "type": "string",
                    "pattern": "^NCT\\d{8}$",
                    "description": "ClinicalTrials.gov identifier (e.g. 'NCT01234567')"
                }
            }
        }),
    )
    // Tool 3 — full adverse event table by treatment arm
    .fill_tool(
        "get-ae-by-arm",
        "Retrieve the full adverse event frequency table for a clinical study, \
         broken down by treatment arm. Returns event terms, participant counts, \
         and event rates per arm. Supports safety signal detection across \
         experimental vs. control groups.",
        "/study/{nctId}/results",
        json!({
            "type": "object",
            "required": ["nctId"],
            "properties": {
                "nctId": {
                    "type": "string",
                    "pattern": "^NCT\\d{8}$",
                    "description": "ClinicalTrials.gov identifier (e.g. 'NCT01234567')"
                },
                "arm": {
                    "type": "string",
                    "description": "Optional arm label filter (e.g. 'Placebo', 'Drug 10mg')"
                },
                "threshold": {
                    "type": "number",
                    "description": "Optional minimum event frequency (%) to include in output"
                }
            }
        }),
    )
    // Tool 4 — protocol safety section (oversight, stopping rules, monitoring)
    .fill_tool(
        "get-safety-protocol",
        "Extract the safety management sections from a study protocol on \
         ClinicalTrials.gov, including stopping rules, safety monitoring plan \
         description, and Data Safety Monitoring Board (DSMB) oversight details. \
         Relevant for protocol quality assessment and risk-based monitoring review.",
        "/study/{nctId}",
        json!({
            "type": "object",
            "required": ["nctId"],
            "properties": {
                "nctId": {
                    "type": "string",
                    "pattern": "^NCT\\d{8}$",
                    "description": "ClinicalTrials.gov identifier (e.g. 'NCT01234567')"
                }
            }
        }),
    )
    // Tool 5 — browse results with phase + status filters
    .fill_tool(
        "browse-results",
        "Browse ClinicalTrials.gov results pages filtered by phase, status, and \
         therapeutic area. Returns a paginated list of studies with posted results, \
         including primary completion date and whether safety data is available. \
         Useful for scoping an evidence landscape before targeted extraction.",
        "/search",
        json!({
            "type": "object",
            "properties": {
                "condition": {
                    "type": "string",
                    "description": "Therapeutic area or disease condition"
                },
                "phase": {
                    "type": "string",
                    "enum": ["PHASE1", "PHASE2", "PHASE3", "PHASE4"],
                    "description": "Phase filter"
                },
                "hasResults": {
                    "type": "boolean",
                    "default": true,
                    "description": "If true, only return studies with posted results"
                },
                "sponsor": {
                    "type": "string",
                    "description": "Sponsor or lead organisation name filter"
                },
                "page": {
                    "type": "integer",
                    "default": 1,
                    "description": "Results page number"
                }
            }
        }),
    )
    // Tool 6 — primary outcomes and safety endpoints
    .fill_tool(
        "get-safety-endpoints",
        "Retrieve the primary and secondary outcome measures for a study, \
         with focus on safety-designated endpoints. Returns outcome description, \
         time frame, and posted metric values where available. Supports \
         endpoint-level benefit-risk decomposition.",
        "/study/{nctId}",
        json!({
            "type": "object",
            "required": ["nctId"],
            "properties": {
                "nctId": {
                    "type": "string",
                    "pattern": "^NCT\\d{8}$",
                    "description": "ClinicalTrials.gov identifier"
                },
                "endpointType": {
                    "type": "string",
                    "enum": ["PRIMARY", "SECONDARY", "ALL"],
                    "default": "ALL",
                    "description": "Filter by endpoint type"
                }
            }
        }),
    )
    // Tool 7 — study demographics (enrolled population safety context)
    .fill_tool(
        "get-study-demographics",
        "Extract enrolled population demographics from a study's results page \
         on ClinicalTrials.gov. Returns age distribution, sex breakdown, \
         race/ethnicity where reported, and total enrolment by arm. Provides \
         population context for interpreting adverse event rates.",
        "/study/{nctId}/results",
        json!({
            "type": "object",
            "required": ["nctId"],
            "properties": {
                "nctId": {
                    "type": "string",
                    "pattern": "^NCT\\d{8}$",
                    "description": "ClinicalTrials.gov identifier"
                }
            }
        }),
    )
    // Tool 8 — navigate to study record (entry point for manual agent browsing)
    .navigate_tool(
        "view-study",
        "Navigate directly to a ClinicalTrials.gov study record by NCT ID. \
         Returns the full study page for an agent to inspect protocol details, \
         eligibility criteria, contacts, and posted results. Use before targeted \
         extraction tools when the study structure is unknown.",
        "/study/{nctId}",
    )
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AccessTier, ExecutionType, PvVertical};

    #[test]
    fn clinical_trials_domain() {
        let cfg = config();
        assert_eq!(cfg.domain, "clinicaltrials.gov");
        assert_eq!(cfg.vertical, PvVertical::ClinicalTrials);
    }

    #[test]
    fn clinical_trials_tool_count() {
        let cfg = config();
        assert_eq!(cfg.total_tools(), 8);
    }

    #[test]
    fn clinical_trials_all_public() {
        let cfg = config();
        assert_eq!(cfg.public_tool_count(), 8);
        assert_eq!(cfg.gated_tool_count(), 0);
        assert_eq!(cfg.premium_tool_count(), 0);
    }

    #[test]
    fn clinical_trials_mix_of_execution_types() {
        let cfg = config();
        let fill_count = cfg
            .tools
            .iter()
            .filter(|t| t.execution_type == ExecutionType::Fill)
            .count();
        let navigate_count = cfg
            .tools
            .iter()
            .filter(|t| t.execution_type == ExecutionType::Navigate)
            .count();
        // 7 fill tools + 1 navigate tool
        assert_eq!(fill_count, 7);
        assert_eq!(navigate_count, 1);
    }

    #[test]
    fn clinical_trials_required_tags() {
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
    fn clinical_trials_has_disclaimer() {
        let cfg = config();
        assert!(cfg.description.contains("DISCLAIMER"));
        assert!(cfg.description.contains("NexVigilant"));
    }

    #[test]
    fn clinical_trials_contributor() {
        let cfg = config();
        assert_eq!(cfg.contributor, "MatthewCampCorp");
    }

    #[test]
    fn sae_table_requires_nct_id() {
        let cfg = config();
        let tool = cfg.tools.iter().find(|t| t.name == "get-sae-table").unwrap();
        let schema = tool.input_schema.as_ref().unwrap();
        let required = schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v == "nctId"));
    }

    #[test]
    fn search_trials_phase_enum_correct() {
        let cfg = config();
        let tool = cfg.tools.iter().find(|t| t.name == "search-trials").unwrap();
        let schema = tool.input_schema.as_ref().unwrap();
        let phase_enum = &schema["properties"]["phase"]["enum"];
        assert!(phase_enum.is_array());
        // PHASE1 through PHASE4 plus NA
        assert_eq!(phase_enum.as_array().unwrap().len(), 5);
    }

    #[test]
    fn view_study_has_no_input_schema() {
        let cfg = config();
        let tool = cfg.tools.iter().find(|t| t.name == "view-study").unwrap();
        // Navigate tools don't carry an input schema in the builder
        assert!(tool.input_schema.is_none());
    }
}
