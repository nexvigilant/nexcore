// ========================================================================
// System Tools (4)
// ========================================================================

#[tool(
    description = "Health check for NexCore MCP server. Returns version, tool count, and status."
)]
pub async fn nexcore_health(&self) -> Result<CallToolResult, McpError> {
    let tool_count = self.tool_router.list_all().len();
    let health =
        serde_json::json!({
        "status": "healthy",
        "server": "nexcore-mcp",
        "version": env!("CARGO_PKG_VERSION"),
        "tool_count": tool_count,
        "domains": ["foundation", "pv", "vigilance", "skills", "guidelines", "faers", "gcloud", "wolfram", "principles", "hooks"]
    });
    Ok(
        CallToolResult::success(
            vec![
                rmcp::model::Content::text(
                    serde_json::to_string_pretty(&health).unwrap_or_default()
                )
            ]
        )
    )
}

#[tool(
    description = "Validate Claude Code configuration. Checks for security issues, path validity, and configuration errors. Returns validation status and any warnings."
)]
pub async fn config_validate(&self) -> Result<CallToolResult, McpError> {
    use nexcore_config::Validate;

    if let Some(cfg) = &self.config {
        match cfg.validate() {
            Ok(_) =>
                Ok(
                    CallToolResult::success(
                        vec![
                            rmcp::model::Content::text(
                                "✅ Configuration valid:\n\
                                 • No security issues detected\n\
                                 • All paths validated\n\
                                 • MCP servers properly configured".to_string()
                            )
                        ]
                    )
                ),
            Err(e) =>
                Ok(
                    CallToolResult::success(
                        vec![
                            rmcp::model::Content::text(
                                format!("⚠️  Configuration warnings:\n{}", e)
                            )
                        ]
                    )
                ),
        }
    } else {
        Ok(
            CallToolResult::success(
                vec![
                    rmcp::model::Content::text(
                        "ℹ️  No configuration loaded\n\
                         Configuration file not found at:\n\
                         • ~/nexcore/config.toml\n\
                         • ~/.claude.json".to_string()
                    )
                ]
            )
        )
    }
}

#[tool(
    description = "List all configured MCP servers. Returns server names, commands, and arguments for both global and optionally project-specific servers."
)]
pub async fn mcp_servers_list(
    &self,
    Parameters(params): Parameters<params::system::McpServersListParams>
) -> Result<CallToolResult, McpError> {
    if let Some(cfg) = &self.config {
        let mut servers: Vec<serde_json::Value> = cfg.mcp_servers
            .iter()
            .map(|(name, config)| {
                let (command, args) = match config {
                    nexcore_config::McpServerConfig::Stdio { command, args, .. } => {
                        (command.clone(), args.clone())
                    }
                };
                serde_json::json!({
                    "name": name,
                    "scope": "global",
                    "command": command,
                    "args": args
                })
            })
            .collect();

        if params.include_projects {
            for (path, project) in &cfg.projects {
                for (name, config) in &project.mcp_servers {
                    let (command, args) = match config {
                        nexcore_config::McpServerConfig::Stdio { command, args, .. } => {
                            (command.clone(), args.clone())
                        }
                    };
                    servers.push(
                        serde_json::json!({
                        "name": name,
                        "scope": "project",
                        "project": path.display().to_string(),
                        "command": command,
                        "args": args
                    })
                    );
                }
            }
        }

        let result =
            serde_json::json!({
            "total": servers.len(),
            "servers": servers
        });

        Ok(
            CallToolResult::success(
                vec![
                    rmcp::model::Content::text(
                        serde_json::to_string_pretty(&result).unwrap_or_default()
                    )
                ]
            )
        )
    } else {
        Ok(
            CallToolResult::success(
                vec![
                    rmcp::model::Content::text(
                        "ℹ️  No configuration loaded - cannot list MCP servers".to_string()
                    )
                ]
            )
        )
    }
}

#[tool(
    description = "Get configuration details for a specific MCP server by name. Returns command, args, and environment variables."
)]
pub async fn mcp_server_get(
    &self,
    Parameters(params): Parameters<params::system::McpServerGetParams>
) -> Result<CallToolResult, McpError> {
    if let Some(cfg) = &self.config {
        if let Some(server) = cfg.mcp_servers.get(&params.name) {
            let result = match server {
                nexcore_config::McpServerConfig::Stdio { command, args, env } => {
                    serde_json::json!({
                        "name": params.name,
                        "type": "stdio",
                        "command": command,
                        "args": args,
                        "env": env
                    })
                }
            };
            Ok(
                CallToolResult::success(
                    vec![
                        rmcp::model::Content::text(
                            serde_json::to_string_pretty(&result).unwrap_or_default()
                        )
                    ]
                )
            )
        } else {
            Ok(
                CallToolResult::success(
                    vec![
                        rmcp::model::Content::text(
                            format!("❌ MCP server '{}' not found", params.name)
                        )
                    ]
                )
            )
        }
    } else {
        Ok(
            CallToolResult::success(
                vec![rmcp::model::Content::text("ℹ️  No configuration loaded".to_string())]
            )
        )
    }
}

// ========================================================================
// Foundation Tools (7)
// ========================================================================

#[tool(
    description = "Calculate Levenshtein edit distance and similarity between two strings. Returns distance, similarity (0-1), and string lengths. 63x faster than Python."
)]
pub async fn foundation_levenshtein(
    &self,
    Parameters(params): Parameters<params::foundation::LevenshteinParams>
) -> Result<CallToolResult, McpError> {
    tools::foundation::calc_levenshtein(params)
}

#[tool(
    description = "Calculate bounded Levenshtein distance with early termination. Returns distance if within max_distance, null if exceeded. Faster than unbounded for filtering large candidate sets."
)]
pub async fn foundation_levenshtein_bounded(
    &self,
    Parameters(params): Parameters<params::foundation::LevenshteinBoundedParams>
) -> Result<CallToolResult, McpError> {
    tools::foundation::calc_levenshtein_bounded(params)
}

#[tool(
    description = "Batch fuzzy search: find best matches for a query against candidates. Returns matches sorted by similarity (descending)."
)]
pub async fn foundation_fuzzy_search(
    &self,
    Parameters(params): Parameters<params::foundation::FuzzySearchParams>
) -> Result<CallToolResult, McpError> {
    tools::foundation::fuzzy_search(params)
}

#[tool(
    description = "Calculate SHA-256 hash of input string. Returns hex-encoded hash. 20x faster than Python."
)]
pub async fn foundation_sha256(
    &self,
    Parameters(params): Parameters<params::foundation::Sha256Params>
) -> Result<CallToolResult, McpError> {
    tools::foundation::sha256(params)
}

#[tool(
    description = "Parse YAML content to JSON. 7x faster than Python. Returns parsed JSON or error message."
)]
pub async fn foundation_yaml_parse(
    &self,
    Parameters(params): Parameters<params::foundation::YamlParseParams>
) -> Result<CallToolResult, McpError> {
    tools::foundation::yaml_parse(params)
}

#[tool(
    description = "Topological sort of a directed acyclic graph (DAG). Returns nodes in dependency order. Detects cycles."
)]
pub async fn foundation_graph_topsort(
    &self,
    Parameters(params): Parameters<params::foundation::GraphTopsortParams>
) -> Result<CallToolResult, McpError> {
    tools::foundation::graph_topsort(params)
}

#[tool(
    description = "Compute parallel execution levels for a DAG. Groups independent nodes that can run concurrently."
)]
pub async fn foundation_graph_levels(
    &self,
    Parameters(params): Parameters<params::foundation::GraphLevelsParams>
) -> Result<CallToolResult, McpError> {
    tools::foundation::graph_levels(params)
}

#[tool(
    description = "FSRS spaced repetition: calculate next review interval based on current state and rating."
)]
pub async fn foundation_fsrs_review(
    &self,
    Parameters(params): Parameters<params::foundation::FsrsReviewParams>
) -> Result<CallToolResult, McpError> {
    tools::foundation::fsrs_review(params)
}

#[tool(
    description = "Expand a concept into all deterministic search variants: case forms (lower/UPPER/Title/camelCase/snake_case/kebab-case), singular/plural, abbreviation, truncated stems, and optional section markers. Returns patterns and combined regex. 100% deterministic, zero I/O."
)]
pub async fn foundation_concept_grep(
    &self,
    Parameters(params): Parameters<params::foundation::ConceptGrepParams>
) -> Result<CallToolResult, McpError> {
    tools::foundation::concept_grep(params)
}

// ========================================================================
// Formula-Derived Tools (5) \u2014 KU extraction pipeline \u2192 MCP tools
// ========================================================================

#[tool(
    description = "Compute PV signal strength composite: S = Unexpectedness \uI00d7 Robustness \u00d7 Therapeutic_importance. All inputs in [0,1]. Higher = stronger signal. Classifies as strong/moderate/weak/negligible."
)]
pub async fn pv_signal_strength(
    &self,
    Parameters(params): Parameters<params::formula::SignalStrengthParams>
) -> Result<CallToolResult, McpError> {
    tools::formula::signal_strength(params)
}

#[tool(
    description = "Compute domain distance via weighted primitive overlap. Distance in [0,1] where 0=identical, 1=maximally distant. Uses tier-weighted Jaccard: d = 1 - (w1\u00d7T1 + w2\u00d7T2 + w3\u00d7T3). Classifies domains as very_close to very_distant."
)]
pub async fn foundation_domain_distance(
    &self,
    Parameters(params): Parameters<params::formula::DomainDistanceParams>
) -> Result<CallToolResult, McpError> {
    tools::formula::domain_distance(params)
}

#[tool(
    description = "Compute flywheel velocity from paired failure/fix timestamps (ms since epoch). velocity = 1/avg(fix-failure). Target: < 24 hours. Classifies as exceptional/target/acceptable/slow."
)]
pub async fn foundation_flywheel_velocity(
    &self,
    Parameters(params): Parameters<params::formula::FlywheelVelocityParams>
) -> Result<CallToolResult, McpError> {
    tools::formula::flywheel_velocity(params)
}

#[tool(
    description = "Compute LLM token-to-operation ratio for code generation efficiency. ratio = tokens/operations. Target: \u2264 1.0. Lower = more efficient. Classifies as excellent/target/verbose/wasteful."
)]
pub async fn foundation_token_ratio(
    &self,
    Parameters(params): Parameters<params::formula::TokenRatioParams>
) -> Result<CallToolResult, McpError> {
    tools::formula::token_ratio(params)
}

#[tool(
    description = "Compute spectral overlap (cosine similarity) between two feature vectors. overlap = (a\u00b7b)/(\u2016a\u2016\u00d7\u2016b\u2016) in [-1,1]. For autocorrelation spectra, typically [0,1]. Classifies as highly_similar to anti_correlated."
)]
pub async fn foundation_spectral_overlap(
    &self,
    Parameters(params): Parameters<params::formula::SpectralOverlapParams>
) -> Result<CallToolResult, McpError> {
    tools::formula::spectral_overlap(params)
}

// ========================================================================
// Lex Primitiva Tools (4) - T1 Symbolic Foundation
// ========================================================================

#[tool(
    description = "List all 15 Lex Primitiva symbols (\u03c3 \u03bc \u03c2 \u03c1 \u2205 \u2202 \u03bd \u2203 \u03c0 \u2192 \u03ba N \u03bb \u221d \u03a3). The irreducible T1 primitives that ground all higher-tier types."
)]
pub async fn lex_primitiva_list(
    &self,
    Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaListParams>
) -> Result<CallToolResult, McpError> {
    tools::lex_primitiva::list_primitives(params)
}

#[tool(
    description = "Get details about a specific Lex Primitiva by name or symbol. Returns description, Rust manifestation, and tier."
)]
pub async fn lex_primitiva_get(
    &self,
    Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaGetParams>
) -> Result<CallToolResult, McpError> {
    tools::lex_primitiva::get_primitive(params)
}

#[tool(
    description = "Classify a type's grounding tier (T1-Universal, T2-Primitive, T2-Composite, T3-DomainSpecific) based on its primitive composition."
)]
pub async fn lex_primitiva_tier(
    &self,
    Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaTierParams>
) -> Result<CallToolResult, McpError> {
    tools::lex_primitiva::classify_tier(params)
}

#[tool(
    description = "Get the primitive composition for a grounded type. Shows which T1 primitives compose the type, the dominant primitive, and confidence score."
)]
pub async fn lex_primitiva_composition(
    &self,
    Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaCompositionParams>
) -> Result<CallToolResult, McpError> {
    tools::lex_primitiva::get_composition(params)
}

#[tool(
    description = "Reverse compose: given T1 primitives, synthesize upward through the tier DAG to discover patterns, interactions, dominant primitive, tier classification, and completion suggestions. Input primitive names like 'Boundary', 'Comparison'."
)]
pub async fn lex_primitiva_reverse_compose(
    &self,
    Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaReverseComposeParams>
) -> Result<CallToolResult, McpError> {
    tools::lex_primitiva::reverse_compose(params)
}

#[tool(
    description = "Reverse lookup: find grounded types whose primitive composition matches a set of T1 primitives. Supports 'exact', 'superset' (default), and 'subset' match modes."
)]
pub async fn lex_primitiva_reverse_lookup(
    &self,
    Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaReverseLookupParams>
) -> Result<CallToolResult, McpError> {
    tools::lex_primitiva::reverse_lookup(params)
}

#[tool(
    description = "Compute the molecular weight of a word/concept from its T1 primitive decomposition (Algorithm A76). Provide primitive names and get Shannon information-theoretic weight in daltons, transfer class, and predicted cross-domain transfer confidence. MW anti-correlates with transferability: light words transfer easily, heavy words are domain-locked."
)]
pub async fn lex_primitiva_molecular_weight(
    &self,
    Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaMolecularWeightParams>
) -> Result<CallToolResult, McpError> {
    tools::lex_primitiva::molecular_weight(params)
}

#[tool(
    description = "Dominant shift (phase transition) analysis: detect whether adding a new T1 primitive to a base composition changes the dominant primitive. Returns old/new dominant, tier transition, coherence delta, and a shifted flag. A 'shifted = true' result signals a phase transition — the structural character of the composition has reorganized. Example: adding 'Boundary' to ['Comparison'] triggers the Gatekeeper pattern, shifting dominance from Comparison to Boundary."
)]
pub async fn lex_primitiva_dominant_shift(
    &self,
    Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaDominantShiftParams>
) -> Result<CallToolResult, McpError> {
    tools::lex_primitiva::dominant_shift(params)
}

// ========================================================================
// Principles Knowledge Base Tools (3)
// ========================================================================

#[tool(
    description = "List all available principles in the knowledge base. Returns principle IDs, titles, and file paths. Includes Ray Dalio's Principles, KISS, First Principles, etc."
)]
pub async fn principles_list(
    &self,
    Parameters(params): Parameters<params::principles::PrinciplesListParams>
) -> Result<CallToolResult, McpError> {
    tools::principles::list_principles(params)
}

#[tool(
    description = "Get a specific principle by name. Returns the full markdown content. Use 'dalio-principles' for Ray Dalio's decision-making framework, 'kiss' for Keep It Simple, 'first-principles' for foundational reasoning."
)]
pub async fn principles_get(
    &self,
    Parameters(params): Parameters<params::principles::PrinciplesGetParams>
) -> Result<CallToolResult, McpError> {
    tools::principles::get_principle(params)
}

#[tool(
    description = "Search principles by keyword. Returns matching sections with context. Use for finding relevant decision-making guidance. Examples: 'open-minded', 'meritocracy', 'believability', 'expected value'"
)]
pub async fn principles_search(
    &self,
    Parameters(params): Parameters<params::principles::PrinciplesSearchParams>
) -> Result<CallToolResult, McpError> {
    tools::principles::search_principles(params)
}

// ========================================================================
// Edit Distance Framework (4 tools)
// ========================================================================

#[tool(
    description = "Compute edit distance between two strings. Supports algorithms: 'levenshtein' (default), 'damerau' (adds transposition), 'lcs' (indel-only). Returns distance, similarity, and string lengths."
)]
pub async fn edit_distance_compute(
    &self,
    Parameters(params): Parameters<params::edit_distance::EditDistanceParams>
) -> Result<CallToolResult, McpError> {
    tools::edit_distance::edit_distance_compute(params)
}

#[tool(
    description = "Compute string similarity with threshold check. Returns similarity score (0-1), distance, and whether similarity meets threshold (default 0.8). Useful for PV drug name matching."
)]
pub async fn edit_distance_similarity(
    &self,
    Parameters(params): Parameters<params::edit_distance::EditDistanceSimilarityParams>
) -> Result<CallToolResult, McpError> {
    tools::edit_distance::edit_distance_similarity(params)
}

#[tool(
    description = "Compute edit distance with full traceback: returns the sequence of insert/delete/substitute operations to transform source into target. Uses full-matrix DP solver."
)]
pub async fn edit_distance_traceback(
    &self,
    Parameters(params): Parameters<params::edit_distance::EditDistanceTracebackParams>
) -> Result<CallToolResult, McpError> {
    tools::edit_distance::edit_distance_traceback(params)
}

#[tool(
    description = "Look up cross-domain transfer confidence for edit distance concepts. Pre-computed maps: text/unicode \u2194 bioinformatics/dna, spell-checking, nlp/tokens, pharmacovigilance; bioinformatics/dna \u2194 music/melody. Returns structural/functional/contextual scores and caveats."
)]
pub async fn edit_distance_transfer(
    &self,
    Parameters(params): Parameters<params::edit_distance::EditDistanceTransferParams>
) -> Result<CallToolResult, McpError> {
    tools::edit_distance::edit_distance_transfer(params)
}

#[tool(
    description = "Batch edit distance: compare a query string against multiple candidates. Returns matches sorted by similarity (descending). Supports min_similarity filter, limit, and algorithm selection (levenshtein/damerau/lcs)."
)]
pub async fn edit_distance_batch(
    &self,
    Parameters(params): Parameters<params::edit_distance::EditDistanceBatchParams>
) -> Result<CallToolResult, McpError> {
    tools::edit_distance::edit_distance_batch(params)
}
