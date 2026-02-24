//! API route modules

pub mod academy;
pub mod benefit_risk;
pub mod billing;
pub mod brain;
pub mod circles;
pub mod common;
pub mod community;
pub mod core;
pub mod core_api;
pub mod delegation;
pub mod foundation;
pub mod guardian;
pub mod guardian_product;
pub mod guardian_ws;
pub mod health;
pub mod mesh;
pub mod messages;
pub mod pv;
pub mod pvdsl;
pub mod reporting;
pub mod signal;
pub mod skills;
pub mod sos;
pub mod space_pv;
pub mod ventures;
pub mod vigil;
pub mod vigil_sys;
pub mod vigilance;

// Compliance (audit trails, GDPR, export controls, SOC 2)
pub mod compliance;

// Platform ML (model catalog, inference, training, active learning, aggregation)
pub mod platform_ml;

// Admin (user management, system configuration, audit)
pub mod admin;
// Benchmarks (performance metrics, regression tracking)
pub mod benchmarks;
// Career (career pathways, progression tracking)
pub mod career;
// FAERS (FDA Adverse Event Reporting System queries)
pub mod faers;
// Graph Layout (node/edge visualization, force-directed layouts)
pub mod graph_layout;
// Learning (learning paths, progress tracking, assessments)
pub mod learning;
// Marketplace (skill marketplace, listings, transactions)
pub mod marketplace;
// MCP (Model Context Protocol bridge — in-process tool execution)
pub mod mcp;
// Telemetry (system metrics, usage analytics, performance monitoring)
pub mod telemetry;
// Tenant (multi-tenant isolation, tenant management)
pub mod tenant;

// Regulatory Intelligence (FDA Guidance Documents + ICH/CIOMS glossary)
pub mod regulatory_intelligence;
// ICSR (Individual Case Safety Report construction + validation)
pub mod icsr;
