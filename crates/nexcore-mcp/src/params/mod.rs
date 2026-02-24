pub mod serde_lenient;

// Academy Forge (extract IR, validate academy content)
pub mod academy_forge;
pub use academy_forge::*;

// CLI wrapper parameters (git, gh, systemctl, npm, filesystem)
pub mod fs;
pub mod gh;
pub mod git;
pub mod npm;
pub mod service;

pub mod adventure;
pub mod algovigilance;
pub mod anatomy;
pub mod anatomy_db;
pub mod antitransformer;
pub mod axioms;
pub mod biology;
pub mod brain;
pub mod brain_db;
pub mod brand;
pub mod browser;
pub mod ca;
pub mod caesura;
pub mod cargo;
pub mod ccp;
pub mod cep;
pub mod chemistry;
pub mod claude_fs;
pub mod clearance;
pub mod cognition;
pub mod compendious;
pub mod compliance;
pub mod compositor;
pub mod cortex;
pub mod crew;
pub mod declension;
pub mod dna;
pub mod docs;
pub mod docs_claude;
pub mod dtree;
pub mod edu;
pub mod energy;
pub mod faers;
pub mod forge;
pub mod formula;
pub mod foundation;
pub mod frontend;
pub mod gcloud;
pub mod gsheets;
pub mod guardian;
pub mod hitl;
pub mod hud;
pub mod immunity;
pub mod insight;
pub mod integrity;
pub mod knowledge;
pub mod lab;
pub mod lessons;
pub mod lex_primitiva;
pub mod math;
pub mod measure;
pub mod mesh;
pub mod miner;
pub mod network;
pub mod perplexity;
pub mod pom;
pub mod pv;
pub mod pv_embeddings;
pub mod pvdsl;
pub mod reddit;
pub mod regulatory;
pub mod repl;
pub mod reproductive;
pub mod retrieval;
pub mod ribosome;
pub mod security;
pub mod sentinel;
pub mod skills;
pub mod sop_anatomy;
pub mod sos;
pub mod stem;
pub mod system;
pub mod telemetry;
pub mod trace;
pub mod transcriptase;
pub mod transform;
pub mod trust;
pub mod user;
pub mod validation;
pub mod vigil;
pub mod vigilance;
pub mod viz;
pub mod viz_advanced;
pub mod viz_biologics;
pub mod viz_foundation;
pub mod viz_physics;
pub mod watchtower;
pub mod wolfram;

// Crate Development Framework (scaffold + audit)
pub mod crate_dev;
pub use crate_dev::*;

// Crate X-Ray (deep inspection, CTVP trials, dev goals)
pub mod crate_xray;
pub use crate_xray::*;

// Knowledge Engine (ingest, compress, compile, query, stats)
pub mod knowledge_engine;
pub use knowledge_engine::*;

// New param modules
pub mod domain_primitives;
pub mod edit_distance;
pub mod epidemiology;
pub mod faers_analytics;
pub mod faers_etl;
pub mod fda;
pub mod game_theory;
pub mod ich;
pub mod molecular_weight;
pub mod organize;
pub mod pharos;
pub mod prima;
pub mod primitive_scanner;
pub mod primitive_validation;
pub mod principles;
pub mod registry;
pub mod relay;
pub mod value_mining;
pub mod visual;

// Re-export all param types for flat import paths (crate::params::FooParams)
pub use fs::*;
pub use gh::*;
pub use git::*;
pub use npm::*;
pub use service::*;

pub use adventure::*;
pub use algovigilance::*;
pub use anatomy::*;
pub use antitransformer::*;
pub use axioms::*;
pub use biology::*;
pub use brain::*;
pub use brain_db::*;
pub use brand::*;
pub use browser::*;
pub use ca::*;
pub use caesura::*;
pub use cargo::*;
pub use ccp::*;
pub use cep::*;
pub use chemistry::*;
pub use claude_fs::*;
pub use clearance::*;
pub use cognition::*;
pub use compendious::*;
pub use compliance::*;
pub use compositor::*;
pub use cortex::*;
pub use crew::*;
pub use declension::*;
pub use dna::*;
pub use docs::*;
pub use docs_claude::*;
pub use dtree::*;
pub use edu::*;
pub use energy::*;
pub use faers::*;
pub use fda::*;
pub use forge::*;
pub use formula::*;
pub use foundation::*;
pub use frontend::*;
pub use gcloud::*;
pub use gsheets::*;
pub use guardian::*;
pub use hitl::*;
pub use hud::*;
pub use immunity::*;
pub use insight::*;
pub use integrity::*;
pub use knowledge::*;
pub use lab::*;
pub use lessons::*;
pub use lex_primitiva::*;
pub use math::*;
pub use measure::*;
pub use mesh::*;
pub use miner::*;
pub use network::*;
pub use perplexity::*;
pub use pom::*;
pub use pv::*;
pub use pv_embeddings::*;
pub use pvdsl::*;
pub use reddit::*;
pub use regulatory::*;
pub use repl::*;
pub use reproductive::*;
pub use retrieval::*;
pub use ribosome::*;
pub use security::*;
pub use sentinel::*;
pub use skills::*;
pub use sos::*;
pub use stem::*;
pub use system::*;
pub use telemetry::*;
pub use trace::*;
pub use transcriptase::*;
pub use transform::*;
pub use trust::*;
pub use user::*;
pub use validation::*;
pub use vigil::*;
pub use vigilance::*;
pub use viz::*;
pub use viz_advanced::*;
pub use viz_biologics::*;
pub use viz_foundation::*;
pub use viz_physics::*;
pub use watchtower::*;
pub use wolfram::*;

// New module re-exports
pub use domain_primitives::*;
pub use epidemiology::*;
// edit_distance, game_theory, primitive_scanner are re-export-only modules
// faers_analytics, faers_etl, ich are re-exported through their parent modules (faers, knowledge)
pub use molecular_weight::*;
pub use organize::*;
pub use prima::*;
pub use primitive_validation::*;
pub use principles::*;
pub use registry::*;
pub use relay::*;
pub use value_mining::*;
pub use visual::*;

// Oracle (Bayesian event prediction)
pub mod oracle;
pub use oracle::*;

// AI Engineering Bible Round 2 improvements
pub mod drift;
pub mod rank_fusion;
pub mod rate_limit;
pub use drift::*;
pub use rank_fusion::*;
pub use rate_limit::*;

// AI Engineering Bible Round 3 improvements
pub mod observability;
pub mod security_posture;
pub use observability::*;
pub use security_posture::*;

// GROUNDED (epistemological substrate — uncertainty, evidence chains, confidence gating)
pub mod grounded;
pub use grounded::*;

// Disney Loop (forward-only compound discovery — ρ(t) → ∂(¬σ⁻¹) → ∃(ν) → ρ(t+1))
pub mod disney_loop;
pub use disney_loop::*;

// Digital Highways (infrastructure acceleration — Chatburn 1923)
pub mod highway;
pub use highway::*;

// Tool Routing (deterministic dispatch — σ+→+μ+∂)
pub mod routing;
pub use routing::*;

// Validify (8-gate crate validation)
pub mod validify;
pub use validify::*;

// CTVP (Clinical Trial Validation Paradigm)
pub mod ctvp;
pub use ctvp::*;

// Code Inspection (FDA-inspired audit)
pub mod code_inspect;
pub use code_inspect::*;

// Primitive Coverage (T1 coverage analysis)
pub mod primitive_coverage;
pub use primitive_coverage::*;

// Model Delegation (task→model routing)
pub mod model_delegation;
pub use model_delegation::*;

// Prompt Kinetics (ADME PK model)
pub mod prompt_kinetics;
pub use prompt_kinetics::*;

// Compounding Engine (learning velocity)
pub mod compounding_engine;
pub use compounding_engine::*;

// Polymer (hook pipeline composition)
pub mod polymer;
pub use polymer::*;

// Theory of Vigilance (ToV direct — signal strength, stability shells, epistemic trust)
pub mod tov;
pub use tov::*;

// PV Core (IVF axioms, severity classification)
pub mod pv_core;
pub use pv_core::*;

// HTTP Request (curl replacement)
pub mod http;
pub use http::*;

// BAS Organ Systems (Biological Analogy System)
pub mod nervous;
pub use nervous::*;
pub mod cardiovascular;
pub use cardiovascular::*;
pub mod circulatory;
pub use circulatory::*;
pub mod digestive;
pub use digestive::*;
pub mod lymphatic;
pub use lymphatic::*;
pub mod muscular;
pub use muscular::*;
pub mod phenotype;
pub use phenotype::*;
pub mod respiratory;
pub use respiratory::*;
pub mod skeletal;
pub use skeletal::*;
pub mod urinary;
pub use urinary::*;
pub mod integumentary;
pub use integumentary::*;

// PHAROS — Autonomous Signal Surveillance
pub use pharos::*;

// Kellnr (consolidated from kellnr-mcp: 40 tools — PK, thermo, stats, graph, dtree, surveillance, registry)
pub mod kellnr;
pub use kellnr::*;

// Observatory Phase 9 — Career transitions, learning DAG, graph layout
pub mod career;
pub use career::*;
pub mod graph_layout;
pub use graph_layout::*;
pub mod learning_dag;
pub use learning_dag::*;

// Observatory Personalization (detect, get, set, validate)
pub mod observatory;
pub use observatory::*;

// Stoichiometry (encode/decode concepts as balanced primitive equations)
pub mod stoichiometry;
pub use stoichiometry::*;

// TRIAL Framework (universal experimentation — protocol, power, randomize, interim, safety, endpoint, multiplicity, adapt, report)
pub mod trial;
pub use trial::*;

// The Foundry (dual-pipeline assembly line — validate, cascade, infer, render, VDAG)
pub mod foundry;
pub use foundry::*;

// Chemivigilance (SMILES parsing, descriptors, QSAR, metabolites, SafetyBrief — 15 tools)
pub mod chemivigilance;
pub use chemivigilance::*;

// NMD Surveillance (anti-hallucination pipeline)
pub mod nmd;
pub use nmd::*;

// PV Pharmacokinetics (AUC, clearance, half-life, steady-state, ionization, Michaelis-Menten)
pub mod pk;
pub use pk::*;

// PV Causality Assessment (RUCAM hepatotoxicity, UCAS unified)
pub mod causality;
pub use causality::*;

// PV Temporal Analysis (time-to-onset, challenge assessment, plausibility)
pub mod temporal;
pub use temporal::*;

// QSAR Granular Predictions (mutagenicity, hepatotoxicity, cardiotoxicity, domain assessment)
pub mod qsar;
pub use qsar::*;

// NotebookLM (library, sessions, browser-automated research queries)
pub mod notebooklm;
pub use notebooklm::*;

// Cloud Intelligence (17 tools — query, infrastructure, reasoning)
pub mod cloud;
pub use cloud::*;

// Rust Development (error types, derive advisor, match gen, borrow explain)
pub mod rust_dev;
pub use rust_dev::*;

// Topology (TDA + graph analysis)
pub mod topology;
pub use topology::*;

// Zeta function telescope pipeline (LMFDB, batch, scaling, Cayley, operator hunt)
pub mod zeta;
pub use zeta::*;

// Signal detection pipeline (PRR/ROR/IC/EBGM, Evans thresholds, relay)
pub mod signal_pipeline;
pub use signal_pipeline::*;

// Preemptive PV: 3-tier signal detection (Reactive → Predictive → Preemptive)
pub mod preemptive_pv;
pub use preemptive_pv::*;

// OpenFDA: live FDA database access (drugs, devices, food, substances)
pub mod openfda;
pub use openfda::*;

// Compound Registry: chemical compound resolution and caching
pub mod compound_registry;
pub use compound_registry::*;

// FHIR R4: pharmacovigilance-focused HL7 resource parsing and signal extraction
pub mod fhir;
pub use fhir::*;

// Retrocasting: retrospective signal-to-structure analysis and ML training
pub mod retrocasting;
pub use retrocasting::*;

// Engram: unified knowledge store (search, decay, ingest, duplicates)
pub mod engram;
pub use engram::*;

// Ghost: privacy-by-design pseudonymization, PII detection, anonymization boundaries
pub mod ghost;
pub use ghost::*;

// Pharma R&D: predictive taxonomy, cross-domain transfer, Chomsky classification
pub mod pharma_rd;
pub use pharma_rd::*;

// Combinatorics: Dudeney-derived algorithms (Catalan, derangement, Josephus, grid paths)
pub mod combinatorics;
pub use combinatorics::*;

// Theory of Vigilance (Grounded): signal strength, safety margin, stability shells
pub mod tov_grounded;
pub use tov_grounded::*;

// Statemind: DNA pipeline word analysis and constellation resonance
pub mod statemind;
pub use statemind::*;

// Reason: causal DAG construction, inference, counterfactual evaluation
pub mod reason;
pub use reason::*;

// Word: binary word trait algebra (popcount, rotation, GCD, alignment)
pub mod word;
pub use word::*;

// Harm Taxonomy: ToV §9 harm classification (types A-I, conservation laws, axiom connections)
pub mod harm_taxonomy;
pub use harm_taxonomy::*;

// Antibodies: adaptive immune recognition (affinity, Ig class, response classification)
pub mod antibodies;
pub use antibodies::*;

// Jeopardy: game theory strategy engine (wagers, buzz decisions, compound velocity)
pub mod jeopardy;
pub use jeopardy::*;

// Audio: sample conversion, spec computation, codec catalog, pan law
pub mod audio;
pub use audio::*;

// Compilation Space: 7D transform algebra (axes, points, transforms, chains)
pub mod compilation_space;
pub use compilation_space::*;

// Pharmacovigilance Taxonomy (4-tier WHO-grounded PV concept encoder)
pub mod pharmacovigilance;
pub use pharmacovigilance::*;

// Vault (AES-256-GCM encryption, PBKDF2 key derivation)
pub mod vault;
pub use vault::*;

// Build Orchestrator (CI/CD pipeline management)
pub mod build_orchestrator;
pub use build_orchestrator::*;

// Skills Engine (advanced skill analysis and quality metrics)
pub mod skills_engine;
pub use skills_engine::*;

// NCBI (National Center for Biotechnology Information — ESearch, ESummary, EFetch, ELink)
pub mod ncbi;
pub use ncbi::*;
