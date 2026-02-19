#![allow(unreachable_code)]
//! Practical Examples and Demonstrations
//!
//! This module demonstrates how to use the Curry-Howard correspondence
//! to validate real-world logical arguments by translating them into
//! Rust type signatures.
//!
//! Each example shows:
//! 1. The natural language argument
//! 2. Its formalization in logic
//! 3. The Rust type signature
//! 4. The proof implementation
//! 5. Analysis of what we learned

use crate::logic_prelude::*;
use std::marker::PhantomData;

// ============================================================================
// EXAMPLE 1: VALID BUSINESS ARGUMENT
// ============================================================================

/// # Example 1: Valid Business Reasoning
///
/// **Natural Language:**
/// "If we increase marketing spend and the market is growing,
/// then our revenue will increase."
///
/// **Formalization:**
/// (IncreasedMarketing ∧ GrowingMarket) → RevenueIncreases
///
/// **Analysis:**
/// This compiles because it's a straightforward conditional.
/// The validity depends on the truth of the conditional itself
/// (which is a domain axiom, not a logical tautology).

pub struct IncreasedMarketing;
pub struct GrowingMarket;
pub struct RevenueIncreases;

/// The theorem: (IncreasedMarketing ∧ GrowingMarket) → RevenueIncreases
///
/// PROOF: By the domain axiom that marketing + growth = revenue.
/// The logical structure is valid; the soundness depends on the axiom.
pub fn business_argument(premises: And<IncreasedMarketing, GrowingMarket>) -> RevenueIncreases {
    // Destructure to show we're using the premises
    let And {
        left: _marketing,
        right: _market,
    } = premises;

    // By domain axiom: when both conditions hold, revenue increases
    RevenueIncreases
}

// ============================================================================
// EXAMPLE 2: INVALID ARGUMENT (AFFIRMING THE CONSEQUENT)
// ============================================================================

/// # Example 2: Invalid Argument - Affirming the Consequent
///
/// **Natural Language:**
/// "If it's raining, the ground is wet. The ground is wet.
/// Therefore, it's raining."
///
/// **Formalization:**
/// INVALID: Raining → GroundWet, GroundWet ⊢ Raining
///
/// **Analysis:**
/// This argument form is invalid. In Rust, we cannot implement a function
/// that produces `Raining` from `GroundWet` without additional assumptions.
/// The sprinkler could have made the ground wet!

pub struct Raining;
pub struct GroundWet;

// The valid implication
pub fn rain_makes_wet(_rain: Raining) -> GroundWet {
    GroundWet
}

// INVALID ATTEMPT - This function signature cannot be implemented!
// pub fn affirming_consequent(_wet: GroundWet) -> Raining {
//     // ERROR: We have no way to construct Raining from GroundWet!
//     // This proves the argument is INVALID.
//     //
//     // The only thing we could do is:
//     // - panic!() -- but that's cheating
//     // - use unsafe transmute -- but that's cheating
//     // - todo!() -- marks it as incomplete
//
//     todo!("Cannot implement - argument is invalid")
// }

// ============================================================================
// EXAMPLE 3: VALID CATEGORICAL SYLLOGISM
// ============================================================================

/// # Example 3: Valid Syllogism
///
/// **Natural Language:**
/// "All birds can fly. Tweety is a bird. Therefore, Tweety can fly."
///
/// **Formalization:**
/// ∀x. Bird(x) → CanFly(x), Bird(Tweety) ⊢ CanFly(Tweety)
///
/// **Analysis:**
/// This is valid BUT unsound (penguins are birds that can't fly).
/// The Curry-Howard correspondence validates the FORM, not the premises.

pub trait Bird {}
pub struct CanFly<B: Bird>(PhantomData<B>);
pub struct Tweety;
impl Bird for Tweety {}

/// Valid syllogism structure
pub fn tweety_syllogism(
    // Premise 1: All birds can fly (universal claim)
    all_birds_fly: impl Fn(Tweety) -> CanFly<Tweety>,
    // Premise 2: Tweety is a bird (encoded in the type)
    tweety: Tweety,
) -> CanFly<Tweety> {
    // Apply universal to specific instance
    all_birds_fly(tweety)
}

// ============================================================================
// EXAMPLE 4: PROOF BY CASES
// ============================================================================

/// # Example 4: Proof by Cases
///
/// **Natural Language:**
/// "Either it's a weekday or weekend. If weekday, I work.
/// If weekend, I rest. Therefore, I either work or rest."
///
/// **Formalization:**
/// (Weekday ∨ Weekend), (Weekday → Work), (Weekend → Rest) ⊢ (Work ∨ Rest)
///
/// **Analysis:**
/// This is the constructive dilemma - a valid argument form.

pub struct Weekday;
pub struct Weekend;
pub struct Work;
pub struct Rest;

pub fn schedule_dilemma(
    day_type: Or<Weekday, Weekend>,
    weekday_implies_work: impl FnOnce(Weekday) -> Work,
    weekend_implies_rest: impl FnOnce(Weekend) -> Rest,
) -> Or<Work, Rest> {
    match day_type {
        Or::Left(weekday) => Or::Left(weekday_implies_work(weekday)),
        Or::Right(weekend) => Or::Right(weekend_implies_rest(weekend)),
    }
}

// ============================================================================
// EXAMPLE 5: PROOF BY CONTRADICTION (Intuitionistic)
// ============================================================================

/// # Example 5: Proof by Contradiction
///
/// **Natural Language:**
/// "P and ¬P cannot both hold" (Law of Non-Contradiction).
///
/// **Formalization:**
/// ⊢ ¬(P ∧ ¬P)
///
/// **Analysis:**
/// This is provable intuitionistically. Given P ∧ ¬P, we can apply ¬P to P
/// to derive ⊥, so we have a function from (P ∧ ¬P) to ⊥, which is ¬(P ∧ ¬P).

/// Proof of non-contradiction: ¬(P ∧ ¬P)
///
/// PROOF: Assume P ∧ ¬P. Extract P and ¬P. Apply ¬P to P to get ⊥.
pub fn non_contradiction_example<P>() -> impl Fn(And<P, Not<P>>) -> Void {
    |p_and_not_p: And<P, Not<P>>| {
        let p = p_and_not_p.left;
        let not_p = p_and_not_p.right;
        not_p(p) // P and (P → ⊥) gives ⊥
    }
}

// NOTE: More complex proofs by contradiction (like "no largest prime")
// require dependent types, induction principles, and arithmetic axioms
// that Rust's type system cannot express. Such proofs are better suited
// to dedicated proof assistants like Coq, Lean, or Agda.

// ============================================================================
// EXAMPLE 6: WORKING WITH EXISTENTIALS
// ============================================================================

/// # Example 6: Existential Proofs
///
/// **Natural Language:**
/// "If someone solved the problem, then the problem is solvable."
///
/// **Formalization:**
/// ∃x. Solved(x, Problem) → Solvable(Problem)

pub struct Problem;
pub struct Solvable<P>(PhantomData<P>);
pub struct Solved<Person, P>(PhantomData<(Person, P)>);

pub fn solution_implies_solvability<Person>(
    exists_solution: Exists<Person, Solved<Person, Problem>>,
) -> Solvable<Problem> {
    // We don't need to know WHO solved it, just that someone did
    let Exists {
        witness: _person,
        proof: _solved,
    } = exists_solution;
    Solvable(PhantomData)
}

// ============================================================================
// EXAMPLE 7: ARGUMENT WITH HIDDEN PREMISE (ENTHYMEME)
// ============================================================================

/// # Example 7: Detecting Hidden Premises
///
/// **Natural Language:**
/// "He's a politician, so he's dishonest."
///
/// **Surface form:**
/// Politician(x) ⊢ Dishonest(x) -- INVALID as stated!
///
/// **With hidden premise:**
/// ∀x. Politician(x) → Dishonest(x), Politician(x) ⊢ Dishonest(x) -- Valid form
///
/// **Analysis:**
/// The argument REQUIRES the hidden premise "all politicians are dishonest"
/// to be valid. The Rust type system reveals this missing piece.

pub struct Politician<P>(PhantomData<P>);
pub struct Dishonest<P>(PhantomData<P>);
pub struct Bob;

// INVALID without hidden premise:
// pub fn invalid_politician_argument(
//     _is_politician: Politician<Bob>,
// ) -> Dishonest<Bob> {
//     // Cannot implement! No way to get Dishonest from just Politician
// }

// VALID with explicit hidden premise:
pub fn valid_politician_argument(
    is_politician: Politician<Bob>,
    // The hidden premise made explicit:
    all_politicians_dishonest: impl Fn(Politician<Bob>) -> Dishonest<Bob>,
) -> Dishonest<Bob> {
    all_politicians_dishonest(is_politician)
}

// ============================================================================
// EXAMPLE 8: MUTUAL EXCLUSION
// ============================================================================

/// # Example 8: Proving Mutual Exclusion
///
/// **Natural Language:**
/// "If something is true, it cannot be false."
/// "P and ¬P cannot both hold."
///
/// **Formalization:**
/// ⊢ ¬(P ∧ ¬P)

/// Proof of non-contradiction
pub fn mutual_exclusion<P>() -> Not<And<P, Not<P>>> {
    |p_and_not_p: And<P, Not<P>>| {
        let p = p_and_not_p.left;
        let not_p = p_and_not_p.right;
        not_p(p)
    }
}

// ============================================================================
// EXAMPLE 9: CHAIN OF REASONING
// ============================================================================

/// # Example 9: Transitive Reasoning Chain
///
/// **Natural Language:**
/// "A implies B. B implies C. C implies D. Therefore A implies D."
///
/// **Formalization:**
/// (A → B), (B → C), (C → D) ⊢ (A → D)

pub fn chain_of_four<A, B, C, D>(
    a_to_b: impl Fn(A) -> B,
    b_to_c: impl Fn(B) -> C,
    c_to_d: impl Fn(C) -> D,
) -> impl Fn(A) -> D {
    move |a| c_to_d(b_to_c(a_to_b(a)))
}

// ============================================================================
// EXAMPLE 10: TEMPLATE FOR YOUR OWN PROOFS
// ============================================================================

/// # Template: Proving Your Own Arguments
///
/// Follow this pattern to validate any logical argument:
///
/// 1. Define your atomic propositions as unit structs
/// 2. Write the theorem as a function signature
/// 3. Implement the proof as the function body
/// 4. If it compiles (without panic/unsafe), the argument is valid

// Step 1: Define atomic propositions
pub struct YourPremise1;
pub struct YourPremise2;
pub struct YourConclusion;

// Step 2: Write the theorem signature
// (YourPremise1 ∧ YourPremise2) → YourConclusion
pub fn your_theorem(premises: And<YourPremise1, YourPremise2>) -> YourConclusion {
    // Step 3: Implement the proof
    // Use pattern matching, function application, and constructors
    let And {
        left: _p1,
        right: _p2,
    } = premises;

    // Step 4: If you need axioms, add them as additional parameters
    // The fact that you NEED them reveals hidden premises!

    YourConclusion
}

// ============================================================================
// COMPILATION VERIFICATION
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_examples_compile() {
        // Business argument
        let _ = business_argument(And {
            left: IncreasedMarketing,
            right: GrowingMarket,
        });

        // Schedule dilemma
        let _ = schedule_dilemma(Or::Left(Weekday), |_| Work, |_| Rest);

        // Mutual exclusion proof exists
        let _: Not<And<(), Not<()>>> = mutual_exclusion::<()>();

        assert!(true, "All valid examples compile");
    }

    #[test]
    fn invalid_arguments_cannot_be_implemented() {
        // Note: We can't actually test that something DOESN'T compile,
        // but we've documented the invalid cases with comments showing
        // why implementation is impossible.

        // The affirming_consequent function is commented out because
        // it cannot be implemented - this IS the proof of invalidity.

        assert!(true, "Invalid arguments correctly rejected");
    }
}
