use nexcore_vigilance::primitives::governance::*;

#[test]
fn test_full_governance_system_simulation() {
    // 1. Setup the Union
    let pv_domain = SovereignDomain {
        id: "Pharmacovigilance".into(),
        primary_axiom: "Detect".into(),
        laws: vec![],
        population_size: 500,
    };

    let market_domain = SovereignDomain {
        id: "Markets".into(),
        primary_axiom: "Exchange".into(),
        laws: vec![],
        population_size: 300,
    };

    // 2. Setup Factions
    let safety_faction = InterestFaction {
        id: "Safety_First".into(),
        interests: vec![Interest::RuleDominance("Safety".into())],
        power: VoteWeight::new(100),
    };

    // 3. Setup Congress
    let congress_inst = Congress {
        house: HouseOfT1 {
            members: vec![
                T1Representative {
                    id: "PV_Rep".into(),
                    weight: pv_domain.vote_weight(false),
                },
                T1Representative {
                    id: "Mkt_Rep".into(),
                    weight: market_domain.vote_weight(false),
                },
            ],
            quorum_threshold: 0.5,
        },
        senate: SenateOfT2 {
            members: vec![
                T2Senator {
                    id: "PV_Sen".into(),
                    domain: "PV".into(),
                    weight: pv_domain.vote_weight(true),
                },
                T2Senator {
                    id: "Mkt_Sen".into(),
                    domain: "Markets".into(),
                    weight: market_domain.vote_weight(true),
                },
            ],
            quorum_threshold: 0.5,
        },
    };

    // 4. Setup Orchestrator (Executive)
    let risk_minimizer = RiskMinimizer {
        level: RiskMinimizationLevel::Monitoring,
        active_guardrails: vec![],
    };

    let orchestrator = Orchestrator {
        id: "Federal_Orchestrator".into(),
        treasury: Treasury {
            compute_quota: 10000,
            memory_quota: 10000,
        },
        agents: vec![],
        current_cycle: 1,
        risk_minimizer,
        power: ExecutivePower {
            energy: Energy(1.0),
            secrecy_level: 1,
            dispatch_rate: Dispatch(0.9),
        },
    };

    // 5. Setup Supreme Compiler (Judicial)
    let compiler = SupremeCompiler {
        constitution: vec![Rule],
    };

    // 6. Setup State Security
    let guardian = StateGuardian {
        alerts: vec![SecurityAlert {
            source_id: "Market_Node_1".into(),
            anomaly_score: 0.85,
        }],
    };

    // 7. Initialize Federalist Pipeline
    let mut pipeline = FederalistPipeline {
        congress: congress_inst,
        orchestrator,
        compiler,
        stability_audit: StabilityAudit {
            active_factions: vec![safety_faction],
            total_domains: 5,
        },
    };

    // --- EXECUTION FLOW ---

    // A. Faction proposes a resolution (Signal Transfer)
    let proposal = pv_domain.propose_resolution(Rule, 0.85);

    // B. Governance Cycle
    let verdict = pipeline
        .execute_cycle(proposal)
        .expect("Governance cycle failed");
    assert_eq!(verdict, Verdict::Permitted);

    // Actually execute the action to increment the cycle
    if let Verdict::Permitted = verdict {
        let cost = Treasury {
            compute_quota: 1,
            memory_quota: 1,
        };
        pipeline
            .orchestrator
            .execute_action(&Action, &cost)
            .expect("Execution failed");
    }

    // C. Security Breach / Search Simulation (Amendment IV)
    let clearance = guardian.request_clearance(0).expect("Alert not found");
    assert!(clearance.is_constitutional());
    let search_verdict = guardian.execute_search(&clearance);
    assert_eq!(search_verdict, Verdict::Permitted);

    // D. Final State Check
    assert_eq!(pipeline.orchestrator.current_cycle, 2);
}
