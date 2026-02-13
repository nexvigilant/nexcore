// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Effect System
//!
//! Static effect tracking for verifiable computation.
//!
//! ## Philosophy
//!
//! "Code that compiles is mathematically true."
//!
//! Effects make hidden side-effects explicit. If a function is Pure,
//! it is guaranteed referentially transparent. If it has IO, the type
//! system tells you it interacts with the world.
//!
//! ## Tier: T2-P (ς + π + → + ∅)
//!
//! ## Effect Categories
//!
//! | Effect | Symbol | Grounding | Meaning |
//! |--------|--------|-----------|---------|
//! | Pure   | `∅`    | Void      | No effects, referentially transparent |
//! | IO     | `π`    | Persistence | External world interaction |
//! | State  | `ς`    | State     | Mutable state modification |
//! | Diverge| `ρ`    | Recursion | May not terminate |
//!
//! ## Composition Rules
//!
//! Effects compose via union: `f(IO) ; g(State) → IO ∪ State`
//!
//! A function's effect is the join of all expressions within it.

use lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{BitAnd, BitOr};

/// Individual effect kinds.
///
/// Each maps to a T1 primitive from Lex Primitiva.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Effect {
    /// Pure computation — grounded to ∅ (Void).
    /// No observable effects, referentially transparent.
    Pure,

    /// IO effect — grounded to π (Persistence).
    /// Reads from or writes to external world (console, files, network).
    IO,

    /// State effect — grounded to ς (State).
    /// Modifies mutable state.
    State,

    /// Divergence effect — grounded to ρ (Recursion).
    /// May not terminate (unbounded recursion, infinite loops).
    Diverge,
}

impl Effect {
    /// Get the primitive this effect grounds to.
    #[must_use]
    pub const fn primitive(self) -> LexPrimitiva {
        match self {
            Self::Pure => LexPrimitiva::Void,
            Self::IO => LexPrimitiva::Persistence,
            Self::State => LexPrimitiva::State,
            Self::Diverge => LexPrimitiva::Recursion,
        }
    }

    /// Get the symbolic representation.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Pure => "∅",
            Self::IO => "π",
            Self::State => "ς",
            Self::Diverge => "ρ",
        }
    }

    /// Parse from symbol.
    #[must_use]
    pub fn from_symbol(s: &str) -> Option<Self> {
        match s {
            "∅" | "pure" | "Pure" => Some(Self::Pure),
            "π" | "io" | "IO" => Some(Self::IO),
            "ς" | "state" | "State" => Some(Self::State),
            "ρ" | "diverge" | "Diverge" => Some(Self::Diverge),
            _ => None,
        }
    }

    /// Human-readable name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pure => "Pure",
            Self::IO => "IO",
            Self::State => "State",
            Self::Diverge => "Diverge",
        }
    }
}

impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EFFECT SET — Σ (Sum) of effect possibilities
// ═══════════════════════════════════════════════════════════════════════════

/// A set of effects, represented as bitflags for efficiency.
///
/// ## Composition
///
/// Effect sets compose via union (join):
/// - `Pure ∪ X = X` (pure is identity)
/// - `IO ∪ State = {IO, State}`
///
/// ## Tier: T2-P (Σ + σ)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct EffectSet(u8);

impl EffectSet {
    const PURE_BIT: u8 = 0b0000;
    const IO_BIT: u8 = 0b0001;
    const STATE_BIT: u8 = 0b0010;
    const DIVERGE_BIT: u8 = 0b0100;

    /// Empty effect set (pure).
    pub const PURE: Self = Self(Self::PURE_BIT);

    /// IO effect only.
    pub const IO: Self = Self(Self::IO_BIT);

    /// State effect only.
    pub const STATE: Self = Self(Self::STATE_BIT);

    /// Diverge effect only.
    pub const DIVERGE: Self = Self(Self::DIVERGE_BIT);

    /// Create from a single effect.
    #[must_use]
    pub const fn singleton(effect: Effect) -> Self {
        match effect {
            Effect::Pure => Self::PURE,
            Effect::IO => Self::IO,
            Effect::State => Self::STATE,
            Effect::Diverge => Self::DIVERGE,
        }
    }

    /// Create an empty (pure) effect set.
    #[must_use]
    pub const fn empty() -> Self {
        Self::PURE
    }

    /// Check if the set is pure (no effects).
    #[must_use]
    pub const fn is_pure(self) -> bool {
        self.0 == Self::PURE_BIT
    }

    /// Check if the set contains IO.
    #[must_use]
    pub const fn has_io(self) -> bool {
        (self.0 & Self::IO_BIT) != 0
    }

    /// Check if the set contains State.
    #[must_use]
    pub const fn has_state(self) -> bool {
        (self.0 & Self::STATE_BIT) != 0
    }

    /// Check if the set may diverge.
    #[must_use]
    pub const fn may_diverge(self) -> bool {
        (self.0 & Self::DIVERGE_BIT) != 0
    }

    /// Check if set contains a specific effect.
    #[must_use]
    pub const fn contains(self, effect: Effect) -> bool {
        match effect {
            Effect::Pure => self.is_pure(),
            Effect::IO => self.has_io(),
            Effect::State => self.has_state(),
            Effect::Diverge => self.may_diverge(),
        }
    }

    /// Union two effect sets (join operation).
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Intersection of two effect sets.
    #[must_use]
    pub const fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Check if self is a subset of other.
    ///
    /// Used for effect checking: can we call `f` from context with `allowed` effects?
    #[must_use]
    pub const fn is_subset_of(self, allowed: Self) -> bool {
        (self.0 & !allowed.0) == 0
    }

    /// Iterate over contained effects.
    pub fn iter(self) -> impl Iterator<Item = Effect> {
        let mut effects = Vec::with_capacity(4);
        if self.is_pure() {
            effects.push(Effect::Pure);
        } else {
            if self.has_io() {
                effects.push(Effect::IO);
            }
            if self.has_state() {
                effects.push(Effect::State);
            }
            if self.may_diverge() {
                effects.push(Effect::Diverge);
            }
        }
        effects.into_iter()
    }

    /// Get primitive composition for this effect set.
    #[must_use]
    pub fn composition(self) -> PrimitiveComposition {
        let primitives: Vec<_> = self.iter().map(Effect::primitive).collect();
        PrimitiveComposition::new(primitives)
    }

    /// Count non-pure effects.
    #[must_use]
    pub const fn count(self) -> usize {
        self.0.count_ones() as usize
    }
}

impl fmt::Debug for EffectSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_pure() {
            write!(f, "EffectSet(∅)")
        } else {
            let effects: Vec<_> = self.iter().map(|e| e.symbol()).collect();
            write!(f, "EffectSet({})", effects.join(" ∪ "))
        }
    }
}

impl fmt::Display for EffectSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_pure() {
            write!(f, "∅")
        } else {
            let effects: Vec<_> = self.iter().map(|e| e.symbol()).collect();
            write!(f, "{}", effects.join(" ∪ "))
        }
    }
}

impl BitOr for EffectSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl BitAnd for EffectSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersect(rhs)
    }
}

impl From<Effect> for EffectSet {
    fn from(effect: Effect) -> Self {
        Self::singleton(effect)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EFFECT SIGNATURE — μ[params → return ! effects]
// ═══════════════════════════════════════════════════════════════════════════

/// Effect signature for a function.
///
/// Syntax: `fn name(params) → RetType ! Effects`
///
/// ## Examples
///
/// - `fn pure(x: N) → N ! ∅` — Pure function
/// - `fn print(x: A) → ∅ ! π` — IO effect
/// - `fn mutate(x: &mut A) → ∅ ! ς` — State effect
///
/// ## Tier: T2-C (μ + → + Σ)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectSig {
    /// Effects this function may perform.
    pub effects: EffectSet,
    /// Whether effects were explicitly declared (vs inferred).
    pub explicit: bool,
}

impl EffectSig {
    /// Create a pure signature.
    #[must_use]
    pub const fn pure() -> Self {
        Self {
            effects: EffectSet::PURE,
            explicit: true,
        }
    }

    /// Create an IO signature.
    #[must_use]
    pub const fn io() -> Self {
        Self {
            effects: EffectSet::IO,
            explicit: true,
        }
    }

    /// Create a state signature.
    #[must_use]
    pub const fn state() -> Self {
        Self {
            effects: EffectSet::STATE,
            explicit: true,
        }
    }

    /// Create from an effect set.
    #[must_use]
    pub const fn from_effects(effects: EffectSet, explicit: bool) -> Self {
        Self { effects, explicit }
    }

    /// Check if this signature is compatible with a required signature.
    ///
    /// `self` can substitute for `required` if `self.effects ⊆ required.effects`.
    #[must_use]
    pub const fn compatible_with(&self, required: &Self) -> bool {
        self.effects.is_subset_of(required.effects)
    }
}

impl Default for EffectSig {
    fn default() -> Self {
        Self::pure()
    }
}

impl fmt::Display for EffectSig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "! {}", self.effects)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// BUILTIN EFFECT MAPPING — κ (comparison) of known effects
// ═══════════════════════════════════════════════════════════════════════════

/// Get the effect signature for a builtin function.
///
/// ## IO Functions (π)
/// - `print`, `ω`, `println`, `ωn`
///
/// ## Pure Functions (∅)
/// - All math, sequence, string operations that don't mutate
///
/// ## State Functions (ς)
/// - None currently (Prima is immutable by default)
#[must_use]
pub fn builtin_effect(name: &str) -> EffectSig {
    match name {
        // IO effects — grounded to π (Persistence)
        "print" | "ω" | "println" | "ωn" => EffectSig::io(),

        // All other builtins are pure — grounded to ∅ (Void)
        _ => EffectSig::pure(),
    }
}

/// Check if a builtin is known to be pure.
#[must_use]
pub fn is_pure_builtin(name: &str) -> bool {
    builtin_effect(name).effects.is_pure()
}

/// Check if a builtin has IO effects.
#[must_use]
pub fn has_io_builtin(name: &str) -> bool {
    builtin_effect(name).effects.has_io()
}

// ═══════════════════════════════════════════════════════════════════════════
// EFFECT CONTEXT — ς (State) for effect inference
// ═══════════════════════════════════════════════════════════════════════════

use std::collections::HashMap;

/// Context for effect inference and checking.
///
/// Tracks:
/// - Known function effects
/// - Current allowed effects (for checking)
/// - Accumulated effects in current scope
///
/// ## Tier: T2-C (ς + μ + σ)
#[derive(Debug, Clone, Default)]
pub struct EffectContext {
    /// Known function effect signatures.
    functions: HashMap<String, EffectSig>,
    /// Effects allowed in current context.
    allowed: EffectSet,
    /// Effects accumulated in current scope.
    accumulated: EffectSet,
}

impl EffectContext {
    /// Create a new effect context allowing all effects.
    #[must_use]
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            allowed: EffectSet(0xFF), // Allow all
            accumulated: EffectSet::empty(),
        }
    }

    /// Create a pure context (only pure operations allowed).
    #[must_use]
    pub fn pure() -> Self {
        Self {
            functions: HashMap::new(),
            allowed: EffectSet::PURE,
            accumulated: EffectSet::empty(),
        }
    }

    /// Register a function's effect signature.
    pub fn register(&mut self, name: impl Into<String>, sig: EffectSig) {
        self.functions.insert(name.into(), sig);
    }

    /// Look up a function's effect signature.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&EffectSig> {
        self.functions.get(name)
    }

    /// Get the effect signature for a name (function or builtin).
    #[must_use]
    pub fn effect_of(&self, name: &str) -> EffectSig {
        self.functions
            .get(name)
            .cloned()
            .unwrap_or_else(|| builtin_effect(name))
    }

    /// Record an effect occurrence.
    pub fn record(&mut self, effect: Effect) {
        self.accumulated = self.accumulated.union(effect.into());
    }

    /// Record multiple effects.
    pub fn record_set(&mut self, effects: EffectSet) {
        self.accumulated = self.accumulated.union(effects);
    }

    /// Get accumulated effects.
    #[must_use]
    pub const fn accumulated(&self) -> EffectSet {
        self.accumulated
    }

    /// Get allowed effects.
    #[must_use]
    pub const fn allowed(&self) -> EffectSet {
        self.allowed
    }

    /// Check if an effect is allowed in current context.
    #[must_use]
    pub fn is_allowed(&self, effect: Effect) -> bool {
        EffectSet::singleton(effect).is_subset_of(self.allowed)
    }

    /// Check if accumulated effects exceed allowed.
    #[must_use]
    pub const fn has_violation(&self) -> bool {
        !self.accumulated.is_subset_of(self.allowed)
    }

    /// Set allowed effects (for entering a pure context).
    pub fn with_allowed(&mut self, allowed: EffectSet) {
        self.allowed = allowed;
    }

    /// Reset accumulated effects (for new scope).
    pub fn reset_accumulated(&mut self) {
        self.accumulated = EffectSet::empty();
    }

    /// Create a child context with same registrations but fresh accumulator.
    #[must_use]
    pub fn child(&self) -> Self {
        Self {
            functions: self.functions.clone(),
            allowed: self.allowed,
            accumulated: EffectSet::empty(),
        }
    }

    /// Merge child's accumulated effects into parent.
    pub fn merge_child(&mut self, child: &Self) {
        self.accumulated = self.accumulated.union(child.accumulated);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EFFECT ERROR — ∂ (Boundary) violations
// ═══════════════════════════════════════════════════════════════════════════

use crate::token::Span;

/// Error when an effect is used in a context that doesn't allow it.
///
/// ## Tier: T2-P (∂ + Σ)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectViolation {
    /// The effect that was used.
    pub effect: Effect,
    /// Effects allowed in context.
    pub allowed: EffectSet,
    /// Location of violation.
    pub span: Span,
    /// Descriptive message.
    pub message: String,
}

impl EffectViolation {
    /// Create a new violation.
    #[must_use]
    pub fn new(effect: Effect, allowed: EffectSet, span: Span) -> Self {
        let message = format!(
            "effect {} not allowed in context (allowed: {})",
            effect.symbol(),
            allowed
        );
        Self {
            effect,
            allowed,
            span,
            message,
        }
    }

    /// Create an IO-in-pure-context violation.
    #[must_use]
    pub fn io_in_pure(span: Span) -> Self {
        Self::new(Effect::IO, EffectSet::PURE, span)
    }

    /// Create a state-in-pure-context violation.
    #[must_use]
    pub fn state_in_pure(span: Span) -> Self {
        Self::new(Effect::State, EffectSet::PURE, span)
    }
}

impl fmt::Display for EffectViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "effect violation at {}: {} (grounded: {})",
            self.span,
            self.message,
            self.effect.primitive()
        )
    }
}

impl std::error::Error for EffectViolation {}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS — ν (Invariant) verification
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // Effect tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_effect_symbols() {
        assert_eq!(Effect::Pure.symbol(), "∅");
        assert_eq!(Effect::IO.symbol(), "π");
        assert_eq!(Effect::State.symbol(), "ς");
        assert_eq!(Effect::Diverge.symbol(), "ρ");
    }

    #[test]
    fn test_effect_primitives() {
        assert_eq!(Effect::Pure.primitive(), LexPrimitiva::Void);
        assert_eq!(Effect::IO.primitive(), LexPrimitiva::Persistence);
        assert_eq!(Effect::State.primitive(), LexPrimitiva::State);
        assert_eq!(Effect::Diverge.primitive(), LexPrimitiva::Recursion);
    }

    #[test]
    fn test_effect_from_symbol() {
        assert_eq!(Effect::from_symbol("∅"), Some(Effect::Pure));
        assert_eq!(Effect::from_symbol("pure"), Some(Effect::Pure));
        assert_eq!(Effect::from_symbol("π"), Some(Effect::IO));
        assert_eq!(Effect::from_symbol("io"), Some(Effect::IO));
        assert_eq!(Effect::from_symbol("invalid"), None);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EffectSet tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_effect_set_pure() {
        let pure = EffectSet::PURE;
        assert!(pure.is_pure());
        assert!(!pure.has_io());
        assert!(!pure.has_state());
        assert!(!pure.may_diverge());
    }

    #[test]
    fn test_effect_set_union() {
        let io = EffectSet::IO;
        let state = EffectSet::STATE;
        let combined = io | state;

        assert!(!combined.is_pure());
        assert!(combined.has_io());
        assert!(combined.has_state());
        assert!(!combined.may_diverge());
    }

    #[test]
    fn test_effect_set_subset() {
        let io = EffectSet::IO;
        let io_state = EffectSet::IO | EffectSet::STATE;
        let pure = EffectSet::PURE;

        assert!(io.is_subset_of(io_state));
        assert!(!io_state.is_subset_of(io));
        assert!(pure.is_subset_of(io));
        assert!(pure.is_subset_of(pure));
    }

    #[test]
    fn test_effect_set_display() {
        assert_eq!(format!("{}", EffectSet::PURE), "∅");
        assert_eq!(format!("{}", EffectSet::IO), "π");
        assert_eq!(format!("{}", EffectSet::IO | EffectSet::STATE), "π ∪ ς");
    }

    #[test]
    fn test_effect_set_count() {
        assert_eq!(EffectSet::PURE.count(), 0);
        assert_eq!(EffectSet::IO.count(), 1);
        assert_eq!((EffectSet::IO | EffectSet::STATE).count(), 2);
        assert_eq!(
            (EffectSet::IO | EffectSet::STATE | EffectSet::DIVERGE).count(),
            3
        );
    }

    #[test]
    fn test_effect_set_iter() {
        let effects: Vec<_> = EffectSet::PURE.iter().collect();
        assert_eq!(effects, vec![Effect::Pure]);

        let effects: Vec<_> = (EffectSet::IO | EffectSet::STATE).iter().collect();
        assert!(effects.contains(&Effect::IO));
        assert!(effects.contains(&Effect::State));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EffectSig tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_effect_sig_pure() {
        let sig = EffectSig::pure();
        assert!(sig.effects.is_pure());
        assert!(sig.explicit);
    }

    #[test]
    fn test_effect_sig_compatibility() {
        let pure = EffectSig::pure();
        let io = EffectSig::io();
        let io_state = EffectSig::from_effects(EffectSet::IO | EffectSet::STATE, true);

        // Pure is compatible with anything
        assert!(pure.compatible_with(&io));
        assert!(pure.compatible_with(&io_state));

        // IO is compatible with io_state but not pure
        assert!(io.compatible_with(&io_state));
        assert!(!io.compatible_with(&pure));

        // io_state requires both
        assert!(!io_state.compatible_with(&io));
        assert!(!io_state.compatible_with(&pure));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Builtin effect tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_builtin_effects() {
        // IO builtins
        assert!(has_io_builtin("print"));
        assert!(has_io_builtin("ω"));
        assert!(has_io_builtin("println"));
        assert!(has_io_builtin("ωn"));

        // Pure builtins
        assert!(is_pure_builtin("len"));
        assert!(is_pure_builtin("#"));
        assert!(is_pure_builtin("map"));
        assert!(is_pure_builtin("Φ"));
        assert!(is_pure_builtin("abs"));
        assert!(is_pure_builtin("±"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EffectContext tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_effect_context_accumulation() {
        let mut ctx = EffectContext::new();
        assert!(ctx.accumulated().is_pure());

        ctx.record(Effect::IO);
        assert!(ctx.accumulated().has_io());
        assert!(!ctx.accumulated().has_state());

        ctx.record(Effect::State);
        assert!(ctx.accumulated().has_io());
        assert!(ctx.accumulated().has_state());
    }

    #[test]
    fn test_effect_context_violation() {
        let mut ctx = EffectContext::pure();
        assert!(!ctx.has_violation());

        ctx.record(Effect::IO);
        assert!(ctx.has_violation());
    }

    #[test]
    fn test_effect_context_child() {
        let mut parent = EffectContext::new();
        parent.record(Effect::IO);

        let mut child = parent.child();
        assert!(child.accumulated().is_pure()); // Fresh accumulator

        child.record(Effect::State);
        parent.merge_child(&child);

        assert!(parent.accumulated().has_io());
        assert!(parent.accumulated().has_state());
    }

    #[test]
    fn test_effect_context_lookup() {
        let mut ctx = EffectContext::new();
        ctx.register("my_io_fn", EffectSig::io());

        assert!(ctx.lookup("my_io_fn").is_some());
        assert_eq!(ctx.effect_of("my_io_fn").effects, EffectSet::IO);
        assert_eq!(ctx.effect_of("unknown").effects, EffectSet::PURE);
        assert_eq!(ctx.effect_of("print").effects, EffectSet::IO);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EffectViolation tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_effect_violation_display() {
        let v = EffectViolation::io_in_pure(Span::new(0, 5, 1));
        let msg = format!("{v}");
        assert!(msg.contains("π")); // IO symbol
        assert!(msg.contains("∅")); // Pure allowed
        assert!(msg.contains("Persistence")); // Primitive name
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Composition grounding tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_effect_set_composition() {
        let io_state = EffectSet::IO | EffectSet::STATE;
        let comp = io_state.composition();

        // Should contain π and ς
        let unique = comp.unique();
        assert!(unique.contains(&LexPrimitiva::Persistence));
        assert!(unique.contains(&LexPrimitiva::State));
    }

    #[test]
    fn test_pure_composition_is_void() {
        let pure = EffectSet::PURE;
        let comp = pure.composition();

        // Pure maps to ∅
        let unique = comp.unique();
        assert!(unique.contains(&LexPrimitiva::Void));
    }
}
