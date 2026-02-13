use nexcore_vigilance::primitives::governance::*;
use nexcore_vigilance::primitives::measurement::{Confidence, Measured};

#[test]
fn test_state_of_the_union_simulation() {
    // 1. Setup the Three Branches (Federalist No. 51)
    let congress = Congress {
        house: HouseOfT1 {
            members: vec![
                T1Representative {
                    id: "Rep1".into(),
                    weight: VoteWeight::new(50),
                },
                T1Representative {
                    id: "Rep2".into(),
                    weight: VoteWeight::new(50),
                },
            ],
            quorum_threshold: 0.5,
        },
        senate: SenateOfT2 {
            members: vec![
                T2Senator {
                    id: "Sen1".into(),
                    domain: "PV".into(),
                    weight: VoteWeight::new(50),
                },
                T2Senator {
                    id: "Sen2".into(),
                    domain: "Markets".into(),
                    weight: VoteWeight::new(50),
                },
            ],
            quorum_threshold: 0.5,
        },
    };

    let orchestrator = Orchestrator {
        id: "The_Orchestrator".into(),
        treasury: Treasury {
            compute_quota: 5000,
            memory_quota: 5000,
        },
        agents: vec![Agent {
            id: "Sentinel1".into(),
            department: "Executive".into(),
            capability: 0.95,
        }],
        current_cycle: 0,
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

    let supreme_compiler = SupremeCompiler {
        constitution: vec![Rule],
    };

    let mut pipeline = FederalistPipeline {
        congress,
        orchestrator,
        compiler: supreme_compiler,
        stability_audit: StabilityAudit {
            active_factions: vec![],
            total_domains: 5,
        },
    };

    // 2. Scenario A: A Constitutional Resolution (Safe)
    let safe_resolution = Measured::uncertain(Rule, Confidence::new(0.9)); // High rigor

    let verdict_a = pipeline
        .execute_cycle(safe_resolution)
        .expect("Cycle failed");
    assert_eq!(verdict_a, Verdict::Permitted, "Safe resolution should pass");

    // 3. Scenario B: An Unconstitutional Resolution (Low Rigor)
    let risky_resolution = Measured::uncertain(Rule, Confidence::new(0.2)); // Low rigor

    let verdict_b = pipeline
        .execute_cycle(risky_resolution)
        .expect("Cycle failed");
    assert_eq!(
        verdict_b,
        Verdict::Rejected,
        "Risky resolution should be rejected by Congress or Compiler"
    );

    // 4. Scenario C: Executive Veto (Moderate Rigor, but Executive disagrees)
    let veto_resolution = Measured::uncertain(Rule, Confidence::new(0.65)); // Passes Congress barrier (0.5) but not Executive barrier (0.7)

    let verdict_c = pipeline
        .execute_cycle(veto_resolution)
        .expect("Cycle failed");
    assert_eq!(
        verdict_c,
        Verdict::Rejected,
        "Resolution should be vetoed by the Orchestrator"
    );
}
