//! # nexcore-pv-intelligence
//!
//! Pharmacovigilance intelligence graph — Company × Drug × Disease
//! composition with competitive analysis queries.
//!
//! ## Architecture
//!
//! ```text
//! Company --owns--> Drug --treats--> Disease
//!    |                |                |
//!    v                v                v
//! CompanyAnalysis  DrugAnalysis   DiseaseAnalysis
//!          \          |           /
//!           nexcore-pv-intelligence
//! ```
//!
//! The graph uses **string IDs** throughout. Entity crates
//! (`nexcore-pharma`, `nexcore-drug`, `nexcore-disease`) supply the data;
//! this crate supplies the query engine. They connect at runtime through
//! the [`builder::GraphBuilder`].
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_pv_intelligence::build_top10_graph;
//!
//! let graph = build_top10_graph();
//! let rankings = graph.safest_company_for_disease("t2dm");
//! assert!(!rankings.is_empty());
//! ```
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol |
//! |---------|-----------|--------|
//! | Graph node collections | State | ς |
//! | Edge variants | Sum (Coproduct) | Σ |
//! | Query pipelines | Sequence | σ |
//! | Node → result transforms | Mapping | μ |
//! | PRR / count comparisons | Comparison | κ |
//! | Signal thresholds | Boundary | ∂ |
//! | Numeric scores | Quantity | N |
//! | Optional fields | Void | ∅ |
//! | Constructors / builders | Existence | ∃ |
//! | Serialize / Deserialize | Persistence | π |
//! | Query trigger → result | Causality | → |
//! | `build()` finalization | Irreversibility | ∝ |
//!
//! ## Modules
//!
//! - [`graph`]: Node types (`CompanyNode`, `DrugNode`, `DiseaseNode`) and `Edge` enum
//! - [`builder`]: `GraphBuilder` — consuming builder for constructing graphs
//! - [`ranking`]: Result types for all query methods
//! - [`queries`]: `IntelligenceGraph` query methods (impl block)

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod builder;
pub mod graph;
pub mod queries;
pub mod ranking;

pub use builder::GraphBuilder;
pub use graph::{CompanyNode, DiseaseNode, DrugNode, Edge, IntelligenceGraph};
pub use ranking::{
    ClassEffectResult, CompanyRanking, HeadToHead, PipelineOverlap, SafetyGap, SharedSignal,
    TherapeuticLandscape,
};

/// Build a pre-seeded graph with top-10 pharma companies, key drugs,
/// and the 10 major disease states with all ownership and indication edges.
///
/// This is the seed graph — enriched at runtime by Station tools.
///
/// # Examples
///
/// ```
/// use nexcore_pv_intelligence::build_top10_graph;
///
/// let graph = build_top10_graph();
/// assert!(graph.companies.len() >= 10);
/// assert!(graph.drugs.len() >= 10);
/// assert!(graph.diseases.len() >= 10);
/// ```
pub fn build_top10_graph() -> IntelligenceGraph {
    GraphBuilder::new()
        // ── Companies ──────────────────────────────────────────────────────
        .add_company(
            "lilly",
            "Eli Lilly",
            &["tirzepatide", "donanemab", "dulaglutide"],
        )
        .add_company(
            "novo-nordisk",
            "Novo Nordisk",
            &["semaglutide", "liraglutide"],
        )
        .add_company("pfizer", "Pfizer", &["paxlovid", "eliquis"])
        .add_company("bms", "Bristol Myers Squibb", &["nivolumab", "apixaban"])
        .add_company("merck", "Merck", &["pembrolizumab", "sitagliptin"])
        .add_company("abbvie", "AbbVie", &["adalimumab", "upadacitinib"])
        .add_company(
            "astrazeneca",
            "AstraZeneca",
            &["dapagliflozin", "osimertinib"],
        )
        .add_company(
            "roche",
            "Roche",
            &["pembrolizumab-biosimilar", "ocrelizumab"],
        )
        .add_company("jnj", "Johnson & Johnson", &["ibrutinib", "ustekinumab"])
        .add_company("novartis", "Novartis", &["secukinumab", "ribociclib"])
        // ── Drugs ──────────────────────────────────────────────────────────
        .add_drug(
            "tirzepatide",
            "tirzepatide",
            "GLP-1/GIP Dual Agonist",
            Some("lilly"),
        )
        .add_drug(
            "donanemab",
            "donanemab",
            "Anti-Amyloid Antibody",
            Some("lilly"),
        )
        .add_drug(
            "dulaglutide",
            "dulaglutide",
            "GLP-1 Receptor Agonist",
            Some("lilly"),
        )
        .add_drug(
            "semaglutide",
            "semaglutide",
            "GLP-1 Receptor Agonist",
            Some("novo-nordisk"),
        )
        .add_drug(
            "liraglutide",
            "liraglutide",
            "GLP-1 Receptor Agonist",
            Some("novo-nordisk"),
        )
        .add_drug(
            "paxlovid",
            "nirmatrelvir/ritonavir",
            "Protease Inhibitor",
            Some("pfizer"),
        )
        .add_drug("eliquis", "apixaban", "Anticoagulant", Some("pfizer"))
        .add_drug(
            "nivolumab",
            "nivolumab",
            "PD-1 Checkpoint Inhibitor",
            Some("bms"),
        )
        .add_drug("apixaban", "apixaban", "Anticoagulant", Some("bms"))
        .add_drug(
            "pembrolizumab",
            "pembrolizumab",
            "PD-1 Checkpoint Inhibitor",
            Some("merck"),
        )
        .add_drug(
            "sitagliptin",
            "sitagliptin",
            "DPP-4 Inhibitor",
            Some("merck"),
        )
        .add_drug("adalimumab", "adalimumab", "Anti-TNF", Some("abbvie"))
        .add_drug(
            "upadacitinib",
            "upadacitinib",
            "JAK Inhibitor",
            Some("abbvie"),
        )
        .add_drug(
            "dapagliflozin",
            "dapagliflozin",
            "SGLT2 Inhibitor",
            Some("astrazeneca"),
        )
        .add_drug(
            "osimertinib",
            "osimertinib",
            "EGFR TKI",
            Some("astrazeneca"),
        )
        .add_drug(
            "pembrolizumab-biosimilar",
            "pembrolizumab biosimilar",
            "PD-1 Checkpoint Inhibitor",
            Some("roche"),
        )
        .add_drug("ocrelizumab", "ocrelizumab", "Anti-CD20", Some("roche"))
        .add_drug("ibrutinib", "ibrutinib", "BTK Inhibitor", Some("jnj"))
        .add_drug("ustekinumab", "ustekinumab", "Anti-IL-12/23", Some("jnj"))
        .add_drug(
            "secukinumab",
            "secukinumab",
            "Anti-IL-17A",
            Some("novartis"),
        )
        .add_drug(
            "ribociclib",
            "ribociclib",
            "CDK4/6 Inhibitor",
            Some("novartis"),
        )
        // ── Diseases ───────────────────────────────────────────────────────
        .add_disease("t2dm", "Type 2 Diabetes Mellitus", "Metabolic")
        .add_disease("obesity", "Obesity", "Metabolic")
        .add_disease("alzheimers", "Alzheimer's Disease", "Neuroscience")
        .add_disease("nsclc", "Non-Small Cell Lung Cancer", "Oncology")
        .add_disease("ra", "Rheumatoid Arthritis", "Immunology")
        .add_disease("breast-cancer", "Breast Cancer (HR+/HER2-)", "Oncology")
        .add_disease("ms", "Multiple Sclerosis", "Neuroscience")
        .add_disease("cll", "Chronic Lymphocytic Leukaemia", "Hematology")
        .add_disease("psoriasis", "Psoriasis", "Dermatology")
        .add_disease("covid19", "COVID-19", "Infectious")
        .add_disease("afib", "Atrial Fibrillation", "Cardiovascular")
        // ── Indication edges ───────────────────────────────────────────────
        .add_treats("tirzepatide", "t2dm")
        .add_treats("tirzepatide", "obesity")
        .add_treats("donanemab", "alzheimers")
        .add_treats("dulaglutide", "t2dm")
        .add_treats("semaglutide", "t2dm")
        .add_treats("semaglutide", "obesity")
        .add_treats("liraglutide", "t2dm")
        .add_treats("liraglutide", "obesity")
        .add_treats("paxlovid", "covid19")
        .add_treats("eliquis", "afib")
        .add_treats("nivolumab", "nsclc")
        .add_treats("apixaban", "afib")
        .add_treats("pembrolizumab", "nsclc")
        .add_treats("pembrolizumab", "breast-cancer")
        .add_treats("sitagliptin", "t2dm")
        .add_treats("adalimumab", "ra")
        .add_treats("adalimumab", "psoriasis")
        .add_treats("upadacitinib", "ra")
        .add_treats("dapagliflozin", "t2dm")
        .add_treats("osimertinib", "nsclc")
        .add_treats("pembrolizumab-biosimilar", "nsclc")
        .add_treats("ocrelizumab", "ms")
        .add_treats("ibrutinib", "cll")
        .add_treats("ustekinumab", "psoriasis")
        .add_treats("secukinumab", "psoriasis")
        .add_treats("secukinumab", "ra")
        .add_treats("ribociclib", "breast-cancer")
        // ── Competition edges ──────────────────────────────────────────────
        .add_competes("semaglutide", "tirzepatide", "t2dm")
        .add_competes("semaglutide", "tirzepatide", "obesity")
        .add_competes("pembrolizumab", "nivolumab", "nsclc")
        .add_competes("adalimumab", "upadacitinib", "ra")
        .add_competes("eliquis", "apixaban", "afib")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_top10_graph_has_expected_counts() {
        let graph = build_top10_graph();
        assert!(graph.companies.len() >= 10, "expected >= 10 companies");
        assert!(graph.drugs.len() >= 10, "expected >= 10 drugs");
        assert!(graph.diseases.len() >= 10, "expected >= 10 diseases");
        assert!(!graph.edges.is_empty(), "expected edges");
    }

    #[test]
    fn build_top10_graph_owns_edges_present() {
        let graph = build_top10_graph();
        let owns_count = graph
            .edges
            .iter()
            .filter(|e| matches!(e, Edge::Owns { .. }))
            .count();
        assert!(
            owns_count >= 10,
            "expected >= 10 Owns edges, got {owns_count}"
        );
    }

    #[test]
    fn build_top10_graph_treats_edges_present() {
        let graph = build_top10_graph();
        let treats_count = graph
            .edges
            .iter()
            .filter(|e| matches!(e, Edge::Treats { .. }))
            .count();
        assert!(
            treats_count >= 10,
            "expected >= 10 Treats edges, got {treats_count}"
        );
    }

    #[test]
    fn build_top10_graph_lilly_owns_tirzepatide() {
        let graph = build_top10_graph();
        let has_edge = graph.edges.iter().any(|e| {
            matches!(e, Edge::Owns { company, drug }
                if company == "lilly" && drug == "tirzepatide")
        });
        assert!(has_edge, "Lilly should own tirzepatide");
    }

    #[test]
    fn build_top10_graph_t2dm_rankings_non_empty() {
        let graph = build_top10_graph();
        let rankings = graph.safest_company_for_disease("t2dm");
        assert!(
            !rankings.is_empty(),
            "safest_company_for_disease(t2dm) should return results"
        );
    }

    #[test]
    fn build_top10_graph_serializes() {
        let graph = build_top10_graph();
        let json = serde_json::to_string(&graph).expect("graph must serialize");
        assert!(json.contains("tirzepatide"));
        assert!(json.contains("t2dm"));
    }
}
