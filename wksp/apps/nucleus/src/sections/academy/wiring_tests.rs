#![cfg(test)]

#[test]
fn academy_routes_include_framework_and_gvp_surfaces() {
    let app_rs = include_str!("../../app.rs");
    for route in [
        r#"StaticSegment("pv-framework")"#,
        r#"StaticSegment("gvp-modules")"#,
        r#"StaticSegment("gvp-curriculum")"#,
        r#"StaticSegment("gvp-progress")"#,
        r#"StaticSegment("gvp-assessments")"#,
        r#"StaticSegment("gvp-practicum")"#,
        r#"StaticSegment("evidence-ledger")"#,
    ] {
        assert!(app_rs.contains(route), "missing route marker: {route}");
    }
}

#[test]
fn academy_pages_link_to_guardian_and_framework() {
    let files = [
        include_str!("courses.rs"),
        include_str!("dashboard.rs"),
        include_str!("gvp_modules.rs"),
        include_str!("gvp_curriculum.rs"),
        include_str!("gvp_progress.rs"),
        include_str!("gvp_assessments.rs"),
        include_str!("gvp_practicum.rs"),
        include_str!("pv_framework.rs"),
    ];
    for content in files {
        assert!(
            content.contains("/vigilance/guardian"),
            "page missing guardian integration link"
        );
    }
    assert!(
        include_str!("skills.rs").contains("/academy/pv-framework"),
        "skills page missing framework link"
    );
}
