/// MLM Batch with masked tokens and labels
#[derive(Debug, Clone)]
pub struct MLMBatch {
    pub input_ids: Vec<Vec<i64>>,
    #[allow(dead_code)]
    pub attention_mask: Vec<Vec<f32>>,
    pub masked_ids: Vec<Vec<i64>>,
    pub masked_positions: Vec<Vec<i64>>,
}

/// DNA nucleotide vocabulary: char-level tokenization
/// Special tokens: [PAD]=0, [UNK]=1, [CLS]=2, [SEP]=3, [MASK]=4
/// Nucleotides: A=5, T=6, G=7, C=8
pub struct DnaVocabulary {
    #[allow(dead_code)]
    pub vocab_size: usize,
}

impl DnaVocabulary {
    pub const PAD: i64 = 0;
    pub const UNK: i64 = 1;
    pub const CLS: i64 = 2;
    pub const SEP: i64 = 3;
    pub const MASK: i64 = 4;
    pub const A: i64 = 5;
    pub const T: i64 = 6;
    pub const G: i64 = 7;
    pub const C: i64 = 8;
    pub const VOCAB_SIZE: usize = 9;

    pub fn new() -> Self {
        Self {
            vocab_size: Self::VOCAB_SIZE,
        }
    }

    pub fn encode_nucleotide(&self, ch: char) -> i64 {
        match ch {
            'A' | 'a' => Self::A,
            'T' | 't' => Self::T,
            'G' | 'g' => Self::G,
            'C' | 'c' => Self::C,
            _ => Self::UNK,
        }
    }

    #[allow(dead_code)]
    pub fn decode(&self, id: i64) -> char {
        match id {
            0 => '_', // PAD
            1 => '?', // UNK
            2 => '[', // CLS
            3 => ']', // SEP
            4 => '*', // MASK
            5 => 'A',
            6 => 'T',
            7 => 'G',
            8 => 'C',
            _ => '?',
        }
    }
}

/// In-memory DNA dataset with MLM preprocessing
pub struct TextDataset {
    sequences: Vec<String>,
    pub vocab: DnaVocabulary,
    batch_size: usize,
    seq_length: usize,
    mask_prob: f32,
}

impl TextDataset {
    pub fn new(batch_size: usize, seq_length: usize) -> Self {
        let vocab = DnaVocabulary::new();

        // Real DNA sequences for training
        let sequences = vec![
            // Coding sequences (start codon + coding + stop)
            "ATGGCTAGCGATCGATCGATCGATCGTAA".to_string(),
            "ATGCCCGGGAAATTTCCCGGGAAATTTCGATAG".to_string(),
            "ATGATGATGATGATGATGATGATGATGATGTAA".to_string(),
            "ATGCGATCGATCGATCGATCGATCGATCGTGA".to_string(),
            "ATGGGGCCCAAAGGGCCCAAAGGGCCCAAATAG".to_string(),
            "ATGAAAAAAAATTTTTTTTCCCCCCCCGGGGTAA".to_string(),
            "ATGCAGCAGCAGCAGCAGCAGCAGCAGCAGTGA".to_string(),
            "ATGTCATCATCATCATCATCATCATCATCATAG".to_string(),
            // Regulatory regions (promoter-like)
            "TATAAAAGGCGCGCGCGCGCATATATATATATA".to_string(),
            "CCAATCCCGCGCGCATATATATGCGCGCGCGC".to_string(),
            // Repetitive elements
            "ATATATATATATATATATATATATATATATATAT".to_string(),
            "GCGCGCGCGCGCGCGCGCGCGCGCGCGCGCGC".to_string(),
            "CAGCAGCAGCAGCAGCAGCAGCAGCAGCAGCAG".to_string(),
            // GC-rich and AT-rich regions
            "GCCGCCGCCGCCGCCGCCGCCGCCGCCGCCGCC".to_string(),
            "ATTAATTAATTAATTAATTAATTAATTAATTAAT".to_string(),
            // Mixed patterns
            "ATCGATCGATCGATCGATCGATCGATCGATCGAT".to_string(),
            "TAGCTAGCTAGCTAGCTAGCTAGCTAGCTAGCTA".to_string(),
            "AGTCAGTCAGTCAGTCAGTCAGTCAGTCAGTCA".to_string(),
            "GACTGACTGACTGACTGACTGACTGACTGACTGA".to_string(),
            "ACGTACGTACGTACGTACGTACGTACGTACGTAC".to_string(),
            // Longer coding sequences
            "ATGGCGATCGATCGATCGAATCGATCGATCGATCGATCGTAA".to_string(),
            "ATGCCCAAAGGGTTTATGCCCAAAGGGTTTATGCCCTAG".to_string(),
            "ATGTTTAAACCCGGGATGTTTAAACCCGGGATGTTTAAATGA".to_string(),
            "ATGGGCGGCGGCATGATGATGATGCCCAAATTTGGGTAA".to_string(),
        ];

        Self {
            sequences,
            vocab,
            batch_size,
            seq_length,
            mask_prob: 0.15,
        }
    }

    /// Tokenize DNA sequence to token IDs with [CLS] and [SEP]
    fn tokenize(&self, sequence: &str) -> Vec<i64> {
        let mut tokens = Vec::with_capacity(self.seq_length);
        tokens.push(DnaVocabulary::CLS);

        for ch in sequence.chars() {
            if tokens.len() >= self.seq_length - 1 {
                break;
            }
            tokens.push(self.vocab.encode_nucleotide(ch));
        }

        tokens.push(DnaVocabulary::SEP);

        // Pad to seq_length
        while tokens.len() < self.seq_length {
            tokens.push(DnaVocabulary::PAD);
        }

        tokens.truncate(self.seq_length);
        tokens
    }

    /// Create MLM batch with masking (80% mask, 10% random, 10% keep)
    fn create_mlm_batch(&self, tokens: &[i64]) -> (Vec<i64>, Vec<i64>, Vec<i64>) {
        let mut masked_tokens = tokens.to_vec();
        let mut masked_ids = Vec::new();
        let mut masked_positions = Vec::new();

        for (pos, &token) in tokens.iter().enumerate() {
            // Skip special tokens (PAD, UNK, CLS, SEP, MASK) — only mask nucleotides
            if token >= DnaVocabulary::A && rand::random::<f32>() < self.mask_prob {
                masked_ids.push(token);
                masked_positions.push(pos as i64);

                let rand_val = rand::random::<f32>();
                if rand_val < 0.8 {
                    masked_tokens[pos] = DnaVocabulary::MASK;
                } else if rand_val < 0.9 {
                    // Random nucleotide (5-8)
                    masked_tokens[pos] = (rand::random::<u32>() % 4 + 5) as i64;
                }
                // else: keep original (10%)
            }
        }

        (masked_tokens, masked_ids, masked_positions)
    }

    /// Get next batch by index
    pub fn get_batch(&self, batch_idx: usize) -> Option<MLMBatch> {
        let start_idx = batch_idx * self.batch_size;
        if start_idx >= self.sequences.len() {
            return None;
        }

        let end_idx = (start_idx + self.batch_size).min(self.sequences.len());
        let batch_seqs = &self.sequences[start_idx..end_idx];

        let mut input_ids = Vec::new();
        let mut attention_mask = Vec::new();
        let mut all_masked_ids = Vec::new();
        let mut all_masked_positions = Vec::new();

        for seq in batch_seqs {
            let tokens = self.tokenize(seq);

            // Attention mask: 1.0 for real tokens, 0.0 for padding
            let mask: Vec<f32> = tokens
                .iter()
                .map(|&t| if t != DnaVocabulary::PAD { 1.0 } else { 0.0 })
                .collect();

            let (masked_tokens, masked_ids, masked_positions) = self.create_mlm_batch(&tokens);

            input_ids.push(masked_tokens);
            attention_mask.push(mask);
            all_masked_ids.push(masked_ids);
            all_masked_positions.push(masked_positions);
        }

        Some(MLMBatch {
            input_ids,
            attention_mask,
            masked_ids: all_masked_ids,
            masked_positions: all_masked_positions,
        })
    }

    pub fn num_batches(&self) -> usize {
        (self.sequences.len() + self.batch_size - 1) / self.batch_size
    }

    #[allow(dead_code)]
    pub fn vocab_size(&self) -> usize {
        self.vocab.vocab_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vocab_encode_nucleotides() {
        let vocab = DnaVocabulary::new();
        assert_eq!(vocab.encode_nucleotide('A'), DnaVocabulary::A);
        assert_eq!(vocab.encode_nucleotide('T'), DnaVocabulary::T);
        assert_eq!(vocab.encode_nucleotide('G'), DnaVocabulary::G);
        assert_eq!(vocab.encode_nucleotide('C'), DnaVocabulary::C);
        assert_eq!(vocab.encode_nucleotide('X'), DnaVocabulary::UNK);
        // Case insensitive
        assert_eq!(vocab.encode_nucleotide('a'), DnaVocabulary::A);
    }

    #[test]
    fn test_vocab_decode_roundtrip() {
        let vocab = DnaVocabulary::new();
        for ch in ['A', 'T', 'G', 'C'] {
            let id = vocab.encode_nucleotide(ch);
            assert_eq!(vocab.decode(id), ch);
        }
    }

    #[test]
    fn test_dataset_creates_batches() {
        let ds = TextDataset::new(4, 32);
        assert!(ds.num_batches() > 0);
        let batch = ds.get_batch(0).expect("first batch should exist");
        assert_eq!(batch.input_ids.len(), 4);
        assert_eq!(batch.input_ids[0].len(), 32);
    }

    #[test]
    fn test_batch_has_cls_sep() {
        let ds = TextDataset::new(2, 64);
        let batch = ds.get_batch(0).unwrap();
        for seq in &batch.input_ids {
            assert_eq!(seq[0], DnaVocabulary::CLS, "First token should be CLS");
            // SEP should appear somewhere before padding
            assert!(seq.contains(&DnaVocabulary::SEP), "Should contain SEP");
        }
    }

    #[test]
    fn test_batch_has_masks() {
        let ds = TextDataset::new(4, 64);
        let batch = ds.get_batch(0).unwrap();
        // At least some positions should be masked
        let total_masks: usize = batch.masked_positions.iter().map(|p| p.len()).sum();
        assert!(total_masks > 0, "Should have masked positions");
        // masked_ids should match masked_positions count
        let total_ids: usize = batch.masked_ids.iter().map(|p| p.len()).sum();
        assert_eq!(total_masks, total_ids);
    }

    #[test]
    fn test_masked_ids_are_nucleotides() {
        let ds = TextDataset::new(4, 64);
        let batch = ds.get_batch(0).unwrap();
        for ids in &batch.masked_ids {
            for &id in ids {
                assert!(
                    id >= DnaVocabulary::A && id <= DnaVocabulary::C,
                    "Masked IDs should be nucleotide tokens, got {}",
                    id
                );
            }
        }
    }

    #[test]
    fn test_out_of_range_batch_returns_none() {
        let ds = TextDataset::new(4, 32);
        assert!(ds.get_batch(9999).is_none());
    }
}
