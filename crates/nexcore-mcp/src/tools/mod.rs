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

// Academy Forge (structured knowledge extraction + academy validation)
pub mod academy_forge;

pub mod api;
pub mod asm;
pub mod benefit_risk;
pub mod boundary_detector;
pub mod brain;

// CLI wrapper tools (git, gh, systemctl, npm, filesystem)
pub mod fs;
pub mod gh;
pub mod git;
pub mod npm;
pub mod service;

// DAG Publish introspection (plan, dry-run, status — read-only, no cargo publish)
pub mod dag_publish;

// Cargo Toolchain (structured build/check/test/clippy/fmt/tree)
pub mod brain_db;
pub mod brain_verify;
pub mod brand_semantics;
pub mod cargo;
pub mod ccim;
pub mod chemistry;
pub mod commandments;
pub mod compliance;
pub mod cortex;
pub mod learning;

// Crate Development Framework (scaffold + audit)
pub mod crate_dev;

// Crate X-Ray (deep inspection, CTVP trials, dev goals)
pub mod crate_xray;
pub mod crew;
pub mod docs;
pub mod faers;
pub mod forge;
pub mod foundation;

// Frontend & Accessibility (WCAG contrast, touch targets, type scale, spacing, a11y audit)
pub mod fda_guidance;
pub mod frontend;
pub mod gcloud;
pub mod grammar;
pub mod guardian;
pub mod guidelines;
pub mod hitl;
pub mod hud;
pub mod ich_glossary;
pub mod perplexity;
pub mod primitive_validation;
pub mod principles;
pub mod pv;
pub mod pv_axioms;
pub mod pv_embeddings;
pub mod pv_pipeline;
pub mod pvdsl;
pub mod registry;
pub mod regulatory;
pub mod reproductive;
pub mod retrieval;
pub mod signal;
pub mod skills;

// SOP-Anatomy-Code triple mapping, reactor, transfer protocol
pub mod sop_anatomy;
pub mod synapse;
pub mod synth;
pub mod validation;
pub mod vigil;
pub mod vigilance;
pub mod wolfram;

// Endocrine system (persistent behavioral state)
pub mod hormones;

// Cognition (transformer algorithm — attention, generation, metrics)
pub mod cognition;

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

// Topology (TDA: persistent homology, Betti numbers, Vietoris-Rips + graph analysis)
pub mod topology;

// Visualization (SVG diagrams for STEM taxonomy, type composition, DAGs)
pub mod viz;
pub mod viz_advanced;
pub mod viz_biologics;
pub mod viz_foundation;
pub mod viz_physics;

// Algorithmic Vigilance (ICSR deduplication, signal triage)
pub mod algovigilance;

// DNA-based Computation (nucleotide encoding, Codon VM, PV signals)
pub mod dna;

// Decision Tree Engine (CART binary splitting, pruning, importance)
pub mod dtree;

// ML Pipeline (autonomous PV signal detection via random forest)
pub mod ml_pipeline;

// Edit Distance Framework (generic ops, costs, solvers, transfer)
pub mod edit_distance;

// Sentinel (SSH brute-force protection — Rust fail2ban)
pub mod sentinel;

// Measure (workspace quality measurement — information theory, graph theory, statistics)
pub mod measure;

// Anatomy (workspace structural analysis — dependency graph, layers, blast radius, Chomsky)
pub mod anatomy;
pub mod anatomy_db;

// FAERS ETL (local bulk data pipeline — ingest, signal detection, known-pair validation)
pub mod faers_etl;

// PHAROS — Pharmacovigilance Autonomous Reconnaissance and Observation System
pub mod pharos;

// FAERS Analytics (novel signal detection — A77 velocity, A80 cascade, A82 outcome)
pub mod faers_analytics;

// Lex Primitiva (T1 symbolic foundation queries)
pub mod lex_primitiva;

// Primitive Brain (T1 decomposition, distance, conservation, composition)
pub mod primitive_brain;

// Ouverse Chain (forward enablement — Anti-Why direction of conservation law)
pub mod ouverse;

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
pub mod cybercinetics;
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

// CCCP (Consultant's Client Care Process — 5-phase PV consulting pipeline)
pub mod cccp;

// Education Machine (Bayesian mastery, 5-phase FSM, spaced repetition)
pub mod education;

// Adversarial prompt detection (statistical fingerprints)
pub mod adversarial;

// AI text detection (5-feature statistical fingerprints)
pub mod antitransformer;

// Epidemiology (Domain 7 — measures of association, impact, survival)
pub mod epidemiology;

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

// Flywheel Loop Engine (5-loop cascade, VDAG, live metrics — ς+κ+∂+→+N)
pub mod flywheel;

// Formula-Derived Tools (KU extraction → MCP tools: F-004, F-007, F-008, F-011, F-015)
pub mod formula;

// Declension System (Latin-inspired architectural primitives — ∂+ς+μ+∅+×)
pub mod declension;

// Vigil System (π(∂·ν)|∝ — Vigilance engine: ledger, boundary gate, consequence pipeline)
pub mod vigil_system;

// Theory of Vigilance (ToV direct — signal strength, stability, epistemic trust)
pub mod tov;

// PV Core (IVF axioms, severity classification)
pub mod pv_core;

// Knowledge (KSB article index — 628 PV articles across 15 domains)
pub mod knowledge;

// Knowledge Engine (π+μ+σ+N+∝ — knowledge compression, compilation, and query)
pub mod knowledge_engine;

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

// Statistical Drift Detection (KS test, PSI, Jensen-Shannon divergence)
pub mod drift_detection;

// Rate Limiting (token bucket, sliding window)
pub mod rate_limiter;

// Rank Fusion (RRF, hybrid interpolation, Borda count)
pub mod rank_fusion;

// Relay Fidelity (chain construction, verification, pipeline tracking — →+∂+π)
pub mod relay;

// Security Posture Assessment (compliance scorecards, threat readiness, gap analysis)
pub mod security_posture;

// AI Observability Metrics (inference latency, data freshness, throughput)
pub mod observability;

// ORGANIZE (8-step file organization pipeline — ∃ κ μ → ∂ Σ ∅ ς)
pub mod organize;

// GROUNDED (epistemological substrate — uncertainty, evidence chains, confidence gating)
pub mod grounded;

// Digital Highways (infrastructure acceleration — Chatburn 1923 transfer)
pub mod highway;

// Disney Loop (forward-only compound discovery — ρ(t) → ∂(¬σ⁻¹) → ∃(ν) → ρ(t+1))
pub mod disney_loop;

// Tool Routing (deterministic dispatch + DAG execution planning — σ+→+μ+∂)
pub mod routing;

// Validify (8-gate crate validation — V-A-L-I-D-I-F-Y)
pub mod validify;

// CTVP (Clinical Trial Validation Paradigm — 5-phase software validation)
pub mod ctvp;

// Code Inspection (FDA-inspired audit — safety/efficacy/purity)
pub mod code_inspect;

// Primitive Coverage (T1 Lex Primitiva coverage analysis)
pub mod primitive_coverage;

// Model Delegation (task→model routing with scoring)
pub mod model_delegation;

// Prompt Kinetics (ADME pharmacokinetic model for prompts)
pub mod prompt_kinetics;

// Compounding Engine (learning pipeline velocity metrics)
pub mod compounding_engine;

// Polymer (hook pipeline composition with stoichiometry)
pub mod polymer;

// BAS Organ Systems (Biological Analogy System — 11 organ crate wrappers)
pub mod cardiovascular;
pub mod circulatory;
pub mod digestive;
pub mod integumentary;
pub mod lymphatic;
pub mod muscular;
pub mod nervous;
pub mod phenotype;
pub mod respiratory;
pub mod skeletal;
pub mod urinary;

// HTTP Request (MCP-native curl replacement — →+λ+μ+∂)
pub mod http;

// Oracle (Bayesian event prediction — σ+→+ν+κ+N+π)
pub mod oracle;

// MCP Call Telemetry (self-reporting metrics)
#[cfg(feature = "telemetry")]
pub mod telemetry;

// Terminal Remote Controller (Claude accessibility bridge)
pub mod terminal_remote;

// Kellnr computation & registry (consolidated from kellnr-mcp)
pub mod kellnr_dtree;
pub mod kellnr_graph;
pub mod kellnr_pk;
pub mod kellnr_registry;
pub mod kellnr_signal;
pub mod kellnr_stats;
pub mod kellnr_thermo;

// Observatory Phase 9 — Career transitions, learning DAG, graph layout
pub mod career;
pub mod graph_layout;
pub mod learning_dag;

// Observatory Personalization (detect, get, set, validate)
pub mod observatory;

// Stoichiometry (encode/decode concepts as balanced primitive equations)
pub mod stoichiometry;

// TRIAL Framework (universal experimentation — 10 MCP tools)
pub mod trial;

// The Foundry (dual-pipeline assembly line — validate, cascade, infer, render, VDAG)
pub mod foundry;

// Chemivigilance (SMILES, descriptors, QSAR, metabolites, SafetyBrief — 15 tools)
pub mod chemivigilance;

// NMD Surveillance (Nonsense-Mediated mRNA Decay — anti-hallucination pipeline)
pub mod nmd;

// PV Pharmacokinetics (AUC, clearance, half-life, steady-state, ionization, Michaelis-Menten)
pub mod pk;

// PV Causality Assessment (RUCAM hepatotoxicity, UCAS unified)
pub mod causality;

// PV Temporal Analysis (time-to-onset, challenge assessment, plausibility)
pub mod temporal;

// QSAR Granular Predictions (mutagenicity, hepatotoxicity, cardiotoxicity, domain assessment)
pub mod qsar;

// NotebookLM (library CRUD, session management, browser-automated queries)
pub mod notebooklm;

// Cloud Intelligence (35-type taxonomy → 17 MCP tools)
pub mod cloud;

// Rust Development (error types, derive advisor, match gen, borrow explain)
pub mod rust_dev;

// Zeta function telescope pipeline (LMFDB, batch, scaling, Cayley, operator hunt)
pub mod zeta;

// Signal detection pipeline (PRR/ROR/IC/EBGM, contingency tables, Evans thresholds, relay)
pub mod signal_pipeline;

// Preemptive PV: 3-tier signal detection (Reactive → Predictive → Preemptive)
pub mod preemptive_pv;

// OpenFDA: live FDA drug, device, food, and substance database access
pub mod openfda;

// Compound Registry: 3-layer chemical compound resolution (Cache → PubChem → ChEMBL)
pub mod compound_registry;

// FHIR R4: HL7 resource parsing, AdverseEvent→SignalInput conversion, validation
pub mod fhir;

// Retrocasting: signal-to-structure linking, clustering, alert correlation, ML features
pub mod retrocasting;

// Engram: unified knowledge daemon (TF-IDF search, temporal decay, multi-source ingest)
pub mod engram;

// Ghost: privacy enforcement, PII scanning, anonymization boundary checking
pub mod ghost;

// Pharma R&D: taxonomy, transfer confidence, pipeline stages, Chomsky classification
pub mod pharma_rd;

// Combinatorics: Catalan, derangement, cycle decomposition, Josephus, grid paths, linear extensions
pub mod combinatorics;

// Theory of Vigilance (Grounded): signal strength, safety margin, stability shells, harm types
pub mod tov_grounded;

// Statemind: DNA chemistry word analysis and constellation resonance
pub mod statemind;

// Reason: causal DAG inference and counterfactual evaluation
pub mod reason;

// Word: binary word trait algebra (popcount, hamming, rotation, GCD, alignment)
pub mod word;

// Harm Taxonomy: ToV §9 harm classification (types A-I, laws, axioms)
pub mod harm_taxonomy;

// Anti-Vector: structured countermeasures that annihilate harm vectors
pub mod antivector;

// Antibodies: adaptive immune recognition (affinity, Ig class)
pub mod antibodies;

// Jeopardy: game theory strategy engine (wagers, buzz, selection, compound velocity)
pub mod jeopardy;

// Audio: sample conversion, spec computation, codec catalog, pan law, stream states
pub mod audio;

// Compilation Space: 7D transform algebra (points, transforms, chains, catalog)
pub mod compilation_space;

// Pharmacovigilance Taxonomy (4-tier WHO-grounded PV concept encoder)
pub mod pharmacovigilance;

// Vault (AES-256-GCM encryption, PBKDF2 key derivation, salt generation)
pub mod vault;

// Knowledge Vault (Obsidian-compatible markdown vault — read, search, write, tags)
pub mod knowledge_vault;

// Build Orchestrator (CI/CD pipeline: dry-run, stages, workspace, history, metrics)
pub mod build_orchestrator;

// Skills Engine (advanced SQI, maturity, KSB verify, ecosystem, dependency graph, gap analysis, evolution)
pub mod skills_engine;

// NCBI (National Center for Biotechnology Information — PubMed, Gene, Protein search)
pub mod ncbi;

// Entropy (Shannon, cross, KL, mutual information, normalized, conditional)
pub mod entropy;

// Graph (centrality, components, shortest paths, PageRank, communities, SCC, topo sort)
pub mod graph;

// Markov chains (stationary distribution, n-step probabilities, ergodicity, classification)
pub mod markov;

// DataFrame (sovereign columnar engine — describe, query, aggregate, counter, stats, construct, transform, save)
pub mod dataframe;

// Chrono (sovereign datetime engine — now, parse, format, duration)
pub mod chrono;

// NexChat (AI chat status, config, tool discovery)
pub mod nexchat;

// AST Query (structural Rust code search via syn parsing)
pub mod ast_query;

// Diagram rendering (DOT/Graphviz to SVG/PNG/PDF)
pub mod diagram;

// Hook testing harness
pub mod hook_error_id;
pub mod hook_test;

// Jupyter & Voila (kernel management, server status, notebook rendering)
pub mod jupyter;

// Test History (cross-session test result tracking — query, flaky detection)
pub mod test_history;

// Station (WebMCP Hub config rail management — build, add-tool, list, export, coverage)
pub mod station;

// Microgram Decision Trees (run, test, catalog, coverage, bench, chain execution)
pub mod microgram;

// Publishing (DOCX → EPUB pipeline, EPUB reader)
pub mod publishing;

// PV Intelligence (competitive analysis — head-to-head, class effects, safety gaps, landscape)
pub mod pv_intelligence;

// Drug entity tools (profile, signals, compare, class members — catalog of 10 drugs)
pub mod drug_tools;

// Pharma company entity tools (profile, signals, pipeline, boxed warnings — 12 companies)
pub mod pharma_tools;

// Generic Processor Framework (∂(σ(μ)) + {ς} — pipeline, boundary, batch)
pub mod processor;
