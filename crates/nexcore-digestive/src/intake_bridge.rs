//! # Intake Bridge
//!
//! Inter-crate pipeline: Respiratory → Digestive → Energy.
//!
//! Converts respiratory `Extracted` output into digestive `Fragment` input,
//! and digestive `Metabolized` output into energy-compatible token replenishment.
//!
//! ```text
//! Respiratory::Extracted → Fragment → [digest] → Metabolized → token_yield (u64)
//! ```

use nexcore_respiratory::{Extracted, ExchangeResult, InputSource};

use crate::{DataKind, Fragment, Metabolized};

/// Convert a respiratory `Extracted` item into a digestive `Fragment`.
///
/// **Biological mapping**: Swallowing — food passes from airway exchange
/// surface into the digestive tract for decomposition.
pub fn extracted_to_fragment(extracted: &Extracted, index: usize) -> Fragment {
    let kind = classify_source(extracted.source);
    Fragment {
        index,
        content: extracted.content.clone(),
        kind,
    }
}

/// Convert all extracted items from a respiratory exchange cycle into fragments.
///
/// **Biological mapping**: Bolus formation — grouped intake ready for stomach.
pub fn exchange_to_fragments(result: &ExchangeResult) -> Vec<Fragment> {
    result
        .extracted
        .iter()
        .enumerate()
        .map(|(i, ext)| extracted_to_fragment(ext, i))
        .collect()
}

/// Compute energy token yield from a metabolized digestive output.
///
/// Returns the number of tokens that can be recycled back into the energy pool
/// via `TokenPool::recycle()`. The yield is the `energy` field from `Metabolized`,
/// representing information content extracted during liver processing.
///
/// **Biological mapping**: Nutrient absorption → ATP synthesis.
/// Digested food becomes glucose → enters mitochondria → produces ATP.
pub fn metabolized_to_token_yield(metabolized: &Metabolized) -> u64 {
    // energy field is usize (information content proxy from liver).
    // Map directly to token count — each unit of digestive energy = 1 recyclable token.
    metabolized.energy as u64
}

/// Compute total token yield from a batch of metabolized outputs.
///
/// Suitable for passing to `TokenPool::recycle(total)`.
pub fn batch_token_yield(metabolized: &[Metabolized]) -> u64 {
    metabolized
        .iter()
        .map(|m| metabolized_to_token_yield(m))
        .sum()
}

/// Map respiratory `InputSource` to digestive `DataKind`.
fn classify_source(source: InputSource) -> DataKind {
    match source {
        InputSource::Api => DataKind::Object,
        InputSource::File => DataKind::Text,
        InputSource::Config => DataKind::Object,
        InputSource::Event => DataKind::Text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DigestiveTract, Mouth, Stomach};

    #[test]
    fn test_extracted_to_fragment() {
        let extracted = Extracted {
            content: "api response data".to_string(),
            source: InputSource::Api,
            priority: 3,
        };

        let fragment = extracted_to_fragment(&extracted, 0);

        assert_eq!(fragment.index, 0);
        assert_eq!(fragment.content, "api response data");
        assert_eq!(fragment.kind, DataKind::Object);
    }

    #[test]
    fn test_exchange_to_fragments() {
        let result = ExchangeResult {
            extracted: vec![
                Extracted {
                    content: "first item".to_string(),
                    source: InputSource::File,
                    priority: 3,
                },
                Extracted {
                    content: "second item".to_string(),
                    source: InputSource::Event,
                    priority: 1,
                },
            ],
            exhaled: nexcore_respiratory::Exhaled::default(),
        };

        let fragments = exchange_to_fragments(&result);

        assert_eq!(fragments.len(), 2);
        assert_eq!(fragments[0].index, 0);
        assert_eq!(fragments[0].kind, DataKind::Text);
        assert_eq!(fragments[1].index, 1);
        assert_eq!(fragments[1].content, "second item");
    }

    #[test]
    fn test_metabolized_to_token_yield() {
        let metabolized = Metabolized {
            original: "hello world".to_string(),
            processed: "Hello world".to_string(),
            energy: 11,
        };

        let yield_tokens = metabolized_to_token_yield(&metabolized);
        assert_eq!(yield_tokens, 11);
    }

    #[test]
    fn test_batch_token_yield() {
        let batch = vec![
            Metabolized {
                original: "a".to_string(),
                processed: "A".to_string(),
                energy: 5,
            },
            Metabolized {
                original: "bb".to_string(),
                processed: "Bb".to_string(),
                energy: 8,
            },
        ];

        assert_eq!(batch_token_yield(&batch), 13);
    }

    #[test]
    fn test_full_pipeline_respiratory_to_energy() {
        // 1. Simulate respiratory output
        let exchange_result = ExchangeResult {
            extracted: vec![
                Extracted {
                    content: "drug:aspirin count:150".to_string(),
                    source: InputSource::Api,
                    priority: 3,
                },
                Extracted {
                    content: "patient data received".to_string(),
                    source: InputSource::File,
                    priority: 2,
                },
            ],
            exhaled: nexcore_respiratory::Exhaled::default(),
        };

        // 2. Convert respiratory → digestive fragments
        let fragments = exchange_to_fragments(&exchange_result);
        assert_eq!(fragments.len(), 2);

        // 3. Feed through digestive tract (stomach + intestine + liver)
        let mut tract = DigestiveTract::default();
        // Use mouth to re-chew (the fragments carry raw content)
        // But we can also directly feed the stomach
        let mut stomach = Stomach::default();
        stomach.ingest_batch(fragments);
        let nutrients = stomach.digest();
        assert!(!nutrients.is_empty());

        // 4. Liver metabolizes stored content
        let mut liver = crate::Liver::default();
        for n in &nutrients {
            // Store the fats (string values) for metabolization
            liver.store(&n.fats);
            // Proteins are structural keys — also store
            liver.store(&n.proteins);
        }
        let metabolized = liver.metabolize();

        // 5. Convert digestive output → energy token yield
        let total_yield = batch_token_yield(&metabolized);

        // The yield should be > 0 (we processed real content)
        assert!(total_yield > 0, "Pipeline should produce energy tokens");

        // 6. Verify the yield could feed TokenPool::recycle
        // (We don't import energy here — just prove the u64 is valid)
        assert!(
            total_yield < 1_000_000,
            "Yield should be bounded and reasonable"
        );
    }

    #[test]
    fn test_classify_source_mapping() {
        assert_eq!(classify_source(InputSource::Api), DataKind::Object);
        assert_eq!(classify_source(InputSource::File), DataKind::Text);
        assert_eq!(classify_source(InputSource::Config), DataKind::Object);
        assert_eq!(classify_source(InputSource::Event), DataKind::Text);
    }
}
