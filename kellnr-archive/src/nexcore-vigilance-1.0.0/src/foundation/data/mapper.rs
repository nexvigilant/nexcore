//! # Data Mapping Utilities
//!
//! High-performance fuzzy mapping for data normalization (e.g. drug name mapping).

use super::super::algorithms::levenshtein::levenshtein_distance;
use super::super::traits::Calculable;
use serde::{Deserialize, Serialize};

/// A single mapping result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingResult {
    /// The original input name
    pub source: String,
    /// The matched target name
    pub target: String,
    /// The edit distance
    pub distance: usize,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

/// Data mapper for fuzzy string matching
pub struct DataMapper {
    /// Reference list of target names (e.g. standard drug names)
    pub targets: Vec<String>,
    /// Maximum distance allowed for a match
    pub max_distance: usize,
}

impl DataMapper {
    /// Create a new mapper with targets and threshold
    pub fn new(targets: Vec<String>, max_distance: usize) -> Self {
        Self {
            targets,
            max_distance,
        }
    }

    /// Map a single name to the best target
    pub fn map(&self, source: &str) -> Option<MappingResult> {
        let mut best_match: Option<(String, usize)> = None;

        for target in &self.targets {
            let dist = levenshtein_distance(source, target);

            if dist <= self.max_distance {
                match best_match {
                    None => best_match = Some((target.clone(), dist)),
                    Some((_, current_best)) if dist < current_best => {
                        best_match = Some((target.clone(), dist));
                    }
                    _ => {}
                }
            }
        }

        best_match.map(|(target, distance)| {
            let max_len = source.len().max(target.len());
            let confidence = if max_len == 0 {
                1.0
            } else {
                1.0 - (distance as f64 / max_len as f64)
            };

            MappingResult {
                source: source.to_string(),
                target,
                distance,
                confidence,
            }
        })
    }
}

impl Calculable for DataMapper {
    type Input = String;
    type Output = Option<MappingResult>;

    fn calculate(&self, input: Self::Input) -> Self::Output {
        self.map(&input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_mapper_exact() {
        let targets = vec!["ASPIRIN".to_string(), "METFORMIN".to_string()];
        let mapper = DataMapper::new(targets, 2);
        let result = mapper.map("ASPIRIN").unwrap();
        assert_eq!(result.target, "ASPIRIN");
        assert_eq!(result.distance, 0);
    }

    #[test]
    fn test_data_mapper_fuzzy() {
        let targets = vec!["ATORVASTATIN".to_string()];
        let mapper = DataMapper::new(targets, 3);
        let result = mapper.map("ATORVASTATN").unwrap(); // Missing 'I'
        assert_eq!(result.target, "ATORVASTATIN");
        assert_eq!(result.distance, 1);
        assert!(result.confidence > 0.9);
    }
}
