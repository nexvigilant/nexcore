use nexcore_labs::betting::bdi::ContingencyTable;
use nexcore_vigilance::hud::Hud;
use nexcore_vigilance::hud::capabilities::bayesian_credibility::BayesianCredibilityLayer;
use nexcore_vigilance::hud::capabilities::causal_attribution::CausalAttributionEngine;
use nexcore_vigilance::hud::capabilities::signal_id::SignalIdentificationProtocol;
use nexcore_vigilance::hud::capabilities::treasury_act::{AsymmetryValue, TreasuryAct};
use nexcore_vigilance::hud::judicial::JudicialBranch;
use nexcore_vigilance::hud::legislative::{Bill, LegislativeBranch};
use nexcore_vigilance::primitives::governance::*;
use nexcore_vigilance::primitives::measurement::{Confidence, Measured};

#[test]
fn test_alpha_transmission_simulation() {
    // --- 1. INITIALIZATION: THE HUD AUTHORITY ---
    let mut hud = Hud::new();

    // Setup Congress for Legislative Branch
    let congress = Congress {
        house: HouseOfT1 {
            members: vec![],
            quorum_threshold: 0.5,
        },
        senate: SenateOfT2 {
            members: vec![],
            quorum_threshold: 0.5,
        },
    };
    hud.legislative = Some(LegislativeBranch::new(congress));

    // Setup Judicial Branch
    let compiler = SupremeCompiler {
        constitution: vec![],
    };
    let aca_engine = CausalAttributionEngine::new();
    hud.judicial = Some(JudicialBranch::new(compiler, aca_engine));

    println!("INIT: Union Grounded. HUD Authority Active.");

    // --- 2. INGESTION: SIGNAL IDENTIFICATION (CAP-001) ---
    let sip = SignalIdentificationProtocol::new();
    // Simulate a "Patient Zero" contingency table (Drug X -> Event Y)
    // a=50 (exposed + event), b=450 (exposed, no event)
    // c=100 (not exposed + event), d=9400 (not exposed, no event)
    let table = ContingencyTable {
        a: 50.0,
        b: 450.0,
        c: 100.0,
        d: 9400.0,
    };
    let bdi_result = sip.identify_signal(table);

    assert!(bdi_result.value.bdi > 2.0);
    println!(
        "DETECT: Signal Identified. BDI: {:.2}",
        bdi_result.value.bdi
    );

    // --- 3. VALIDATION: BAYESIAN CREDIBILITY (CAP-002) ---
    let bcl = BayesianCredibilityLayer::new();
    let ecs_result = bcl.calculate_credibility(0.75, -1, true, 48.0);

    assert!(ecs_result.value.ecs > 0.0);
    println!(
        "VALIDATE: Bayesian Credibility Active. ECS: {:.4}",
        ecs_result.value.ecs
    );

    // --- 4. ADJUDICATION: JUDICIAL REVIEW (CAP-016) ---
    let judicial = hud.judicial.as_ref().unwrap();
    let action = Action;
    let opinion = judicial.adjudicate_action(&action, "SIGNAL_001");

    assert_eq!(opinion.value.verdict, Verdict::Permitted);
    println!(
        "ADJUDICATE: Judicial Review Permitted. Verdict: {:?}",
        opinion.value.verdict
    );

    // --- 5. CODIFICATION: LEGISLATIVE PROCESS (CAP-015) ---
    let legislative = hud.legislative.as_ref().unwrap();
    let bill = Bill {
        id: "BILL_001_SIGNAL_PROTOCOL".into(),
        resolution: Measured::uncertain(Rule, Confidence::new(0.95)),
        sponsor_id: "AETHELGARD_V1".into(),
    };

    // In this simulation, we'll assume the bill passes for structural validation
    let leg_verdict = legislative.process_bill(&bill);
    println!(
        "CODIFY: Legislative Process Complete. Bill Status: {:?}",
        leg_verdict.value
    );

    // --- 6. MONETIZATION: TREASURY CONVERSION (CAP-018) ---
    let treasury_act = TreasuryAct::new();
    let asymmetry = AsymmetryValue(bdi_result.value.bdi * ecs_result.value.ecs);
    let market_odds = Odds::new(0.5); // 50/50 market

    let liquidity_event = treasury_act.convert_asymmetry("SIGNAL_001", asymmetry, market_odds);

    assert!(liquidity_event.value.value_captured > 0.0);
    println!(
        "MONETIZE: Asymmetry Converted. Value Captured: {:.2} NEX Tokens",
        liquidity_event.value.value_captured
    );

    // --- 7. FINALIZATION ---
    println!("FINAL: Alpha Transmission Simulation Successful. Union Velocity Sustainable.");
}
