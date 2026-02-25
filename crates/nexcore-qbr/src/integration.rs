//! Numerical quadrature — Simpson's composite rule.
//!
//! Sovereign implementation. Zero external dependencies.
//! Exact for polynomials up to degree 3. Well-suited for smooth
//! Hill curves over therapeutic dose ranges.
//!
//! Tier: T2-P (Foundation Primitive)
//! Grounding: N(Quantity) + Σ(Sum) + ∂(Boundary)

use crate::error::QbrError;

/// Composite Simpson's rule for definite integration.
///
/// Computes `∫[a,b] f(x) dx` using `n` subintervals (must be even).
///
/// # Arguments
/// * `f` — The integrand function `f(x) -> f64`
/// * `a` — Lower bound of integration
/// * `b` — Upper bound of integration
/// * `n` — Number of subintervals (must be even and >= 2)
///
/// # Errors
/// Returns `QbrError::Integration` if `n` is odd, zero, or bounds are invalid.
pub fn simpson_integrate<F>(f: F, a: f64, b: f64, n: usize) -> Result<f64, QbrError>
where
    F: Fn(f64) -> f64,
{
    if n < 2 {
        return Err(QbrError::Integration(
            "Number of intervals must be >= 2".to_string(),
        ));
    }
    if n % 2 != 0 {
        return Err(QbrError::Integration(
            "Number of intervals must be even for Simpson's rule".to_string(),
        ));
    }
    if b <= a {
        return Err(QbrError::Integration(format!(
            "Upper bound ({b}) must be greater than lower bound ({a})"
        )));
    }
    if !a.is_finite() || !b.is_finite() {
        return Err(QbrError::Integration(
            "Integration bounds must be finite".to_string(),
        ));
    }

    let h = (b - a) / n as f64;
    let mut sum = f(a) + f(b);

    // Odd indices: coefficient 4
    for i in (1..n).step_by(2) {
        sum += 4.0 * f(a + i as f64 * h);
    }

    // Even indices: coefficient 2
    for i in (2..n).step_by(2) {
        sum += 2.0 * f(a + i as f64 * h);
    }

    Ok(sum * h / 3.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrate_constant() {
        // ∫[0,1] 5 dx = 5.0
        let result = simpson_integrate(|_| 5.0, 0.0, 1.0, 100).unwrap_or(0.0);
        assert!((result - 5.0).abs() < 1e-12, "Expected 5.0, got {result}");
    }

    #[test]
    fn test_integrate_linear() {
        // ∫[0,1] x dx = 0.5
        let result = simpson_integrate(|x| x, 0.0, 1.0, 100).unwrap_or(0.0);
        assert!((result - 0.5).abs() < 1e-12, "Expected 0.5, got {result}");
    }

    #[test]
    fn test_integrate_quadratic() {
        // ∫[0,1] x² dx = 1/3
        let result = simpson_integrate(|x| x * x, 0.0, 1.0, 100).unwrap_or(0.0);
        assert!(
            (result - 1.0 / 3.0).abs() < 1e-10,
            "Expected 0.333..., got {result}"
        );
    }

    #[test]
    fn test_integrate_cubic() {
        // ∫[0,1] x³ dx = 1/4 — Simpson's is exact for degree <= 3
        let result = simpson_integrate(|x| x * x * x, 0.0, 1.0, 2).unwrap_or(0.0);
        assert!((result - 0.25).abs() < 1e-12, "Expected 0.25, got {result}");
    }

    #[test]
    fn test_integrate_sin() {
        // ∫[0,π] sin(x) dx = 2.0
        let result = simpson_integrate(|x| x.sin(), 0.0, std::f64::consts::PI, 1000).unwrap_or(0.0);
        assert!((result - 2.0).abs() < 1e-8, "Expected 2.0, got {result}");
    }

    #[test]
    fn test_odd_intervals_rejected() {
        let result = simpson_integrate(|x| x, 0.0, 1.0, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_intervals_rejected() {
        let result = simpson_integrate(|x| x, 0.0, 1.0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_bounds_rejected() {
        let result = simpson_integrate(|x| x, 1.0, 0.0, 100);
        assert!(result.is_err());
    }
}
