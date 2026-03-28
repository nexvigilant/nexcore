//! Graph builder for constructing `IntelligenceGraph` instances.
//!
//! `GraphBuilder` uses a consuming builder pattern (each method takes
//! ownership of `self` and returns a new `Self`) so call chains compile
//! without intermediate bindings.
//!
//! Edges are inferred automatically from structure:
//! - `add_company(id, name, drugs)` emits an `Edge::Owns` for each listed drug
//!   **after** the drug has been added.
//! - `add_treats(drug, disease)` emits `Edge::Treats`.
//! - `add_competes(drug_a, drug_b, disease)` emits `Edge::CompetesWith`.
//!
//! ## T1 Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|-------------|--------|
//! | Builder accumulation | State | ς |
//! | Constructor `new()` | Existence | ∃ |
//! | `build()` — one-way finalization | Irreversibility | ∝ |
//! | Edge derivation from structure | Causality | → |
//! | Deferred drug_count update | Sequence | σ |

use crate::graph::{CompanyNode, DiseaseNode, DrugNode, Edge, IntelligenceGraph};

/// Builder for [`IntelligenceGraph`].
///
/// Add nodes and relationships in any order, then call [`GraphBuilder::build`]
/// to obtain an immutable graph. The `build` step resolves `drug_count` on
/// company and disease nodes from the edges accumulated during construction.
///
/// # Examples
///
/// ```
/// use nexcore_pv_intelligence::builder::GraphBuilder;
///
/// let graph = GraphBuilder::new()
///     .add_company("pfizer", "Pfizer", &["paxlovid"])
///     .add_drug("paxlovid", "nirmatrelvir/ritonavir", "Protease Inhibitor", Some("pfizer"))
///     .add_disease("covid19", "COVID-19", "Infectious")
///     .add_treats("paxlovid", "covid19")
///     .build();
///
/// assert_eq!(graph.companies[0].drug_count, 1);
/// assert_eq!(graph.diseases[0].drug_count, 1);
/// ```
#[derive(Debug, Default)]
pub struct GraphBuilder {
    companies: Vec<CompanyNode>,
    drugs: Vec<DrugNode>,
    diseases: Vec<DiseaseNode>,
    edges: Vec<Edge>,
    /// Pending ownership associations: (company_id, drug_id). Resolved at build.
    pending_owns: Vec<(String, String)>,
}

impl GraphBuilder {
    /// Create an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a company node and record ownership edges for its listed drugs.
    ///
    /// The `drugs` slice contains drug IDs. Ownership edges are emitted
    /// for any drug ID that resolves to a `DrugNode` present in the graph
    /// at `build()` time. Drugs listed here but not yet added are still
    /// linked — the pending association is resolved at build.
    pub fn add_company(mut self, id: &str, name: &str, drugs: &[&str]) -> Self {
        self.companies.push(CompanyNode {
            id: id.to_string(),
            name: name.to_string(),
            drug_count: 0, // resolved at build()
            therapeutic_areas: vec![],
        });
        for drug_id in drugs {
            self.pending_owns
                .push((id.to_string(), drug_id.to_string()));
        }
        self
    }

    /// Add a drug node.
    ///
    /// `owner` is an optional company ID. If supplied, an `Edge::Owns` is
    /// also emitted (complementing any pending_owns entry from `add_company`).
    pub fn add_drug(mut self, id: &str, name: &str, class: &str, owner: Option<&str>) -> Self {
        self.drugs.push(DrugNode {
            id: id.to_string(),
            generic_name: name.to_string(),
            drug_class: class.to_string(),
            owner: owner.map(|s| s.to_string()),
            indications: vec![],
            signal_count: 0,
            strongest_prr: None,
        });
        self
    }

    /// Add a disease node.
    pub fn add_disease(mut self, id: &str, name: &str, area: &str) -> Self {
        self.diseases.push(DiseaseNode {
            id: id.to_string(),
            name: name.to_string(),
            therapeutic_area: area.to_string(),
            drug_count: 0, // resolved at build()
            prevalence: None,
        });
        self
    }

    /// Record that a drug treats a disease (emits `Edge::Treats`).
    pub fn add_treats(mut self, drug: &str, disease: &str) -> Self {
        // Record indication on the drug node if present
        if let Some(d) = self.drugs.iter_mut().find(|d| d.id == drug) {
            if !d.indications.contains(&disease.to_string()) {
                d.indications.push(disease.to_string());
            }
        }
        self.edges.push(Edge::Treats {
            drug: drug.to_string(),
            disease: disease.to_string(),
        });
        self
    }

    /// Record that two drugs compete in the same indication (emits `Edge::CompetesWith`).
    pub fn add_competes(mut self, drug_a: &str, drug_b: &str, disease: &str) -> Self {
        self.edges.push(Edge::CompetesWith {
            drug_a: drug_a.to_string(),
            drug_b: drug_b.to_string(),
            disease: disease.to_string(),
        });
        self
    }

    /// Finalize and return the intelligence graph.
    ///
    /// This step:
    /// 1. Emits `Edge::Owns` for all pending ownership associations.
    /// 2. Resolves `drug_count` on company nodes.
    /// 3. Resolves `drug_count` on disease nodes from `Edge::Treats` edges.
    /// 4. Resolves `therapeutic_areas` on company nodes from owned drugs.
    pub fn build(mut self) -> IntelligenceGraph {
        // Step 1 — emit Owns edges from pending associations (dedup)
        for (company_id, drug_id) in &self.pending_owns {
            let already_exists = self.edges.iter().any(|e| {
                matches!(e, Edge::Owns { company, drug }
                    if company == company_id && drug == drug_id)
            });
            if !already_exists {
                self.edges.push(Edge::Owns {
                    company: company_id.clone(),
                    drug: drug_id.clone(),
                });
            }
            // Backfill owner on drug node if not set
            if let Some(d) = self.drugs.iter_mut().find(|d| &d.id == drug_id) {
                if d.owner.is_none() {
                    d.owner = Some(company_id.clone());
                }
            }
        }

        // Step 2 — resolve drug_count on company nodes
        for company in &mut self.companies {
            company.drug_count = self
                .edges
                .iter()
                .filter(|e| matches!(e, Edge::Owns { company: c, .. } if c == &company.id))
                .count();
        }

        // Step 3 — resolve drug_count on disease nodes
        for disease in &mut self.diseases {
            disease.drug_count = self
                .edges
                .iter()
                .filter(|e| matches!(e, Edge::Treats { disease: d, .. } if d == &disease.id))
                .count();
        }

        // Step 4 — resolve therapeutic_areas on company nodes from owned drugs
        for company in &mut self.companies {
            let owned_drug_ids: Vec<String> = self
                .edges
                .iter()
                .filter_map(|e| match e {
                    Edge::Owns { company: c, drug } if c == &company.id => Some(drug.clone()),
                    _ => None,
                })
                .collect();

            let mut areas: Vec<String> = owned_drug_ids
                .iter()
                .filter_map(|drug_id| {
                    self.drugs.iter().find(|d| &d.id == drug_id).and_then(|d| {
                        // Look up disease therapeutic_area via Treats edges
                        d.indications.first().and_then(|disease_id| {
                            self.diseases
                                .iter()
                                .find(|dis| &dis.id == disease_id)
                                .map(|dis| dis.therapeutic_area.clone())
                        })
                    })
                })
                .collect();
            areas.sort();
            areas.dedup();
            company.therapeutic_areas = areas;
        }

        IntelligenceGraph {
            companies: self.companies,
            drugs: self.drugs,
            diseases: self.diseases,
            edges: self.edges,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mini_graph() -> IntelligenceGraph {
        GraphBuilder::new()
            .add_company("acme", "Acme Pharma", &["drugX", "drugY"])
            .add_drug("drugX", "drug-x", "GLP-1 Agonist", Some("acme"))
            .add_drug("drugY", "drug-y", "GLP-1 Agonist", None)
            .add_disease("t2dm", "Type 2 Diabetes", "Metabolic")
            .add_treats("drugX", "t2dm")
            .add_treats("drugY", "t2dm")
            .build()
    }

    #[test]
    fn build_resolves_company_drug_count() {
        let g = mini_graph();
        let acme = g.companies.iter().find(|c| c.id == "acme").unwrap();
        assert_eq!(acme.drug_count, 2);
    }

    #[test]
    fn build_resolves_disease_drug_count() {
        let g = mini_graph();
        let t2dm = g.diseases.iter().find(|d| d.id == "t2dm").unwrap();
        assert_eq!(t2dm.drug_count, 2);
    }

    #[test]
    fn build_backfills_drug_owner() {
        let g = mini_graph();
        let drug_y = g.drugs.iter().find(|d| d.id == "drugY").unwrap();
        // pending_owns from add_company backfills owner
        assert_eq!(drug_y.owner.as_deref(), Some("acme"));
    }

    #[test]
    fn add_treats_records_indication_on_drug() {
        let g = mini_graph();
        let drug_x = g.drugs.iter().find(|d| d.id == "drugX").unwrap();
        assert!(drug_x.indications.contains(&"t2dm".to_string()));
    }

    #[test]
    fn add_competes_emits_edge() {
        let g = GraphBuilder::new()
            .add_drug("a", "alpha", "Class1", None)
            .add_drug("b", "beta", "Class1", None)
            .add_disease("d1", "Disease One", "Oncology")
            .add_competes("a", "b", "d1")
            .build();
        let has_competes = g.edges.iter().any(|e| {
            matches!(e, crate::graph::Edge::CompetesWith { drug_a, drug_b, disease }
                if drug_a == "a" && drug_b == "b" && disease == "d1")
        });
        assert!(has_competes);
    }

    #[test]
    fn owns_edges_not_duplicated() {
        // add_company lists drug_a; add_drug does NOT supply owner → only one Owns edge
        let g = GraphBuilder::new()
            .add_company("co", "Co", &["drug_a"])
            .add_drug("drug_a", "Drug A", "Class", None)
            .build();
        let owns_count = g
            .edges
            .iter()
            .filter(|e| matches!(e, Edge::Owns { .. }))
            .count();
        assert_eq!(owns_count, 1);
    }
}
