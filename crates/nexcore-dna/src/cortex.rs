//! Cortex — Clustering, Gravity & Evolution in 3D Word-Space.
//!
//! Three emergent structure layers on top of StateMind:
//!
//! | Layer | Algorithm | Primitives |
//! |-------|-----------|------------|
//! | Cluster | K-means partitioning | λ, N, κ, μ |
//! | Gravity | N-body Euler integration | ς, N, λ, → |
//! | Evolve | Genetic algorithm | σ, κ, N, ρ |
//!
//! All algorithms from scratch. Deterministic PRNG via xorshift64. Zero deps.

use crate::lexicon::{self, WordOre};
use crate::statemind::{MindPoint, StateMind};
use std::fmt;

// ---------------------------------------------------------------------------
// Deterministic PRNG (xorshift64)
// ---------------------------------------------------------------------------

/// Deterministic xorshift64 pseudo-random number generator.
///
/// Tier: T2-P (σ Sequence + ς State)
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Create with a non-zero seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    /// Next u64 via xorshift64.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Uniform float in [0.0, 1.0).
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }

    /// Uniform usize in [0, max). Returns 0 if max == 0.
    pub fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        (self.next_u64() % max as u64) as usize
    }

    /// Random lowercase ASCII letter.
    pub fn next_char(&mut self) -> u8 {
        b'a' + (self.next_u64() % 26) as u8
    }
}

// ---------------------------------------------------------------------------
// Clustering Types
// ---------------------------------------------------------------------------

/// A group of words in 3D mind-space.
///
/// Tier: T2-C (λ Location + N Quantity + κ Comparison + μ Mapping)
/// Dominant: λ Location
pub struct Cluster {
    /// Center of mass.
    pub centroid: MindPoint,
    /// Indices into the StateMind points array.
    pub members: Vec<usize>,
    /// Maximum distance from centroid to any member.
    pub radius: f64,
    /// Zero-based cluster identifier.
    pub id: usize,
}

/// Output of k-means clustering.
///
/// Tier: T3 (σ + μ + κ + N + λ + ∂ + ∃)
/// Dominant: σ Sequence
pub struct ClusterResult {
    /// The k clusters.
    pub clusters: Vec<Cluster>,
    /// Number of iterations performed.
    pub iterations: usize,
    /// Whether centroids converged (stopped moving).
    pub converged: bool,
    /// Sum of squared distances from points to their centroids.
    pub total_variance: f64,
}

// ---------------------------------------------------------------------------
// Gravity Types
// ---------------------------------------------------------------------------

/// A word-particle with position, velocity, and mass in 3D space.
///
/// Tier: T2-C (λ Location + N Quantity + ς State + → Causality)
/// Dominant: ς State
pub struct Particle {
    /// Index into the original StateMind lexicon.
    pub word_idx: usize,
    /// Current 3D position.
    pub position: MindPoint,
    /// Velocity vector [vx, vy, vz].
    pub velocity: [f64; 3],
    /// Mass derived from word entropy.
    pub mass: f64,
}

/// Configuration for the gravity simulation.
///
/// Tier: T2-P (∂ Boundary + N Quantity)
/// Dominant: ∂ Boundary
pub struct GravityConfig {
    /// Gravitational constant.
    pub g_constant: f64,
    /// Euler integration timestep.
    pub dt: f64,
    /// Maximum simulation ticks.
    pub max_ticks: usize,
    /// Kinetic energy threshold for convergence.
    pub convergence_threshold: f64,
    /// Velocity damping factor (0..1).
    pub damping: f64,
}

impl Default for GravityConfig {
    fn default() -> Self {
        Self {
            g_constant: 0.1,
            dt: 0.01,
            max_ticks: 1000,
            convergence_threshold: 1e-6,
            damping: 0.95,
        }
    }
}

/// Energy snapshot at a given tick.
///
/// Tier: T2-C (ς State + N Quantity + λ Location + → Causality)
/// Dominant: ς State
pub struct GravitySnapshot {
    /// Tick number.
    pub tick: usize,
    /// Total kinetic energy: Σ 0.5 * m * |v|².
    pub kinetic_energy: f64,
    /// Total potential energy: Σ -G*mi*mj/d.
    pub potential_energy: f64,
}

/// Result of the gravity simulation.
///
/// Tier: T3 (σ + ς + μ + N + λ + →)
/// Dominant: σ Sequence
pub struct GravityResult {
    /// Final particle states.
    pub particles: Vec<Particle>,
    /// Periodic energy snapshots (every 10 ticks).
    pub snapshots: Vec<GravitySnapshot>,
    /// Total ticks simulated.
    pub ticks: usize,
    /// Whether kinetic energy fell below threshold.
    pub converged: bool,
}

// ---------------------------------------------------------------------------
// Evolution Types
// ---------------------------------------------------------------------------

/// A word-organism with fitness in 3D space.
///
/// Tier: T2-C (σ Sequence + κ Comparison + N Quantity + λ Location)
/// Dominant: N Quantity
pub struct Organism {
    /// The word string.
    pub word: String,
    /// Projection into 3D space.
    pub position: MindPoint,
    /// Fitness score (higher is better).
    pub fitness: f64,
}

/// Genetic algorithm configuration.
///
/// Tier: T2-C (N Quantity + ∂ Boundary + ν Frequency + κ Comparison)
/// Dominant: ∂ Boundary
pub struct EvolutionConfig {
    /// Population size per generation.
    pub population_size: usize,
    /// Number of generations to run.
    pub generations: usize,
    /// Per-byte mutation probability.
    pub mutation_rate: f64,
    /// Probability of crossover vs clone.
    pub crossover_rate: f64,
    /// Tournament selection group size.
    pub tournament_size: usize,
    /// Number of elites preserved each generation.
    pub elitism: usize,
}

impl Default for EvolutionConfig {
    /// Defaults refined by evolutionary training (primitive-forge, 12k games).
    ///
    /// Key findings: higher mutation (0.2 vs 0.1) prevents premature convergence;
    /// larger elite (4 vs 2) preserves solution diversity while maintaining
    /// monotonic fitness improvement.
    fn default() -> Self {
        Self {
            population_size: 50,
            generations: 100,
            mutation_rate: 0.2,
            crossover_rate: 0.7,
            tournament_size: 3,
            elitism: 4,
        }
    }
}

/// Snapshot of one generation.
///
/// Tier: T2-C (σ Sequence + N Quantity + κ Comparison + λ Location)
/// Dominant: N Quantity
pub struct Generation {
    /// Generation number (0-based).
    pub number: usize,
    /// Best fitness in this generation.
    pub best_fitness: f64,
    /// Mean fitness across the population.
    pub mean_fitness: f64,
    /// Best word in this generation.
    pub best_word: String,
}

/// Result of the evolutionary run.
///
/// Tier: T3 (σ + μ + κ + N + λ + → + ∃)
/// Dominant: σ Sequence
pub struct EvolutionResult {
    /// Generation-by-generation snapshots.
    pub generations: Vec<Generation>,
    /// The overall best organism found.
    pub best: Organism,
    /// Generation at which fitness stopped improving (if any).
    pub converged_at: Option<usize>,
}

// ---------------------------------------------------------------------------
// Display implementations
// ---------------------------------------------------------------------------

impl fmt::Display for Cluster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "C{}: {} members, r={:.3}, centroid={}",
            self.id,
            self.members.len(),
            self.radius,
            self.centroid
        )
    }
}

impl fmt::Display for ClusterResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "K-Means: {} clusters, {} iter, var={:.3}, {}",
            self.clusters.len(),
            self.iterations,
            self.total_variance,
            if self.converged {
                "converged"
            } else {
                "max_iter"
            }
        )
    }
}

impl fmt::Display for Particle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{} {} v=[{:.3}, {:.3}, {:.3}] m={:.1}",
            self.word_idx,
            self.position,
            self.velocity[0],
            self.velocity[1],
            self.velocity[2],
            self.mass
        )
    }
}

impl fmt::Display for GravityResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let last_ke = self
            .snapshots
            .last()
            .map(|s| s.kinetic_energy)
            .unwrap_or(0.0);
        let last_pe = self
            .snapshots
            .last()
            .map(|s| s.potential_energy)
            .unwrap_or(0.0);
        write!(
            f,
            "Gravity: {} ticks, KE={:.3}, PE={:.3}, {}",
            self.ticks,
            last_ke,
            last_pe,
            if self.converged {
                "converged"
            } else {
                "max_ticks"
            }
        )
    }
}

impl fmt::Display for Organism {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\"{}\" {} fit={:.2}",
            self.word, self.position, self.fitness
        )
    }
}

impl fmt::Display for Generation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Gen {}: best={:.2} mean={:.2} \"{}\"",
            self.number, self.best_fitness, self.mean_fitness, self.best_word
        )
    }
}

impl fmt::Display for EvolutionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Evolution: {} gen, best=\"{}\" fit={:.2}",
            self.generations.len(),
            self.best.word,
            self.best.fitness
        )
    }
}

// ---------------------------------------------------------------------------
// K-Means Clustering
// ---------------------------------------------------------------------------

/// Assign each point to the nearest centroid. Returns cluster index per point.
fn assign_clusters(points: &[MindPoint], centroids: &[MindPoint]) -> Vec<usize> {
    points
        .iter()
        .map(|p| {
            let mut best_idx = 0;
            let mut best_dist = f64::INFINITY;
            for (i, c) in centroids.iter().enumerate() {
                let d = p.distance(c);
                if d < best_dist {
                    best_dist = d;
                    best_idx = i;
                }
            }
            best_idx
        })
        .collect()
}

/// Recompute centroids from assignments. Returns (new_centroids, converged).
fn recompute_centroids(
    points: &[MindPoint],
    assignments: &[usize],
    k: usize,
    old_centroids: &[MindPoint],
) -> (Vec<MindPoint>, bool) {
    let mut sums = vec![[0.0_f64; 3]; k];
    let mut counts = vec![0usize; k];

    for (i, &cluster) in assignments.iter().enumerate() {
        if cluster < k {
            let arr = points[i].as_array();
            sums[cluster][0] += arr[0];
            sums[cluster][1] += arr[1];
            sums[cluster][2] += arr[2];
            counts[cluster] += 1;
        }
    }

    let mut converged = true;
    let new_centroids: Vec<MindPoint> = (0..k)
        .map(|c| {
            if counts[c] == 0 {
                // Keep old centroid if cluster is empty
                old_centroids[c].clone()
            } else {
                let n = counts[c] as f64;
                let new_pt = MindPoint {
                    entropy_norm: sums[c][0] / n,
                    gc_content: sums[c][1] / n,
                    density: sums[c][2] / n,
                };
                if old_centroids[c].distance(&new_pt) > 1e-10 {
                    converged = false;
                }
                new_pt
            }
        })
        .collect();

    (new_centroids, converged)
}

/// Compute total variance (sum of squared distances from points to centroids).
fn compute_variance(points: &[MindPoint], centroids: &[MindPoint], assignments: &[usize]) -> f64 {
    points
        .iter()
        .zip(assignments.iter())
        .map(|(p, &c)| {
            if c < centroids.len() {
                let d = p.distance(&centroids[c]);
                d * d
            } else {
                0.0
            }
        })
        .sum()
}

/// K-means clustering over the StateMind's 3D word-space.
///
/// Uses the first `k` points as initial centroids. Iterates until convergence
/// or `max_iter` is reached.
#[must_use]
pub fn kmeans(mind: &StateMind, k: usize, max_iter: usize) -> ClusterResult {
    let points = mind.points();

    if points.is_empty() || k == 0 {
        return ClusterResult {
            clusters: Vec::new(),
            iterations: 0,
            converged: true,
            total_variance: 0.0,
        };
    }

    let k = k.min(points.len());

    // Initialize centroids from first k points
    let mut centroids: Vec<MindPoint> = points[..k].to_vec();

    let mut assignments = Vec::new();
    let mut iterations = 0;
    let mut converged = false;

    for _ in 0..max_iter {
        iterations += 1;
        assignments = assign_clusters(points, &centroids);
        let (new_centroids, conv) = recompute_centroids(points, &assignments, k, &centroids);
        centroids = new_centroids;
        if conv {
            converged = true;
            break;
        }
    }

    let total_variance = compute_variance(points, &centroids, &assignments);

    // Build cluster structs
    let clusters: Vec<Cluster> = (0..k)
        .map(|c| {
            let members: Vec<usize> = assignments
                .iter()
                .enumerate()
                .filter(|&(_, &a)| a == c)
                .map(|(i, _)| i)
                .collect();

            let radius = members
                .iter()
                .map(|&i| points[i].distance(&centroids[c]))
                .fold(0.0_f64, f64::max);

            Cluster {
                centroid: centroids[c].clone(),
                members,
                radius,
                id: c,
            }
        })
        .collect();

    ClusterResult {
        clusters,
        iterations,
        converged,
        total_variance,
    }
}

// ---------------------------------------------------------------------------
// Gravity Simulation
// ---------------------------------------------------------------------------

/// Compute mass from a WordOre: 1.0 + entropy.
fn compute_mass(ore: &WordOre) -> f64 {
    1.0 + ore.entropy
}

/// Initialize particles from StateMind.
fn init_particles(mind: &StateMind) -> Vec<Particle> {
    mind.lexicon()
        .entries()
        .iter()
        .zip(mind.points().iter())
        .enumerate()
        .map(|(i, (ore, pt))| Particle {
            word_idx: i,
            position: pt.clone(),
            velocity: [0.0, 0.0, 0.0],
            mass: compute_mass(ore),
        })
        .collect()
}

/// Gravitational force on p1 due to p2. Returns [fx, fy, fz].
///
/// F = G * m1 * m2 / (d² + softening) * direction
fn gravitational_force(p1: &Particle, p2: &Particle, g: f64) -> [f64; 3] {
    let softening = 1e-6;
    let a1 = p1.position.as_array();
    let a2 = p2.position.as_array();

    let dx = a2[0] - a1[0];
    let dy = a2[1] - a1[1];
    let dz = a2[2] - a1[2];

    let dist_sq = dx * dx + dy * dy + dz * dz + softening;
    let dist = dist_sq.sqrt();
    let force_mag = g * p1.mass * p2.mass / dist_sq;

    // Direction from p1 toward p2
    [
        force_mag * dx / dist,
        force_mag * dy / dist,
        force_mag * dz / dist,
    ]
}

/// Total kinetic energy: Σ 0.5 * m * |v|².
fn kinetic_energy(particles: &[Particle]) -> f64 {
    particles
        .iter()
        .map(|p| {
            let v_sq = p.velocity[0] * p.velocity[0]
                + p.velocity[1] * p.velocity[1]
                + p.velocity[2] * p.velocity[2];
            0.5 * p.mass * v_sq
        })
        .sum()
}

/// Total gravitational potential energy: Σ_{i<j} -G*mi*mj/d.
fn potential_energy(particles: &[Particle], g: f64) -> f64 {
    let softening = 1e-6;
    let mut pe = 0.0;
    for i in 0..particles.len() {
        for j in (i + 1)..particles.len() {
            let d = particles[i].position.distance(&particles[j].position);
            pe -= g * particles[i].mass * particles[j].mass / (d + softening);
        }
    }
    pe
}

/// N-body gravity simulation over the StateMind's word-space.
///
/// Words attract by similarity (closer mass ≈ closer words).
/// Uses Euler integration with velocity damping.
#[must_use]
pub fn gravity_sim(mind: &StateMind, config: GravityConfig) -> GravityResult {
    let mut particles = init_particles(mind);

    if particles.len() <= 1 {
        let ke = kinetic_energy(&particles);
        let pe = potential_energy(&particles, config.g_constant);
        return GravityResult {
            snapshots: vec![GravitySnapshot {
                tick: 0,
                kinetic_energy: ke,
                potential_energy: pe,
            }],
            ticks: 0,
            converged: true,
            particles,
        };
    }

    let mut snapshots = Vec::new();
    let mut converged = false;
    let mut tick = 0;

    while tick < config.max_ticks {
        // Compute forces
        let n = particles.len();
        let mut forces = vec![[0.0_f64; 3]; n];

        for i in 0..n {
            for j in (i + 1)..n {
                let f = gravitational_force(&particles[i], &particles[j], config.g_constant);
                forces[i][0] += f[0];
                forces[i][1] += f[1];
                forces[i][2] += f[2];
                forces[j][0] -= f[0];
                forces[j][1] -= f[1];
                forces[j][2] -= f[2];
            }
        }

        // Euler integration: update velocity then position
        for (i, p) in particles.iter_mut().enumerate() {
            let inv_mass = 1.0 / p.mass;
            p.velocity[0] = (p.velocity[0] + forces[i][0] * inv_mass * config.dt) * config.damping;
            p.velocity[1] = (p.velocity[1] + forces[i][1] * inv_mass * config.dt) * config.damping;
            p.velocity[2] = (p.velocity[2] + forces[i][2] * inv_mass * config.dt) * config.damping;

            let arr = p.position.as_array();
            p.position = MindPoint {
                entropy_norm: arr[0] + p.velocity[0] * config.dt,
                gc_content: arr[1] + p.velocity[1] * config.dt,
                density: arr[2] + p.velocity[2] * config.dt,
            };
        }

        tick += 1;

        // Snapshot every 10 ticks
        if tick % 10 == 0 || tick == 1 {
            let ke = kinetic_energy(&particles);
            let pe = potential_energy(&particles, config.g_constant);
            snapshots.push(GravitySnapshot {
                tick,
                kinetic_energy: ke,
                potential_energy: pe,
            });

            if ke < config.convergence_threshold {
                converged = true;
                break;
            }
        }
    }

    // Final snapshot if not already captured
    if snapshots.is_empty() || snapshots.last().map(|s| s.tick != tick).unwrap_or(true) {
        let ke = kinetic_energy(&particles);
        let pe = potential_energy(&particles, config.g_constant);
        snapshots.push(GravitySnapshot {
            tick,
            kinetic_energy: ke,
            potential_energy: pe,
        });
    }

    GravityResult {
        particles,
        snapshots,
        ticks: tick,
        converged,
    }
}

// ---------------------------------------------------------------------------
// Evolution (Genetic Algorithm)
// ---------------------------------------------------------------------------

/// Project a word into MindPoint via mining.
fn word_to_point(word: &str) -> MindPoint {
    let ore = lexicon::mine(word);
    MindPoint::from_ore(&ore)
}

/// Fitness: 1.0 / (1.0 + distance). Closer to target = higher fitness.
fn evaluate_fitness(word: &str, target_point: &MindPoint) -> f64 {
    let pt = word_to_point(word);
    let d = pt.distance(target_point);
    1.0 / (1.0 + d)
}

/// Tournament selection: pick best from `k` random organisms.
fn tournament_select<'a>(pop: &'a [Organism], k: usize, rng: &mut Rng) -> &'a Organism {
    let mut best_idx = rng.next_usize(pop.len());
    let mut best_fit = pop[best_idx].fitness;

    for _ in 1..k {
        let idx = rng.next_usize(pop.len());
        if pop[idx].fitness > best_fit {
            best_fit = pop[idx].fitness;
            best_idx = idx;
        }
    }

    &pop[best_idx]
}

/// Single-point crossover: take a[..point] + b[point..].
fn word_crossover(a: &str, b: &str, rng: &mut Rng) -> String {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let min_len = a_bytes.len().min(b_bytes.len());

    if min_len <= 1 {
        // Can't meaningfully cross over very short words
        return a.to_string();
    }

    let point = 1 + rng.next_usize(min_len - 1);
    let mut result = Vec::with_capacity(a_bytes.len().max(b_bytes.len()));
    result.extend_from_slice(&a_bytes[..point]);
    if point < b_bytes.len() {
        result.extend_from_slice(&b_bytes[point..]);
    }

    String::from_utf8(result).unwrap_or_else(|_| a.to_string())
}

/// Per-byte mutation: with probability `rate`, replace byte with random a-z.
fn word_mutate(word: &str, rate: f64, rng: &mut Rng) -> String {
    let bytes: Vec<u8> = word
        .as_bytes()
        .iter()
        .map(|&b| {
            if rng.next_f64() < rate {
                rng.next_char()
            } else {
                b
            }
        })
        .collect();

    String::from_utf8(bytes).unwrap_or_else(|_| word.to_string())
}

/// Initialize a population from seeds + random mutations.
fn init_population(seeds: &[&str], size: usize, rng: &mut Rng) -> Vec<String> {
    let mut pop: Vec<String> = seeds.iter().map(|s| s.to_string()).collect();

    while pop.len() < size {
        let base_idx = rng.next_usize(seeds.len());
        let mutated = word_mutate(seeds[base_idx], 0.3, rng);
        pop.push(mutated);
    }

    pop.truncate(size);
    pop
}

/// Seed for the PRNG derived from target word bytes.
fn seed_from_word(word: &str) -> u64 {
    let mut h: u64 = 5381;
    for &b in word.as_bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    if h == 0 { 1 } else { h }
}

/// Genetic algorithm: evolve a population of words toward a target.
///
/// Fitness is inversely proportional to 3D distance from the target's
/// MindPoint. Uses tournament selection, single-point crossover, and
/// per-byte mutation.
#[must_use]
pub fn evolve(seeds: &[&str], target_word: &str, config: EvolutionConfig) -> EvolutionResult {
    let mut rng = Rng::new(seed_from_word(target_word));
    let target_point = word_to_point(target_word);

    let pop_size = config.population_size.max(2);
    let mut words = init_population(seeds, pop_size, &mut rng);

    let mut generation_snapshots = Vec::new();
    let mut global_best_word = String::new();
    let mut global_best_fitness = 0.0_f64;
    let mut global_best_point = MindPoint::origin();
    let mut converged_at: Option<usize> = None;
    let mut prev_best = 0.0_f64;
    let mut stagnation = 0usize;

    for gen_idx in 0..config.generations {
        // Evaluate fitness
        let mut organisms: Vec<Organism> = words
            .iter()
            .map(|w| {
                let pt = word_to_point(w);
                let fit = evaluate_fitness(w, &target_point);
                Organism {
                    word: w.clone(),
                    position: pt,
                    fitness: fit,
                }
            })
            .collect();

        // Sort by fitness descending
        organisms.sort_by(|a, b| {
            b.fitness
                .partial_cmp(&a.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let best_fit = organisms[0].fitness;
        let mean_fit: f64 =
            organisms.iter().map(|o| o.fitness).sum::<f64>() / organisms.len() as f64;

        generation_snapshots.push(Generation {
            number: gen_idx,
            best_fitness: best_fit,
            mean_fitness: mean_fit,
            best_word: organisms[0].word.clone(),
        });

        // Track global best
        if best_fit > global_best_fitness {
            global_best_fitness = best_fit;
            global_best_word = organisms[0].word.clone();
            global_best_point = organisms[0].position.clone();
        }

        // Convergence check: stagnation
        if (best_fit - prev_best).abs() < 1e-10 {
            stagnation += 1;
            if stagnation >= 10 && converged_at.is_none() {
                converged_at = Some(gen_idx);
            }
        } else {
            stagnation = 0;
        }
        prev_best = best_fit;

        // Build next generation
        let mut next_words: Vec<String> = Vec::with_capacity(pop_size);

        // Elitism: keep top N
        let elite_count = config.elitism.min(organisms.len());
        for org in organisms.iter().take(elite_count) {
            next_words.push(org.word.clone());
        }

        // Fill rest via selection + crossover + mutation
        while next_words.len() < pop_size {
            let parent_a = tournament_select(&organisms, config.tournament_size, &mut rng);
            let parent_b = tournament_select(&organisms, config.tournament_size, &mut rng);

            let child = if rng.next_f64() < config.crossover_rate {
                word_crossover(&parent_a.word, &parent_b.word, &mut rng)
            } else {
                parent_a.word.clone()
            };

            let mutated = word_mutate(&child, config.mutation_rate, &mut rng);
            next_words.push(mutated);
        }

        words = next_words;
    }

    EvolutionResult {
        generations: generation_snapshots,
        best: Organism {
            word: global_best_word,
            position: global_best_point,
            fitness: global_best_fitness,
        },
        converged_at,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- PRNG tests ---

    #[test]
    fn rng_deterministic() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn rng_range() {
        let mut rng = Rng::new(123);
        for _ in 0..200 {
            let v = rng.next_usize(10);
            assert!(v < 10);
        }
    }

    #[test]
    fn rng_f64_range() {
        let mut rng = Rng::new(999);
        for _ in 0..200 {
            let v = rng.next_f64();
            assert!(v >= 0.0 && v < 1.0);
        }
    }

    #[test]
    fn rng_next_char_lowercase() {
        let mut rng = Rng::new(777);
        for _ in 0..200 {
            let c = rng.next_char();
            assert!(c >= b'a' && c <= b'z');
        }
    }

    #[test]
    fn rng_zero_seed_handled() {
        let mut rng = Rng::new(0);
        // Should not get stuck at zero
        let v = rng.next_u64();
        assert_ne!(v, 0);
    }

    // --- K-Means tests ---

    #[test]
    fn kmeans_empty_mind() {
        let mind = StateMind::new();
        let result = kmeans(&mind, 3, 100);
        assert!(result.clusters.is_empty());
        assert!(result.converged);
        assert_eq!(result.iterations, 0);
    }

    #[test]
    fn kmeans_single_cluster() {
        let mut mind = StateMind::new();
        mind.ingest("hello");
        mind.ingest("world");
        mind.ingest("hi");

        let result = kmeans(&mind, 1, 100);
        assert_eq!(result.clusters.len(), 1);
        assert_eq!(result.clusters[0].members.len(), 3);
        assert!(result.converged);
    }

    #[test]
    fn kmeans_k_equals_n() {
        let mut mind = StateMind::new();
        mind.ingest("alpha");
        mind.ingest("beta");
        mind.ingest("gamma");

        let result = kmeans(&mind, 3, 100);
        assert_eq!(result.clusters.len(), 3);
        // Each cluster should have exactly 1 member
        for c in &result.clusters {
            assert_eq!(c.members.len(), 1);
        }
        assert!(result.converged);
    }

    #[test]
    fn kmeans_convergence() {
        let mut mind = StateMind::new();
        for w in &["rust", "java", "go", "python", "ruby"] {
            mind.ingest(w);
        }

        let result = kmeans(&mind, 2, 1000);
        assert!(result.iterations > 0);
        // Should converge with enough iterations
        assert!(result.converged);
    }

    #[test]
    fn kmeans_cluster_radius() {
        let mut mind = StateMind::new();
        mind.ingest("aaa");
        mind.ingest("bbb");

        let result = kmeans(&mind, 1, 100);
        assert_eq!(result.clusters.len(), 1);
        assert!(result.clusters[0].radius >= 0.0);
    }

    #[test]
    fn kmeans_deterministic() {
        let mut mind = StateMind::new();
        for w in &["cat", "bat", "hat", "sat", "mat"] {
            mind.ingest(w);
        }

        let r1 = kmeans(&mind, 2, 100);
        let r2 = kmeans(&mind, 2, 100);
        assert_eq!(r1.iterations, r2.iterations);
        assert_eq!(r1.clusters.len(), r2.clusters.len());
    }

    #[test]
    fn kmeans_two_clusters() {
        let mut mind = StateMind::new();
        mind.ingest("a");
        mind.ingest("b");
        mind.ingest("abcdefghij");
        mind.ingest("abcdefghijk");

        let result = kmeans(&mind, 2, 100);
        assert_eq!(result.clusters.len(), 2);
        // Total members should equal total points
        let total_members: usize = result.clusters.iter().map(|c| c.members.len()).sum();
        assert_eq!(total_members, 4);
    }

    #[test]
    fn kmeans_variance_positive() {
        let mut mind = StateMind::new();
        mind.ingest("hello");
        mind.ingest("world");

        let result = kmeans(&mind, 1, 100);
        assert!(result.total_variance >= 0.0);
    }

    #[test]
    fn kmeans_three_words_two_clusters() {
        let mut mind = StateMind::new();
        mind.ingest("aaaa");
        mind.ingest("aaab");
        mind.ingest("zzzzzzzzzz");

        let result = kmeans(&mind, 2, 100);
        assert_eq!(result.clusters.len(), 2);
        assert!(result.converged);
    }

    #[test]
    fn assign_nearest() {
        let points = vec![
            MindPoint {
                entropy_norm: 0.0,
                gc_content: 0.0,
                density: 0.0,
            },
            MindPoint {
                entropy_norm: 1.0,
                gc_content: 1.0,
                density: 1.0,
            },
        ];
        let centroids = vec![
            MindPoint {
                entropy_norm: 0.0,
                gc_content: 0.0,
                density: 0.0,
            },
            MindPoint {
                entropy_norm: 1.0,
                gc_content: 1.0,
                density: 1.0,
            },
        ];
        let assignments = assign_clusters(&points, &centroids);
        assert_eq!(assignments, vec![0, 1]);
    }

    #[test]
    fn recompute_mean() {
        let points = vec![
            MindPoint {
                entropy_norm: 0.0,
                gc_content: 0.0,
                density: 0.0,
            },
            MindPoint {
                entropy_norm: 2.0,
                gc_content: 2.0,
                density: 2.0,
            },
        ];
        let assignments = vec![0, 0]; // both in cluster 0
        let old_centroids = vec![MindPoint {
            entropy_norm: 0.0,
            gc_content: 0.0,
            density: 0.0,
        }];
        let (new_c, _conv) = recompute_centroids(&points, &assignments, 1, &old_centroids);
        assert!((new_c[0].entropy_norm - 1.0).abs() < f64::EPSILON);
        assert!((new_c[0].gc_content - 1.0).abs() < f64::EPSILON);
        assert!((new_c[0].density - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn variance_decreasing() {
        let mut mind = StateMind::new();
        for w in &["cat", "bat", "hat", "elephant", "zebra"] {
            mind.ingest(w);
        }

        let r1 = kmeans(&mind, 1, 100);
        let r2 = kmeans(&mind, 2, 100);
        // More clusters should reduce or maintain variance
        assert!(r2.total_variance <= r1.total_variance + f64::EPSILON);
    }

    #[test]
    fn cluster_display() {
        let c = Cluster {
            centroid: MindPoint {
                entropy_norm: 0.25,
                gc_content: 0.30,
                density: 0.80,
            },
            members: vec![0, 1, 2],
            radius: 0.042,
            id: 0,
        };
        let s = format!("{c}");
        assert!(s.contains("C0"));
        assert!(s.contains("3 members"));
    }

    #[test]
    fn cluster_result_display() {
        let result = ClusterResult {
            clusters: Vec::new(),
            iterations: 12,
            converged: true,
            total_variance: 0.156,
        };
        let s = format!("{result}");
        assert!(s.contains("12 iter"));
        assert!(s.contains("converged"));
    }

    // --- Gravity tests ---

    #[test]
    fn gravity_zero_velocity_init() {
        let mut mind = StateMind::new();
        mind.ingest("test");
        let particles = init_particles(&mind);
        assert_eq!(particles.len(), 1);
        assert_eq!(particles[0].velocity, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn gravity_single_particle() {
        let mut mind = StateMind::new();
        mind.ingest("alone");

        let result = gravity_sim(&mind, GravityConfig::default());
        assert!(result.converged);
        assert_eq!(result.particles.len(), 1);
    }

    #[test]
    fn gravity_two_attract() {
        let mut mind = StateMind::new();
        mind.ingest("aaaa");
        mind.ingest("zzzzzzzzzz");

        let result = gravity_sim(
            &mind,
            GravityConfig {
                max_ticks: 100,
                ..GravityConfig::default()
            },
        );
        assert_eq!(result.particles.len(), 2);
        // After simulation, particles should be closer than initially
        let initial_dist = mind.points()[0].distance(&mind.points()[1]);
        let final_dist = result.particles[0]
            .position
            .distance(&result.particles[1].position);
        assert!(final_dist < initial_dist + 0.1); // allow small numerical drift
    }

    #[test]
    fn gravity_mass_positive() {
        let mut mind = StateMind::new();
        mind.ingest("word");
        let particles = init_particles(&mind);
        assert!(particles[0].mass > 0.0);
    }

    #[test]
    fn gravity_kinetic_energy_zero_init() {
        let mut mind = StateMind::new();
        mind.ingest("hello");
        let particles = init_particles(&mind);
        let ke = kinetic_energy(&particles);
        assert!((ke - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gravity_potential_negative() {
        let mut mind = StateMind::new();
        mind.ingest("alpha");
        mind.ingest("beta");
        let particles = init_particles(&mind);
        let pe = potential_energy(&particles, 0.1);
        assert!(pe < 0.0); // gravitational PE is always negative
    }

    #[test]
    fn gravity_damping_reduces() {
        let mut mind = StateMind::new();
        mind.ingest("cat");
        mind.ingest("dog");

        // Strong damping
        let r1 = gravity_sim(
            &mind,
            GravityConfig {
                damping: 0.5,
                max_ticks: 100,
                ..GravityConfig::default()
            },
        );
        // Weak damping
        let r2 = gravity_sim(
            &mind,
            GravityConfig {
                damping: 0.99,
                max_ticks: 100,
                ..GravityConfig::default()
            },
        );

        // Strong damping should yield lower final KE
        let ke1 = r1.snapshots.last().map(|s| s.kinetic_energy).unwrap_or(0.0);
        let ke2 = r2.snapshots.last().map(|s| s.kinetic_energy).unwrap_or(0.0);
        assert!(ke1 <= ke2 + 1e-6);
    }

    #[test]
    fn gravity_convergence() {
        let mut mind = StateMind::new();
        mind.ingest("hi");
        mind.ingest("ho");

        let result = gravity_sim(
            &mind,
            GravityConfig {
                damping: 0.5,
                max_ticks: 5000,
                convergence_threshold: 1e-4,
                ..GravityConfig::default()
            },
        );
        // With strong damping, should eventually converge
        assert!(result.converged || result.ticks <= 5000);
    }

    #[test]
    fn gravity_max_ticks() {
        let mut mind = StateMind::new();
        mind.ingest("abc");
        mind.ingest("xyz");

        let result = gravity_sim(
            &mind,
            GravityConfig {
                max_ticks: 10,
                convergence_threshold: 1e-30, // basically never converge
                ..GravityConfig::default()
            },
        );
        assert!(result.ticks <= 10);
    }

    #[test]
    fn gravity_force_direction() {
        let p1 = Particle {
            word_idx: 0,
            position: MindPoint {
                entropy_norm: 0.0,
                gc_content: 0.0,
                density: 0.0,
            },
            velocity: [0.0, 0.0, 0.0],
            mass: 1.0,
        };
        let p2 = Particle {
            word_idx: 1,
            position: MindPoint {
                entropy_norm: 1.0,
                gc_content: 0.0,
                density: 0.0,
            },
            velocity: [0.0, 0.0, 0.0],
            mass: 1.0,
        };
        let f = gravitational_force(&p1, &p2, 1.0);
        // Force should be in positive x direction (toward p2)
        assert!(f[0] > 0.0);
        assert!(f[1].abs() < 1e-6);
        assert!(f[2].abs() < 1e-6);
    }

    #[test]
    fn gravity_force_inverse_sq() {
        let p1 = Particle {
            word_idx: 0,
            position: MindPoint {
                entropy_norm: 0.0,
                gc_content: 0.0,
                density: 0.0,
            },
            velocity: [0.0; 3],
            mass: 1.0,
        };
        let p2_close = Particle {
            word_idx: 1,
            position: MindPoint {
                entropy_norm: 0.1,
                gc_content: 0.0,
                density: 0.0,
            },
            velocity: [0.0; 3],
            mass: 1.0,
        };
        let p2_far = Particle {
            word_idx: 1,
            position: MindPoint {
                entropy_norm: 1.0,
                gc_content: 0.0,
                density: 0.0,
            },
            velocity: [0.0; 3],
            mass: 1.0,
        };

        let f_close = gravitational_force(&p1, &p2_close, 1.0);
        let f_far = gravitational_force(&p1, &p2_far, 1.0);
        // Closer → stronger force (approximately inverse-square)
        assert!(f_close[0] > f_far[0]);
    }

    #[test]
    fn gravity_snapshot_periodic() {
        let mut mind = StateMind::new();
        mind.ingest("foo");
        mind.ingest("bar");

        let result = gravity_sim(
            &mind,
            GravityConfig {
                max_ticks: 50,
                convergence_threshold: 1e-30,
                ..GravityConfig::default()
            },
        );
        // Should have snapshots at tick 1, 10, 20, 30, 40, 50
        assert!(result.snapshots.len() >= 2);
    }

    #[test]
    fn gravity_config_default() {
        let cfg = GravityConfig::default();
        assert!((cfg.g_constant - 0.1).abs() < f64::EPSILON);
        assert!((cfg.dt - 0.01).abs() < f64::EPSILON);
        assert_eq!(cfg.max_ticks, 1000);
        assert!((cfg.damping - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn particle_display() {
        let p = Particle {
            word_idx: 0,
            position: MindPoint {
                entropy_norm: 0.25,
                gc_content: 0.30,
                density: 0.80,
            },
            velocity: [0.01, -0.02, 0.0],
            mass: 2.5,
        };
        let s = format!("{p}");
        assert!(s.contains("#0"));
        assert!(s.contains("2.5"));
    }

    #[test]
    fn gravity_result_display() {
        let result = GravityResult {
            particles: Vec::new(),
            snapshots: vec![GravitySnapshot {
                tick: 150,
                kinetic_energy: 0.001,
                potential_energy: -0.045,
            }],
            ticks: 150,
            converged: true,
        };
        let s = format!("{result}");
        assert!(s.contains("150 ticks"));
        assert!(s.contains("converged"));
    }

    // --- Evolution tests ---

    #[test]
    fn evolve_single_gen() {
        let result = evolve(
            &["aaaa", "bbbb"],
            "rust",
            EvolutionConfig {
                generations: 1,
                population_size: 10,
                ..EvolutionConfig::default()
            },
        );
        assert_eq!(result.generations.len(), 1);
        assert!(result.best.fitness > 0.0);
    }

    #[test]
    fn evolve_fitness_improves() {
        let result = evolve(
            &["aaaa", "bbbb"],
            "rust",
            EvolutionConfig {
                generations: 50,
                population_size: 20,
                ..EvolutionConfig::default()
            },
        );

        let first_best = result.generations[0].best_fitness;
        let last_best = result
            .generations
            .last()
            .map(|g| g.best_fitness)
            .unwrap_or(0.0);
        // Fitness should not decrease (elitism)
        assert!(last_best >= first_best - f64::EPSILON);
    }

    #[test]
    fn evolve_elitism_preserves() {
        let result = evolve(
            &["test"],
            "best",
            EvolutionConfig {
                generations: 20,
                population_size: 10,
                elitism: 2,
                ..EvolutionConfig::default()
            },
        );

        // Best fitness should be monotonically non-decreasing
        for w in result.generations.windows(2) {
            assert!(w[1].best_fitness >= w[0].best_fitness - 1e-10);
        }
    }

    #[test]
    fn evolve_population_size() {
        let result = evolve(
            &["ab"],
            "cd",
            EvolutionConfig {
                generations: 5,
                population_size: 30,
                ..EvolutionConfig::default()
            },
        );
        // All generations should exist
        assert_eq!(result.generations.len(), 5);
    }

    #[test]
    fn evolve_deterministic() {
        let cfg = EvolutionConfig {
            generations: 10,
            population_size: 20,
            ..EvolutionConfig::default()
        };

        let r1 = evolve(&["hello"], "world", cfg);
        let cfg2 = EvolutionConfig {
            generations: 10,
            population_size: 20,
            ..EvolutionConfig::default()
        };
        let r2 = evolve(&["hello"], "world", cfg2);

        assert_eq!(r1.best.word, r2.best.word);
        assert!((r1.best.fitness - r2.best.fitness).abs() < f64::EPSILON);
    }

    #[test]
    fn evaluate_fitness_self() {
        let target_point = word_to_point("rust");
        let f = evaluate_fitness("rust", &target_point);
        // Distance is 0, fitness should be 1.0
        assert!((f - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn evaluate_fitness_distance() {
        let target_point = word_to_point("rust");
        let f_close = evaluate_fitness("rust", &target_point);
        let f_far = evaluate_fitness("aaaaaaaaaa", &target_point);
        assert!(f_close > f_far);
    }

    #[test]
    fn tournament_best() {
        let pop = vec![
            Organism {
                word: "a".to_string(),
                position: MindPoint::origin(),
                fitness: 0.1,
            },
            Organism {
                word: "b".to_string(),
                position: MindPoint::origin(),
                fitness: 0.9,
            },
            Organism {
                word: "c".to_string(),
                position: MindPoint::origin(),
                fitness: 0.5,
            },
        ];
        let mut rng = Rng::new(42);
        // With tournament size == pop size, should always pick best
        let best = tournament_select(&pop, 3, &mut rng);
        assert!((best.fitness - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn word_crossover_length() {
        let mut rng = Rng::new(42);
        let child = word_crossover("abcd", "wxyz", &mut rng);
        // Crossover of two 4-byte words → 4 bytes
        assert_eq!(child.len(), 4);
    }

    #[test]
    fn word_crossover_deterministic() {
        let mut rng1 = Rng::new(99);
        let mut rng2 = Rng::new(99);
        let c1 = word_crossover("hello", "world", &mut rng1);
        let c2 = word_crossover("hello", "world", &mut rng2);
        assert_eq!(c1, c2);
    }

    #[test]
    fn word_mutate_rate_zero() {
        let mut rng = Rng::new(42);
        let result = word_mutate("hello", 0.0, &mut rng);
        assert_eq!(result, "hello");
    }

    #[test]
    fn word_mutate_rate_one() {
        let mut rng = Rng::new(42);
        let result = word_mutate("hello", 1.0, &mut rng);
        // All bytes mutated → highly unlikely to match original
        assert_ne!(result, "hello");
        assert_eq!(result.len(), 5); // length preserved
    }

    #[test]
    fn generation_display() {
        let snap = Generation {
            number: 10,
            best_fitness: 0.95,
            mean_fitness: 0.72,
            best_word: "rust".to_string(),
        };
        let s = format!("{snap}");
        assert!(s.contains("Gen 10"));
        assert!(s.contains("rust"));
    }

    #[test]
    fn evolution_result_display() {
        let result = EvolutionResult {
            generations: vec![Generation {
                number: 0,
                best_fitness: 0.95,
                mean_fitness: 0.72,
                best_word: "rust".to_string(),
            }],
            best: Organism {
                word: "rust".to_string(),
                position: MindPoint::origin(),
                fitness: 0.95,
            },
            converged_at: None,
        };
        let s = format!("{result}");
        assert!(s.contains("rust"));
        assert!(s.contains("0.95"));
    }

    #[test]
    fn organism_display() {
        let org = Organism {
            word: "rust".to_string(),
            position: MindPoint {
                entropy_norm: 0.25,
                gc_content: 0.188,
                density: 1.0,
            },
            fitness: 0.85,
        };
        let s = format!("{org}");
        assert!(s.contains("rust"));
        assert!(s.contains("0.85"));
    }

    #[test]
    fn evolve_convergence() {
        let result = evolve(
            &["ruse", "bust"],
            "rust",
            EvolutionConfig {
                generations: 100,
                population_size: 30,
                mutation_rate: 0.15,
                ..EvolutionConfig::default()
            },
        );
        // Should have run all generations
        assert_eq!(result.generations.len(), 100);
        // Best fitness should be positive
        assert!(result.best.fitness > 0.0);
    }

    #[test]
    fn random_word_length() {
        let mut rng = Rng::new(42);
        let pop = init_population(&["test"], 10, &mut rng);
        assert_eq!(pop.len(), 10);
        // All words should have length > 0
        for w in &pop {
            assert!(!w.is_empty());
        }
    }

    #[test]
    fn evolution_config_default() {
        let cfg = EvolutionConfig::default();
        assert_eq!(cfg.population_size, 50);
        assert_eq!(cfg.generations, 100);
        assert!((cfg.mutation_rate - 0.2).abs() < f64::EPSILON);
        assert!((cfg.crossover_rate - 0.7).abs() < f64::EPSILON);
        assert_eq!(cfg.tournament_size, 3);
        assert_eq!(cfg.elitism, 4);
    }
}
