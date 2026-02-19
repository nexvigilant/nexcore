use nexcore_vigilance::primitives::governance::*;
use nexcore_vigilance::primitives::measurement::Confidence;

#[test]
fn test_economic_and_administrative_union() {
    // 1. Setup Market Domain (Treasury Act)
    let market_state = MarketState {
        market_id: "Market_1".into(),
        odds: Odds::new(0.6), // Market thinks 60% likely
        liquidity: Liquidity::new(1000),
    };

    // 2. Setup PV Intelligence (Guardian)
    let internal_confidence = Confidence::new(0.95); // Guardian knows 95% likely

    // 3. Integration Bridge
    let integration = MarketIntegration {
        pv_domain_id: "PV".into(),
        market_domain_id: "Markets".into(),
        signal_transfer_rule: Rule,
        alpha_threshold: 0.2, // 0.9 - 0.6 = 0.3 (Meets threshold)
    };

    // 4. Propose Arbitrage (Asymmetry detected)
    let _arbitrage_proposal = integration
        .propose_arbitrage(internal_confidence, &market_state)
        .expect("Should detect asymmetry");

    // 5. Setup Orchestrator (Executive Unity - Federalist No. 70)
    let orchestrator = Orchestrator {
        id: "Unity_Exec".into(),
        treasury: Treasury {
            compute_quota: 5000,
            memory_quota: 5000,
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

    // 6. Setup Stability Audit (Faction Control - Federalist No. 10)
    let stability_audit = StabilityAudit {
        active_factions: vec![], // Simulation of high pluralism
        total_domains: 5,
    };

    let mut pipeline = FederalistPipeline {
        congress: Congress {
            house: HouseOfT1 {
                members: vec![],
                quorum_threshold: 0.5,
            },
            senate: SenateOfT2 {
                members: vec![],
                quorum_threshold: 0.5,
            },
        },
        orchestrator,
        compiler: SupremeCompiler {
            constitution: vec![],
        },
        stability_audit,
    };

    // 7. Execute Simulation Cycle
    // For this test, we bypass Congress manually to test the executive dispatch path
    if pipeline
        .orchestrator
        .power
        .execute_with_dispatch(&Action, internal_confidence)
    {
        let cost = integration.transfer_cost();
        pipeline
            .orchestrator
            .execute_action(&Action, &cost)
            .expect("Action failed");
    }

    // 8. Administrative Review (APA)
    let rule = AdministrativeRule {
        target_agency_id: "Orchestrator".into(),
        procedure: Procedure(vec!["Propose".into(), "Execute".into()]),
        constraint: Rule,
    };
    let log = AgencyLog {
        agency_id: "Orchestrator".into(),
        action_taken: "Arbitrage".into(),
        compliance_score: Compliance(0.95),
    };

    assert_eq!(rule.judicial_review(&log), Verdict::Permitted);
    assert_eq!(pipeline.orchestrator.current_cycle, 2);
}
