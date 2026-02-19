use nexcore_vigilance::primitives::governance::*;
use nexcore_vigilance::primitives::measurement::Confidence;

#[test]
fn test_universal_union_simulation() {
    // --- SETUP: THE UNION ---

    // 1. Setup Domains & Factions
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

    // 2. Setup Branches
    let congress = Congress {
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
            quorum_threshold: 0.3, // Lower threshold for high-value asymmetry
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
            quorum_threshold: 0.3,
        },
    };

    let orchestrator = Orchestrator {
        id: "Unity_Exec".into(),
        treasury: Treasury {
            compute_quota: 10000,
            memory_quota: 10000,
        },
        agents: vec![],
        current_cycle: 1,
        risk_minimizer: RiskMinimizer {
            level: RiskMinimizationLevel::Monitoring,
            active_guardrails: vec![],
        },
        power: ExecutivePower {
            energy: Energy(1.0),
            secrecy_level: 1,
            dispatch_rate: Dispatch(0.9),
        },
    };

    let mut pipeline = FederalistPipeline {
        congress,
        orchestrator,
        compiler: SupremeCompiler {
            constitution: vec![],
        },
        stability_audit: StabilityAudit {
            active_factions: vec![],
            total_domains: 5,
        },
    };

    // 3. Setup External Relations & Oracle
    let mut ero = ExternalRelationsOffice {
        active_bonds: vec![],
    };
    let oracle = OracleIntegrationLayer {
        oracle_id: "Deep_Oracle_V1".into(),
        reputation: OracleReputation(0.98),
    };

    // --- EXECUTION: THE SCENARIO ---

    // A. Market Opportunity Detection (Treasury Act)
    let market_state = MarketState {
        market_id: "Drug_Signal_Market".into(),
        odds: Odds::new(0.8), // Market is starting to see it (80%)
        liquidity: Liquidity::new(5000),
    };

    let internal_confidence = Confidence::new(0.98); // Guardian is certain (98%)

    // B. External Validation (Oracle Protocol)
    // Constitutional requirement: External Oracle must confirm high-stakes arbitrage
    let oracle_response = oracle.validate_claim("Signal is Valid", "Query_001");
    let definitive_conf = oracle_response.definitive_confidence();
    assert!(definitive_conf.value() > 0.9);

    // C. Integration Bridge (Market Integration)
    let integration = MarketIntegration {
        pv_domain_id: pv_domain.id.clone(),
        market_domain_id: market_domain.id.clone(),
        signal_transfer_rule: Rule,
        alpha_threshold: 0.1, // Require 10% alpha
    };

    // D. Propose & Execute Governance Cycle (Federalist Pipeline)
    let proposal = integration
        .propose_arbitrage(internal_confidence, &market_state)
        .expect("Arbitrage opportunity should be detected");

    let verdict = pipeline.execute_cycle(proposal).expect("Cycle failed");
    assert_eq!(verdict, Verdict::Permitted);

    // Actually execute the action to increment the cycle
    let cost = integration.transfer_cost();
    pipeline
        .orchestrator
        .execute_action(&Action, &cost)
        .expect("Execution failed");

    // E. External Import (Treaty of Grounding)
    // Naturalize the resulting Market Position data back into the Union
    let foreign_data = ForeignType {
        origin_system: "Polymarket".into(),
        raw_payload: "{ 'position': 'YES', 'shares': 100 }".into(),
    };

    let bond = ero
        .naturalize(foreign_data, "Position_Mapping")
        .expect("Should naturalize foreign data");
    assert_eq!(bond.alignment.0, 0.9);

    // F. Judicial Review (Federalist No. 78)
    let review_engine = JudicialReviewEngine { precedents: vec![] };
    let opinion = review_engine.review_execution(&Action, true); // true = proof of grounding provided
    assert!(!opinion.is_nullified());

    // Final State Check
    assert_eq!(pipeline.orchestrator.current_cycle, 2);
}
