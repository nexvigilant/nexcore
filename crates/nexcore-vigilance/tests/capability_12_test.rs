use nexcore_vigilance::hud::capabilities::*;
use nexcore_vigilance::primitives::governance::{Agent, Verdict};

#[test]
fn test_capability_12_agent_standards_act() {
    let act = AgentStandardsAct::new();
    assert!(act.labor_standard_active);

    let agent = Agent {
        id: "Chain".into(),
        department: "Integration".into(),
        capability: 0.87,
    };

    // 1. BLS Audit: High Capability Agent
    let measured_audit = act.audit_agent(&agent);
    assert_eq!(measured_audit.value.agent_id, "Chain");
    assert!(measured_audit.confidence.value() > 0.95);

    // 2. OSHA Safety Check: Healthy Context
    let verdict_healthy = act.verify_safety(0.44);
    assert_eq!(verdict_healthy, Verdict::Permitted);

    // 3. OSHA Safety Check: Near Election
    let verdict_near = act.verify_safety(0.18);
    assert_eq!(verdict_near, Verdict::Flagged);

    // 4. OSHA Safety Check: Exhaustion Danger
    let verdict_danger = act.verify_safety(0.05);
    assert_eq!(verdict_danger, Verdict::Rejected);
}
