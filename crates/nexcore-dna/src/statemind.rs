//! StateMind — Three-Dimensional Word Space with Auto-Mining Simulations.
//!
//! Projects every mined word into a 3D space where each axis captures a
//! different mathematical property:
//!
//! - **X (entropy)**: Shannon entropy normalized to [0, 1] — information density
//! - **Y (gc_content)**: GC fraction of the DNA encoding — compositional bias
//! - **Z (density)**: Lexical density (unique bytes / total bytes) — byte diversity
//!
//! From this projection, StateMind enables:
//!
//! - 3D nearest-neighbor queries
//! - Mutation simulations that track drift through the space
//! - Auto-mining: generate word variants from seeds, populating the space
//! - Per-axis statistics (min, max, mean, std_dev)
//!
//! All algorithms are deterministic. Zero external dependencies.

use crate::lexicon::{self, Lexicon, WordOre};
use std::fmt;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A word's projection into 3D mind-space.
///
/// Tier: T2-P (N Quantity + λ Location)
/// Dominant: λ Location (positional context in 3D space)
#[derive(Debug, Clone)]
pub struct MindPoint {
    /// X: Shannon entropy normalized by max (8.0 bits). Range [0, 1].
    pub entropy_norm: f64,
    /// Y: GC content of the DNA encoding. Range [0, 1].
    pub gc_content: f64,
    /// Z: Lexical density (unique bytes / total bytes). Range [0, 1].
    pub density: f64,
}

/// Change vector between two MindPoints.
///
/// Tier: T2-P (κ Comparison + N Quantity)
/// Dominant: κ Comparison (measures directional change)
#[derive(Debug, Clone)]
pub struct Drift {
    /// Change in each axis: [dx, dy, dz].
    pub delta: [f64; 3],
    /// Euclidean magnitude: sqrt(dx² + dy² + dz²).
    pub magnitude: f64,
}

/// Result of a single mutation simulation step.
pub struct MutationResult {
    /// Original word.
    pub original: String,
    /// Mutated word.
    pub mutant: String,
    /// Original projection.
    pub original_point: MindPoint,
    /// Mutant projection.
    pub mutant_point: MindPoint,
    /// Drift between original and mutant.
    pub drift: Drift,
}

/// Per-axis statistics.
#[derive(Debug, Clone)]
pub struct DimStats {
    /// Minimum value on this axis.
    pub min: f64,
    /// Maximum value on this axis.
    pub max: f64,
    /// Arithmetic mean.
    pub mean: f64,
    /// Standard deviation.
    pub std_dev: f64,
}

/// Three-dimensional word-space with auto-mining and simulation.
///
/// Tier: T3 (ς + σ + μ + κ + N + λ + ∂ + ∃ + π)
/// Dominant: ς State (the mind's state evolves as words are ingested)
pub struct StateMind {
    lexicon: Lexicon,
    points: Vec<MindPoint>,
}

// ---------------------------------------------------------------------------
// Helper: lexical density
// ---------------------------------------------------------------------------

/// Lexical density: fraction of unique bytes in the word. Range [0, 1].
///
/// Returns 0.0 for empty strings.
#[must_use]
pub fn lexical_density(word: &str) -> f64 {
    if word.is_empty() {
        return 0.0;
    }
    let bytes = word.as_bytes();
    let mut seen = [false; 256];
    let mut unique = 0usize;
    for &b in bytes {
        if !seen[b as usize] {
            seen[b as usize] = true;
            unique += 1;
        }
    }
    unique as f64 / bytes.len() as f64
}

/// Mutate a single byte in a word. Returns None if position is out of bounds
/// or the replacement produces invalid UTF-8.
#[must_use]
pub fn mutate_word(word: &str, position: usize, replacement: u8) -> Option<String> {
    if position >= word.len() {
        return None;
    }
    let mut bytes = word.as_bytes().to_vec();
    bytes[position] = replacement;
    String::from_utf8(bytes).ok()
}

// ---------------------------------------------------------------------------
// MindPoint
// ---------------------------------------------------------------------------

impl MindPoint {
    /// Project a WordOre into 3D mind-space.
    #[must_use]
    pub fn from_ore(ore: &WordOre) -> Self {
        let entropy_norm = if ore.entropy > 0.0 {
            (ore.entropy / 8.0).min(1.0)
        } else {
            0.0
        };
        let density = lexical_density(&ore.word);

        Self {
            entropy_norm,
            gc_content: ore.gc_content,
            density,
        }
    }

    /// Euclidean distance to another point.
    #[must_use]
    pub fn distance(&self, other: &MindPoint) -> f64 {
        let dx = self.entropy_norm - other.entropy_norm;
        let dy = self.gc_content - other.gc_content;
        let dz = self.density - other.density;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// The origin point (0, 0, 0).
    #[must_use]
    pub fn origin() -> Self {
        Self {
            entropy_norm: 0.0,
            gc_content: 0.0,
            density: 0.0,
        }
    }

    /// Coordinates as an array.
    #[must_use]
    pub fn as_array(&self) -> [f64; 3] {
        [self.entropy_norm, self.gc_content, self.density]
    }
}

impl fmt::Display for MindPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({:.3}, {:.3}, {:.3})",
            self.entropy_norm, self.gc_content, self.density
        )
    }
}

// ---------------------------------------------------------------------------
// Drift
// ---------------------------------------------------------------------------

impl Drift {
    /// Compute drift vector between two points: from `a` to `b`.
    #[must_use]
    pub fn between(a: &MindPoint, b: &MindPoint) -> Self {
        let dx = b.entropy_norm - a.entropy_norm;
        let dy = b.gc_content - a.gc_content;
        let dz = b.density - a.density;
        let magnitude = (dx * dx + dy * dy + dz * dz).sqrt();
        Self {
            delta: [dx, dy, dz],
            magnitude,
        }
    }

    /// Zero drift (no change).
    #[must_use]
    pub fn zero() -> Self {
        Self {
            delta: [0.0, 0.0, 0.0],
            magnitude: 0.0,
        }
    }
}

impl fmt::Display for Drift {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Δ=[{:.4}, {:.4}, {:.4}] |{:.4}|",
            self.delta[0], self.delta[1], self.delta[2], self.magnitude
        )
    }
}

// ---------------------------------------------------------------------------
// DimStats
// ---------------------------------------------------------------------------

impl DimStats {
    /// Compute statistics over a slice of values.
    #[must_use]
    fn compute(values: &[f64]) -> Self {
        if values.is_empty() {
            return Self {
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                std_dev: 0.0,
            };
        }

        let n = values.len() as f64;
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut sum = 0.0_f64;

        for &v in values {
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
            sum += v;
        }

        let mean = sum / n;
        let variance = values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        Self {
            min,
            max,
            mean,
            std_dev,
        }
    }
}

impl fmt::Display for DimStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:.3}..{:.3}] μ={:.3} σ={:.3}",
            self.min, self.max, self.mean, self.std_dev
        )
    }
}

// ---------------------------------------------------------------------------
// StateMind
// ---------------------------------------------------------------------------

impl StateMind {
    /// Create an empty mind.
    #[must_use]
    pub fn new() -> Self {
        Self {
            lexicon: Lexicon::new(),
            points: Vec::new(),
        }
    }

    /// Ingest a single word: mine it, project to 3D, add to lexicon.
    /// Skips duplicates.
    pub fn ingest(&mut self, word: &str) {
        let before = self.lexicon.len();
        self.lexicon.insert(word);
        if self.lexicon.len() > before {
            let ore = self.lexicon.get(word);
            if let Some(o) = ore {
                self.points.push(MindPoint::from_ore(o));
            }
        }
    }

    /// Ingest all words from a text corpus (splits on whitespace).
    /// Returns the number of new unique words added.
    pub fn ingest_corpus(&mut self, text: &str) -> usize {
        let before = self.lexicon.len();
        for word in text.split_whitespace() {
            // Strip basic punctuation from edges
            let cleaned: &str = word.trim_matches(|c: char| c.is_ascii_punctuation());
            if !cleaned.is_empty() {
                self.ingest(cleaned);
            }
        }
        self.lexicon.len() - before
    }

    /// Number of words in the mind.
    #[must_use]
    pub fn len(&self) -> usize {
        self.lexicon.len()
    }

    /// Whether the mind is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lexicon.is_empty()
    }

    /// Get the 3D projection for a word.
    #[must_use]
    pub fn project(&self, word: &str) -> Option<&MindPoint> {
        let idx = self.lexicon.entries().iter().position(|e| e.word == word)?;
        self.points.get(idx)
    }

    /// Find k nearest neighbors in 3D space to a given point.
    ///
    /// Returns (word ore, mind point, distance) tuples sorted by distance.
    #[must_use]
    pub fn nearest_3d(&self, point: &MindPoint, k: usize) -> Vec<(&WordOre, &MindPoint, f64)> {
        let mut scored: Vec<(&WordOre, &MindPoint, f64)> = self
            .lexicon
            .entries()
            .iter()
            .zip(self.points.iter())
            .map(|(ore, mp)| (ore, mp, point.distance(mp)))
            .collect();

        scored.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }

    /// Centroid (center of mass) of all projected points.
    #[must_use]
    pub fn centroid(&self) -> MindPoint {
        if self.points.is_empty() {
            return MindPoint::origin();
        }

        let n = self.points.len() as f64;
        let sum_x: f64 = self.points.iter().map(|p| p.entropy_norm).sum();
        let sum_y: f64 = self.points.iter().map(|p| p.gc_content).sum();
        let sum_z: f64 = self.points.iter().map(|p| p.density).sum();

        MindPoint {
            entropy_norm: sum_x / n,
            gc_content: sum_y / n,
            density: sum_z / n,
        }
    }

    /// Maximum distance from centroid to any point (bounding radius).
    #[must_use]
    pub fn radius(&self) -> f64 {
        let c = self.centroid();
        self.points
            .iter()
            .map(|p| c.distance(p))
            .fold(0.0_f64, f64::max)
    }

    /// Per-axis statistics: [entropy, gc_content, density].
    #[must_use]
    pub fn dimension_stats(&self) -> [DimStats; 3] {
        let xs: Vec<f64> = self.points.iter().map(|p| p.entropy_norm).collect();
        let ys: Vec<f64> = self.points.iter().map(|p| p.gc_content).collect();
        let zs: Vec<f64> = self.points.iter().map(|p| p.density).collect();

        [
            DimStats::compute(&xs),
            DimStats::compute(&ys),
            DimStats::compute(&zs),
        ]
    }

    /// Simulate mutations on a word and track drift through 3D space.
    ///
    /// Generates deterministic mutations: for each byte position, substitutes
    /// lowercase letters a-z (skipping identity). Stops after `count` results.
    #[must_use]
    pub fn simulate_mutation(&self, word: &str, count: usize) -> Vec<MutationResult> {
        if word.is_empty() || count == 0 {
            return Vec::new();
        }

        let ore = lexicon::mine(word);
        let original_point = MindPoint::from_ore(&ore);
        let bytes = word.as_bytes();
        let mut results = Vec::with_capacity(count);

        'outer: for (pos, &original_byte) in bytes.iter().enumerate() {
            for ch in b'a'..=b'z' {
                if ch == original_byte {
                    continue;
                }
                if let Some(mutant_word) = mutate_word(word, pos, ch) {
                    let mutant_ore = lexicon::mine(&mutant_word);
                    let mutant_point = MindPoint::from_ore(&mutant_ore);
                    let drift = Drift::between(&original_point, &mutant_point);

                    results.push(MutationResult {
                        original: word.to_string(),
                        mutant: mutant_word,
                        original_point: original_point.clone(),
                        mutant_point,
                        drift,
                    });

                    if results.len() >= count {
                        break 'outer;
                    }
                }
            }
        }

        results
    }

    /// Auto-mine: from seed words, generate single-character mutations and
    /// mine all unique results into the mind.
    ///
    /// Returns the number of new words added (including seeds).
    pub fn auto_mine(&mut self, seeds: &[&str]) -> usize {
        let before = self.lexicon.len();

        // Ingest seeds first
        for seed in seeds {
            self.ingest(seed);
        }

        // Generate mutations of each seed
        let seed_copies: Vec<String> = seeds.iter().map(|s| s.to_string()).collect();
        for seed in &seed_copies {
            let bytes = seed.as_bytes();
            for (pos, &original_byte) in bytes.iter().enumerate() {
                for ch in b'a'..=b'z' {
                    if ch == original_byte {
                        continue;
                    }
                    if let Some(mutant) = mutate_word(seed, pos, ch) {
                        self.ingest(&mutant);
                    }
                }
            }
        }

        self.lexicon.len() - before
    }

    /// Access the underlying lexicon.
    #[must_use]
    pub fn lexicon(&self) -> &Lexicon {
        &self.lexicon
    }

    /// Access all projected points.
    #[must_use]
    pub fn points(&self) -> &[MindPoint] {
        &self.points
    }
}

impl Default for StateMind {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for StateMind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "StateMind(empty)")
        } else {
            let c = self.centroid();
            let r = self.radius();
            write!(
                f,
                "StateMind({} words, centroid={}, radius={:.4})",
                self.len(),
                c,
                r
            )
        }
    }
}

impl fmt::Display for MutationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\"{}\" -> \"{}\" {} -> {} {}",
            self.original, self.mutant, self.original_point, self.mutant_point, self.drift
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // MindPoint tests
    // -----------------------------------------------------------------------

    #[test]
    fn mindpoint_from_ore_basic() {
        let ore = lexicon::mine("hello");
        let pt = MindPoint::from_ore(&ore);
        // Entropy should be > 0 (not all same bytes)
        assert!(pt.entropy_norm > 0.0);
        assert!(pt.entropy_norm <= 1.0);
        // GC content in range
        assert!(pt.gc_content >= 0.0 && pt.gc_content <= 1.0);
        // Density: "hello" has 4 unique bytes out of 5
        assert!((pt.density - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn mindpoint_from_ore_empty() {
        let ore = lexicon::mine("");
        let pt = MindPoint::from_ore(&ore);
        assert!((pt.entropy_norm - 0.0).abs() < f64::EPSILON);
        assert!((pt.density - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mindpoint_distance_zero() {
        let ore = lexicon::mine("test");
        let pt = MindPoint::from_ore(&ore);
        assert!((pt.distance(&pt) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mindpoint_distance_known() {
        let a = MindPoint {
            entropy_norm: 0.0,
            gc_content: 0.0,
            density: 0.0,
        };
        let b = MindPoint {
            entropy_norm: 1.0,
            gc_content: 0.0,
            density: 0.0,
        };
        assert!((a.distance(&b) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mindpoint_entropy_normalized_range() {
        // "abcdefghijklmnop" — 16 unique bytes → H = 4.0, norm = 0.5
        let ore = lexicon::mine("abcdefghijklmnop");
        let pt = MindPoint::from_ore(&ore);
        assert!(pt.entropy_norm > 0.0 && pt.entropy_norm <= 1.0);
    }

    #[test]
    fn mindpoint_display() {
        let pt = MindPoint {
            entropy_norm: 0.5,
            gc_content: 0.3,
            density: 0.8,
        };
        let s = format!("{pt}");
        assert!(s.contains("0.500"));
        assert!(s.contains("0.300"));
        assert!(s.contains("0.800"));
    }

    // -----------------------------------------------------------------------
    // Drift tests
    // -----------------------------------------------------------------------

    #[test]
    fn drift_between_same_points() {
        let pt = MindPoint {
            entropy_norm: 0.5,
            gc_content: 0.5,
            density: 0.5,
        };
        let drift = Drift::between(&pt, &pt);
        assert!((drift.magnitude - 0.0).abs() < f64::EPSILON);
        for d in &drift.delta {
            assert!((d - 0.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn drift_between_different_points() {
        let a = MindPoint::origin();
        let b = MindPoint {
            entropy_norm: 3.0,
            gc_content: 4.0,
            density: 0.0,
        };
        let drift = Drift::between(&a, &b);
        // 3-4-5 triangle → magnitude = 5.0
        assert!((drift.magnitude - 5.0).abs() < 1e-10);
    }

    #[test]
    fn drift_magnitude_is_euclidean() {
        let a = MindPoint {
            entropy_norm: 1.0,
            gc_content: 2.0,
            density: 3.0,
        };
        let b = MindPoint {
            entropy_norm: 4.0,
            gc_content: 6.0,
            density: 3.0,
        };
        let drift = Drift::between(&a, &b);
        // dx=3, dy=4, dz=0 → magnitude=5
        assert!((drift.magnitude - 5.0).abs() < 1e-10);
        assert!((drift.delta[0] - 3.0).abs() < f64::EPSILON);
        assert!((drift.delta[1] - 4.0).abs() < f64::EPSILON);
        assert!((drift.delta[2] - 0.0).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // mutate_word tests
    // -----------------------------------------------------------------------

    #[test]
    fn mutate_word_basic() {
        let result = mutate_word("cat", 0, b'b');
        assert_eq!(result, Some("bat".to_string()));
    }

    #[test]
    fn mutate_word_out_of_bounds() {
        let result = mutate_word("cat", 10, b'x');
        assert!(result.is_none());
    }

    #[test]
    fn mutate_word_identity() {
        let result = mutate_word("cat", 0, b'c');
        assert_eq!(result, Some("cat".to_string()));
    }

    #[test]
    fn mutate_word_preserves_valid_utf8() {
        let result = mutate_word("hello", 2, b'x');
        assert!(result.is_some());
        if let Some(s) = result {
            assert_eq!(s, "hexlo");
        }
    }

    // -----------------------------------------------------------------------
    // lexical_density tests
    // -----------------------------------------------------------------------

    #[test]
    fn lexical_density_empty() {
        assert!((lexical_density("") - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn lexical_density_all_unique() {
        // "abcd" → 4 unique / 4 total = 1.0
        assert!((lexical_density("abcd") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn lexical_density_all_same() {
        // "aaaa" → 1 unique / 4 total = 0.25
        assert!((lexical_density("aaaa") - 0.25).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // StateMind core tests
    // -----------------------------------------------------------------------

    #[test]
    fn statemind_new_empty() {
        let mind = StateMind::new();
        assert!(mind.is_empty());
        assert_eq!(mind.len(), 0);
    }

    #[test]
    fn statemind_default() {
        let mind = StateMind::default();
        assert!(mind.is_empty());
    }

    #[test]
    fn statemind_ingest_word() {
        let mut mind = StateMind::new();
        mind.ingest("hello");
        assert_eq!(mind.len(), 1);
        assert!(mind.project("hello").is_some());
    }

    #[test]
    fn statemind_ingest_dedup() {
        let mut mind = StateMind::new();
        mind.ingest("hello");
        mind.ingest("hello");
        assert_eq!(mind.len(), 1);
        assert_eq!(mind.points().len(), 1);
    }

    #[test]
    fn statemind_ingest_corpus() {
        let mut mind = StateMind::new();
        let added = mind.ingest_corpus("the quick brown fox jumps over the lazy dog");
        // "the" appears twice but should be deduped
        assert_eq!(added, 8); // the, quick, brown, fox, jumps, over, lazy, dog
        assert_eq!(mind.len(), 8);
    }

    #[test]
    fn statemind_ingest_corpus_strips_punctuation() {
        let mut mind = StateMind::new();
        mind.ingest_corpus("hello, world!");
        assert!(mind.project("hello").is_some());
        assert!(mind.project("world").is_some());
    }

    #[test]
    fn statemind_project_lookup() {
        let mut mind = StateMind::new();
        mind.ingest("rust");
        let pt = mind.project("rust");
        assert!(pt.is_some());
        if let Some(p) = pt {
            assert!(p.entropy_norm >= 0.0 && p.entropy_norm <= 1.0);
        }
    }

    #[test]
    fn statemind_project_missing() {
        let mind = StateMind::new();
        assert!(mind.project("nothing").is_none());
    }

    #[test]
    fn statemind_centroid_single() {
        let mut mind = StateMind::new();
        mind.ingest("test");
        let c = mind.centroid();
        let p = mind.project("test");
        if let Some(pt) = p {
            assert!((c.entropy_norm - pt.entropy_norm).abs() < f64::EPSILON);
            assert!((c.gc_content - pt.gc_content).abs() < f64::EPSILON);
            assert!((c.density - pt.density).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn statemind_centroid_empty() {
        let mind = StateMind::new();
        let c = mind.centroid();
        assert!((c.entropy_norm - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn statemind_radius() {
        let mut mind = StateMind::new();
        mind.ingest("a");
        mind.ingest("abcdefghij");
        let r = mind.radius();
        assert!(r > 0.0);
    }

    #[test]
    fn statemind_radius_empty() {
        let mind = StateMind::new();
        assert!((mind.radius() - 0.0).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // StateMind 3D query tests
    // -----------------------------------------------------------------------

    #[test]
    fn statemind_nearest_3d_basic() {
        let mut mind = StateMind::new();
        mind.ingest("cat");
        mind.ingest("bat");
        mind.ingest("elephant");

        // Query near "cat"'s projection
        let cat_pt = mind.project("cat");
        if let Some(pt) = cat_pt {
            let neighbors = mind.nearest_3d(pt, 2);
            assert_eq!(neighbors.len(), 2);
            // First should be cat itself (distance 0)
            assert_eq!(neighbors[0].0.word, "cat");
            assert!((neighbors[0].2 - 0.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn statemind_nearest_3d_k_exceeds() {
        let mut mind = StateMind::new();
        mind.ingest("one");
        let neighbors = mind.nearest_3d(&MindPoint::origin(), 10);
        assert_eq!(neighbors.len(), 1);
    }

    #[test]
    fn statemind_dimension_stats_basic() {
        let mut mind = StateMind::new();
        mind.ingest("aaaa"); // low entropy, low density
        mind.ingest("abcd"); // higher entropy, high density

        let stats = mind.dimension_stats();
        // Entropy axis
        assert!(stats[0].min <= stats[0].max);
        assert!(stats[0].std_dev >= 0.0);
        // GC axis
        assert!(stats[1].min >= 0.0 && stats[1].max <= 1.0);
        // Density axis
        assert!(stats[2].min >= 0.0 && stats[2].max <= 1.0);
    }

    #[test]
    fn statemind_dimension_stats_empty() {
        let mind = StateMind::new();
        let stats = mind.dimension_stats();
        for s in &stats {
            assert!((s.mean - 0.0).abs() < f64::EPSILON);
        }
    }

    // -----------------------------------------------------------------------
    // StateMind simulation tests
    // -----------------------------------------------------------------------

    #[test]
    fn simulate_mutation_basic() {
        let mind = StateMind::new();
        let results = mind.simulate_mutation("cat", 5);
        assert_eq!(results.len(), 5);
        // All mutations should differ from original
        for r in &results {
            assert_eq!(r.original, "cat");
            assert_ne!(r.mutant, "cat");
        }
    }

    #[test]
    fn simulate_mutation_tracks_drift() {
        let mind = StateMind::new();
        let results = mind.simulate_mutation("hello", 3);
        for r in &results {
            // Drift magnitude should be >= 0
            assert!(r.drift.magnitude >= 0.0);
        }
    }

    #[test]
    fn simulate_mutation_empty_word() {
        let mind = StateMind::new();
        let results = mind.simulate_mutation("", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn simulate_mutation_zero_count() {
        let mind = StateMind::new();
        let results = mind.simulate_mutation("test", 0);
        assert!(results.is_empty());
    }

    #[test]
    fn auto_mine_generates_mutations() {
        let mut mind = StateMind::new();
        let added = mind.auto_mine(&["go"]);
        // "go" = 2 positions × 25 mutations each + 1 seed = 51
        // But some mutations may collide, so at least seed + some
        assert!(added >= 1);
        assert!(mind.len() > 1);
    }

    #[test]
    fn auto_mine_adds_to_lexicon() {
        let mut mind = StateMind::new();
        mind.auto_mine(&["ab"]);
        // Should have "ab" plus many 2-letter variants
        assert!(mind.lexicon().get("ab").is_some());
        assert!(mind.len() > 10); // at least some mutations added
    }

    #[test]
    fn auto_mine_multiple_seeds() {
        let mut mind = StateMind::new();
        let added = mind.auto_mine(&["a", "b"]);
        assert!(added >= 2); // at least the seeds
    }

    // -----------------------------------------------------------------------
    // Display tests
    // -----------------------------------------------------------------------

    #[test]
    fn display_statemind_empty() {
        let mind = StateMind::new();
        let s = format!("{mind}");
        assert!(s.contains("empty"));
    }

    #[test]
    fn display_statemind_with_entries() {
        let mut mind = StateMind::new();
        mind.ingest("rust");
        mind.ingest("java");
        let s = format!("{mind}");
        assert!(s.contains("2 words"));
        assert!(s.contains("centroid"));
    }

    #[test]
    fn display_drift() {
        let drift = Drift::zero();
        let s = format!("{drift}");
        assert!(s.contains("0.0000"));
    }

    #[test]
    fn display_mutation_result() {
        let mind = StateMind::new();
        let results = mind.simulate_mutation("cat", 1);
        if let Some(r) = results.first() {
            let s = format!("{r}");
            assert!(s.contains("cat"));
        }
    }

    #[test]
    fn display_dimstats() {
        let stats = DimStats::compute(&[1.0, 2.0, 3.0]);
        let s = format!("{stats}");
        assert!(s.contains("1.000"));
        assert!(s.contains("3.000"));
    }
}
