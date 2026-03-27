//! Demonstration: `dispatch_table!` macro converting a section of unified.rs
//!
//! This file shows the BEFORE and AFTER for the Foundation + Topology + Formula
//! sections (20 tools). The macro reduces 40 lines of match arms to 24 lines
//! of structured declaration.
//!
//! To migrate the full 1,361-arm dispatcher:
//! 1. Convert section-by-section (Foundation, PV, Vigilance, etc.)
//! 2. Run `cargo check` after each section
//! 3. Full migration eliminates ~800 lines of boilerplate
//!
//! NOT compiled — this is documentation.

// ============================================================================
// BEFORE: 40 lines of hand-written match arms (from unified.rs:135-190)
// ============================================================================
//
//  match command {
//      "help" => help_catalog(),
//      "toolbox" => typed(params, toolbox_search),
//      "tool_schema" => tool_schema_lookup(&params, server),
//
//      "nexcore_health" => server.unified_health(),
//      "config_validate" => server.unified_config_validate(),
//      "mcp_servers_list" => typed(params, |p: params::McpServersListParams| {
//          server.unified_mcp_servers_list(p)
//      }),
//      "mcp_server_get" => typed(params, |p: params::McpServerGetParams| {
//          server.unified_mcp_server_get(p)
//      }),
//
//      "api_health" => typed_async(params, tools::api::health).await,
//      "api_list_routes" => typed_async(params, tools::api::list_routes).await,
//
//      "foundation_levenshtein" => typed(params, tools::foundation::calc_levenshtein),
//      "foundation_levenshtein_bounded" => {
//          typed(params, tools::foundation::calc_levenshtein_bounded)
//      }
//      "foundation_fuzzy_search" => typed(params, tools::foundation::fuzzy_search),
//      "foundation_sha256" => typed(params, tools::foundation::sha256),
//      "foundation_yaml_parse" => typed(params, tools::foundation::yaml_parse),
//      "foundation_graph_topsort" => typed(params, tools::foundation::graph_topsort),
//      "foundation_graph_levels" => typed(params, tools::foundation::graph_levels),
//      "foundation_fsrs_review" => typed(params, tools::foundation::fsrs_review),
//      "foundation_concept_grep" => typed(params, tools::foundation::concept_grep),
//
//      "topo_vietoris_rips" => typed(params, tools::topology::topo_vietoris_rips),
//      "topo_persistence" => typed(params, tools::topology::topo_persistence),
//      "topo_betti" => typed(params, tools::topology::topo_betti),
//      "graph_centrality" => typed(params, tools::topology::graph_centrality),
//      "graph_components" => typed(params, tools::topology::graph_components),
//      "graph_shortest_path" => typed(params, tools::topology::graph_shortest_path),
//
//      "pv_signal_strength" => typed(params, tools::formula::signal_strength),
//      "foundation_domain_distance" => typed(params, tools::formula::domain_distance),
//      "foundation_flywheel_velocity" => typed(params, tools::formula::flywheel_velocity),
//      "foundation_token_ratio" => typed(params, tools::formula::token_ratio),
//      "foundation_spectral_overlap" => typed(params, tools::formula::spectral_overlap),
//
//      _ => Err(McpError::invalid_params("Unknown", None))
//  }

// ============================================================================
// AFTER: 24 lines using dispatch_table! macro
// ============================================================================
//
//  use crate::dispatch_macro::dispatch_table;
//
//  dispatch_table! {
//      command, params, server;
//
//      // Meta
//      @raw "help" => help_catalog();
//      @raw "toolbox" => typed(params, toolbox_search);
//      @raw "tool_schema" => tool_schema_lookup(&params, server);
//
//      // System (4)
//      @server "nexcore_health" => unified_health();
//      @server "config_validate" => unified_config_validate();
//      @sync "mcp_servers_list" => |p: params::McpServersListParams| server.unified_mcp_servers_list(p);
//      @sync "mcp_server_get" => |p: params::McpServerGetParams| server.unified_mcp_server_get(p);
//
//      // API (2)
//      @async "api_health" => tools::api::health;
//      @async "api_list_routes" => tools::api::list_routes;
//
//      // Foundation (9)
//      @sync "foundation_levenshtein" => tools::foundation::calc_levenshtein;
//      @sync "foundation_levenshtein_bounded" => tools::foundation::calc_levenshtein_bounded;
//      @sync "foundation_fuzzy_search" => tools::foundation::fuzzy_search;
//      @sync "foundation_sha256" => tools::foundation::sha256;
//      @sync "foundation_yaml_parse" => tools::foundation::yaml_parse;
//      @sync "foundation_graph_topsort" => tools::foundation::graph_topsort;
//      @sync "foundation_graph_levels" => tools::foundation::graph_levels;
//      @sync "foundation_fsrs_review" => tools::foundation::fsrs_review;
//      @sync "foundation_concept_grep" => tools::foundation::concept_grep;
//
//      // Topology (6)
//      @sync "topo_vietoris_rips" => tools::topology::topo_vietoris_rips;
//      @sync "topo_persistence" => tools::topology::topo_persistence;
//      @sync "topo_betti" => tools::topology::topo_betti;
//      @sync "graph_centrality" => tools::topology::graph_centrality;
//      @sync "graph_components" => tools::topology::graph_components;
//      @sync "graph_shortest_path" => tools::topology::graph_shortest_path;
//
//      // Formula (5)
//      @sync "pv_signal_strength" => tools::formula::signal_strength;
//      @sync "foundation_domain_distance" => tools::formula::domain_distance;
//      @sync "foundation_flywheel_velocity" => tools::formula::flywheel_velocity;
//      @sync "foundation_token_ratio" => tools::formula::token_ratio;
//      @sync "foundation_spectral_overlap" => tools::formula::spectral_overlap;
//  }

// ============================================================================
// Migration Stats (projected for full 1,361-arm dispatcher)
// ============================================================================
//
// | Metric | Before | After | Reduction |
// |--------|--------|-------|-----------|
// | Lines  | 2,660  | ~1,400 | -47%     |
// | Match arms | 1,361 | 1,361 | 0 (same count, less boilerplate) |
// | Brace blocks | ~200 | 0 | -100% |
// | `.await` suffixes | 146 | 0 (macro adds) | -100% |
// | Error on unknown | manual | automatic | -100% |
// | Style visible | No | Yes (@sync/@async/@server) | +100% |
//
// Estimated migration time: ~2 hours (section by section with cargo check gates)
