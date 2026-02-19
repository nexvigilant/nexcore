//! # Common Traits
//!
//! Foundational traits for execution and calculation patterns.

use anyhow::Result;

/// A trait for types that perform asynchronous execution.
///
/// Used by agents, orchestrators, and skill executors.
pub trait Executable {
    /// Input type for execution
    type Input;
    /// Output type from execution
    type Output;
    /// Error type for execution failures
    type Error;

    /// Execute the operation asynchronously.
    fn execute(
        &self,
        input: Self::Input,
    ) -> impl std::future::Future<Output = Result<Self::Output, Self::Error>> + Send;
}

/// A trait for types that perform synchronous calculations.
///
/// Used by pure computational functions (signal detection, causality, etc.).
///
/// # Example
///
/// ```
/// use nexcore_vigilance::foundation::traits::Calculable;
///
/// struct Adder;
///
/// impl Calculable for Adder {
///     type Input = (i32, i32);
///     type Output = i32;
///
///     fn calculate(&self, input: Self::Input) -> Self::Output {
///         input.0 + input.1
///     }
/// }
///
/// let adder = Adder;
/// assert_eq!(adder.calculate((2, 3)), 5);
/// ```
pub trait Calculable {
    /// Input type for calculation
    type Input;
    /// Output type from calculation
    type Output;

    /// Perform the calculation.
    fn calculate(&self, input: Self::Input) -> Self::Output;
}

/// A result that includes a mandatory safety assessment.
pub struct VigilantResult<T> {
    /// The raw computational result
    pub data: T,
    /// The associated safety margin (d(s))
    pub safety_margin: f32,
    /// The epistemic trust score (0.0-1.0)
    pub trust_score: f64,
}

/// A trait for calculations that must be performed within safety axioms.
pub trait SafeCalculable {
    /// The input type for the calculation.
    type Input;
    /// The output type for the calculation.
    type Output;

    /// Calculate the result and automatically compute the safety manifold distance.
    fn calculate_safe(&self, input: Self::Input) -> VigilantResult<Self::Output>;
}

/// A trait for types that can be validated.
pub trait Validatable {
    /// Error type for validation failures
    type Error;

    /// Validate the instance.
    fn validate(&self) -> Result<(), Self::Error>;

    /// Check if the instance is valid.
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

/// A trait for types that can produce a deterministic hash.
pub trait Hashable {
    /// Produce a deterministic hash of the instance.
    fn content_hash(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Multiplier(i32);

    impl Calculable for Multiplier {
        type Input = i32;
        type Output = i32;

        fn calculate(&self, input: Self::Input) -> Self::Output {
            input * self.0
        }
    }

    #[test]
    fn test_calculable() {
        let multiplier = Multiplier(3);
        assert_eq!(multiplier.calculate(4), 12);
    }
}
