#[cfg(test)]
mod cellular_tests {
    use crate::genetics::{GeneticGuard, GenomeRequirement};
    use crate::phenotypes::{SomaticSpecialization, TissuePhenotype};
    use nexcore_lex_primitiva::LexPrimitiva;

    #[test]
    fn test_germ_line_preservation() {
        let requirement = GenomeRequirement::default();
        let guard = GeneticGuard::new(requirement);

        // Healthy offspring (contains π, ∂, σ)
        let healthy_prims = vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Sequence,
        ];
        assert!(!guard.is_mutation_lethal(&healthy_prims, false));

        // Lethal mutation (missing π Persistence)
        let drifting_prims = vec![LexPrimitiva::Boundary, LexPrimitiva::Sequence];
        assert!(guard.is_mutation_lethal(&drifting_prims, false));

        // Lethal mutation (contains unsafe)
        assert!(guard.is_mutation_lethal(&healthy_prims, true));
    }

    #[test]
    fn test_somatic_specialization() {
        let immune = SomaticSpecialization::for_phenotype(TissuePhenotype::Immune);
        assert!(immune.tool_allowlist.contains(&"immunity_scan".to_string()));
        assert_eq!(immune.target_vsat, 0.85);

        let muscular = SomaticSpecialization::for_phenotype(TissuePhenotype::Muscular);
        assert!(
            muscular
                .tool_allowlist
                .contains(&"run_shell_command".to_string())
        );
    }
}
