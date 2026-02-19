//! Code Generation Templates for the nexcore-dna language.
//!
//! Pre-built patterns that AI agents can invoke with minimal tokens.
//! Each template expands to a complete, optimized source program.
//!
//! ## Available Templates
//!
//! | Template | Tokens | Expands To |
//! |----------|--------|-----------|
//! | `sum(1, N)` | 3 | For loop with accumulator |
//! | `factorial(N)` | 3 | Recursive function |
//! | `fibonacci(N)` | 3 | Iterative with memory |
//! | `gcd(a, b)` | 4 | Euclidean algorithm |
//! | `is_prime(n)` | 3 | Trial division |
//! | `power(base, exp)` | 4 | Exponentiation by squaring |
//! | `abs_val(n)` | 3 | Absolute value via branch |
//! | `max_of(a, b)` | 4 | Binary maximum |
//! | `min_of(a, b)` | 4 | Binary minimum |
//! | `collatz(n)` | 3 | Collatz sequence steps |
//!
//! ## Usage
//!
//! ```rust
//! use nexcore_dna::lang::templates;
//!
//! let source = templates::expand("fibonacci", &[10]);
//! assert!(source.is_ok());
//! ```
//!
//! Tier: T3 (σ Sequence + ρ Recursion + N Quantity + → Causality + μ Mapping + ∂ Boundary)

use crate::error::{DnaError, Result};

// ============================================================================
// Template Registry
// ============================================================================

/// A code template descriptor.
///
/// Tier: T2-P (μ Mapping + σ Sequence + N Quantity)
#[derive(Debug, Clone)]
pub struct Template {
    /// Template name (e.g. "fibonacci").
    pub name: &'static str,
    /// Human description.
    pub description: &'static str,
    /// Parameter names.
    pub params: &'static [&'static str],
    /// The generator function.
    generator: fn(&[i64]) -> Result<String>,
}

/// List all available templates.
pub fn catalog() -> Vec<Template> {
    vec![
        Template {
            name: "sum",
            description: "Sum integers from start to end (inclusive)",
            params: &["start", "end"],
            generator: gen_sum,
        },
        Template {
            name: "factorial",
            description: "Factorial via recursive function",
            params: &["n"],
            generator: gen_factorial,
        },
        Template {
            name: "fibonacci",
            description: "Fibonacci number (iterative, O(n))",
            params: &["n"],
            generator: gen_fibonacci,
        },
        Template {
            name: "gcd",
            description: "Greatest common divisor (Euclidean algorithm)",
            params: &["a", "b"],
            generator: gen_gcd,
        },
        Template {
            name: "is_prime",
            description: "Primality test (trial division)",
            params: &["n"],
            generator: gen_is_prime,
        },
        Template {
            name: "power",
            description: "Exponentiation by squaring",
            params: &["base", "exp"],
            generator: gen_power,
        },
        Template {
            name: "abs_val",
            description: "Absolute value via conditional",
            params: &["n"],
            generator: gen_abs_val,
        },
        Template {
            name: "max_of",
            description: "Maximum of two values",
            params: &["a", "b"],
            generator: gen_max_of,
        },
        Template {
            name: "min_of",
            description: "Minimum of two values",
            params: &["a", "b"],
            generator: gen_min_of,
        },
        Template {
            name: "collatz",
            description: "Collatz sequence step count to reach 1",
            params: &["n"],
            generator: gen_collatz,
        },
    ]
}

/// Expand a template by name with given arguments.
///
/// Returns the generated source code.
pub fn expand(name: &str, args: &[i64]) -> Result<String> {
    let templates = catalog();
    let tmpl = templates
        .iter()
        .find(|t| t.name == name)
        .ok_or_else(|| DnaError::SyntaxError(0, format!("unknown template: '{name}'")))?;

    let expected = tmpl.params.len();
    if args.len() != expected {
        return Err(DnaError::SyntaxError(
            0,
            format!(
                "template '{}' expects {} args ({}), got {}",
                name,
                expected,
                tmpl.params.join(", "),
                args.len()
            ),
        ));
    }

    (tmpl.generator)(args)
}

/// Expand a template and compile+eval it, returning the output.
pub fn expand_eval(name: &str, args: &[i64]) -> Result<Vec<i64>> {
    let source = expand(name, args)?;
    let result = crate::lang::compiler::eval(&source)?;
    Ok(result.output)
}

/// List template names for auto-completion.
pub fn names() -> Vec<&'static str> {
    catalog().iter().map(|t| t.name).collect()
}

// ============================================================================
// Template Generators
// ============================================================================

fn gen_sum(args: &[i64]) -> Result<String> {
    let start = args[0];
    let end = args[1];
    Ok(format!(
        "let total = 0\n\
         for i = {start} to {end} do\n\
         \x20\x20total = total + i\n\
         end\n\
         total"
    ))
}

fn gen_factorial(args: &[i64]) -> Result<String> {
    let n = args[0];
    Ok(format!(
        "fn fact(n) do\n\
         \x20\x20if n <= 1 do\n\
         \x20\x20\x20\x20return 1\n\
         \x20\x20end\n\
         \x20\x20return n * fact(n - 1)\n\
         end\n\
         fact({n})"
    ))
}

fn gen_fibonacci(args: &[i64]) -> Result<String> {
    let n = args[0];
    Ok(format!(
        "fn fib(n) do\n\
         \x20\x20if n <= 1 do\n\
         \x20\x20\x20\x20return n\n\
         \x20\x20end\n\
         \x20\x20let a = 0\n\
         \x20\x20let b = 1\n\
         \x20\x20let i = 2\n\
         \x20\x20while i <= n do\n\
         \x20\x20\x20\x20let temp = b\n\
         \x20\x20\x20\x20b = a + b\n\
         \x20\x20\x20\x20a = temp\n\
         \x20\x20\x20\x20i = i + 1\n\
         \x20\x20end\n\
         \x20\x20return b\n\
         end\n\
         fib({n})"
    ))
}

fn gen_gcd(args: &[i64]) -> Result<String> {
    let a = args[0];
    let b = args[1];
    Ok(format!(
        "fn gcd(a, b) do\n\
         \x20\x20while b > 0 do\n\
         \x20\x20\x20\x20let temp = b\n\
         \x20\x20\x20\x20b = a % b\n\
         \x20\x20\x20\x20a = temp\n\
         \x20\x20end\n\
         \x20\x20return a\n\
         end\n\
         gcd({a}, {b})"
    ))
}

fn gen_is_prime(args: &[i64]) -> Result<String> {
    let n = args[0];
    Ok(format!(
        "fn is_prime(n) do\n\
         \x20\x20if n <= 1 do\n\
         \x20\x20\x20\x20return 0\n\
         \x20\x20end\n\
         \x20\x20if n <= 3 do\n\
         \x20\x20\x20\x20return 1\n\
         \x20\x20end\n\
         \x20\x20if n % 2 == 0 do\n\
         \x20\x20\x20\x20return 0\n\
         \x20\x20end\n\
         \x20\x20let d = 3\n\
         \x20\x20while d * d <= n do\n\
         \x20\x20\x20\x20if n % d == 0 do\n\
         \x20\x20\x20\x20\x20\x20return 0\n\
         \x20\x20\x20\x20end\n\
         \x20\x20\x20\x20d = d + 2\n\
         \x20\x20end\n\
         \x20\x20return 1\n\
         end\n\
         is_prime({n})"
    ))
}

fn gen_power(args: &[i64]) -> Result<String> {
    let base = args[0];
    let exp = args[1];
    Ok(format!(
        "fn power(base, exp) do\n\
         \x20\x20let result = 1\n\
         \x20\x20let b = base\n\
         \x20\x20let e = exp\n\
         \x20\x20while e > 0 do\n\
         \x20\x20\x20\x20if e % 2 == 1 do\n\
         \x20\x20\x20\x20\x20\x20result = result * b\n\
         \x20\x20\x20\x20end\n\
         \x20\x20\x20\x20b = b * b\n\
         \x20\x20\x20\x20e = e / 2\n\
         \x20\x20end\n\
         \x20\x20return result\n\
         end\n\
         power({base}, {exp})"
    ))
}

fn gen_abs_val(args: &[i64]) -> Result<String> {
    let n = args[0];
    Ok(format!(
        "fn abs_val(n) do\n\
         \x20\x20if n < 0 do\n\
         \x20\x20\x20\x20return 0 - n\n\
         \x20\x20end\n\
         \x20\x20return n\n\
         end\n\
         abs_val({n})"
    ))
}

fn gen_max_of(args: &[i64]) -> Result<String> {
    let a = args[0];
    let b = args[1];
    Ok(format!(
        "fn max_of(a, b) do\n\
         \x20\x20if a >= b do\n\
         \x20\x20\x20\x20return a\n\
         \x20\x20end\n\
         \x20\x20return b\n\
         end\n\
         max_of({a}, {b})"
    ))
}

fn gen_min_of(args: &[i64]) -> Result<String> {
    let a = args[0];
    let b = args[1];
    Ok(format!(
        "fn min_of(a, b) do\n\
         \x20\x20if a <= b do\n\
         \x20\x20\x20\x20return a\n\
         \x20\x20end\n\
         \x20\x20return b\n\
         end\n\
         min_of({a}, {b})"
    ))
}

fn gen_collatz(args: &[i64]) -> Result<String> {
    let n = args[0];
    Ok(format!(
        "fn collatz(n) do\n\
         \x20\x20let steps = 0\n\
         \x20\x20let val = n\n\
         \x20\x20while val > 1 do\n\
         \x20\x20\x20\x20if val % 2 == 0 do\n\
         \x20\x20\x20\x20\x20\x20val = val / 2\n\
         \x20\x20\x20\x20else\n\
         \x20\x20\x20\x20\x20\x20val = val * 3 + 1\n\
         \x20\x20\x20\x20end\n\
         \x20\x20\x20\x20steps = steps + 1\n\
         \x20\x20end\n\
         \x20\x20return steps\n\
         end\n\
         collatz({n})"
    ))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_all_templates() {
        let cat = catalog();
        assert_eq!(cat.len(), 10);
        let names_list: Vec<&str> = cat.iter().map(|t| t.name).collect();
        assert!(names_list.contains(&"sum"));
        assert!(names_list.contains(&"factorial"));
        assert!(names_list.contains(&"fibonacci"));
        assert!(names_list.contains(&"gcd"));
        assert!(names_list.contains(&"is_prime"));
        assert!(names_list.contains(&"power"));
        assert!(names_list.contains(&"abs_val"));
        assert!(names_list.contains(&"max_of"));
        assert!(names_list.contains(&"min_of"));
        assert!(names_list.contains(&"collatz"));
    }

    #[test]
    fn names_list() {
        let n = names();
        assert_eq!(n.len(), 10);
    }

    #[test]
    fn template_sum() {
        let result = expand_eval("sum", &[1, 10]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![55]); // 1+2+...+10 = 55
        }
    }

    #[test]
    fn template_sum_single() {
        let result = expand_eval("sum", &[5, 5]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![5]);
        }
    }

    #[test]
    fn template_factorial() {
        let result = expand_eval("factorial", &[5]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![120]);
        }
    }

    #[test]
    fn template_factorial_zero() {
        let result = expand_eval("factorial", &[0]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![1]);
        }
    }

    #[test]
    fn template_fibonacci() {
        let result = expand_eval("fibonacci", &[10]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![55]);
        }
    }

    #[test]
    fn template_fibonacci_zero() {
        let result = expand_eval("fibonacci", &[0]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![0]);
        }
    }

    #[test]
    fn template_fibonacci_one() {
        let result = expand_eval("fibonacci", &[1]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![1]);
        }
    }

    #[test]
    fn template_gcd() {
        let result = expand_eval("gcd", &[48, 18]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![6]);
        }
    }

    #[test]
    fn template_gcd_coprime() {
        let result = expand_eval("gcd", &[13, 7]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![1]);
        }
    }

    #[test]
    fn template_is_prime_true() {
        let result = expand_eval("is_prime", &[17]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![1]);
        }
    }

    #[test]
    fn template_is_prime_false() {
        let result = expand_eval("is_prime", &[15]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![0]);
        }
    }

    #[test]
    fn template_is_prime_two() {
        let result = expand_eval("is_prime", &[2]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![1]);
        }
    }

    #[test]
    fn template_is_prime_one() {
        let result = expand_eval("is_prime", &[1]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![0]);
        }
    }

    #[test]
    fn template_power() {
        let result = expand_eval("power", &[2, 10]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![1024]);
        }
    }

    #[test]
    fn template_power_zero_exp() {
        let result = expand_eval("power", &[5, 0]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![1]);
        }
    }

    #[test]
    fn template_abs_val_positive() {
        let result = expand_eval("abs_val", &[42]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![42]);
        }
    }

    #[test]
    fn template_abs_val_negative() {
        let result = expand_eval("abs_val", &[-7]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![7]);
        }
    }

    #[test]
    fn template_max_of() {
        let result = expand_eval("max_of", &[3, 7]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![7]);
        }
    }

    #[test]
    fn template_min_of() {
        let result = expand_eval("min_of", &[3, 7]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![3]);
        }
    }

    #[test]
    fn template_collatz() {
        // collatz(6): 6→3→10→5→16→8→4→2→1 = 8 steps
        let result = expand_eval("collatz", &[6]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![8]);
        }
    }

    #[test]
    fn template_collatz_one() {
        let result = expand_eval("collatz", &[1]);
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, vec![0]); // already at 1
        }
    }

    // --- Error handling ---

    #[test]
    fn template_unknown() {
        let result = expand("nonexistent", &[1]);
        assert!(result.is_err());
    }

    #[test]
    fn template_wrong_arity() {
        let result = expand("fibonacci", &[1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn template_expand_source() {
        // Verify source text is valid
        let source = expand("sum", &[1, 5]);
        assert!(source.is_ok());
        if let Ok(s) = source {
            assert!(s.contains("total"));
            assert!(s.contains("for"));
        }
    }
}
