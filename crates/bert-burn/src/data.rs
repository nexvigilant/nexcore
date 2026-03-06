/// MLM Batch with masked tokens and labels
#[derive(Debug, Clone)]
pub struct MLMBatch {
    pub input_ids: Vec<Vec<i64>>,
    /// Padding mask: 1.0 for real tokens, 0.0 for [PAD].
    /// Converted to `Tensor<B, 2, Bool>` in runner.rs and wired into
    /// `MhaInput::mask_pad` so attention ignores padding positions.
    pub attention_mask: Vec<Vec<f32>>,
    pub masked_ids: Vec<Vec<i64>>,
    pub masked_positions: Vec<Vec<i64>>,
}

/// DNA nucleotide vocabulary: char-level tokenization
/// Special tokens: [PAD]=0, [UNK]=1, [CLS]=2, [SEP]=3, [MASK]=4
/// Nucleotides: A=5, T=6, G=7, C=8
pub struct DnaVocabulary {
    #[allow(
        dead_code,
        reason = "held for runtime introspection; constants are used directly"
    )]
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

    #[allow(dead_code, reason = "used in tests and future CLI decode display")]
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

/// Biological sequence categories for structured training data
#[derive(Debug, Clone, Copy)]
enum SeqClass {
    /// ATG → in-frame codons → stop codon (TAA/TAG/TGA)
    CodingRegion,
    /// TATA box + downstream, GC box patterns
    Promoter,
    /// High GC content (>70%), frequent CpG dinucleotides
    CpgIsland,
    /// AT-rich intergenic filler
    Intergenic,
    /// Short tandem repeats (microsatellites)
    Microsatellite,
}

/// Generate biologically structured DNA sequences with controlled randomness.
///
/// Uses a seeded RNG derived from `seed` so training data is deterministic
/// across runs but diverse across seeds.
fn generate_sequences(count: usize, min_len: usize, seed: u64) -> Vec<String> {
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    let mut rng = SmallRng::seed_from_u64(seed);
    let mut sequences = Vec::with_capacity(count);

    // Human codon usage table (top 20 codons by frequency, simplified)
    // Each codon is weighted by approximate human codon usage bias
    let codons_weighted: &[(&str, u32)] = &[
        ("GCT", 18),
        ("GCC", 28),
        ("GCA", 16),
        ("GCG", 7), // Ala
        ("TGT", 10),
        ("TGC", 12), // Cys
        ("GAT", 22),
        ("GAC", 25), // Asp
        ("GAA", 29),
        ("GAG", 40), // Glu
        ("TTT", 17),
        ("TTC", 20), // Phe
        ("GGT", 11),
        ("GGC", 22),
        ("GGA", 16),
        ("GGG", 16), // Gly
        ("CAT", 11),
        ("CAC", 15), // His
        ("ATT", 16),
        ("ATC", 21),
        ("ATA", 7), // Ile
        ("AAA", 24),
        ("AAG", 32), // Lys
        ("CTT", 13),
        ("CTC", 20),
        ("CTA", 7),
        ("CTG", 40), // Leu
        ("TTA", 8),
        ("TTG", 13), // Leu
        ("ATG", 22), // Met (start)
        ("AAT", 17),
        ("AAC", 19), // Asn
        ("CCT", 18),
        ("CCC", 20),
        ("CCA", 17),
        ("CCG", 7), // Pro
        ("CAA", 12),
        ("CAG", 34), // Gln
        ("CGT", 5),
        ("CGC", 10),
        ("CGA", 6),
        ("CGG", 11), // Arg
        ("AGA", 12),
        ("AGG", 12), // Arg
        ("TCT", 15),
        ("TCC", 18),
        ("TCA", 12),
        ("TCG", 4), // Ser
        ("AGT", 12),
        ("AGC", 19), // Ser
        ("ACT", 13),
        ("ACC", 19),
        ("ACA", 15),
        ("ACG", 6), // Thr
        ("GTT", 11),
        ("GTC", 14),
        ("GTA", 7),
        ("GTG", 28), // Val
        ("TGG", 13), // Trp
        ("TAT", 12),
        ("TAC", 15), // Tyr
    ];
    let codon_total: u32 = codons_weighted.iter().map(|(_, w)| w).sum();

    let stop_codons = ["TAA", "TAG", "TGA"];

    let pick_codon = |rng: &mut SmallRng| -> &str {
        let mut r = rng.gen_range(0..codon_total);
        for &(codon, weight) in codons_weighted {
            if r < weight {
                return codon;
            }
            r -= weight;
        }
        "GCT" // fallback
    };

    let random_nuc = |rng: &mut SmallRng| -> char {
        match rng.gen_range(0..4u8) {
            0 => 'A',
            1 => 'T',
            2 => 'G',
            _ => 'C',
        }
    };

    // GC-biased nucleotide (70% GC for CpG islands)
    let gc_biased_nuc = |rng: &mut SmallRng| -> char {
        if rng.r#gen::<f32>() < 0.7 {
            if rng.r#gen::<bool>() { 'G' } else { 'C' }
        } else {
            if rng.r#gen::<bool>() { 'A' } else { 'T' }
        }
    };

    // AT-biased nucleotide (70% AT for intergenic)
    let at_biased_nuc = |rng: &mut SmallRng| -> char {
        if rng.r#gen::<f32>() < 0.7 {
            if rng.r#gen::<bool>() { 'A' } else { 'T' }
        } else {
            if rng.r#gen::<bool>() { 'G' } else { 'C' }
        }
    };

    // Microsatellite repeat units (2-6bp)
    let repeat_units: &[&str] = &[
        "CA", "GT", "AT", "GC", // dinucleotide
        "CAG", "CTG", "CGG", "GAA", // trinucleotide (disease-associated)
        "AATG", "GATA", "TAGA", // tetranucleotide
    ];

    // Class distribution: 40% coding, 15% promoter, 15% CpG, 15% intergenic, 15% microsatellite
    let classes = [
        (SeqClass::CodingRegion, 40),
        (SeqClass::Promoter, 15),
        (SeqClass::CpgIsland, 15),
        (SeqClass::Intergenic, 15),
        (SeqClass::Microsatellite, 15),
    ];
    let class_total: u32 = classes.iter().map(|(_, w)| w).sum();

    for _ in 0..count {
        let target_len = min_len + rng.gen_range(0..min_len / 2);

        // Pick class
        let mut r = rng.gen_range(0..class_total);
        let mut class = SeqClass::CodingRegion;
        for &(c, w) in &classes {
            if r < w {
                class = c;
                break;
            }
            r -= w;
        }

        let mut seq = String::with_capacity(target_len + 10);

        match class {
            SeqClass::CodingRegion => {
                // Kozak-like context + ATG + codons + stop
                // Simplified Kozak: GCCACC before ATG
                if rng.r#gen::<f32>() < 0.6 {
                    seq.push_str("GCCACC");
                }
                seq.push_str("ATG");
                while seq.len() + 3 < target_len {
                    seq.push_str(pick_codon(&mut rng));
                }
                seq.push_str(stop_codons[rng.gen_range(0..3)]);
            }
            SeqClass::Promoter => {
                // 5' flanking → TATA box → spacer → initiator
                for _ in 0..(target_len / 4) {
                    seq.push(gc_biased_nuc(&mut rng));
                }
                // TATA box (TATAAA with common variants)
                let tata_variants = ["TATAAA", "TATAAAA", "TATATAA", "TATAAAT"];
                seq.push_str(tata_variants[rng.gen_range(0..tata_variants.len())]);
                // 25-30bp spacer to transcription start
                for _ in 0..rng
                    .gen_range(20..30)
                    .min(target_len.saturating_sub(seq.len()))
                {
                    seq.push(at_biased_nuc(&mut rng));
                }
                // Initiator element (PyPyAN(T/A)PyPy pattern, simplified)
                if seq.len() + 6 <= target_len {
                    seq.push_str("CCAATC");
                }
                while seq.len() < target_len {
                    seq.push(random_nuc(&mut rng));
                }
            }
            SeqClass::CpgIsland => {
                // High CG content, frequent CpG dinucleotides
                // Real CpG islands: >200bp, >50% GC, observed/expected CpG >0.6
                while seq.len() < target_len {
                    if rng.r#gen::<f32>() < 0.3 {
                        seq.push('C');
                        seq.push('G');
                    } else {
                        seq.push(gc_biased_nuc(&mut rng));
                    }
                }
            }
            SeqClass::Intergenic => {
                // AT-rich with occasional simple repeats
                while seq.len() < target_len {
                    if rng.r#gen::<f32>() < 0.15 {
                        // Short poly-A/T run (3-8bp)
                        let run_len = rng.gen_range(3..=8);
                        let base = if rng.r#gen::<bool>() { 'A' } else { 'T' };
                        for _ in 0..run_len.min(target_len - seq.len()) {
                            seq.push(base);
                        }
                    } else {
                        seq.push(at_biased_nuc(&mut rng));
                    }
                }
            }
            SeqClass::Microsatellite => {
                // Tandem repeats of a short unit
                let unit = repeat_units[rng.gen_range(0..repeat_units.len())];
                // Flanking sequence
                for _ in 0..rng.gen_range(3..8) {
                    seq.push(random_nuc(&mut rng));
                }
                while seq.len() + unit.len() < target_len - 5 {
                    seq.push_str(unit);
                }
                for _ in 0..rng.gen_range(3..6) {
                    seq.push(random_nuc(&mut rng));
                }
            }
        }

        seq.truncate(target_len);
        sequences.push(seq);
    }

    sequences
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
    /// Create dataset with biologically structured sequences.
    ///
    /// Generates 500 sequences across 5 biological classes:
    /// coding regions (40%), promoters (15%), CpG islands (15%),
    /// intergenic (15%), microsatellites (15%).
    ///
    /// Coding sequences use human codon usage bias. Promoter regions
    /// include TATA box variants. CpG islands have >70% GC content.
    pub fn new(batch_size: usize, seq_length: usize) -> Self {
        let vocab = DnaVocabulary::new();
        let sequences = generate_sequences(500, seq_length.saturating_sub(4), 42);

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

    pub fn num_sequences(&self) -> usize {
        self.sequences.len()
    }

    #[allow(
        dead_code,
        reason = "introspection helper for external callers; internal code uses VOCAB_SIZE constant"
    )]
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
        // Aggregate across all batches to eliminate probabilistic flake
        // (P(zero masks in any single batch) ≈ 1.2e-8; across all batches ≈ 0)
        let mut total_masks = 0_usize;
        let mut total_ids = 0_usize;
        for batch_idx in 0..ds.num_batches() {
            if let Some(batch) = ds.get_batch(batch_idx) {
                let batch_masks: usize = batch.masked_positions.iter().map(|p| p.len()).sum();
                let batch_ids: usize = batch.masked_ids.iter().map(|p| p.len()).sum();
                assert_eq!(
                    batch_masks, batch_ids,
                    "mask count must equal id count in batch {}",
                    batch_idx
                );
                total_masks += batch_masks;
                total_ids += batch_ids;
            }
        }
        assert!(
            total_masks > 0,
            "Should have masked positions across all batches"
        );
        assert_eq!(total_masks, total_ids);
    }

    #[test]
    fn test_masked_ids_are_nucleotides() {
        // Aggregate across all batches: guarantees we actually check some IDs
        // (single-batch tests risk tautology if mask_prob yields no masks by chance,
        // even though P(zero masks in 4 seqs × 30 nucleotides) ≈ 5e-10).
        let ds = TextDataset::new(4, 64);
        let mut total_checked = 0_usize;
        for batch_idx in 0..ds.num_batches() {
            let batch = ds.get_batch(batch_idx).unwrap();
            for ids in &batch.masked_ids {
                for &id in ids {
                    assert!(
                        id >= DnaVocabulary::A && id <= DnaVocabulary::C,
                        "Masked IDs should be nucleotide tokens (5-8), got {}",
                        id
                    );
                    total_checked += 1;
                }
            }
        }
        assert!(
            total_checked > 0,
            "No masked IDs were checked — masking produced no output"
        );
    }

    #[test]
    fn test_out_of_range_batch_returns_none() {
        let ds = TextDataset::new(4, 32);
        assert!(ds.get_batch(9999).is_none());
    }
}
