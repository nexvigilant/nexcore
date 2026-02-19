//! Declarative macros for GROUNDED.
//!
//! `uncertain_match!` — Exhaustive confidence band matching.
//! `verify!` — Specification-based assertion with evidence.

/// Match on confidence bands exhaustively.
///
/// Forces the caller to handle all four confidence levels (High, Medium, Low, Negligible),
/// preventing silent propagation of low-confidence values.
///
/// # Examples
/// ```
/// use grounded::{Uncertain, Confidence, uncertain_match};
///
/// let prediction = Uncertain::new(42.0_f64, Confidence::new(0.87).unwrap());
///
/// let action = uncertain_match!(prediction,
///     High(val) => format!("act on {val}"),
///     Medium(val) => format!("act with fallback for {val}"),
///     Low(val) => format!("need more evidence for {val}"),
///     Negligible(val) => format!("cannot use {val}"),
/// );
///
/// assert!(action.contains("fallback"));
/// ```
#[macro_export]
macro_rules! uncertain_match {
    ($uncertain:expr,
        High($h:ident) => $high:expr,
        Medium($m:ident) => $med:expr,
        Low($l:ident) => $low:expr,
        Negligible($n:ident) => $neg:expr $(,)?
    ) => {{
        let __uncertain = $uncertain;
        match __uncertain.band() {
            $crate::ConfidenceBand::High => {
                let $h = __uncertain.value();
                $high
            }
            $crate::ConfidenceBand::Medium => {
                let $m = __uncertain.value();
                $med
            }
            $crate::ConfidenceBand::Low => {
                let $l = __uncertain.value();
                $low
            }
            $crate::ConfidenceBand::Negligible => {
                let $n = __uncertain.value();
                $neg
            }
        }
    }};
}

/// Verify a specification at runtime with evidence tracking.
///
/// Returns `Ok(value)` if the specification holds, `Err(GroundedError)` if violated.
///
/// # Examples
/// ```
/// use grounded::verify;
///
/// fn sort(mut v: Vec<i32>) -> Vec<i32> {
///     v.sort();
///     v
/// }
///
/// let input = vec![3, 1, 4, 1, 5];
/// let output = sort(input.clone());
///
/// let verified: Result<Vec<i32>, grounded::GroundedError> = verify!(output,
///     len: output.len() == input.len(),
///     sorted: output.windows(2).all(|w| w[0] <= w[1]),
///     permutation: {
///         let mut a = input.clone();
///         let mut b = output.clone();
///         a.sort();
///         b.sort();
///         a == b
///     },
/// );
///
/// assert!(verified.is_ok());
/// ```
#[macro_export]
macro_rules! verify {
    ($value:expr, $($name:ident : $check:expr),+ $(,)?) => {{
        // Evaluate checks BEFORE moving the value (checks may borrow it)
        let mut __violations: Vec<String> = Vec::new();

        $(
            if !($check) {
                __violations.push(format!(
                    "specification '{}' violated",
                    stringify!($name)
                ));
            }
        )+

        if __violations.is_empty() {
            Ok($value)
        } else {
            Err($crate::GroundedError::SpecificationViolated(
                __violations.join("; ")
            ))
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::{Confidence, GroundedError, Uncertain};

    #[test]
    fn uncertain_match_exhaustive() {
        let cases: Vec<(f64, &str)> = vec![
            (0.96, "high"),
            (0.85, "medium"),
            (0.60, "low"),
            (0.30, "negligible"),
        ];

        for (conf_val, expected_label) in cases {
            let u = Uncertain::new(42, Confidence::new(conf_val).unwrap_or(Confidence::NONE));
            let label = uncertain_match!(u,
                High(_v) => "high",
                Medium(_v) => "medium",
                Low(_v) => "low",
                Negligible(_v) => "negligible",
            );
            assert_eq!(label, expected_label, "failed for confidence {conf_val}");
        }
    }

    #[test]
    fn verify_passing() {
        let data = vec![1, 2, 3, 4, 5];
        let result: Result<Vec<i32>, GroundedError> = verify!(data,
            non_empty: !data.is_empty(),
            sorted: data.windows(2).all(|w| w[0] <= w[1]),
            bounded: data.iter().all(|&x| x > 0 && x < 100),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn verify_failing() {
        let data = vec![5, 3, 1];
        let result: Result<Vec<i32>, GroundedError> = verify!(data,
            non_empty: !data.is_empty(),
            sorted: data.windows(2).all(|w| w[0] <= w[1]),
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("sorted"),
            "error should mention 'sorted': {msg}"
        );
    }
}
