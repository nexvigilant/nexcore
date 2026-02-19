//! Tool implementations for nexcore MCP Server
//!
//! Organized by domain:
//! - foundation: Algorithms, crypto, YAML, graph, FSRS
//! - pv: Signal detection, causality assessment
//! - vigilance: Safety margin, risk, ToV, harm types
//! - guardian: Homeostasis control loop, threat sensing, response execution
//! - vigil: AI orchestrator control, event bus, memory layer
//! - skills: Registry, validation, taxonomy
//! - validation: Universal L1-L5 validation
//! - guidelines: ICH, CIOMS, EMA GVP guideline search and lookup
//! - ich_glossary: O(1) ICH/CIOMS term lookup for 894+ pharmacovigilance terms
//! - faers: FDA Adverse Event Reporting System queries
//! - gcloud: Google Cloud CLI operations
//! - wolfram: Wolfram Alpha computational knowledge engine
//! - brain: Antigravity-style working memory (sessions, artifacts, code tracking)
//! - primitive_validation: Corpus-backed validation with professional citations

pub mod benefit_risk;
pub mod bonding;
pub mod brain;
pub mod brain_db;
pub mod brain_verify;
pub mod brand_semantics;
pub mod chemistry;
pub mod commandments;
pub mod compliance;
pub mod cortex;
pub mod docs;
pub mod faers;
pub mod forge;
pub mod foundation;
pub mod gcloud;
pub mod grammar;
pub mod guardian;
pub mod guidelines;
pub mod hooks;
pub mod hud;
pub mod ich_glossary;
pub mod perplexity;
pub mod primitive_validation;
pub mod principles;
pub mod pv;
pub mod pv_axioms;
pub mod pv_pipeline;
pub mod pvdsl;
pub mod regulatory;
pub mod reproductive;
pub mod signal;
pub mod skills;
pub mod synapse;
pub mod synth;
pub mod validation;
pub mod vigil;
pub mod vigilance;
pub mod wolfram;

// Endocrine system (persistent behavioral state)
pub mod hormones;

// Cognitive Evolution Pipeline (8-stage knowledge discovery)
pub mod cep;
pub mod mcp_lock;
pub mod mesh;
pub mod node_hunter;
#[allow(missing_docs)]
pub mod watchtower;

// Telemetry Intelligence (external source monitoring)
pub mod telemetry_intel;

// Antipattern Immunity (self-regulating code quality)
#[allow(missing_docs)]
pub mod immunity;

// Primitive Scanner (domain primitive extraction)
pub mod primitive_scanner;

// STEM primitives (Science, Technology, Engineering, Mathematics)
pub mod stem;

// Visualization (SVG diagrams for STEM taxonomy, type composition, DAGs)
pub mod viz;

// Algorithmic Vigilance (ICSR deduplication, signal triage)
pub mod algovigilance;

// DNA-based Computation (nucleotide encoding, Codon VM, PV signals)
pub mod dna;

// Decision Tree Engine (CART binary splitting, pruning, importance)
pub mod dtree;

// Edit Distance Framework (generic ops, costs, solvers, transfer)
pub mod edit_distance;

// Sentinel (SSH brute-force protection — Rust fail2ban)
pub mod sentinel;

// Measure (workspace quality measurement — information theory, graph theory, statistics)
pub mod measure;

// Anatomy (workspace structural analysis — dependency graph, layers, blast radius, Chomsky)
pub mod anatomy;

// FAERS ETL (local bulk data pipeline — ingest, signal detection, known-pair validation)
pub mod faers_etl;

// FAERS Analytics (novel signal detection — A77 velocity, A80 cascade, A82 outcome)
pub mod faers_analytics;

// Lex Primitiva (T1 symbolic foundation queries)
pub mod lex_primitiva;

// Molecular Biology (Central Dogma, ADME, codon translation)
pub mod molecular;

// Skill Crates (typed Rust skill implementations)
pub mod skill_crates;

// Skill Knowledge Assist (intent-based skill search)
pub mod assist;

// Skill Token Analysis (context optimization)
pub mod skill_tokens;

// Visual Primitives (shape/color pattern matching for Prima)
pub mod visual;

// FDA AI Credibility Assessment (7-step regulatory framework)
pub mod fda;
pub mod fda_metrics;

// Cytokine Signaling (typed event bus based on immune system patterns)
pub mod cytokine;

// Value Mining (economic signal detection using PV algorithms)
pub mod value_mining;

// Game Theory (normal-form equilibrium analysis)
pub mod game_theory;

// Signal Theory (Universal Theory of Signals — axioms, theorems, detection, SDT)
pub mod signal_theory;

// Signal Fence (process-level network signal container — default-deny)
pub mod signal_fence;

// Prima Language (primitive-first programming, .true files)
pub mod prima;

// Aggregate (T1 fold/recursive/ranked combinators — Σ + ρ + κ)
pub mod aggregate;

// Compound Growth (primitive basis velocity tracking)
pub mod compound;

// Compound Growth Detector (phase + bottleneck detection)
pub mod compound_detector;

// Claude Care Process (PK engine for AI support interventions)
pub mod ccp;

// Education Machine (Bayesian mastery, 5-phase FSM, spaced repetition)
pub mod education;

// Adversarial prompt detection (statistical fingerprints)
pub mod adversarial;

// AI text detection (5-feature statistical fingerprints)
pub mod antitransformer;

// Token-as-Energy (ATP/ADP biochemistry for token budget management)
pub mod energy;

// Reverse Transcriptase (schema inference + data generation)
pub mod transcriptase;

// Ribosome (schema contract registry + drift detection)
pub mod ribosome;

// Domain Primitives (tier taxonomy + transfer confidence)
pub mod domain_primitives;

// State Operating System (15-layer state machine runtime)
pub mod sos;

// Skill Quality Index (chemistry-derived scoring — 7 dimensions)
pub mod sqi;

// Development System Monitoring (anomaly detection from telemetry)
pub mod monitoring;

// Security Classification (5-level clearance system with tiered enforcement)
pub mod clearance;

// Secure Boot Chain (TPM-style measured boot with PCR extend)
pub mod secure_boot;

// User Management (authentication, sessions, access control)
pub mod user;

// Consolidated satellite MCP servers
pub mod claude_fs;
pub mod compendious;
pub mod docs_claude;
pub mod gsheets;
pub mod reddit;
pub mod trust;

// Molecular Weight of Words (Algorithm A76 — Shannon information-theoretic mass)
pub mod molecular_weight;

// Laboratory (virtual word/concept experiment engine — decompose, weigh, react)
pub mod laboratory;

// Primitive Trace (concept → T1 → full-stack interaction mapping)
pub mod primitive_trace;

// Text Transformation (cross-domain rewriting engine, 5 tools)
pub mod transform;

// Integrity Assessment (AI text detection for KSB assessment)
pub mod integrity;

// Counter-Awareness (detection/counter-detection matrix)
pub mod counter_awareness;

// Mesh Network (runtime mesh with discovery, gossip, resilience)
pub mod mesh_network;

// Insight Engine (pattern detection, novelty, connection, compression)
pub mod insight;

// Caesura Detection (structural seam detection — ∂+ς+∝+ν)
pub mod caesura;

// Formula-Derived Tools (KU extraction → MCP tools: F-004, F-007, F-008, F-011, F-015)
pub mod formula;

// Declension System (Latin-inspired architectural primitives — ∂+ς+μ+∅+×)
pub mod declension;

// Vigil System (π(∂·ν)|∝ — Vigilance engine: ledger, boundary gate, consequence pipeline)
pub mod vigil_system;

// Lessons Learned (π+μ+σ+∃ — persistent lesson storage with primitive extraction)
pub mod lessons;

// Claude REPL (σ+∂+ς+→ — CLI bridge to Claude Code subprocesses)
pub mod claude_repl;

// Adventure HUD (ς+σ+μ+N — session tracking for Claude Code adventures)
pub mod adventure;

// Borrow Miner (ς+N+κ+∂ — ore mining game with FDA signal detection)
pub mod borrow_miner;

// Proof of Meaning (σ+κ+∂+→+N — chemistry-inspired semantic equivalence engine)
pub mod proof_of_meaning;

// MCP Call Telemetry (self-reporting metrics)
#[cfg(feature = "telemetry")]
pub mod telemetry;
