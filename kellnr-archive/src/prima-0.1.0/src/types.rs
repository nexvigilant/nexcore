// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Type System
//!
//! Types grounded in primitive compositions.
//!
//! ## Mathematical Foundation
//!
//! Every type has a `PrimitiveComposition` tracking its grounding.
//! The tier system classifies types by composition complexity.
//!
//! ## Type Hierarchy
//!
//! | Tier | Composition Size | Examples |
//! |------|------------------|----------|
//! | T1 | 1 primitive | N, σ, μ |
//! | T2-P | 2-3 primitives | σ[N], μ[N→N] |
//! | T2-C | 4-5 primitives | Complex data |
//! | T3 | 6+ primitives | Domain types |

use crate::ast::TypeExpr;
use lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition, Tier};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A Prima type with its composition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrimaType {
    /// Type name.
    pub name: String,
    /// Primitive composition.
    pub composition: PrimitiveComposition,
    /// Generic parameters.
    pub params: Vec<PrimaType>,
}

impl PrimaType {
    /// Create a new type.
    #[must_use]
    pub fn new(name: impl Into<String>, composition: PrimitiveComposition) -> Self {
        Self {
            name: name.into(),
            composition,
            params: Vec::new(),
        }
    }

    /// Create with parameters.
    #[must_use]
    pub fn with_params(mut self, params: Vec<PrimaType>) -> Self {
        // Merge parameter compositions
        for p in &params {
            for prim in &p.composition.primitives {
                if !self.composition.primitives.contains(prim) {
                    self.composition.primitives.push(*prim);
                }
            }
        }
        self.params = params;
        self
    }

    /// Get the tier of this type.
    #[must_use]
    pub fn tier(&self) -> Tier {
        Tier::classify(&self.composition)
    }

    /// Check if type is pure (single primitive).
    #[must_use]
    pub fn is_pure(&self) -> bool {
        self.composition.is_pure()
    }

    /// Get transfer confidence to another domain.
    #[must_use]
    pub fn transfer_confidence(&self) -> f64 {
        self.tier().transfer_multiplier() * self.composition.confidence
    }
}

impl std::fmt::Display for PrimaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.composition)
    }
}

/// Type environment.
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    types: HashMap<String, PrimaType>,
}

impl TypeEnv {
    /// Create with built-in types.
    #[must_use]
    pub fn with_builtins() -> Self {
        let mut env = Self::default();
        env.register_builtins();
        env
    }

    fn register_builtins(&mut self) {
        // T1 primitive types
        self.insert(PrimaType::new(
            "N",
            PrimitiveComposition::new(vec![LexPrimitiva::Quantity]),
        ));
        self.insert(PrimaType::new(
            "∅",
            PrimitiveComposition::new(vec![LexPrimitiva::Void]),
        ));
        self.insert(PrimaType::new(
            "Σ",
            PrimitiveComposition::new(vec![LexPrimitiva::Sum]),
        ));
        self.insert(PrimaType::new(
            "σ",
            PrimitiveComposition::new(vec![LexPrimitiva::Sequence]),
        ));
        self.insert(PrimaType::new(
            "μ",
            PrimitiveComposition::new(vec![LexPrimitiva::Mapping]),
        ));
        self.insert(PrimaType::new(
            "→",
            PrimitiveComposition::new(vec![LexPrimitiva::Causality]),
        ));

        // Aliases
        self.insert(PrimaType::new(
            "Int",
            PrimitiveComposition::new(vec![LexPrimitiva::Quantity]),
        ));
        self.insert(PrimaType::new(
            "Float",
            PrimitiveComposition::new(vec![LexPrimitiva::Quantity]),
        ));
        self.insert(PrimaType::new(
            "Bool",
            PrimitiveComposition::new(vec![LexPrimitiva::Sum]),
        ));
        self.insert(PrimaType::new(
            "String",
            PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Quantity]),
        ));
        self.insert(PrimaType::new(
            "Void",
            PrimitiveComposition::new(vec![LexPrimitiva::Void]),
        ));
    }

    /// Insert a type.
    pub fn insert(&mut self, ty: PrimaType) {
        self.types.insert(ty.name.clone(), ty);
    }

    /// Get a type by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&PrimaType> {
        self.types.get(name)
    }

    /// Resolve a type expression to a PrimaType.
    #[must_use]
    pub fn resolve(&self, expr: &TypeExpr) -> Option<PrimaType> {
        use crate::ast::TypeKind;
        match &expr.kind {
            TypeKind::Primitive(p) => Some(PrimaType::new(
                p.symbol(),
                PrimitiveComposition::new(vec![*p]),
            )),
            TypeKind::Named(name) => self.get(name).cloned(),
            TypeKind::Sequence(inner) => {
                let inner_ty = self.resolve(inner)?;
                Some(
                    PrimaType::new("σ", PrimitiveComposition::new(vec![LexPrimitiva::Sequence]))
                        .with_params(vec![inner_ty]),
                )
            }
            TypeKind::Mapping(from, to) => {
                let from_ty = self.resolve(from)?;
                let to_ty = self.resolve(to)?;
                Some(
                    PrimaType::new("μ", PrimitiveComposition::new(vec![LexPrimitiva::Mapping]))
                        .with_params(vec![from_ty, to_ty]),
                )
            }
            TypeKind::Sum(variants) => {
                let variant_tys: Vec<_> = variants.iter().filter_map(|v| self.resolve(v)).collect();
                Some(
                    PrimaType::new("Σ", PrimitiveComposition::new(vec![LexPrimitiva::Sum]))
                        .with_params(variant_tys),
                )
            }
            TypeKind::Function(params, ret) => {
                let param_tys: Vec<_> = params.iter().filter_map(|p| self.resolve(p)).collect();
                let ret_ty = self.resolve(ret)?;
                let mut all = param_tys;
                all.push(ret_ty);
                Some(
                    PrimaType::new(
                        "→",
                        PrimitiveComposition::new(vec![LexPrimitiva::Causality]),
                    )
                    .with_params(all),
                )
            }
            TypeKind::Optional(inner) => {
                let inner_ty = self.resolve(inner)?;
                Some(
                    PrimaType::new(
                        "?",
                        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Void]),
                    )
                    .with_params(vec![inner_ty]),
                )
            }
            TypeKind::Void => Some(PrimaType::new(
                "∅",
                PrimitiveComposition::new(vec![LexPrimitiva::Void]),
            )),
            TypeKind::Infer => Some(PrimaType::new(
                "?",
                PrimitiveComposition::new(vec![LexPrimitiva::Existence]),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_types() {
        let env = TypeEnv::with_builtins();
        assert!(env.get("N").is_some());
        assert!(env.get("∅").is_some());
        assert!(env.get("String").is_some());
    }

    #[test]
    fn test_type_tier() {
        let n = PrimaType::new("N", PrimitiveComposition::new(vec![LexPrimitiva::Quantity]));
        assert_eq!(n.tier(), Tier::T1Universal);
        assert!(n.is_pure());
    }

    #[test]
    fn test_type_with_params() {
        let n = PrimaType::new("N", PrimitiveComposition::new(vec![LexPrimitiva::Quantity]));
        let seq = PrimaType::new("σ", PrimitiveComposition::new(vec![LexPrimitiva::Sequence]))
            .with_params(vec![n]);

        assert_eq!(seq.tier(), Tier::T2Primitive);
        assert!(!seq.is_pure());
    }

    #[test]
    fn test_transfer_confidence() {
        let t1 = PrimaType::new("N", PrimitiveComposition::new(vec![LexPrimitiva::Quantity]));
        assert!((t1.transfer_confidence() - 1.0).abs() < f64::EPSILON);
    }
}
