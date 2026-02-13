//! # Cognitive Power Analysis
//!
//! Statistical power analysis for AI context sufficiency, adapted from
//! clinical trial sample size calculations.
//!
//! ## Theoretical Foundation
//!
//! This module applies pharmaceutical clinical trial methodology to AI cognition:
//! - **Effect Size** (d) = unique_knowledge / task_complexity
//! - **Alpha** = probability of hallucination (Type I Error)
//! - **Power** = probability of successful resolution (1 - Type II Error)
//!
//! ## Clinical Standard
//!
//! The 80% power threshold is the same standard used in FDA Phase III trials.
//!
//! ## Example
//!
//! ```
//! use nexcore_vigilance::stark::cognitive_power::CognitivePowerAnalyzer;
//!
//! // High knowledge (100 units) for low complexity (10 units)
//! let power = CognitivePowerAnalyzer::calculate_context_power(100, 10, 0.05);
//! assert!(CognitivePowerAnalyzer::is_sufficient(power));
//!
//! // Low knowledge (2 units) for high complexity (100 units)
//! let power = CognitivePowerAnalyzer::calculate_context_power(2, 100, 0.1);
//! assert!(!CognitivePowerAnalyzer::is_sufficient(power));
//! ```

/// Cognitive Power Analyzer for AI context sufficiency.
///
/// Translates clinical statistical power concepts into AI context measurement.
pub struct CognitivePowerAnalyzer;

impl CognitivePowerAnalyzer {
    /// Calculate the 'Statistical Power' of an AI context.
    ///
    /// Analogous to clinical sample size calculation:
    /// - Effect Size (d) = unique_knowledge_count / task_complexity
    /// - Alpha = probability of hallucination (Type I Error)
    /// - Power = probability of successful resolution (1 - Type II Error)
    ///
    /// # Arguments
    ///
    /// * `unique_knowledge` - Number of unique knowledge units available
    /// * `task_complexity` - Measure of task complexity (e.g., token count, steps)
    /// * `hallucination_probability` - Estimated probability of hallucination (0.0-1.0)
    ///
    /// # Returns
    ///
    /// Power score between 0.0 and 1.0
    #[must_use]
    pub fn calculate_context_power(
        unique_knowledge: usize,
        task_complexity: usize,
        hallucination_probability: f64,
    ) -> f64 {
        if task_complexity == 0 {
            return 1.0;
        }

        let effect_size = (unique_knowledge as f64) / (task_complexity as f64);

        // Simplified sigmoid-based power curve
        // As knowledge/complexity increases, power approaches 1.0
        // Hallucination probability acts as a constant dampener
        let power = 1.0 / (1.0 + (-5.0 * (effect_size - 0.5)).exp());

        power * (1.0 - hallucination_probability)
    }

    /// Determine if the agent has sufficient 'Cognitive Power' to proceed.
    ///
    /// Uses the clinical standard of 80% power threshold.
    #[must_use]
    pub fn is_sufficient(power: f64) -> bool {
        power >= 0.8 // 80% power threshold - Clinical Standard
    }

    /// Calculate the minimum knowledge required for a given complexity.
    ///
    /// Solves for the knowledge level that would achieve 80% power.
    #[must_use]
    pub fn minimum_knowledge_for_complexity(
        task_complexity: usize,
        hallucination_probability: f64,
    ) -> usize {
        // Target power = 0.80, solve for knowledge
        // This is an approximation using the sigmoid inverse
        let target_power = 0.8;
        let adjusted_target = target_power / (1.0 - hallucination_probability);

        // Sigmoid inverse: x = ln(y / (1 - y)) / 5 + 0.5
        // where x = effect_size = knowledge / complexity
        let effect_size = if adjusted_target >= 1.0 {
            2.0 // Maximum practical effect size
        } else if adjusted_target <= 0.0 {
            0.0
        } else {
            (adjusted_target / (1.0 - adjusted_target)).ln() / 5.0 + 0.5
        };

        ((effect_size * (task_complexity as f64)).ceil() as usize).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_power() {
        // High knowledge (100) for low complexity (10)
        let p = CognitivePowerAnalyzer::calculate_context_power(100, 10, 0.05);
        assert!(p > 0.9);
        assert!(CognitivePowerAnalyzer::is_sufficient(p));
    }

    #[test]
    fn test_low_power() {
        // Low knowledge (2) for high complexity (100)
        let p = CognitivePowerAnalyzer::calculate_context_power(2, 100, 0.1);
        assert!(p < 0.5);
        assert!(!CognitivePowerAnalyzer::is_sufficient(p));
    }

    #[test]
    fn test_zero_complexity() {
        // Zero complexity should return full power
        let p = CognitivePowerAnalyzer::calculate_context_power(10, 0, 0.0);
        assert!((p - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_minimum_knowledge_calculation() {
        let complexity = 100;
        let hallucination_prob = 0.05;
        let min_knowledge = CognitivePowerAnalyzer::minimum_knowledge_for_complexity(
            complexity,
            hallucination_prob,
        );

        // The calculated minimum should achieve ~80% power
        let power = CognitivePowerAnalyzer::calculate_context_power(
            min_knowledge,
            complexity,
            hallucination_prob,
        );
        assert!(power >= 0.75, "Expected power >= 0.75, got {power}");
    }
}
