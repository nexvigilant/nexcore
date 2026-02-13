use nexcore_vigilance::primitives::governance::*;

#[test]
fn test_cabinet_initialization_1to1() {
    let union = Union::new("NexVigilant_Union");

    // Verify 1:1 matching with US Executive Departments
    assert_eq!(union.cabinet.state.head.agency, "State");
    assert_eq!(union.cabinet.treasury.head.agency, "Treasury");
    assert_eq!(union.cabinet.defense.head.agency, "Defense");
    assert_eq!(union.cabinet.justice.head.agency, "Justice");
    assert_eq!(union.cabinet.agriculture.head.agency, "Agriculture");
    assert_eq!(union.cabinet.commerce.head.agency, "Commerce");
    assert_eq!(union.cabinet.labor.head.agency, "Labor");
    assert_eq!(union.cabinet.health_and_human_services.head.agency, "HHS");
    assert_eq!(union.cabinet.energy.head.agency, "Energy");
    assert_eq!(
        union.cabinet.homeland_security.head.agency,
        "HomelandSecurity"
    );

    // Verify protection of nexcore (Structural grounding)
    assert!(union.cabinet.housing_and_urban_development.head.agency == "HUD");
}
