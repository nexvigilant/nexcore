#[cfg(test)]
mod tests {
    use crate::load::*;

    #[test]
    fn test_load_prioritization() {
        let mut loads = [
            Load {
                id: 1,
                tier: LoadTier::Comms,
                nominal_w: 100.0,
                min_w: 50.0,
            },
            Load {
                id: 0,
                tier: LoadTier::Safety,
                nominal_w: 50.0,
                min_w: 10.0,
            },
        ];

        // Scenario 1: Only Safety can be powered
        let active = prioritize(20.0, &mut loads);
        assert!(active[0]);
        assert!(!active[1]);

        // Scenario 2: Safety and Comms can be powered
        let active = prioritize(150.0, &mut loads);
        assert!(active[0]);
        assert!(active[1]);
    }
}
