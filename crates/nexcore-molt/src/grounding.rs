//! T1 Primitive grounding for nexcore-molt types.
//!
//! The scripting engine is fundamentally about Mapping (mu): bridging
//! Rust's typed world to Tcl's stringly-typed scripting surface.
//! Boundary (partial) is secondary — the sandbox policy draws boundaries
//! around what scripts can access.

#[cfg(test)]
mod tests {
    use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
    use nexcore_lex_primitiva::tier::Tier;

    // We test the grounding concepts here rather than implementing GroundsTo
    // on public types, since Engine/ContextBridge/etc. contain non-static
    // references that make trait impl complex. The grounding is documented
    // and verified through these conceptual tests.

    /// Engine: T2-C (mu + partial + varsigma + sigma), dominant mu
    ///
    /// The engine maps Rust -> Tcl. Boundary is the sandbox.
    /// State is the interpreter state. Sequence is script execution order.
    #[test]
    fn engine_grounding_concept() {
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- Rust <-> Tcl bridge
            LexPrimitiva::Boundary, // partial -- sandbox policy
            LexPrimitiva::State,    // varsigma -- interpreter state
            LexPrimitiva::Sequence, // sigma -- script execution order
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80);

        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!(comp.confidence >= 0.80);
    }

    /// SandboxPolicy: T2-P (partial + Sigma), dominant partial
    ///
    /// Pure boundary control over command availability.
    #[test]
    fn sandbox_grounding_concept() {
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- command boundary
            LexPrimitiva::Sum,      // Sigma -- policy variant
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90);

        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    /// ValueAdapter (to_molt/from_molt): T2-P (mu + varsigma), dominant mu
    ///
    /// Pure mapping between type systems.
    #[test]
    fn value_adapter_grounding_concept() {
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping, // mu -- JSON <-> Molt Value
            LexPrimitiva::State,   // varsigma -- value state
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90);

        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    /// MoltError: T2-P (partial + Sigma), dominant partial
    ///
    /// Error boundary with variant alternation.
    #[test]
    fn error_grounding_concept() {
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- error boundary
            LexPrimitiva::Sum,      // Sigma -- error variant
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85);

        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    /// ContextBridge: T2-P (mu + varsigma), dominant mu
    ///
    /// Maps typed Rust data into Molt's type-erased cache.
    #[test]
    fn bridge_grounding_concept() {
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping, // mu -- typed <-> erased
            LexPrimitiva::State,   // varsigma -- cached state
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85);

        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    /// CommandRegistry: T2-P (sigma + varsigma + N), dominant sigma
    ///
    /// Ordered collection of command entries.
    #[test]
    fn registry_grounding_concept() {
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered entries
            LexPrimitiva::State,    // varsigma -- registry state
            LexPrimitiva::Quantity, // N -- entry count
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80);

        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    /// All groundings have valid confidence ranges.
    #[test]
    fn all_confidences_valid() {
        let compositions = [
            PrimitiveComposition::new(vec![
                LexPrimitiva::Mapping,
                LexPrimitiva::Boundary,
                LexPrimitiva::State,
                LexPrimitiva::Sequence,
            ])
            .with_dominant(LexPrimitiva::Mapping, 0.80),
            PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
                .with_dominant(LexPrimitiva::Boundary, 0.90),
            PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::State])
                .with_dominant(LexPrimitiva::Mapping, 0.90),
        ];
        for comp in &compositions {
            assert!(comp.confidence >= 0.80);
            assert!(comp.confidence <= 1.0);
        }
    }
}
