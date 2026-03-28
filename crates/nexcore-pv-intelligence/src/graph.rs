//! Intelligence graph node and edge types.
//!
//! The `IntelligenceGraph` is the central data structure for the
//! pharmacovigilance intelligence layer. It stores three families of typed
//! nodes (Company, Drug, Disease) and the directed edges that connect them.
//!
//! ## T1 Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|-------------|--------|
//! | Node collections | State | ς |
//! | Edge variants | Sum (Coproduct) | Σ |
//! | Option fields (owner, prevalence) | Void | ∅ |
//! | Serialize / Deserialize | Persistence | π |
//! | `::new()` / builders | Existence | ∃ |

use serde::{Deserialize, Serialize};

/// A pharmaceutical company node in the intelligence graph.
///
/// Holds denormalised identity and aggregate counts for fast query access.
///
/// # Examples
///
/// ```
/// use nexcore_pv_intelligence::graph::CompanyNode;
///
/// let node = CompanyNode {
///     id: "lilly".to_string(),
///     name: "Eli Lilly".to_string(),
///     drug_count: 3,
///     therapeutic_areas: vec!["Metabolic".to_string(), "Neuroscience".to_string()],
/// };
/// assert_eq!(node.id, "lilly");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyNode {
    /// Stable string identifier (e.g. `"lilly"`)
    pub id: String,
    /// Display name
    pub name: String,
    /// Number of drugs owned by this company in the graph
    pub drug_count: usize,
    /// Therapeutic areas the company is active in
    pub therapeutic_areas: Vec<String>,
}

/// A drug node in the intelligence graph.
///
/// Carries pharmacological class, ownership linkage, indication linkages,
/// and aggregated signal counts for competitive analysis.
///
/// # Examples
///
/// ```
/// use nexcore_pv_intelligence::graph::DrugNode;
///
/// let node = DrugNode {
///     id: "semaglutide".to_string(),
///     generic_name: "semaglutide".to_string(),
///     drug_class: "GLP-1 Receptor Agonist".to_string(),
///     owner: Some("novo-nordisk".to_string()),
///     indications: vec!["t2dm".to_string(), "obesity".to_string()],
///     signal_count: 4,
///     strongest_prr: Some(3.1),
/// };
/// assert_eq!(node.drug_class, "GLP-1 Receptor Agonist");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugNode {
    /// Stable string identifier (e.g. `"semaglutide"`)
    pub id: String,
    /// INN / generic name
    pub generic_name: String,
    /// Pharmacological mechanism class string
    pub drug_class: String,
    /// ID of the owning company node, if known
    pub owner: Option<String>,
    /// IDs of disease nodes this drug treats
    pub indications: Vec<String>,
    /// Total number of detected pharmacovigilance signals
    pub signal_count: usize,
    /// Strongest PRR across all signals, if any exist
    pub strongest_prr: Option<f64>,
}

/// A disease node in the intelligence graph.
///
/// Captures epidemiological context, therapeutic classification, and
/// the count of drugs in the graph targeting this disease.
///
/// # Examples
///
/// ```
/// use nexcore_pv_intelligence::graph::DiseaseNode;
///
/// let node = DiseaseNode {
///     id: "t2dm".to_string(),
///     name: "Type 2 Diabetes Mellitus".to_string(),
///     therapeutic_area: "Metabolic".to_string(),
///     drug_count: 5,
///     prevalence: Some(0.085),
/// };
/// assert_eq!(node.id, "t2dm");
/// assert_eq!(node.prevalence, Some(0.085));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiseaseNode {
    /// Stable string identifier (e.g. `"t2dm"`)
    pub id: String,
    /// Full display name
    pub name: String,
    /// Therapeutic area classification
    pub therapeutic_area: String,
    /// Number of drugs in the graph treating this disease
    pub drug_count: usize,
    /// Estimated adult population prevalence (0.0–1.0), if known
    pub prevalence: Option<f64>,
}

/// A directed relationship edge between nodes in the intelligence graph.
///
/// Four edge types capture the key PV intelligence relationships:
/// ownership, therapeutic indication, competitive overlap, and class effect.
///
/// ## T1 Grounding
///
/// `Edge` is a Sum type (Σ) — it partitions relationships into four
/// mutually exclusive variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Edge {
    /// A company owns (markets) a drug
    Owns {
        /// Company node ID
        company: String,
        /// Drug node ID
        drug: String,
    },
    /// A drug has a therapeutic indication for a disease
    Treats {
        /// Drug node ID
        drug: String,
        /// Disease node ID
        disease: String,
    },
    /// Two drugs compete in the same indication
    CompetesWith {
        /// First drug node ID
        drug_a: String,
        /// Second drug node ID
        drug_b: String,
        /// Disease node ID (the shared indication)
        disease: String,
    },
    /// Multiple drugs in the same class share an adverse event signal
    ClassEffect {
        /// Drug class label
        drug_class: String,
        /// Adverse event term
        event: String,
        /// Drug node IDs exhibiting the effect
        drugs: Vec<String>,
    },
}

/// The central intelligence graph.
///
/// Stores company, drug, and disease nodes together with typed edges.
/// Populated by [`crate::builder::GraphBuilder`] and queried via
/// [`crate::queries`] methods.
///
/// # Examples
///
/// ```
/// use nexcore_pv_intelligence::builder::GraphBuilder;
///
/// let graph = GraphBuilder::new()
///     .add_company("lilly", "Eli Lilly", &["tirzepatide"])
///     .add_drug("tirzepatide", "tirzepatide", "GLP-1/GIP Dual Agonist", Some("lilly"))
///     .add_disease("t2dm", "Type 2 Diabetes Mellitus", "Metabolic")
///     .add_treats("tirzepatide", "t2dm")
///     .build();
///
/// assert_eq!(graph.companies.len(), 1);
/// assert_eq!(graph.drugs.len(), 1);
/// assert_eq!(graph.diseases.len(), 1);
/// assert_eq!(graph.edges.len(), 2); // Owns + Treats
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntelligenceGraph {
    /// All company nodes
    pub companies: Vec<CompanyNode>,
    /// All drug nodes
    pub drugs: Vec<DrugNode>,
    /// All disease nodes
    pub diseases: Vec<DiseaseNode>,
    /// All relationship edges
    pub edges: Vec<Edge>,
}
