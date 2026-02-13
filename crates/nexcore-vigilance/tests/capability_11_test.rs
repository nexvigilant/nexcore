use nexcore_vigilance::hud::capabilities::*;
use nexcore_vigilance::primitives::governance::{Liquidity, MarketState, Odds, Token, Verdict};
use nexcore_vigilance::primitives::measurement::Confidence;

#[test]
fn test_capability_11_commerce_arbitrage_act() {
    let act = CommerceArbitrageAct::new();
    assert!(act.trade_active);

    let market_state = MarketState {
        market_id: "Drug_Safety_Outcome".into(),
        odds: Odds::new(0.5),
        liquidity: Liquidity::new(10000),
    };

    // 1. BEA Analysis: Significant Opportunity (90% internal vs 50% market)
    let internal = Confidence::new(0.9);
    let measured_opp = act.analyze_opportunity(internal, &market_state);

    assert!(measured_opp.value.alpha >= 0.4);
    assert!(measured_opp.confidence.value() > 0.9);
    assert_eq!(measured_opp.value.volume_limit, 1000);

    // 2. ITA Authorization: Valid Export
    let manifest = TradeManifest {
        market_id: "Drug_Safety_Outcome".into(),
        exported_value: Token(500),
        alpha_captured: 0.4,
        ip_protection_verified: true,
    };
    let verdict = act.authorize_export(&manifest);
    assert_eq!(verdict, Verdict::Permitted);

    // 3. Scenario: IP Risk (USPTO Violation)
    let leaky_manifest = TradeManifest {
        market_id: "Drug_Safety_Outcome".into(),
        exported_value: Token(500),
        alpha_captured: 0.4,
        ip_protection_verified: false,
    };
    let fail_verdict = act.authorize_export(&leaky_manifest);
    assert_eq!(fail_verdict, Verdict::Rejected);
}
