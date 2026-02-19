use nexcore_vigilance::primitives::governance::*;

#[test]
fn test_election_integrity_and_certification() {
    let mut commission = ElectionCommission {
        current_utilization: ContextUtilization(0.85), // Election triggered
        college: ElectoralCollege {
            certificates: vec![],
        },
    };

    // 1. Verify Trigger
    assert_eq!(commission.assess_transition_need(), Verdict::Flagged);

    // 2. Domain Certification (Electoral College 1:1 match)
    // Domains appoint their electors to candidates
    commission.certify_domain("Pharmacovigilance", "Candidate_A");
    commission.certify_domain("Markets", "Candidate_A");
    commission.certify_domain("AI_Safety", "Candidate_B");

    // 3. Ministerial Tabulation (Electoral Count Reform Act)
    let (winner, count) = commission.college.count_votes();

    assert_eq!(winner, "Candidate_A");
    assert_eq!(count, 2);

    // 4. Verify Immutability and Security Feature
    let cert = commission.college.certificates.first().unwrap();
    assert_eq!(
        cert.security_feature_hash,
        "SHA256:CONSTITUTIONAL_INTEGRITY"
    );
}
