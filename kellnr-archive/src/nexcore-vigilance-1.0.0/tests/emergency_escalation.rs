use nexcore_vigilance::primitives::governance::*;
use nexcore_vigilance::primitives::measurement::{Confidence, Measured};

#[test]
fn test_emergency_escalation_simulation() {
    // 1. Setup Branches
    let congress = Congress {
        house: HouseOfT1 {
            members: vec![T1Representative {
                id: "Rep1".into(),
                weight: VoteWeight::new(100),
            }],
            quorum_threshold: 0.5,
        },
        senate: SenateOfT2 {
            members: vec![T2Senator {
                id: "Sen1".into(),
                domain: "PV".into(),
                weight: VoteWeight::new(100),
            }],
            quorum_threshold: 0.5,
        },
    };

    let risk_minimizer = RiskMinimizer {
        level: RiskMinimizationLevel::Monitoring,
        active_guardrails: vec![Guardrail::ConfidenceThreshold(0.8)],
    };

    let orchestrator = Orchestrator {
        id: "The_Orchestrator".into(),
        treasury: Treasury {
            compute_quota: 10000,
            memory_quota: 10000,
        },
        agents: vec![Agent {
            id: "Sentry".into(),
            department: "Safety".into(),
            capability: 1.0,
        }],
        current_cycle: 0,
        risk_minimizer,
        power: ExecutivePower {
            energy: Energy(1.0),
            secrecy_level: 0,
            dispatch_rate: Dispatch(1.0),
        },
    };

    let supreme_compiler = SupremeCompiler {
        constitution: vec![],
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

    // 2. Scenario: A Critical Failure is detected (Severity 4)
    let escalation = pipeline.congress.classify_escalation(4);
    assert_eq!(escalation, EscalationLevel::Emergency);

    // 3. Emergency Action: Congress passes an emergency Suspension Bill
    let emergency_bill = Measured::uncertain(Rule, Confidence::new(1.0)); // Absolute certainty of danger

    let verdict = pipeline
        .execute_cycle(emergency_bill)
        .expect("Emergency cycle failed");
    assert_eq!(verdict, Verdict::Permitted);

    // 4. Executive Response: Orchestrator increases Risk Minimization Level
    pipeline.orchestrator.risk_minimizer.level = RiskMinimizationLevel::Suspension;
    pipeline
        .orchestrator
        .risk_minimizer
        .active_guardrails
        .push(Guardrail::HumanReviewRequired);

    assert_eq!(
        pipeline.orchestrator.risk_minimizer.level,
        RiskMinimizationLevel::Suspension
    );
    assert!(pipeline.orchestrator.risk_minimizer.active_guardrails.len() >= 2);
}
