//! API route modules

pub mod academy;
pub mod benefit_risk;
pub mod billing;
pub mod brain;
pub mod circles;
pub mod common;
// Projects & Deliverables (R&D workspaces within circles)
pub mod projects;
// Project Tools (MCP tool integration within project context)
pub mod project_tools;
// Publications & Collaboration (inter-circle research sharing)
pub mod community;
pub mod core;
pub mod core_api;
pub mod delegation;
pub mod dna_ml;
pub mod foundation;
pub mod guardian;
pub mod guardian_product;
pub mod guardian_ws;
pub mod health;
pub mod mesh;
pub mod messages;
pub mod microgram;
pub mod ml_pipeline;
pub mod publications;
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

// Terminal (multi-tenant AI-augmented terminal WebSocket)
pub mod terminal_bridge;
pub mod terminal_ws;

// AI Client (sovereign Claude API client — SSE streaming + tool dispatch)
pub mod ai_client;
// AI Bridge (zero-wiring MCP tool auto-discovery for Claude tool_use)
pub mod ai_bridge;

#[cfg(test)]
mod circles_tests;
#[cfg(test)]
mod project_tools_tests;
#[cfg(test)]
mod projects_tests;
#[cfg(test)]
mod publications_tests;
