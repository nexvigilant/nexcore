//! Built-in Pharmacovigilance (PV) domain taxonomy.
//!
//! 30 primitives: 8 T1 + 6 T2-P + 6 T2-C + 10 T3
//! nexcore home domain. Extracted 2026-02-05.

use crate::taxonomy::{DomainTaxonomy, Primitive, Tier};
use crate::transfer::{DomainTransfer, TransferScore};

/// Construct the full Pharmacovigilance taxonomy with pre-computed transfer scores.
pub fn pharmacovigilance() -> DomainTaxonomy {
    let mut tax = DomainTaxonomy::new(
        "Pharmacovigilance",
        "Science and activities relating to the detection, assessment, understanding, \
         and prevention of adverse effects or any other drug-related problem. \
         Encompasses signal detection (PRR, ROR, IC, EBGM), case processing (ICSR/E2B), \
         regulatory reporting (PSUR/PBRER), and risk management (RMP/REMS).",
    );

    // ── T1: Universal ──────────────────────────────────────────────────
    tax.primitives.push(
        Primitive::new(
            "detection",
            "Recognition of signal against background noise",
            Tier::T1,
        )
        .with_domains(&[
            "radar",
            "immune-system",
            "fraud",
            "quality-control",
            "hypothesis-testing",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "threshold",
            "Boundary value that triggers state change when crossed",
            Tier::T1,
        )
        .with_domains(&["all"]),
    );
    tax.primitives.push(
        Primitive::new(
            "tracking",
            "Continuous estimation of state across time",
            Tier::T1,
        )
        .with_domains(&["kalman-filters", "logistics", "epidemiology", "investments"]),
    );
    tax.primitives.push(
        Primitive::new(
            "classification",
            "Categorization of entities into discrete types based on shared attributes",
            Tier::T1,
        )
        .with_domains(&[
            "biology",
            "library-science",
            "machine-learning",
            "linguistics",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "causality",
            "Assessment of cause-effect relationship between stimulus and outcome",
            Tier::T1,
        )
        .with_domains(&[
            "epidemiology",
            "forensics",
            "root-cause-analysis",
            "philosophy",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "reporting",
            "Structured transmission of observed information to decision-makers",
            Tier::T1,
        )
        .with_domains(&[
            "journalism",
            "financial-reporting",
            "incident-response",
            "scientific-publishing",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "aggregation",
            "Combining individual observations into population-level summary",
            Tier::T1,
        )
        .with_domains(&["statistics", "data-warehousing", "census", "meta-analysis"]),
    );
    tax.primitives.push(
        Primitive::new(
            "risk",
            "Probability of harm occurrence given exposure",
            Tier::T1,
        )
        .with_domains(&["insurance", "engineering", "finance", "public-health"]),
    );

    // ── T2-P: Cross-Domain Primitives ──────────────────────────────────
    tax.primitives.push(
        Primitive::new(
            "disproportionality",
            "Statistical detection of imbalance between observed and expected event rates \
             (PRR, ROR, IC, EBGM)",
            Tier::T2P,
        )
        .with_deps(&["detection", "threshold", "aggregation"])
        .with_domains(&[
            "pharmacoepidemiology",
            "fraud-detection",
            "quality-control",
            "epidemiology",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "case-series",
            "Temporal clustering of related individual events suggesting a common cause",
            Tier::T2P,
        )
        .with_deps(&["tracking", "classification", "aggregation"])
        .with_domains(&[
            "epidemiology",
            "outbreak-investigation",
            "reliability-engineering",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "benefit-risk",
            "Bidirectional weighing of therapeutic benefit against potential harm",
            Tier::T2P,
        )
        .with_deps(&["risk", "causality", "aggregation"])
        .with_domains(&[
            "regulatory-science",
            "health-economics",
            "decision-theory",
            "engineering-tradeoffs",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "signal-refinement",
            "Progressive narrowing from statistical noise to confirmed safety signal",
            Tier::T2P,
        )
        .with_deps(&["detection", "threshold", "causality"])
        .with_domains(&["radar-processing", "anomaly-detection", "scientific-method"]),
    );
    tax.primitives.push(
        Primitive::new(
            "denominator-estimation",
            "Approximation of exposed population to contextualize observed event counts",
            Tier::T2P,
        )
        .with_deps(&["aggregation", "tracking"])
        .with_domains(&["epidemiology", "market-sizing", "census", "sampling-theory"]),
    );
    tax.primitives.push(
        Primitive::new(
            "confounding-control",
            "Systematic isolation and adjustment for variables that distort causal inference",
            Tier::T2P,
        )
        .with_deps(&["causality", "classification", "aggregation"])
        .with_domains(&[
            "clinical-trials",
            "econometrics",
            "social-science",
            "A/B-testing",
        ]),
    );

    // ── T2-C: Cross-Domain Composites ──────────────────────────────────
    tax.primitives.push(
        Primitive::new(
            "signal-detection-pipeline",
            "End-to-end process: data mining, filtering, clinical evaluation, and regulatory action",
            Tier::T2C,
        )
        .with_deps(&["detection", "disproportionality", "signal-refinement", "threshold"])
        .with_domains(&["fraud-detection-pipelines", "anomaly-detection-systems", "quality-gates"]),
    );
    tax.primitives.push(
        Primitive::new(
            "periodic-safety-update",
            "Recurring comprehensive assessment of a product's risk profile over defined intervals",
            Tier::T2C,
        )
        .with_deps(&[
            "tracking",
            "case-series",
            "benefit-risk",
            "aggregation",
            "reporting",
        ])
        .with_domains(&[
            "financial-quarterly-reports",
            "compliance-audits",
            "infrastructure-reviews",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "risk-management-plan",
            "Structured strategy for identifying, characterizing, and minimizing known risks",
            Tier::T2C,
        )
        .with_deps(&["risk", "benefit-risk", "tracking", "reporting"])
        .with_domains(&[
            "enterprise-risk-management",
            "project-management",
            "insurance-underwriting",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "labeling-update-cycle",
            "Evidence-triggered revision of official product information and regulatory text",
            Tier::T2C,
        )
        .with_deps(&[
            "signal-refinement",
            "benefit-risk",
            "reporting",
            "classification",
        ])
        .with_domains(&[
            "software-changelogs",
            "regulatory-amendments",
            "standards-revision",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "post-marketing-surveillance",
            "Continuous Phase IV monitoring of safety and effectiveness in real-world populations",
            Tier::T2C,
        )
        .with_deps(&[
            "detection",
            "tracking",
            "aggregation",
            "denominator-estimation",
            "reporting",
        ])
        .with_domains(&[
            "product-recalls",
            "environmental-monitoring",
            "fleet-monitoring",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "pharmacoepidemiologic-study",
            "Designed observational investigation using population data to quantify drug-event associations",
            Tier::T2C,
        )
        .with_deps(&["aggregation", "denominator-estimation", "confounding-control", "causality"])
        .with_domains(&["clinical-epidemiology", "health-services-research", "outcomes-research"]),
    );

    // ── T3: Domain-Specific (Pharmacovigilance) ──────────────────────
    tax.primitives.push(
        Primitive::new(
            "ICSR",
            "Individual Case Safety Report: structured record of a single adverse event in E2B/R3 format",
            Tier::T3,
        )
        .with_deps(&["reporting", "classification", "causality"]),
    );
    tax.primitives.push(
        Primitive::new(
            "MedDRA-coding",
            "Medical Dictionary for Regulatory Activities: hierarchical terminology (SOC/HLGT/HLT/PT/LLT) \
             for adverse event classification",
            Tier::T3,
        )
        .with_deps(&["classification"]),
    );
    tax.primitives.push(
        Primitive::new(
            "PBRER",
            "Periodic Benefit-Risk Evaluation Report: ICH E2C(R2) comprehensive benefit-risk assessment \
             submitted at defined intervals",
            Tier::T3,
        )
        .with_deps(&["periodic-safety-update", "benefit-risk", "signal-refinement"]),
    );
    tax.primitives.push(
        Primitive::new(
            "REMS",
            "Risk Evaluation and Mitigation Strategy: FDA-mandated program ensuring benefits outweigh risks \
             through prescriber/patient requirements",
            Tier::T3,
        )
        .with_deps(&["risk-management-plan", "benefit-risk", "risk"]),
    );
    tax.primitives.push(
        Primitive::new(
            "PSUR",
            "Periodic Safety Update Report: EU-centric recurring safety summary covering global data \
             for authorized medicinal products",
            Tier::T3,
        )
        .with_deps(&["periodic-safety-update", "tracking", "aggregation"]),
    );
    tax.primitives.push(
        Primitive::new(
            "SUSAR",
            "Suspected Unexpected Serious Adverse Reaction: serious ADR not consistent with \
             applicable product information requiring expedited reporting",
            Tier::T3,
        )
        .with_deps(&["ICSR", "classification", "causality", "reporting"]),
    );
    tax.primitives.push(
        Primitive::new(
            "CIOMS-form",
            "Council for International Organizations of Medical Sciences reporting standard: \
             structured one-page format for expedited ADR notification",
            Tier::T3,
        )
        .with_deps(&["ICSR", "reporting"]),
    );
    tax.primitives.push(
        Primitive::new(
            "EudraVigilance",
            "European Medicines Agency adverse reaction database: centralized EU repository \
             for ICSR collection, management, and signal detection",
            Tier::T3,
        )
        .with_deps(&["ICSR", "signal-detection-pipeline", "aggregation"]),
    );
    tax.primitives.push(
        Primitive::new(
            "FAERS",
            "FDA Adverse Event Reporting System: US national repository of spontaneous reports \
             and medication error data supporting post-market safety surveillance",
            Tier::T3,
        )
        .with_deps(&["ICSR", "signal-detection-pipeline", "aggregation"]),
    );
    tax.primitives.push(
        Primitive::new(
            "signal-of-disproportionate-reporting",
            "Confirmed statistical anomaly (SDR): drug-event pair exceeding disproportionality \
             thresholds after clinical review and confounding assessment",
            Tier::T3,
        )
        .with_deps(&[
            "signal-detection-pipeline",
            "disproportionality",
            "signal-refinement",
            "confounding-control",
        ]),
    );

    // ── Pre-computed transfer scores ───────────────────────────────────
    tax.transfers = build_transfers();

    tax
}

/// Pre-computed transfer confidence scores across 4 target domains.
///
/// Target domains: missile-defense, cybersecurity, finance, manufacturing-qc.
/// PV primitives transfer HIGH to epidemiology/medicine concepts,
/// MEDIUM to other signal-detection domains, LOW to purely mechanical domains.
fn build_transfers() -> Vec<DomainTransfer> {
    let data: &[(&str, &[(&str, f64, f64, f64, &str)])] = &[
        // ── T1 ───────────────────────────────────────────────────────────
        (
            "detection",
            &[
                ("missile-defense", 0.99, 0.96, 0.95, "time scale"),
                ("cybersecurity", 0.99, 0.96, 0.95, "time scale"),
                ("finance", 0.92, 0.90, 0.91, "market noise"),
                ("manufacturing-qc", 0.99, 0.98, 0.97, "sample rate"),
            ],
        ),
        (
            "threshold",
            &[
                ("missile-defense", 0.99, 0.99, 0.99, "none"),
                ("cybersecurity", 0.99, 0.99, 0.99, "none"),
                ("finance", 0.99, 0.99, 0.99, "none"),
                ("manufacturing-qc", 0.99, 0.99, 0.99, "none"),
            ],
        ),
        (
            "tracking",
            &[
                ("missile-defense", 0.93, 0.91, 0.88, "temporal granularity"),
                ("cybersecurity", 0.91, 0.88, 0.87, "attribution difficulty"),
                ("finance", 0.95, 0.94, 0.93, "market hours"),
                ("manufacturing-qc", 0.89, 0.86, 0.85, "batch granularity"),
            ],
        ),
        (
            "classification",
            &[
                (
                    "missile-defense",
                    0.82,
                    0.80,
                    0.76,
                    "threat typology differs",
                ),
                ("cybersecurity", 0.90, 0.88, 0.85, "CVE ontology"),
                ("finance", 0.88, 0.86, 0.84, "asset classes"),
                ("manufacturing-qc", 0.93, 0.91, 0.88, "defect taxonomy"),
            ],
        ),
        (
            "causality",
            &[
                ("missile-defense", 0.62, 0.60, 0.55, "deterministic system"),
                ("cybersecurity", 0.78, 0.76, 0.72, "attribution complexity"),
                ("finance", 0.75, 0.73, 0.70, "multi-factor models"),
                ("manufacturing-qc", 0.85, 0.83, 0.80, "root-cause analysis"),
            ],
        ),
        (
            "reporting",
            &[
                (
                    "missile-defense",
                    0.80,
                    0.78,
                    0.75,
                    "classification barriers",
                ),
                ("cybersecurity", 0.92, 0.90, 0.87, "incident formats"),
                ("finance", 0.95, 0.93, 0.91, "regulatory filings"),
                ("manufacturing-qc", 0.93, 0.91, 0.88, "CAPA reports"),
            ],
        ),
        (
            "aggregation",
            &[
                ("missile-defense", 0.78, 0.76, 0.72, "sparse data"),
                ("cybersecurity", 0.88, 0.86, 0.83, "log volume"),
                ("finance", 0.94, 0.92, 0.90, "portfolio aggregation"),
                ("manufacturing-qc", 0.96, 0.94, 0.91, "SPC charts"),
            ],
        ),
        (
            "risk",
            &[
                ("missile-defense", 0.85, 0.83, 0.80, "adversarial dynamics"),
                ("cybersecurity", 0.90, 0.88, 0.85, "threat models"),
                ("finance", 0.95, 0.93, 0.91, "portfolio risk"),
                ("manufacturing-qc", 0.91, 0.89, 0.86, "FMEA"),
            ],
        ),
        // ── T2-P ─────────────────────────────────────────────────────────
        (
            "disproportionality",
            &[
                (
                    "missile-defense",
                    0.55,
                    0.52,
                    0.48,
                    "non-statistical paradigm",
                ),
                (
                    "cybersecurity",
                    0.78,
                    0.75,
                    0.71,
                    "anomaly detection analogy",
                ),
                ("finance", 0.82, 0.80, 0.76, "outlier detection"),
                ("manufacturing-qc", 0.85, 0.83, 0.80, "SPC deviation"),
            ],
        ),
        (
            "case-series",
            &[
                ("missile-defense", 0.50, 0.48, 0.44, "event clustering rare"),
                ("cybersecurity", 0.82, 0.80, 0.76, "incident clustering"),
                ("finance", 0.76, 0.74, 0.70, "correlated defaults"),
                ("manufacturing-qc", 0.88, 0.86, 0.83, "defect clusters"),
            ],
        ),
        (
            "benefit-risk",
            &[
                ("missile-defense", 0.70, 0.68, 0.64, "strategic tradeoffs"),
                ("cybersecurity", 0.74, 0.72, 0.68, "usability vs security"),
                ("finance", 0.92, 0.90, 0.87, "risk-return tradeoff"),
                ("manufacturing-qc", 0.78, 0.76, 0.72, "cost-quality balance"),
            ],
        ),
        (
            "signal-refinement",
            &[
                ("missile-defense", 0.72, 0.70, 0.66, "sensor refinement"),
                ("cybersecurity", 0.85, 0.83, 0.80, "alert triage"),
                ("finance", 0.80, 0.78, 0.74, "alpha refinement"),
                ("manufacturing-qc", 0.83, 0.81, 0.78, "defect root cause"),
            ],
        ),
        (
            "denominator-estimation",
            &[
                ("missile-defense", 0.45, 0.42, 0.38, "no population concept"),
                ("cybersecurity", 0.68, 0.65, 0.61, "install base estimation"),
                ("finance", 0.80, 0.78, 0.74, "market size estimation"),
                ("manufacturing-qc", 0.85, 0.83, 0.80, "production volume"),
            ],
        ),
        (
            "confounding-control",
            &[
                (
                    "missile-defense",
                    0.48,
                    0.45,
                    0.41,
                    "controlled environment",
                ),
                ("cybersecurity", 0.72, 0.70, 0.66, "multi-vector attacks"),
                ("finance", 0.84, 0.82, 0.78, "factor models"),
                ("manufacturing-qc", 0.80, 0.78, 0.74, "DOE methods"),
            ],
        ),
    ];

    let mut transfers = Vec::new();
    for &(prim_name, domain_scores) in data {
        for &(domain, s, f, c, limiting) in domain_scores {
            transfers.push(DomainTransfer {
                primitive_name: prim_name.to_string(),
                target_domain: domain.to_string(),
                score: TransferScore::new(s, f, c),
                limiting_description: limiting.to_string(),
            });
        }
    }
    transfers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pharmacovigilance_loads() {
        let tax = pharmacovigilance();
        assert_eq!(tax.name, "Pharmacovigilance");
        assert_eq!(tax.primitives.len(), 30);
    }

    #[test]
    fn tier_distribution() {
        let tax = pharmacovigilance();
        assert_eq!(tax.by_tier(Tier::T1).len(), 8);
        assert_eq!(tax.by_tier(Tier::T2P).len(), 6);
        assert_eq!(tax.by_tier(Tier::T2C).len(), 6);
        assert_eq!(tax.by_tier(Tier::T3).len(), 10);
    }

    #[test]
    fn irreducible_atom_count() {
        let tax = pharmacovigilance();
        let atoms = tax.irreducible_atoms();
        assert_eq!(atoms.len(), 14); // 8 T1 + 6 T2-P
    }

    #[test]
    fn all_primitives_have_definitions() {
        let tax = pharmacovigilance();
        for prim in &tax.primitives {
            assert!(
                !prim.definition.is_empty(),
                "Primitive '{}' has empty definition",
                prim.name
            );
        }
    }

    #[test]
    fn all_t1_have_domain_examples() {
        let tax = pharmacovigilance();
        for prim in tax.by_tier(Tier::T1) {
            assert!(
                !prim.domain_examples.is_empty(),
                "T1 primitive '{}' has no domain examples",
                prim.name
            );
        }
    }

    #[test]
    fn all_t2p_have_domain_examples() {
        let tax = pharmacovigilance();
        for prim in tax.by_tier(Tier::T2P) {
            assert!(
                !prim.domain_examples.is_empty(),
                "T2-P primitive '{}' has no domain examples",
                prim.name
            );
        }
    }

    #[test]
    fn all_t2c_have_domain_examples() {
        let tax = pharmacovigilance();
        for prim in tax.by_tier(Tier::T2C) {
            assert!(
                !prim.domain_examples.is_empty(),
                "T2-C primitive '{}' has no domain examples",
                prim.name
            );
        }
    }

    #[test]
    fn all_dependencies_resolve() {
        let tax = pharmacovigilance();
        for prim in &tax.primitives {
            for dep in &prim.dependencies {
                assert!(
                    tax.get(dep).is_some(),
                    "Primitive '{}' has unresolved dependency '{}'",
                    prim.name,
                    dep
                );
            }
        }
    }

    #[test]
    fn t1_have_no_dependencies() {
        let tax = pharmacovigilance();
        for prim in tax.by_tier(Tier::T1) {
            assert!(
                prim.dependencies.is_empty(),
                "T1 primitive '{}' should have no dependencies, found {:?}",
                prim.name,
                prim.dependencies
            );
        }
    }

    #[test]
    fn t3_have_dependencies() {
        let tax = pharmacovigilance();
        for prim in tax.by_tier(Tier::T3) {
            assert!(
                !prim.dependencies.is_empty(),
                "T3 primitive '{}' should have at least one dependency",
                prim.name
            );
        }
    }

    #[test]
    fn threshold_highest_transfer() {
        let tax = pharmacovigilance();
        let threshold_transfers: Vec<f64> = tax
            .transfers
            .iter()
            .filter(|t| t.primitive_name == "threshold")
            .map(|t| t.confidence())
            .collect();
        assert!(!threshold_transfers.is_empty());
        for c in &threshold_transfers {
            assert!(*c > 0.95, "threshold should transfer at >0.95, got {c}");
        }
    }

    #[test]
    fn denominator_estimation_low_missile_transfer() {
        let tax = pharmacovigilance();
        let missile_conf: Vec<f64> = tax
            .transfers
            .iter()
            .filter(|t| {
                t.primitive_name == "denominator-estimation" && t.target_domain == "missile-defense"
            })
            .map(|t| t.confidence())
            .collect();
        assert!(!missile_conf.is_empty());
        for c in &missile_conf {
            assert!(
                *c < 0.50,
                "denominator-estimation should have low missile-defense transfer, got {c}"
            );
        }
    }

    #[test]
    fn t1_avg_exceeds_t2p_avg() {
        let tax = pharmacovigilance();
        let t1_avg = tax.avg_transfer_confidence(Tier::T1);
        let t2p_avg = tax.avg_transfer_confidence(Tier::T2P);
        assert!(
            t1_avg > t2p_avg,
            "T1 avg ({t1_avg}) should exceed T2-P avg ({t2p_avg})"
        );
    }

    #[test]
    fn decompose_signal_of_disproportionate_reporting() {
        let tax = pharmacovigilance();
        let node = tax.decompose("signal-of-disproportionate-reporting");
        assert!(node.is_some());
        let node = node.unwrap_or_else(|| crate::taxonomy::DecompositionNode {
            name: String::new(),
            tier: Tier::T1,
            children: vec![],
        });
        let leaves = node.leaves();
        // Should bottom out at T1 atoms
        for leaf in &leaves {
            let prim = tax.get(leaf);
            assert!(prim.is_some(), "leaf {leaf} not found in taxonomy");
        }
    }

    #[test]
    fn decompose_faers() {
        let tax = pharmacovigilance();
        let node = tax.decompose("FAERS");
        assert!(node.is_some());
        let node = node.unwrap_or_else(|| crate::taxonomy::DecompositionNode {
            name: String::new(),
            tier: Tier::T1,
            children: vec![],
        });
        // FAERS depends on ICSR which depends on reporting, classification, causality (all T1)
        assert!(
            node.depth() >= 2,
            "FAERS decomposition should be at least 2 deep"
        );
    }

    #[test]
    fn transfer_count() {
        let tax = pharmacovigilance();
        // 14 primitives (T1+T2-P) x 4 domains = 56 transfers
        assert_eq!(tax.transfers.len(), 56);
    }

    #[test]
    fn all_transfers_in_valid_range() {
        let tax = pharmacovigilance();
        for t in &tax.transfers {
            let c = t.confidence();
            assert!(
                (0.0..=1.0).contains(&c),
                "Transfer {} -> {} has confidence {c} outside [0,1]",
                t.primitive_name,
                t.target_domain
            );
        }
    }

    #[test]
    fn shared_t1_with_golden_dome() {
        let pv = pharmacovigilance();
        let gd = crate::golden_dome::golden_dome();
        let pv_t1: Vec<&str> = pv
            .by_tier(Tier::T1)
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        let gd_t1: Vec<&str> = gd
            .by_tier(Tier::T1)
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        // detection, threshold, tracking should be in both
        let shared = ["detection", "threshold", "tracking"];
        for name in &shared {
            assert!(
                pv_t1.contains(name),
                "PV taxonomy missing shared T1 '{name}'"
            );
            assert!(
                gd_t1.contains(name),
                "Golden Dome taxonomy missing shared T1 '{name}'"
            );
        }
    }

    #[test]
    fn unique_primitive_names() {
        let tax = pharmacovigilance();
        let mut names: Vec<&str> = tax.primitives.iter().map(|p| p.name.as_str()).collect();
        let original_len = names.len();
        names.sort();
        names.dedup();
        assert_eq!(
            names.len(),
            original_len,
            "Duplicate primitive names detected"
        );
    }

    #[test]
    fn pv_specific_primitives_present() {
        let tax = pharmacovigilance();
        let expected = [
            "ICSR",
            "MedDRA-coding",
            "PBRER",
            "REMS",
            "PSUR",
            "SUSAR",
            "CIOMS-form",
            "EudraVigilance",
            "FAERS",
            "signal-of-disproportionate-reporting",
        ];
        for name in &expected {
            assert!(
                tax.get(name).is_some(),
                "Missing PV-specific primitive: {name}"
            );
        }
    }

    #[test]
    fn dag_is_acyclic() {
        // Verify no cycles in the dependency graph by checking topological sort succeeds
        let tax = pharmacovigilance();
        let sorted = crate::analysis::topological_sort(&tax);
        assert!(
            sorted.is_ok(),
            "Dependency graph has cycles (topological sort failed)"
        );
        let sorted = sorted.unwrap_or_default();
        assert_eq!(
            sorted.len(),
            30,
            "Topological sort should include all 30 primitives"
        );
    }

    #[test]
    fn disproportionality_key_dependency_chain() {
        // detection -> disproportionality -> signal-detection-pipeline -> SDR
        let tax = pharmacovigilance();
        let disprop = tax.get("disproportionality");
        assert!(disprop.is_some());
        let disprop = disprop.unwrap_or_else(|| &tax.primitives[0]);
        assert!(disprop.dependencies.contains(&"detection".to_string()));

        let sdp = tax.get("signal-detection-pipeline");
        assert!(sdp.is_some());
        let sdp = sdp.unwrap_or_else(|| &tax.primitives[0]);
        assert!(sdp.dependencies.contains(&"disproportionality".to_string()));

        let sdr = tax.get("signal-of-disproportionate-reporting");
        assert!(sdr.is_some());
        let sdr = sdr.unwrap_or_else(|| &tax.primitives[0]);
        assert!(
            sdr.dependencies
                .contains(&"signal-detection-pipeline".to_string())
        );
    }

    #[test]
    fn reporting_to_icsr_chain() {
        let tax = pharmacovigilance();
        let icsr = tax.get("ICSR");
        assert!(icsr.is_some());
        let icsr = icsr.unwrap_or_else(|| &tax.primitives[0]);
        assert!(icsr.dependencies.contains(&"reporting".to_string()));

        let faers = tax.get("FAERS");
        assert!(faers.is_some());
        let faers = faers.unwrap_or_else(|| &tax.primitives[0]);
        assert!(faers.dependencies.contains(&"ICSR".to_string()));

        let ev = tax.get("EudraVigilance");
        assert!(ev.is_some());
        let ev = ev.unwrap_or_else(|| &tax.primitives[0]);
        assert!(ev.dependencies.contains(&"ICSR".to_string()));
    }
}
