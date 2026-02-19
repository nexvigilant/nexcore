use nexcore_vigilance::primitives::governance::*;
use nexcore_vigilance::primitives::measurement::{Confidence, Measured};

#[test]
fn test_governance_simulation() {
    // 1. Setup Legislative Branch
    let congress = Congress {
        house: HouseOfT1 {
            members: vec![
                T1Representative {
                    id: "Rep1".into(),
                    weight: VoteWeight::new(10),
                },
                T1Representative {
                    id: "Rep2".into(),
                    weight: VoteWeight::new(10),
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
                    domain: "AI".into(),
                    weight: VoteWeight::new(50),
                },
            ],
            quorum_threshold: 0.5,
        },
    };

    // 2. Setup Executive Branch
    let mut orchestrator = Orchestrator {
        id: "Central_Exec".into(),
        treasury: Treasury {
            compute_quota: 1000,
            memory_quota: 1000,
        },
        agents: vec![Agent {
            id: "Agent1".into(),
            department: "Treasury".into(),
            capability: 0.9,
        }],
        current_cycle: 0,
        risk_minimizer: RiskMinimizer {
            level: RiskMinimizationLevel::Information,
            active_guardrails: vec![],
        },
        power: ExecutivePower {
            energy: Energy(1.0),
            secrecy_level: 0,
            dispatch_rate: Dispatch(0.5),
        },
    };

    // 3. Setup Judicial Branch
    let supreme_compiler = SupremeCompiler {
        constitution: vec![Rule],
    };

    // 4. A Resolution is proposed
    let resolution = Measured::uncertain(Rule, Confidence::new(0.8)); // High confidence

    // 5. Legislative Review
    assert!(
        congress.pass_bill(&resolution),
        "Bill failed to pass Congress"
    );

    // 6. Judicial Pre-review
    assert_eq!(
        supreme_compiler.review_resolution(&resolution),
        Verdict::Permitted
    );

    // 7. Executive Action
    if let Some(action) = orchestrator.sign_resolution(&resolution) {
        let cost = Treasury {
            compute_quota: 10,
            memory_quota: 10,
        };
        let execution_verdict = orchestrator
            .execute_action(&action, &cost)
            .expect("Execution failed");

        // 8. Judicial Final Review
        assert_eq!(supreme_compiler.review_action(&action), execution_verdict);
    } else {
        panic!("President vetoed a high-confidence bill");
    }

    assert_eq!(orchestrator.current_cycle, 1);
}
