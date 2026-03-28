//! Intelligence query methods on `IntelligenceGraph`.
//!
//! All queries are pure read operations over the graph — no mutation,
//! no I/O. Each method returns an owned result type from [`crate::ranking`].
//!
//! ## T1 Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|-------------|--------|
//! | Query pipelines | Sequence | σ |
//! | Node → result transform | Mapping | μ |
//! | PRR / count comparisons | Comparison | κ |
//! | Signal thresholds | Boundary | ∂ |
//! | Numeric scores | Quantity | N |
//! | Missing data | Void | ∅ |
//! | Query trigger → result | Causality | → |

use crate::{
    graph::{Edge, IntelligenceGraph},
    ranking::{
        ClassEffectResult, CompanyRanking, HeadToHead, PipelineOverlap, SafetyGap, SharedSignal,
        TherapeuticLandscape,
    },
};

impl IntelligenceGraph {
    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Drugs (by ID) that treat the given disease.
    fn drugs_for_disease<'a>(&'a self, disease_id: &str) -> Vec<&'a crate::graph::DrugNode> {
        let drug_ids: Vec<&str> = self
            .edges
            .iter()
            .filter_map(|e| match e {
                Edge::Treats { drug, disease } if disease == disease_id => Some(drug.as_str()),
                _ => None,
            })
            .collect();

        self.drugs
            .iter()
            .filter(|d| drug_ids.contains(&d.id.as_str()))
            .collect()
    }

    /// Owner company ID for a drug, resolved from Owns edges (fallback to DrugNode.owner).
    fn owner_of(&self, drug_id: &str) -> Option<&str> {
        self.edges
            .iter()
            .find_map(|e| match e {
                Edge::Owns { company, drug } if drug == drug_id => Some(company.as_str()),
                _ => None,
            })
            .or_else(|| {
                self.drugs
                    .iter()
                    .find(|d| d.id == drug_id)
                    .and_then(|d| d.owner.as_deref())
            })
    }

    /// Build a `CompanyRanking` for a company over a specific set of drug IDs.
    fn rank_company(
        &self,
        company_id: &str,
        drug_ids: &[String],
        rank: u32,
    ) -> Option<CompanyRanking> {
        let company = self.companies.iter().find(|c| c.id == company_id)?;

        let mut total_prr = 0.0_f64;
        let mut signal_count = 0_usize;
        let mut boxed_warnings = 0_u32;

        for drug_id in drug_ids {
            if let Some(drug) = self.drugs.iter().find(|d| &d.id == drug_id) {
                total_prr += drug.strongest_prr.unwrap_or(1.0) * drug.signal_count as f64;
                signal_count += drug.signal_count;
                // Use signal_count as a proxy for boxed-warning weight
                // (callers can enrich nodes with real boxed_warning data)
                if drug.strongest_prr.unwrap_or(0.0) > 3.0 {
                    boxed_warnings += 1;
                }
            }
        }

        let avg_signal_strength = if signal_count > 0 {
            total_prr / signal_count as f64
        } else {
            1.0 // background rate
        };

        Some(CompanyRanking {
            company: company.name.clone(),
            drugs: drug_ids.to_vec(),
            avg_signal_strength,
            boxed_warnings,
            rank,
        })
    }

    // -----------------------------------------------------------------------
    // Public query API
    // -----------------------------------------------------------------------

    /// Rank companies by portfolio safety for a given disease.
    ///
    /// Only companies that own at least one drug treating `disease_id` are
    /// included. Companies are sorted ascending by `avg_signal_strength`
    /// (lower PRR = safer), with rank 1 being the safest.
    ///
    /// Returns an empty `Vec` when no drugs treat the disease.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_pv_intelligence::builder::GraphBuilder;
    ///
    /// let graph = GraphBuilder::new()
    ///     .add_company("co_a", "Company A", &["drug1"])
    ///     .add_company("co_b", "Company B", &["drug2"])
    ///     .add_drug("drug1", "Drug 1", "Class A", Some("co_a"))
    ///     .add_drug("drug2", "Drug 2", "Class A", Some("co_b"))
    ///     .add_disease("dis1", "Disease 1", "Metabolic")
    ///     .add_treats("drug1", "dis1")
    ///     .add_treats("drug2", "dis1")
    ///     .build();
    ///
    /// let rankings = graph.safest_company_for_disease("dis1");
    /// assert_eq!(rankings.len(), 2);
    /// assert_eq!(rankings[0].rank, 1);
    /// ```
    pub fn safest_company_for_disease(&self, disease_id: &str) -> Vec<CompanyRanking> {
        let drugs = self.drugs_for_disease(disease_id);
        if drugs.is_empty() {
            return vec![];
        }

        // Group drugs by owning company
        let mut by_company: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for drug in &drugs {
            let owner = self.owner_of(&drug.id).unwrap_or("unknown").to_string();
            by_company.entry(owner).or_default().push(drug.id.clone());
        }

        // Build preliminary rankings (rank=0, will be assigned after sort)
        let mut rankings: Vec<CompanyRanking> = by_company
            .iter()
            .filter_map(|(company_id, drug_ids)| self.rank_company(company_id, drug_ids, 0))
            .collect();

        // Sort ascending by avg_signal_strength (safer = lower PRR = rank 1)
        rankings.sort_by(|a, b| {
            a.avg_signal_strength
                .partial_cmp(&b.avg_signal_strength)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.company.cmp(&b.company))
        });

        // Assign 1-based ranks
        for (i, r) in rankings.iter_mut().enumerate() {
            r.rank = (i + 1) as u32;
        }

        rankings
    }

    /// Head-to-head safety comparison between two drugs.
    ///
    /// Compares shared adverse event events (matched by name across
    /// `DrugNode` signal data stored on the graph). Returns a [`HeadToHead`]
    /// summarising shared signals, drug-unique events, and an overall
    /// safety advantage where one exists.
    ///
    /// When either drug is not in the graph, returns a result with empty
    /// signal collections and `safer_drug = None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_pv_intelligence::{builder::GraphBuilder, graph::DrugNode};
    ///
    /// let mut graph = GraphBuilder::new()
    ///     .add_drug("drug_a", "Alpha", "Class1", None)
    ///     .add_drug("drug_b", "Beta", "Class1", None)
    ///     .build();
    ///
    /// // Enrich with signal data
    /// if let Some(d) = graph.drugs.iter_mut().find(|d| d.id == "drug_a") {
    ///     d.signal_count = 2;
    ///     d.strongest_prr = Some(2.5);
    /// }
    ///
    /// let h2h = graph.head_to_head("drug_a", "drug_b");
    /// assert_eq!(h2h.drug_a, "drug_a");
    /// assert_eq!(h2h.drug_b, "drug_b");
    /// ```
    pub fn head_to_head(&self, drug_a: &str, drug_b: &str) -> HeadToHead {
        let node_a = self.drugs.iter().find(|d| d.id == drug_a);
        let node_b = self.drugs.iter().find(|d| d.id == drug_b);

        // Collect events from ClassEffect edges where both drugs appear
        let class_events_a = self.class_events_for_drug(drug_a);
        let class_events_b = self.class_events_for_drug(drug_b);

        let mut shared_signals: Vec<SharedSignal> = vec![];
        let mut unique_to_a: Vec<String> = vec![];
        let mut unique_to_b: Vec<String> = vec![];

        // Events from ClassEffect edges
        let all_events: std::collections::HashSet<String> = class_events_a
            .iter()
            .chain(class_events_b.iter())
            .cloned()
            .collect();

        for event in &all_events {
            let in_a = class_events_a.contains(event);
            let in_b = class_events_b.contains(event);

            match (in_a, in_b) {
                (true, true) => {
                    let prr_a = node_a.and_then(|n| n.strongest_prr).unwrap_or(1.0);
                    let prr_b = node_b.and_then(|n| n.strongest_prr).unwrap_or(1.0);
                    let advantage = if (prr_a - prr_b).abs() < 0.1 {
                        "comparable".to_string()
                    } else if prr_a < prr_b {
                        "drug_a".to_string()
                    } else {
                        "drug_b".to_string()
                    };
                    shared_signals.push(SharedSignal {
                        event: event.clone(),
                        prr_a,
                        prr_b,
                        advantage,
                    });
                }
                (true, false) => unique_to_a.push(event.clone()),
                (false, true) => unique_to_b.push(event.clone()),
                (false, false) => {}
            }
        }

        // Determine overall safer drug
        let safer_drug = {
            let prr_a = node_a.and_then(|n| n.strongest_prr).unwrap_or(1.0);
            let prr_b = node_b.and_then(|n| n.strongest_prr).unwrap_or(1.0);
            let sig_a = node_a.map(|n| n.signal_count).unwrap_or(0);
            let sig_b = node_b.map(|n| n.signal_count).unwrap_or(0);

            // Lower PRR + fewer signals = safer
            let score_a = prr_a + sig_a as f64 * 0.1;
            let score_b = prr_b + sig_b as f64 * 0.1;

            if (score_a - score_b).abs() < 0.05 {
                None
            } else if score_a < score_b {
                Some(drug_a.to_string())
            } else {
                Some(drug_b.to_string())
            }
        };

        shared_signals.sort_by(|a, b| a.event.cmp(&b.event));
        unique_to_a.sort();
        unique_to_b.sort();

        HeadToHead {
            drug_a: drug_a.to_string(),
            drug_b: drug_b.to_string(),
            shared_signals,
            unique_to_a,
            unique_to_b,
            safer_drug,
        }
    }

    /// Collect adverse event labels from `ClassEffect` edges for a drug.
    fn class_events_for_drug(&self, drug_id: &str) -> std::collections::HashSet<String> {
        self.edges
            .iter()
            .filter_map(|e| match e {
                Edge::ClassEffect { drugs, event, .. } if drugs.iter().any(|d| d == drug_id) => {
                    Some(event.clone())
                }
                _ => None,
            })
            .collect()
    }

    /// Detect class effects — adverse events shared across drugs of the same class.
    ///
    /// Scans `Edge::ClassEffect` entries for the given `drug_class` label.
    /// Returns one [`ClassEffectResult`] per distinct event.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_pv_intelligence::{builder::GraphBuilder, graph::Edge};
    ///
    /// let mut graph = GraphBuilder::new()
    ///     .add_drug("sema", "semaglutide", "GLP-1 Agonist", None)
    ///     .add_drug("lira", "liraglutide", "GLP-1 Agonist", None)
    ///     .build();
    ///
    /// graph.edges.push(Edge::ClassEffect {
    ///     drug_class: "GLP-1 Agonist".to_string(),
    ///     event: "pancreatitis".to_string(),
    ///     drugs: vec!["sema".to_string(), "lira".to_string()],
    /// });
    ///
    /// let effects = graph.class_effects("GLP-1 Agonist");
    /// assert_eq!(effects.len(), 1);
    /// assert_eq!(effects[0].event, "pancreatitis");
    /// assert_eq!(effects[0].affected_count, 2);
    /// ```
    pub fn class_effects(&self, drug_class: &str) -> Vec<ClassEffectResult> {
        let mut results: Vec<ClassEffectResult> = self
            .edges
            .iter()
            .filter_map(|e| match e {
                Edge::ClassEffect {
                    drug_class: cls,
                    event,
                    drugs,
                } if cls == drug_class => Some(ClassEffectResult {
                    drug_class: cls.clone(),
                    event: event.clone(),
                    affected_count: drugs.len(),
                    drugs: drugs.clone(),
                }),
                _ => None,
            })
            .collect();

        results.sort_by(|a, b| {
            b.affected_count
                .cmp(&a.affected_count)
                .then(a.event.cmp(&b.event))
        });
        results
    }

    /// Find safety gaps — drugs with elevated signals that are off-label for the disease.
    ///
    /// A gap is any drug treating `disease_id` that has `signal_count > 0`
    /// and `strongest_prr > 2.0` (a commonly used disproportionality threshold).
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_pv_intelligence::builder::GraphBuilder;
    ///
    /// let mut graph = GraphBuilder::new()
    ///     .add_drug("drug_x", "Drug X", "ClassA", None)
    ///     .add_disease("dis1", "Disease 1", "Oncology")
    ///     .add_treats("drug_x", "dis1")
    ///     .build();
    ///
    /// // Enrich signal data
    /// if let Some(d) = graph.drugs.iter_mut().find(|d| d.id == "drug_x") {
    ///     d.signal_count = 3;
    ///     d.strongest_prr = Some(4.2);
    /// }
    ///
    /// let gaps = graph.safety_gaps("dis1");
    /// assert_eq!(gaps.len(), 1);
    /// assert_eq!(gaps[0].drug, "drug_x");
    /// ```
    pub fn safety_gaps(&self, disease_id: &str) -> Vec<SafetyGap> {
        const PRR_THRESHOLD: f64 = 2.0;

        let mut gaps: Vec<SafetyGap> = self
            .drugs_for_disease(disease_id)
            .into_iter()
            .filter_map(|drug| {
                let prr = drug.strongest_prr?;
                if prr <= PRR_THRESHOLD || drug.signal_count == 0 {
                    return None;
                }
                let opportunity = format!(
                    "{} has {} signal(s) with PRR {:.1} — review for unlabelled risk",
                    drug.generic_name, drug.signal_count, prr
                );
                Some(SafetyGap {
                    drug: drug.id.clone(),
                    event: format!("signal_cluster_{}", drug.signal_count),
                    prr,
                    on_label: false,
                    opportunity,
                })
            })
            .collect();

        gaps.sort_by(|a, b| {
            b.prr
                .partial_cmp(&a.prr)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        gaps
    }

    /// Competitive landscape for a therapeutic area.
    ///
    /// Returns all companies with drugs in the area, ranked by safety,
    /// along with aggregate signal totals and the dominant company (most drugs).
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_pv_intelligence::builder::GraphBuilder;
    ///
    /// let graph = GraphBuilder::new()
    ///     .add_company("co_a", "Company A", &["drug1", "drug2"])
    ///     .add_drug("drug1", "Drug 1", "GLP-1", Some("co_a"))
    ///     .add_drug("drug2", "Drug 2", "GLP-1", Some("co_a"))
    ///     .add_disease("t2dm", "T2DM", "Metabolic")
    ///     .add_treats("drug1", "t2dm")
    ///     .add_treats("drug2", "t2dm")
    ///     .build();
    ///
    /// let landscape = graph.therapeutic_landscape("Metabolic");
    /// assert_eq!(landscape.total_drugs, 2);
    /// assert_eq!(landscape.dominant_company.as_deref(), Some("Company A"));
    /// ```
    pub fn therapeutic_landscape(&self, area: &str) -> TherapeuticLandscape {
        // Drugs in this area = drugs whose indications include a disease in this area
        let disease_ids_in_area: Vec<&str> = self
            .diseases
            .iter()
            .filter(|d| d.therapeutic_area == area)
            .map(|d| d.id.as_str())
            .collect();

        let drugs_in_area: Vec<&crate::graph::DrugNode> = self
            .drugs
            .iter()
            .filter(|drug| {
                drug.indications
                    .iter()
                    .any(|ind| disease_ids_in_area.contains(&ind.as_str()))
            })
            .collect();

        let total_drugs = drugs_in_area.len();
        let total_signals: usize = drugs_in_area.iter().map(|d| d.signal_count).sum();

        // Group by company
        let mut by_company: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for drug in &drugs_in_area {
            let owner = self.owner_of(&drug.id).unwrap_or("unknown").to_string();
            by_company.entry(owner).or_default().push(drug.id.clone());
        }

        let mut companies: Vec<CompanyRanking> = by_company
            .iter()
            .filter_map(|(company_id, drug_ids)| self.rank_company(company_id, drug_ids, 0))
            .collect();

        companies.sort_by(|a, b| {
            a.avg_signal_strength
                .partial_cmp(&b.avg_signal_strength)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.company.cmp(&b.company))
        });
        for (i, r) in companies.iter_mut().enumerate() {
            r.rank = (i + 1) as u32;
        }

        // Dominant = company with most drugs
        let dominant_company = by_company
            .iter()
            .max_by_key(|(_, drugs)| drugs.len())
            .and_then(|(company_id, _)| {
                self.companies
                    .iter()
                    .find(|c| &c.id == company_id)
                    .map(|c| c.name.clone())
            });

        TherapeuticLandscape {
            area: area.to_string(),
            companies,
            total_drugs,
            total_signals,
            dominant_company,
        }
    }

    /// Pipeline overlap — companies that compete in the same indication.
    ///
    /// Returns one [`PipelineOverlap`] per pair of companies that both own
    /// drugs treating `disease_id`. The `competition_phase` is set to
    /// `"head-to-head"` for marketed drugs (determined by presence in the
    /// graph with signal data) and `"emerging"` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_pv_intelligence::builder::GraphBuilder;
    ///
    /// let graph = GraphBuilder::new()
    ///     .add_company("co_a", "Company A", &["drug1"])
    ///     .add_company("co_b", "Company B", &["drug2"])
    ///     .add_drug("drug1", "Drug 1", "ClassA", Some("co_a"))
    ///     .add_drug("drug2", "Drug 2", "ClassA", Some("co_b"))
    ///     .add_disease("dis1", "Disease 1", "Oncology")
    ///     .add_treats("drug1", "dis1")
    ///     .add_treats("drug2", "dis1")
    ///     .build();
    ///
    /// let overlaps = graph.pipeline_overlap("dis1");
    /// assert_eq!(overlaps.len(), 1);
    /// assert_eq!(overlaps[0].shared_indication, "dis1");
    /// ```
    pub fn pipeline_overlap(&self, disease_id: &str) -> Vec<PipelineOverlap> {
        let drugs = self.drugs_for_disease(disease_id);

        // Map drug → owner company
        let drug_owners: Vec<(String, String)> = drugs
            .iter()
            .filter_map(|drug| {
                self.owner_of(&drug.id)
                    .map(|owner| (drug.id.clone(), owner.to_string()))
            })
            .collect();

        // Collect unique companies
        let companies: Vec<&str> = {
            let mut seen = std::collections::HashSet::new();
            drug_owners
                .iter()
                .filter_map(|(_, owner)| {
                    if seen.insert(owner.as_str()) {
                        Some(owner.as_str())
                    } else {
                        None
                    }
                })
                .collect()
        };

        // Produce all unique pairs
        let mut overlaps: Vec<PipelineOverlap> = vec![];
        for i in 0..companies.len() {
            for j in (i + 1)..companies.len() {
                let a = companies[i];
                let b = companies[j];

                // Determine competition phase from signal data
                let a_has_signals =
                    drug_owners
                        .iter()
                        .filter(|(_, owner)| owner == a)
                        .any(|(drug_id, _)| {
                            self.drugs
                                .iter()
                                .find(|d| &d.id == drug_id)
                                .map(|d| d.signal_count > 0)
                                .unwrap_or(false)
                        });
                let b_has_signals =
                    drug_owners
                        .iter()
                        .filter(|(_, owner)| owner == b)
                        .any(|(drug_id, _)| {
                            self.drugs
                                .iter()
                                .find(|d| &d.id == drug_id)
                                .map(|d| d.signal_count > 0)
                                .unwrap_or(false)
                        });

                let competition_phase = if a_has_signals && b_has_signals {
                    "head-to-head"
                } else {
                    "emerging"
                }
                .to_string();

                overlaps.push(PipelineOverlap {
                    company_a: a.to_string(),
                    company_b: b.to_string(),
                    shared_indication: disease_id.to_string(),
                    competition_phase,
                });
            }
        }

        overlaps.sort_by(|a, b| {
            a.company_a
                .cmp(&b.company_a)
                .then(a.company_b.cmp(&b.company_b))
        });
        overlaps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builder::GraphBuilder, graph::Edge};

    /// Build a 2-company, 3-drug, 2-disease mini graph for query tests.
    fn test_graph() -> IntelligenceGraph {
        let mut graph = GraphBuilder::new()
            .add_company("novo", "Novo Nordisk", &["sema", "lira"])
            .add_company("lilly", "Eli Lilly", &["tirze"])
            .add_drug("sema", "semaglutide", "GLP-1 Agonist", Some("novo"))
            .add_drug("lira", "liraglutide", "GLP-1 Agonist", Some("novo"))
            .add_drug(
                "tirze",
                "tirzepatide",
                "GLP-1/GIP Dual Agonist",
                Some("lilly"),
            )
            .add_disease("t2dm", "Type 2 Diabetes Mellitus", "Metabolic")
            .add_disease("obesity", "Obesity", "Metabolic")
            .add_treats("sema", "t2dm")
            .add_treats("sema", "obesity")
            .add_treats("lira", "t2dm")
            .add_treats("tirze", "t2dm")
            .add_treats("tirze", "obesity")
            .build();

        // Enrich signal data
        if let Some(d) = graph.drugs.iter_mut().find(|d| d.id == "sema") {
            d.signal_count = 4;
            d.strongest_prr = Some(3.1);
        }
        if let Some(d) = graph.drugs.iter_mut().find(|d| d.id == "lira") {
            d.signal_count = 2;
            d.strongest_prr = Some(2.4);
        }
        if let Some(d) = graph.drugs.iter_mut().find(|d| d.id == "tirze") {
            d.signal_count = 1;
            d.strongest_prr = Some(1.8);
        }

        // Add a class effect for GLP-1 agonists
        graph.edges.push(Edge::ClassEffect {
            drug_class: "GLP-1 Agonist".to_string(),
            event: "pancreatitis".to_string(),
            drugs: vec!["sema".to_string(), "lira".to_string()],
        });

        graph
    }

    #[test]
    fn safest_company_for_disease_returns_ranked_list() {
        let graph = test_graph();
        let rankings = graph.safest_company_for_disease("t2dm");
        // Both companies have drugs treating t2dm
        assert_eq!(rankings.len(), 2);
        assert_eq!(rankings[0].rank, 1);
        assert_eq!(rankings[1].rank, 2);
        // Ranks are 1-based and contiguous
        assert!(rankings[0].avg_signal_strength <= rankings[1].avg_signal_strength);
    }

    #[test]
    fn safest_company_empty_for_unknown_disease() {
        let graph = test_graph();
        let rankings = graph.safest_company_for_disease("nonexistent");
        assert!(rankings.is_empty());
    }

    #[test]
    fn head_to_head_shared_signals_via_class_effect() {
        let graph = test_graph();
        let h2h = graph.head_to_head("sema", "lira");
        // Both share pancreatitis via ClassEffect edge
        assert_eq!(
            h2h.shared_signals
                .iter()
                .find(|s| s.event == "pancreatitis")
                .is_some(),
            true
        );
        assert_eq!(h2h.drug_a, "sema");
        assert_eq!(h2h.drug_b, "lira");
    }

    #[test]
    fn head_to_head_safer_drug_determined() {
        let graph = test_graph();
        // tirze has lower PRR than sema
        let h2h = graph.head_to_head("sema", "tirze");
        // tirze (drug_b) should be safer
        assert_eq!(h2h.safer_drug.as_deref(), Some("tirze"));
    }

    #[test]
    fn head_to_head_unknown_drugs_returns_empty_result() {
        let graph = test_graph();
        let h2h = graph.head_to_head("unknown_a", "unknown_b");
        assert!(h2h.shared_signals.is_empty());
        assert!(h2h.unique_to_a.is_empty());
        assert!(h2h.unique_to_b.is_empty());
    }

    #[test]
    fn class_effects_glp1_returns_pancreatitis() {
        let graph = test_graph();
        let effects = graph.class_effects("GLP-1 Agonist");
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].event, "pancreatitis");
        assert_eq!(effects[0].affected_count, 2);
        assert!(effects[0].drugs.contains(&"sema".to_string()));
        assert!(effects[0].drugs.contains(&"lira".to_string()));
    }

    #[test]
    fn class_effects_unknown_class_returns_empty() {
        let graph = test_graph();
        let effects = graph.class_effects("Unknown Class");
        assert!(effects.is_empty());
    }

    #[test]
    fn safety_gaps_detects_elevated_prr() {
        let graph = test_graph();
        // sema has PRR 3.1 > threshold 2.0
        let gaps = graph.safety_gaps("t2dm");
        assert!(!gaps.is_empty());
        assert!(gaps.iter().any(|g| g.drug == "sema"));
        // tirze has PRR 1.8 < 2.0 — not a gap
        assert!(!gaps.iter().any(|g| g.drug == "tirze"));
    }

    #[test]
    fn safety_gaps_sorted_descending_by_prr() {
        let graph = test_graph();
        let gaps = graph.safety_gaps("t2dm");
        for i in 0..gaps.len().saturating_sub(1) {
            assert!(gaps[i].prr >= gaps[i + 1].prr);
        }
    }

    #[test]
    fn therapeutic_landscape_metabolic_area() {
        let graph = test_graph();
        let landscape = graph.therapeutic_landscape("Metabolic");
        // All 3 drugs treat diseases in Metabolic area
        assert_eq!(landscape.total_drugs, 3);
        assert!(landscape.total_signals > 0);
        assert!(!landscape.companies.is_empty());
    }

    #[test]
    fn therapeutic_landscape_dominant_company() {
        let graph = test_graph();
        let landscape = graph.therapeutic_landscape("Metabolic");
        // Novo has 2 drugs (sema, lira), Lilly has 1 (tirze) — Novo is dominant
        assert_eq!(landscape.dominant_company.as_deref(), Some("Novo Nordisk"));
    }

    #[test]
    fn pipeline_overlap_two_companies_one_indication() {
        let graph = test_graph();
        let overlaps = graph.pipeline_overlap("t2dm");
        // Novo and Lilly both have drugs in t2dm
        assert_eq!(overlaps.len(), 1);
        assert_eq!(overlaps[0].shared_indication, "t2dm");
    }

    #[test]
    fn pipeline_overlap_empty_for_single_company_disease() {
        // Only one company's drug treats the disease
        let graph = GraphBuilder::new()
            .add_company("sole", "Sole Pharma", &["only_drug"])
            .add_drug("only_drug", "Only Drug", "ClassX", Some("sole"))
            .add_disease("rare", "Rare Condition", "RareDisease")
            .add_treats("only_drug", "rare")
            .build();
        let overlaps = graph.pipeline_overlap("rare");
        assert!(overlaps.is_empty());
    }
}
