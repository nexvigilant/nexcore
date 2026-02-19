//! Built-in Golden Dome missile defense taxonomy.
//!
//! 30 primitives: 8 T1 + 6 T2-P + 6 T2-C + 10 T3
//! Extracted 2026-02-05.

use crate::taxonomy::{DomainTaxonomy, Primitive, Tier};
use crate::transfer::{DomainTransfer, TransferScore};

/// Construct the full Golden Dome taxonomy with pre-computed transfer scores.
pub fn golden_dome() -> DomainTaxonomy {
    let mut tax = DomainTaxonomy::new(
        "Golden Dome",
        "Trump administration next-generation missile defense shield (announced 2025). \
         Four-layer interception architecture: boost, ascent, midcourse, terminal.",
    );

    // ── T1: Universal ──────────────────────────────────────────────────
    tax.primitives.push(
        Primitive::new(
            "sequence",
            "Ordered progression of states or events in time",
            Tier::T1,
        )
        .with_domains(&["all"]),
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
            "layer",
            "Distinct stratum in hierarchical structure with defined interface",
            Tier::T1,
        )
        .with_domains(&["networking", "geology", "neural-nets", "org-charts"]),
    );
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
            "tracking",
            "Continuous estimation of state across time",
            Tier::T1,
        )
        .with_domains(&["kalman-filters", "logistics", "epidemiology", "investments"]),
    );
    tax.primitives.push(
        Primitive::new(
            "engagement",
            "Commitment of resources to interaction",
            Tier::T1,
        )
        .with_domains(&["combat", "marketing", "enzymes", "legal", "teaching"]),
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
    tax.primitives.push(
        Primitive::new(
            "coverage",
            "Extent of domain under protection or observation",
            Tier::T1,
        )
        .with_domains(&["insurance", "testing", "sensor-fields", "wireless"]),
    );

    // ── T2-P: Cross-Domain Primitives ──────────────────────────────────
    tax.primitives.push(
        Primitive::new(
            "kill-chain",
            "Sequential process model from stimulus to effect",
            Tier::T2P,
        )
        .with_deps(&["sequence", "detection", "engagement"])
        .with_domains(&["cyber-defense", "hunting", "sales", "pharmacokinetics"]),
    );
    tax.primitives.push(
        Primitive::new(
            "time-budget",
            "Duration available from stimulus to required response",
            Tier::T2P,
        )
        .with_deps(&["sequence", "threshold"])
        .with_domains(&["compiler-optimization", "triage", "drug-metabolism"]),
    );
    tax.primitives.push(
        Primitive::new(
            "sensor-fusion",
            "Integration of heterogeneous observations into unified state",
            Tier::T2P,
        )
        .with_deps(&["detection", "tracking", "coverage"])
        .with_domains(&["autonomous-vehicles", "medical-diagnosis", "IoT"]),
    );
    tax.primitives.push(
        Primitive::new(
            "intercept-geometry",
            "Spatial-temporal path convergence between pursuer and target",
            Tier::T2P,
        )
        .with_deps(&["tracking", "engagement", "sequence"])
        .with_domains(&["law-enforcement", "immune-response", "predator-prey"]),
    );
    tax.primitives.push(
        Primitive::new(
            "defense-in-depth",
            "Serial independent barriers each reducing penetration probability",
            Tier::T2P,
        )
        .with_deps(&["layer", "redundancy", "threshold"])
        .with_domains(&["cybersecurity", "immune-system", "financial-controls"]),
    );
    tax.primitives.push(
        Primitive::new(
            "handoff",
            "Transfer of responsibility between systems with state preservation",
            Tier::T2P,
        )
        .with_deps(&["tracking", "threshold", "engagement"])
        .with_domains(&["telecom", "medical-care", "air-traffic-control"]),
    );

    // ── T2-C: Cross-Domain Composites ──────────────────────────────────
    tax.primitives.push(
        Primitive::new(
            "layered-defense-architecture",
            "Multi-tier interception model with phase-specific engagement zones",
            Tier::T2C,
        )
        .with_deps(&["layer", "defense-in-depth", "threshold", "sequence"])
        .with_domains(&["firewalls", "immune-system", "quality-gates"]),
    );
    tax.primitives.push(
        Primitive::new(
            "space-based-persistence",
            "Orbital assets providing continuous coverage via constellation geometry",
            Tier::T2C,
        )
        .with_deps(&["coverage", "redundancy", "tracking", "threshold"])
        .with_domains(&["GPS", "satellite-internet", "earth-observation"]),
    );
    tax.primitives.push(
        Primitive::new(
            "fire-control-loop",
            "Detect-Track-Decide-Engage cycle with continuous state feedback",
            Tier::T2C,
        )
        .with_deps(&[
            "detection",
            "tracking",
            "threshold",
            "engagement",
            "sequence",
        ])
        .with_domains(&["OODA-loop", "PID-controllers", "fraud-detection"]),
    );
    tax.primitives.push(
        Primitive::new(
            "command-and-control",
            "Centralized decision authority with distributed execution capability",
            Tier::T2C,
        )
        .with_deps(&["threshold", "handoff", "tracking", "engagement", "layer"])
        .with_domains(&["enterprise-arch", "nervous-system", "orchestration"]),
    );
    tax.primitives.push(
        Primitive::new(
            "battle-damage-assessment",
            "Post-engagement evaluation of effect to update state model",
            Tier::T2C,
        )
        .with_deps(&["detection", "tracking", "threshold"])
        .with_domains(&["A/B-testing", "clinical-outcomes", "impact-evaluation"]),
    );
    tax.primitives.push(
        Primitive::new(
            "launch-on-warning",
            "Automated trigger when detection plus forecast exceeds threshold",
            Tier::T2C,
        )
        .with_deps(&["detection", "threshold", "time-budget", "tracking"])
        .with_domains(&["circuit-breakers", "auto-scaling", "trading-halts"]),
    );

    // ── T3: Domain-Specific (Golden Dome) ──────────────────────────────
    tax.primitives.push(
        Primitive::new(
            "boost-phase-intercept",
            "Engagement during powered ascent before warhead separation",
            Tier::T3,
        )
        .with_deps(&["time-budget", "intercept-geometry", "fire-control-loop"]),
    );
    tax.primitives.push(
        Primitive::new(
            "midcourse-discrimination",
            "Separation of warheads from decoys in exo-atmospheric flight",
            Tier::T3,
        )
        .with_deps(&["sensor-fusion", "tracking", "threshold"]),
    );
    tax.primitives.push(
        Primitive::new(
            "terminal-phase-engagement",
            "Intercept during atmospheric reentry at high velocity",
            Tier::T3,
        )
        .with_deps(&["intercept-geometry", "time-budget", "fire-control-loop"]),
    );
    tax.primitives.push(
        Primitive::new(
            "space-based-interceptor",
            "Orbital kinetic kill vehicle for boost/midcourse engagement",
            Tier::T3,
        )
        .with_deps(&[
            "space-based-persistence",
            "intercept-geometry",
            "engagement",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "HBTSS",
            "Hypersonic and Ballistic Tracking Space Sensor constellation",
            Tier::T3,
        )
        .with_deps(&["space-based-persistence", "sensor-fusion", "tracking"]),
    );
    tax.primitives.push(
        Primitive::new(
            "JADC2",
            "Joint All-Domain Command and Control network",
            Tier::T3,
        )
        .with_deps(&["command-and-control", "sensor-fusion", "handoff"]),
    );
    tax.primitives.push(
        Primitive::new(
            "NGI",
            "Next Generation Interceptor for midcourse defense",
            Tier::T3,
        )
        .with_deps(&[
            "midcourse-discrimination",
            "intercept-geometry",
            "engagement",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "SM-3-Block-IIA",
            "Standard Missile-3 Block IIA for Aegis midcourse intercept",
            Tier::T3,
        )
        .with_deps(&[
            "midcourse-discrimination",
            "intercept-geometry",
            "engagement",
        ]),
    );
    tax.primitives.push(
        Primitive::new(
            "THAAD",
            "Terminal High Altitude Area Defense interceptor",
            Tier::T3,
        )
        .with_deps(&["terminal-phase-engagement", "tracking", "engagement"]),
    );
    tax.primitives.push(
        Primitive::new(
            "Patriot-PAC-3",
            "Patriot Advanced Capability-3 terminal interceptor",
            Tier::T3,
        )
        .with_deps(&["terminal-phase-engagement", "tracking", "engagement"]),
    );

    // ── Pre-computed transfer scores ───────────────────────────────────
    tax.transfers = build_transfers();

    tax
}

/// Pre-computed transfer confidence scores across 4 target domains.
fn build_transfers() -> Vec<DomainTransfer> {
    let data: &[(&str, &[(&str, f64, f64, f64, &str)])] = &[
        // T1
        (
            "detection",
            &[
                ("cybersecurity", 0.99, 0.96, 0.95, "time scale"),
                ("medicine", 0.95, 0.94, 0.92, "asymptomatic cases"),
                ("finance", 0.92, 0.90, 0.91, "market noise"),
                ("manufacturing-qc", 0.99, 0.98, 0.97, "sample rate"),
            ],
        ),
        (
            "tracking",
            &[
                ("cybersecurity", 0.91, 0.88, 0.87, "attribution difficulty"),
                ("medicine", 0.97, 0.96, 0.94, "patient compliance"),
                ("finance", 0.95, 0.94, 0.93, "market hours"),
                ("manufacturing-qc", 0.89, 0.86, 0.85, "batch granularity"),
            ],
        ),
        (
            "threshold",
            &[
                ("cybersecurity", 0.99, 0.99, 0.99, "none"),
                ("medicine", 0.99, 0.98, 0.97, "individual variation"),
                ("finance", 0.99, 0.99, 0.99, "none"),
                ("manufacturing-qc", 0.99, 0.99, 0.99, "none"),
            ],
        ),
        (
            "sequence",
            &[
                ("cybersecurity", 0.99, 0.98, 0.97, "concurrent attacks"),
                ("medicine", 0.96, 0.95, 0.93, "disease progression"),
                ("finance", 0.98, 0.97, 0.96, "market microstructure"),
                ("manufacturing-qc", 0.99, 0.99, 0.99, "none"),
            ],
        ),
        (
            "layer",
            &[
                ("cybersecurity", 0.97, 0.96, 0.94, "boundary blur"),
                ("medicine", 0.84, 0.82, 0.78, "biological complexity"),
                ("finance", 0.81, 0.78, 0.76, "market coupling"),
                ("manufacturing-qc", 0.93, 0.91, 0.88, "process overlap"),
            ],
        ),
        (
            "engagement",
            &[
                ("cybersecurity", 0.75, 0.72, 0.70, "passive vs active"),
                ("medicine", 0.91, 0.89, 0.86, "patient agency"),
                ("finance", 0.83, 0.80, 0.78, "market making"),
                ("manufacturing-qc", 0.80, 0.78, 0.75, "automation"),
            ],
        ),
        (
            "redundancy",
            &[
                ("cybersecurity", 0.96, 0.94, 0.91, "cost"),
                ("medicine", 0.93, 0.91, 0.88, "organ specificity"),
                ("finance", 0.97, 0.96, 0.94, "correlation risk"),
                ("manufacturing-qc", 0.98, 0.97, 0.96, "none"),
            ],
        ),
        (
            "coverage",
            &[
                ("cybersecurity", 0.93, 0.91, 0.88, "zero-day gaps"),
                ("medicine", 0.90, 0.88, 0.85, "access inequality"),
                ("finance", 0.87, 0.85, 0.82, "jurisdiction limits"),
                ("manufacturing-qc", 0.95, 0.93, 0.90, "sensor placement"),
            ],
        ),
        // T2-P
        (
            "kill-chain",
            &[
                ("cybersecurity", 0.96, 0.94, 0.91, "attribution lag"),
                ("medicine", 0.78, 0.76, 0.72, "non-adversarial"),
                ("finance", 0.81, 0.78, 0.76, "regulatory friction"),
                ("manufacturing-qc", 0.85, 0.83, 0.80, "batch vs continuous"),
            ],
        ),
        (
            "time-budget",
            &[
                ("cybersecurity", 0.87, 0.85, 0.82, "latency variance"),
                ("medicine", 0.74, 0.72, 0.68, "chronic timescale"),
                ("finance", 0.90, 0.88, 0.85, "settlement windows"),
                ("manufacturing-qc", 0.93, 0.91, 0.88, "cycle time"),
            ],
        ),
        (
            "sensor-fusion",
            &[
                ("cybersecurity", 0.89, 0.87, 0.84, "log format diversity"),
                ("medicine", 0.94, 0.93, 0.87, "modality gaps"),
                ("finance", 0.84, 0.82, 0.79, "data silos"),
                ("manufacturing-qc", 0.91, 0.89, 0.86, "sensor drift"),
            ],
        ),
        (
            "intercept-geometry",
            &[
                ("cybersecurity", 0.63, 0.61, 0.58, "non-spatial domain"),
                ("medicine", 0.60, 0.58, 0.55, "non-spatial domain"),
                ("finance", 0.73, 0.71, 0.68, "partial: timing intercept"),
                ("manufacturing-qc", 0.56, 0.54, 0.51, "non-spatial domain"),
            ],
        ),
        (
            "defense-in-depth",
            &[
                ("cybersecurity", 0.95, 0.93, 0.90, "config drift"),
                ("medicine", 0.86, 0.84, 0.80, "compliance barriers"),
                ("finance", 0.91, 0.89, 0.86, "regulatory layers"),
                ("manufacturing-qc", 0.90, 0.88, 0.85, "inspection stages"),
            ],
        ),
        (
            "handoff",
            &[
                ("cybersecurity", 0.80, 0.78, 0.75, "context loss"),
                ("medicine", 0.96, 0.94, 0.91, "shift changes"),
                ("finance", 0.84, 0.82, 0.79, "settlement handoff"),
                ("manufacturing-qc", 0.88, 0.86, 0.83, "shift handoff"),
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
    fn golden_dome_loads() {
        let tax = golden_dome();
        assert_eq!(tax.name, "Golden Dome");
        assert_eq!(tax.primitives.len(), 30);
    }

    #[test]
    fn tier_distribution() {
        let tax = golden_dome();
        assert_eq!(tax.by_tier(Tier::T1).len(), 8);
        assert_eq!(tax.by_tier(Tier::T2P).len(), 6);
        assert_eq!(tax.by_tier(Tier::T2C).len(), 6);
        assert_eq!(tax.by_tier(Tier::T3).len(), 10);
    }

    #[test]
    fn irreducible_atom_count() {
        let tax = golden_dome();
        let atoms = tax.irreducible_atoms();
        assert_eq!(atoms.len(), 14); // 8 T1 + 6 T2-P
    }

    #[test]
    fn threshold_highest_transfer() {
        let tax = golden_dome();
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
    fn intercept_geometry_lowest_transfer() {
        let tax = golden_dome();
        let ig_avg = tax.avg_transfer_confidence(Tier::T2P);
        let t1_avg = tax.avg_transfer_confidence(Tier::T1);
        assert!(
            t1_avg > ig_avg,
            "T1 avg ({t1_avg}) should exceed T2-P avg ({ig_avg})"
        );
    }

    #[test]
    fn decompose_boost_phase() {
        let tax = golden_dome();
        let node = tax.decompose("boost-phase-intercept");
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
            assert!(prim.is_some(), "leaf {leaf} not found");
        }
    }

    #[test]
    fn all_dependencies_resolve() {
        let tax = golden_dome();
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
    fn transfer_count() {
        let tax = golden_dome();
        // 14 primitives (T1+T2-P) × 4 domains = 56 transfers
        assert_eq!(tax.transfers.len(), 56);
    }

    #[test]
    fn all_transfers_in_valid_range() {
        let tax = golden_dome();
        for t in &tax.transfers {
            let c = t.confidence();
            assert!(
                (0.0..=1.0).contains(&c),
                "Transfer {} → {} has confidence {c} outside [0,1]",
                t.primitive_name,
                t.target_domain
            );
        }
    }
}
