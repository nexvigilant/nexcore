//! Built-in Cybersecurity domain taxonomy.
//!
//! 30 primitives: 8 T1 + 6 T2-P + 6 T2-C + 10 T3
//! Extracted 2026-02-05.

use crate::taxonomy::{DomainTaxonomy, Primitive, Tier};
use crate::transfer::{DomainTransfer, TransferScore};

/// Construct the full Cybersecurity taxonomy with pre-computed transfer scores.
pub fn cybersecurity() -> DomainTaxonomy {
    let mut tax = DomainTaxonomy::new(
        "Cybersecurity",
        "Science and practice of protecting networks, systems, and data from digital attacks. \
         Encompasses threat detection (SIEM, EDR), identity management (IAM, PKI), \
         vulnerability management (CVE, penetration testing), and incident response (CSIRT, SOAR).",
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
            "authentication",
            "Verification of claimed identity through proof of knowledge, possession, or inherence",
            Tier::T1,
        )
        .with_domains(&[
            "border-control",
            "banking",
            "physical-access",
            "digital-signatures",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "encryption",
            "Reversible transformation of data to ensure confidentiality using cryptographic keys",
            Tier::T1,
        )
        .with_domains(&[
            "military-communications",
            "banking",
            "diplomacy",
            "messaging",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "isolation",
            "Containment of entities within boundaries to prevent unauthorized interaction",
            Tier::T1,
        )
        .with_domains(&[
            "quarantine",
            "sandboxing",
            "clean-rooms",
            "network-segmentation",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "redundancy",
            "Multiple independent paths to same outcome",
            Tier::T1,
        )
        .with_domains(&[
            "fault-tolerance",
            "genetics",
            "supply-chains",
            "info-theory",
        ]),
    );

    // ── T2-P: Cross-Domain Primitives ──────────────────────────────────
    tax.primitives.push(
        Primitive::new(
            "defense-in-depth",
            "Serial independent barriers each reducing penetration probability",
            Tier::T2P,
        )
        .with_deps(&["isolation", "redundancy", "threshold"])
        .with_domains(&["missile-defense", "immune-system", "financial-controls"]),
    );
    tax.primitives.push(
        Primitive::new(
            "threat-intelligence",
            "Systematic collection and analysis of adversary patterns, capabilities, and intent",
            Tier::T2P,
        )
        .with_deps(&["detection", "tracking", "classification"])
        .with_domains(&[
            "military-intelligence",
            "epidemiology",
            "competitive-analysis",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "access-control",
            "Permission-based restriction of resource availability to authorized entities",
            Tier::T2P,
        )
        .with_deps(&["authentication", "classification"])
        .with_domains(&[
            "physical-security",
            "database-systems",
            "operating-systems",
            "healthcare-records",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "vulnerability-scanning",
            "Systematic identification of weaknesses through automated probing and analysis",
            Tier::T2P,
        )
        .with_deps(&["detection", "threshold", "classification"])
        .with_domains(&[
            "quality-assurance",
            "structural-engineering",
            "medical-screening",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "incident-response",
            "Structured workflow for detecting, containing, eradicating, and recovering from breaches",
            Tier::T2P,
        )
        .with_deps(&["detection", "tracking", "isolation"])
        .with_domains(&[
            "emergency-management",
            "medical-triage",
            "disaster-recovery",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "log-correlation",
            "Multi-source event correlation linking disparate observations to unified incidents",
            Tier::T2P,
        )
        .with_deps(&["detection", "tracking", "classification"])
        .with_domains(&["forensics", "epidemiology", "supply-chain-tracking"]),
    );

    // ── T2-C: Cross-Domain Composites ──────────────────────────────────
    tax.primitives.push(
        Primitive::new(
            "SIEM",
            "Security Information and Event Management: real-time log aggregation, correlation, and alert pipeline",
            Tier::T2C,
        )
        .with_deps(&["detection", "log-correlation", "classification", "threshold"])
        .with_domains(&["IT-operations", "compliance-monitoring", "fraud-detection"]),
    );
    tax.primitives.push(
        Primitive::new(
            "zero-trust-architecture",
            "Never trust, always verify: continuous authentication and micro-segmentation framework",
            Tier::T2C,
        )
        .with_deps(&["authentication", "access-control", "isolation", "defense-in-depth"])
        .with_domains(&["enterprise-architecture", "cloud-native", "zero-knowledge-proofs"]),
    );
    tax.primitives.push(
        Primitive::new(
            "threat-hunting",
            "Proactive adversary search combining intelligence hypotheses with detection analytics",
            Tier::T2C,
        )
        .with_deps(&[
            "threat-intelligence",
            "detection",
            "log-correlation",
            "tracking",
        ])
        .with_domains(&[
            "epidemiological-investigation",
            "counter-intelligence",
            "fraud-investigation",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "security-orchestration",
            "Automated response workflow integrating tools, processes, and human decisions (SOAR)",
            Tier::T2C,
        )
        .with_deps(&[
            "incident-response",
            "log-correlation",
            "classification",
            "tracking",
        ])
        .with_domains(&[
            "DevOps-automation",
            "emergency-dispatch",
            "supply-chain-orchestration",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "penetration-testing",
            "Authorized attack simulation methodology to validate defensive controls",
            Tier::T2C,
        )
        .with_deps(&[
            "vulnerability-scanning",
            "threat-intelligence",
            "detection",
            "isolation",
        ])
        .with_domains(&["red-teaming", "stress-testing", "quality-assurance"]),
    );
    tax.primitives.push(
        Primitive::new(
            "compliance-framework",
            "Regulatory standard adherence program ensuring controls meet external requirements (SOC2, ISO27001)",
            Tier::T2C,
        )
        .with_deps(&["vulnerability-scanning", "access-control", "log-correlation", "incident-response"])
        .with_domains(&["financial-regulation", "healthcare-compliance", "manufacturing-standards"]),
    );

    // ── T3: Domain-Specific (Cybersecurity) ──────────────────────────
    tax.primitives.push(
        Primitive::new(
            "MITRE-ATT&CK",
            "Adversary tactics, techniques, and procedures knowledge base for threat modeling and detection engineering",
            Tier::T3,
        )
        .with_deps(&["threat-intelligence", "threat-hunting", "classification"]),
    );
    tax.primitives.push(
        Primitive::new(
            "CVE-management",
            "Common Vulnerabilities and Exposures lifecycle: identification, scoring (CVSS), patching, and tracking",
            Tier::T3,
        )
        .with_deps(&["vulnerability-scanning", "threshold", "tracking"]),
    );
    tax.primitives.push(
        Primitive::new(
            "SOC-operations",
            "Security Operations Center workflow: 24/7 monitoring, triage, escalation, and response coordination",
            Tier::T3,
        )
        .with_deps(&["SIEM", "detection", "log-correlation", "incident-response"]),
    );
    tax.primitives.push(
        Primitive::new(
            "EDR",
            "Endpoint Detection and Response: continuous endpoint monitoring, behavioral analysis, and automated containment",
            Tier::T3,
        )
        .with_deps(&["detection", "tracking", "isolation", "log-correlation"]),
    );
    tax.primitives.push(
        Primitive::new(
            "WAF",
            "Web Application Firewall: HTTP traffic filtering using rule sets and behavioral analysis",
            Tier::T3,
        )
        .with_deps(&["detection", "access-control", "threshold", "classification"]),
    );
    tax.primitives.push(
        Primitive::new(
            "IAM",
            "Identity and Access Management platform: centralized authentication, authorization, and audit",
            Tier::T3,
        )
        .with_deps(&["authentication", "access-control", "zero-trust-architecture"]),
    );
    tax.primitives.push(
        Primitive::new(
            "DLP",
            "Data Loss Prevention system: content inspection, policy enforcement, and exfiltration blocking",
            Tier::T3,
        )
        .with_deps(&["classification", "log-correlation", "encryption", "access-control"]),
    );
    tax.primitives.push(
        Primitive::new(
            "CSIRT",
            "Computer Security Incident Response Team: structured organizational unit for coordinated breach response",
            Tier::T3,
        )
        .with_deps(&["incident-response", "tracking", "security-orchestration"]),
    );
    tax.primitives.push(
        Primitive::new(
            "PKI",
            "Public Key Infrastructure: certificate authority hierarchy, issuance, revocation, and trust chain management",
            Tier::T3,
        )
        .with_deps(&["encryption", "authentication"]),
    );
    tax.primitives.push(
        Primitive::new(
            "honeypot",
            "Deception-based threat intelligence collection using deliberately vulnerable decoy systems",
            Tier::T3,
        )
        .with_deps(&["threat-intelligence", "detection", "isolation"]),
    );

    // ── Pre-computed transfer scores ───────────────────────────────────
    tax.transfers = build_transfers();

    tax
}

/// Pre-computed transfer confidence scores across 4 target domains.
///
/// Target domains: missile-defense, pharmacovigilance, finance, manufacturing-qc.
/// Shared T1 primitives (detection, threshold, tracking, redundancy) transfer HIGH (0.85-0.99).
/// Cyber-specific T3 concepts have LOW confidence to non-cyber domains (0.30-0.50).
fn build_transfers() -> Vec<DomainTransfer> {
    let data: &[(&str, &[(&str, f64, f64, f64, &str)])] = &[
        // ── T1 ───────────────────────────────────────────────────────────
        (
            "detection",
            &[
                ("missile-defense", 0.99, 0.96, 0.95, "time scale"),
                ("pharmacovigilance", 0.99, 0.96, 0.95, "time scale"),
                ("finance", 0.92, 0.90, 0.91, "market noise"),
                ("manufacturing-qc", 0.99, 0.98, 0.97, "sample rate"),
            ],
        ),
        (
            "threshold",
            &[
                ("missile-defense", 0.99, 0.99, 0.99, "none"),
                ("pharmacovigilance", 0.99, 0.99, 0.99, "none"),
                ("finance", 0.99, 0.99, 0.99, "none"),
                ("manufacturing-qc", 0.99, 0.99, 0.99, "none"),
            ],
        ),
        (
            "tracking",
            &[
                ("missile-defense", 0.93, 0.91, 0.88, "temporal granularity"),
                ("pharmacovigilance", 0.91, 0.88, 0.87, "patient timescale"),
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
                ("pharmacovigilance", 0.90, 0.88, 0.85, "MedDRA ontology"),
                ("finance", 0.88, 0.86, 0.84, "asset classes"),
                ("manufacturing-qc", 0.93, 0.91, 0.88, "defect taxonomy"),
            ],
        ),
        (
            "authentication",
            &[
                (
                    "missile-defense",
                    0.78,
                    0.76,
                    0.72,
                    "IFF system differences",
                ),
                (
                    "pharmacovigilance",
                    0.65,
                    0.62,
                    0.58,
                    "non-adversarial domain",
                ),
                ("finance", 0.95, 0.93, 0.91, "KYC/AML alignment"),
                (
                    "manufacturing-qc",
                    0.72,
                    0.70,
                    0.66,
                    "operator verification",
                ),
            ],
        ),
        (
            "encryption",
            &[
                ("missile-defense", 0.88, 0.86, 0.83, "COMSEC alignment"),
                (
                    "pharmacovigilance",
                    0.60,
                    0.58,
                    0.54,
                    "data protection only",
                ),
                ("finance", 0.92, 0.90, 0.87, "transaction security"),
                (
                    "manufacturing-qc",
                    0.55,
                    0.52,
                    0.48,
                    "minimal encryption need",
                ),
            ],
        ),
        (
            "isolation",
            &[
                ("missile-defense", 0.85, 0.83, 0.80, "physical separation"),
                ("pharmacovigilance", 0.70, 0.68, 0.64, "quarantine analogy"),
                ("finance", 0.78, 0.76, 0.72, "ring-fencing"),
                ("manufacturing-qc", 0.82, 0.80, 0.76, "clean room isolation"),
            ],
        ),
        (
            "redundancy",
            &[
                ("missile-defense", 0.96, 0.94, 0.91, "cost"),
                ("pharmacovigilance", 0.78, 0.76, 0.72, "reporting channels"),
                ("finance", 0.97, 0.96, 0.94, "correlation risk"),
                ("manufacturing-qc", 0.98, 0.97, 0.96, "none"),
            ],
        ),
        // ── T2-P ─────────────────────────────────────────────────────────
        (
            "defense-in-depth",
            &[
                ("missile-defense", 0.95, 0.93, 0.90, "layered intercept"),
                (
                    "pharmacovigilance",
                    0.72,
                    0.70,
                    0.66,
                    "safety monitoring layers",
                ),
                ("finance", 0.88, 0.86, 0.83, "control layers"),
                ("manufacturing-qc", 0.90, 0.88, 0.85, "inspection stages"),
            ],
        ),
        (
            "threat-intelligence",
            &[
                ("missile-defense", 0.85, 0.83, 0.80, "adversary modeling"),
                (
                    "pharmacovigilance",
                    0.62,
                    0.60,
                    0.56,
                    "non-adversarial domain",
                ),
                ("finance", 0.78, 0.76, 0.72, "market intelligence"),
                ("manufacturing-qc", 0.55, 0.52, 0.48, "no adversary concept"),
            ],
        ),
        (
            "access-control",
            &[
                ("missile-defense", 0.80, 0.78, 0.75, "clearance systems"),
                (
                    "pharmacovigilance",
                    0.68,
                    0.65,
                    0.61,
                    "data access governance",
                ),
                ("finance", 0.92, 0.90, 0.87, "authorization models"),
                ("manufacturing-qc", 0.75, 0.73, 0.70, "operator permissions"),
            ],
        ),
        (
            "vulnerability-scanning",
            &[
                ("missile-defense", 0.72, 0.70, 0.66, "physical vs digital"),
                ("pharmacovigilance", 0.65, 0.62, 0.58, "safety screening"),
                ("finance", 0.78, 0.76, 0.72, "risk assessment"),
                ("manufacturing-qc", 0.85, 0.83, 0.80, "defect screening"),
            ],
        ),
        (
            "incident-response",
            &[
                ("missile-defense", 0.82, 0.80, 0.76, "engagement protocol"),
                (
                    "pharmacovigilance",
                    0.80,
                    0.78,
                    0.74,
                    "adverse event handling",
                ),
                ("finance", 0.85, 0.83, 0.80, "crisis management"),
                (
                    "manufacturing-qc",
                    0.88,
                    0.86,
                    0.83,
                    "nonconformance handling",
                ),
            ],
        ),
        (
            "log-correlation",
            &[
                ("missile-defense", 0.78, 0.76, 0.72, "sensor data fusion"),
                (
                    "pharmacovigilance",
                    0.75,
                    0.73,
                    0.69,
                    "signal mining analogy",
                ),
                ("finance", 0.82, 0.80, 0.76, "transaction correlation"),
                ("manufacturing-qc", 0.85, 0.83, 0.80, "SPC data streams"),
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
    fn cybersecurity_loads() {
        let tax = cybersecurity();
        assert_eq!(tax.name, "Cybersecurity");
        assert_eq!(tax.primitives.len(), 30);
    }

    #[test]
    fn tier_distribution() {
        let tax = cybersecurity();
        assert_eq!(tax.by_tier(Tier::T1).len(), 8);
        assert_eq!(tax.by_tier(Tier::T2P).len(), 6);
        assert_eq!(tax.by_tier(Tier::T2C).len(), 6);
        assert_eq!(tax.by_tier(Tier::T3).len(), 10);
    }

    #[test]
    fn irreducible_atom_count() {
        let tax = cybersecurity();
        let atoms = tax.irreducible_atoms();
        assert_eq!(atoms.len(), 14); // 8 T1 + 6 T2-P
    }

    #[test]
    fn all_primitives_have_definitions() {
        let tax = cybersecurity();
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
        let tax = cybersecurity();
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
        let tax = cybersecurity();
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
        let tax = cybersecurity();
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
        let tax = cybersecurity();
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
        let tax = cybersecurity();
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
        let tax = cybersecurity();
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
        let tax = cybersecurity();
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
    fn t1_avg_exceeds_t2p_avg() {
        let tax = cybersecurity();
        let t1_avg = tax.avg_transfer_confidence(Tier::T1);
        let t2p_avg = tax.avg_transfer_confidence(Tier::T2P);
        assert!(
            t1_avg > t2p_avg,
            "T1 avg ({t1_avg}) should exceed T2-P avg ({t2p_avg})"
        );
    }

    #[test]
    fn transfer_count() {
        let tax = cybersecurity();
        // 14 primitives (T1+T2-P) x 4 domains = 56 transfers
        assert_eq!(tax.transfers.len(), 56);
    }

    #[test]
    fn all_transfers_in_valid_range() {
        let tax = cybersecurity();
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
        let cyber = cybersecurity();
        let gd = crate::golden_dome::golden_dome();
        let cyber_t1: Vec<&str> = cyber
            .by_tier(Tier::T1)
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        let gd_t1: Vec<&str> = gd
            .by_tier(Tier::T1)
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        // detection, threshold, tracking, redundancy should be in both
        let shared = ["detection", "threshold", "tracking", "redundancy"];
        for name in &shared {
            assert!(
                cyber_t1.contains(name),
                "Cybersecurity taxonomy missing shared T1 '{name}'"
            );
            assert!(
                gd_t1.contains(name),
                "Golden Dome taxonomy missing shared T1 '{name}'"
            );
        }
    }

    #[test]
    fn shared_t1_with_pharmacovigilance() {
        let cyber = cybersecurity();
        let pv = crate::pharmacovigilance::pharmacovigilance();
        let cyber_t1: Vec<&str> = cyber
            .by_tier(Tier::T1)
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        let pv_t1: Vec<&str> = pv
            .by_tier(Tier::T1)
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        // detection, threshold, tracking, classification should be in both
        let shared = ["detection", "threshold", "tracking", "classification"];
        for name in &shared {
            assert!(
                cyber_t1.contains(name),
                "Cybersecurity taxonomy missing shared T1 '{name}'"
            );
            assert!(
                pv_t1.contains(name),
                "Pharmacovigilance taxonomy missing shared T1 '{name}'"
            );
        }
    }

    #[test]
    fn shared_t2p_defense_in_depth_with_golden_dome() {
        let cyber = cybersecurity();
        let gd = crate::golden_dome::golden_dome();
        assert!(
            cyber.get("defense-in-depth").is_some(),
            "Cybersecurity missing defense-in-depth"
        );
        assert!(
            gd.get("defense-in-depth").is_some(),
            "Golden Dome missing defense-in-depth"
        );
    }

    #[test]
    fn unique_primitive_names() {
        let tax = cybersecurity();
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
    fn cyber_specific_primitives_present() {
        let tax = cybersecurity();
        let expected = [
            "MITRE-ATT&CK",
            "CVE-management",
            "SOC-operations",
            "EDR",
            "WAF",
            "IAM",
            "DLP",
            "CSIRT",
            "PKI",
            "honeypot",
        ];
        for name in &expected {
            assert!(
                tax.get(name).is_some(),
                "Missing cyber-specific primitive: {name}"
            );
        }
    }

    #[test]
    fn dag_is_acyclic() {
        let tax = cybersecurity();
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
    fn decompose_soc_operations() {
        let tax = cybersecurity();
        let node = tax.decompose("SOC-operations");
        assert!(node.is_some());
        let node = node.unwrap_or_else(|| crate::taxonomy::DecompositionNode {
            name: String::new(),
            tier: Tier::T1,
            children: vec![],
        });
        let leaves = node.leaves();
        for leaf in &leaves {
            let prim = tax.get(leaf);
            assert!(prim.is_some(), "leaf {leaf} not found in taxonomy");
        }
    }

    #[test]
    fn decompose_iam() {
        let tax = cybersecurity();
        let node = tax.decompose("IAM");
        assert!(node.is_some());
        let node = node.unwrap_or_else(|| crate::taxonomy::DecompositionNode {
            name: String::new(),
            tier: Tier::T1,
            children: vec![],
        });
        // IAM -> zero-trust-architecture -> defense-in-depth -> T1s
        assert!(
            node.depth() >= 2,
            "IAM decomposition should be at least 2 deep, got {}",
            node.depth()
        );
    }

    #[test]
    fn detection_to_siem_to_soc_chain() {
        let tax = cybersecurity();
        let siem = tax.get("SIEM");
        assert!(siem.is_some());
        let siem = siem.unwrap_or_else(|| &tax.primitives[0]);
        assert!(siem.dependencies.contains(&"detection".to_string()));
        assert!(siem.dependencies.contains(&"log-correlation".to_string()));

        let soc = tax.get("SOC-operations");
        assert!(soc.is_some());
        let soc = soc.unwrap_or_else(|| &tax.primitives[0]);
        assert!(soc.dependencies.contains(&"SIEM".to_string()));
    }

    #[test]
    fn authentication_to_iam_chain() {
        let tax = cybersecurity();
        let ac = tax.get("access-control");
        assert!(ac.is_some());
        let ac = ac.unwrap_or_else(|| &tax.primitives[0]);
        assert!(ac.dependencies.contains(&"authentication".to_string()));

        let iam = tax.get("IAM");
        assert!(iam.is_some());
        let iam = iam.unwrap_or_else(|| &tax.primitives[0]);
        assert!(iam.dependencies.contains(&"access-control".to_string()));
        assert!(iam.dependencies.contains(&"authentication".to_string()));
    }
}
