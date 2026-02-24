//! NexVigilant Core API Library
//!
//! Provides the core router and state for the nexcore-api.

pub mod audit;
pub mod auth;
pub mod core_types;
pub mod mcp_bridge;
pub mod metering;
pub mod openapi_compat;
pub mod persistence;
pub mod routes;
pub mod subscription_store;
pub mod tenant;

use axum::{
    Router,
    body::Body,
    extract::{FromRef, Query, Request},
    http::{Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::persistence::Persistence;
use crate::persistence::firestore::{FirestorePersistence, MockPersistence};

/// Shared API state
pub struct ApiState {
    pub persistence: Arc<Persistence>,
    pub skill_state: routes::skills::SkillAppState,
}

impl Clone for ApiState {
    fn clone(&self) -> Self {
        Self {
            persistence: Arc::clone(&self.persistence),
            skill_state: self.skill_state.clone(),
        }
    }
}

impl FromRef<ApiState> for routes::skills::SkillAppState {
    fn from_ref(state: &ApiState) -> Self {
        state.skill_state.clone()
    }
}

/// Build the Axum application with all routes
pub fn build_app(state: ApiState) -> Router {
    eprintln!("[DEBUG] Setting up CORS...");
    let cors = setup_cors();
    eprintln!("[DEBUG] Setting up API routes...");
    let api_routes = setup_api_routes(state.clone());

    // Rate limit state
    let rate_limit = Arc::new(RateLimitState::new(
        std::env::var("RATE_LIMIT_RPS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(250),
    ));

    eprintln!("[DEBUG] Merging Scalar/OpenAPI...");
    let app = Router::new()
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
        .route("/openapi.json", get(openapi_json_handler))
        .nest("/health", routes::health::router())
        .nest("/api/v1", api_routes)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(response_headers_layer))
        .layer(middleware::from_fn(audit::audit_layer))
        .layer(CompressionLayer::new())
        .layer(axum::extract::DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(middleware::from_fn(move |req, next| {
            let limiter = Arc::clone(&rate_limit);
            rate_limit_middleware(limiter, req, next)
        }))
        .layer(cors);
    eprintln!("[DEBUG] build_app complete.");
    app
}

fn setup_cors() -> CorsLayer {
    let allowed_origins: Vec<_> = std::env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:3030,http://127.0.0.1:3030,http://localhost:9002".into())
        .split(',')
        .filter_map(|s| s.trim().parse::<axum::http::HeaderValue>().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
}

fn setup_api_routes(state: ApiState) -> Router<ApiState> {
    Router::new()
        .nest("/foundation", routes::foundation::router())
        .nest("/pv", routes::pv::router())
        .nest("/vigilance", routes::vigilance::router())
        .nest("/academy", routes::academy::router())
        .nest(
            "/community",
            routes::community::router()
                .nest("/circles", routes::circles::router())
                .nest("/messages", routes::messages::router()),
        )
        .nest("/ventures", routes::ventures::router())
        .nest("/guardian", routes::guardian::router())
        .nest("/vigil", routes::vigil::router())
        .nest("/vigil-sys", routes::vigil_sys::router())
        .nest("/skills", routes::skills::router())
        .nest("/brain", routes::brain::router())
        .nest("/core", routes::core_api::router())
        .nest("/pvdsl", routes::pvdsl::router())
        .nest("/reporting", routes::reporting::router())
        .nest("/signal", routes::signal::router())
        .nest("/benefit-risk", routes::benefit_risk::router())
        .nest("/sos", routes::sos::router())
        .nest("/mesh", routes::mesh::router())
        .nest("/guardian-product", routes::guardian_product::router())
        .nest("/compliance", routes::compliance::router())
        .nest("/ml", routes::platform_ml::router())
        .nest("/admin", routes::admin::router())
        .nest("/benchmarks", routes::benchmarks::router())
        .nest("/career", routes::career::router())
        .nest("/faers", routes::faers::router())
        .nest("/regulatory", routes::regulatory_intelligence::router())
        .nest("/icsr", routes::icsr::router())
        .nest("/graph-layout", routes::graph_layout::router())
        .nest("/learning", routes::learning::router())
        .nest("/marketplace", routes::marketplace::router())
        .nest("/mcp", routes::mcp::router())
        .nest("/telemetry", routes::telemetry::router())
        .nest("/tenant", routes::tenant::router())
        .route(
            "/guardian/ws/bridge",
            get(routes::guardian_ws::ws_bridge_handler),
        )
        .layer(middleware::from_fn(metering::metering_layer))
        .layer(middleware::from_fn(auth::require_api_key))
        // Billing routes outside auth — checkout is pre-auth, webhook uses Stripe signature
        .nest("/billing", routes::billing::router())
        .with_state(state)
}

// ── Rate Limiter ──────────────────────────

struct RateLimitState {
    count: AtomicU64,
    window_start: AtomicU64,
    max_rps: u64,
}

impl RateLimitState {
    fn new(max_rps: u64) -> Self {
        Self {
            count: AtomicU64::new(0),
            window_start: AtomicU64::new(current_epoch_secs()),
            max_rps,
        }
    }

    fn check(&self) -> bool {
        let now = current_epoch_secs();
        let window = self.window_start.load(Ordering::Relaxed);
        if now != window {
            self.window_start.store(now, Ordering::Relaxed);
            self.count.store(1, Ordering::Relaxed);
            return true;
        }
        let prev = self.count.fetch_add(1, Ordering::Relaxed);
        prev < self.max_rps
    }
}

fn current_epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

async fn rate_limit_middleware(
    limiter: Arc<RateLimitState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if limiter.check() {
        next.run(req).await
    } else {
        (
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(serde_json::json!({
                "code": "RATE_LIMITED",
                "message": "Too many requests. Try again in 1 second."
            })),
        )
            .into_response()
    }
}

// ── Response Headers ─────────────────────────

/// Middleware that adds standard NexVigilant response headers to every response.
///
/// - `X-NexVigilant-Version`: API version (from `NEXVIGILANT_VERSION` env or `"1.0.0"`)
/// - `X-Request-Id`: Unique request identifier for tracing
async fn response_headers_layer(req: Request<Body>, next: Next) -> Response {
    let request_id = nexcore_id::NexId::v4().to_string();
    let mut response = next.run(req).await;

    let headers = response.headers_mut();

    static VERSION: OnceLock<String> = OnceLock::new();
    let version = VERSION.get_or_init(|| {
        std::env::var("NEXVIGILANT_VERSION").unwrap_or_else(|_| "1.0.0".to_string())
    });

    if let Ok(v) = axum::http::HeaderValue::from_str(version) {
        headers.insert("x-nexvigilant-version", v);
    }
    if let Ok(v) = axum::http::HeaderValue::from_str(&request_id) {
        headers.insert("x-request-id", v);
    }

    response
}

// ── OpenAPI spec endpoint ──────────────────

/// Query parameters accepted by [`openapi_json_handler`].
#[derive(Debug, Deserialize)]
struct OpenApiQuery {
    /// Request a downconverted OpenAPI 3.0.3 spec by passing `?version=3.0`.
    /// Any other value (or omitting the parameter entirely) returns the native
    /// OpenAPI 3.1.0 spec emitted by utoipa.
    version: Option<String>,
}

/// Serve the OpenAPI specification.
///
/// - `GET /openapi.json` — returns the native OpenAPI 3.1.0 spec.
/// - `GET /openapi.json?version=3.0` — returns a downconverted OpenAPI 3.0.3
///   spec compatible with progenitor and other 3.0-only client generators.
async fn openapi_json_handler(Query(params): Query<OpenApiQuery>) -> Response {
    let native = match serde_json::to_value(ApiDoc::openapi()) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({
                    "code": "SPEC_SERIALIZATION_ERROR",
                    "message": e.to_string()
                })),
            )
                .into_response();
        }
    };

    let spec = match params.version.as_deref() {
        Some("3.0") => openapi_compat::downconvert_31_to_30(native),
        _ => native,
    };

    axum::Json(spec).into_response()
}

/// API documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "NexVigilant Core API",
        version = "1.0.0",
        description = "NexVigilant Core (NexCore) REST API — The Vigilance Kernel for pharmacovigilance",
        license(name = "MIT"),
        contact(name = "NexVigilant Team", email = "team@nexvigilant.com")
    ),
    paths(
        routes::foundation::levenshtein_handler,
        routes::foundation::levenshtein_bounded_handler,
        routes::foundation::fuzzy_search_handler,
        routes::foundation::sha256_handler,
        routes::foundation::yaml_parse_handler,
        routes::foundation::graph_topsort_handler,
        routes::foundation::graph_levels_handler,
        routes::foundation::fsrs_review_handler,
        // Academy
        routes::academy::list_courses,
        routes::academy::list_enrollments,
        routes::academy::list_ksb_domains,
        routes::academy::list_pathways,
        routes::academy::enroll,
        routes::pv::signal_complete,
        routes::pv::signal_prr,
        routes::pv::signal_ror,
        routes::pv::signal_ic,
        routes::pv::signal_ebgm,
        routes::pv::chi_square,
        routes::pv::naranjo,
        routes::pv::naranjo_full,
        routes::pv::who_umc,
        routes::pv::who_umc_full,
        routes::pv::rucam,
        routes::pv::seriousness,
        routes::pv::expectedness,
        routes::pv::combined,
        routes::pv::ucas,
        routes::vigilance::safety_margin,
        routes::vigilance::risk_score,
        routes::vigilance::harm_types,
        routes::vigilance::map_to_tov,
        routes::guardian::tick,
        routes::guardian::evaluate_pv,
        routes::guardian::status,
        routes::guardian::reset,
        routes::guardian::update_threshold,
        routes::guardian::pause,
        routes::guardian::resume,
        routes::vigil::status,
        routes::vigil::health,
        routes::vigil::emit_event,
        routes::vigil::memory_search,
        routes::vigil::memory_stats,
        routes::vigil::llm_stats,
        routes::skills::list,
        routes::skills::get_skill,
        routes::skills::validate,
        routes::skills::scan,
        routes::skills::taxonomy_list,
        routes::skills::taxonomy_query,
        routes::skills::execute_skill,
        routes::skills::get_schema,
        routes::brain::session_create,
        routes::brain::session_list,
        routes::brain::session_load,
        routes::brain::artifact_save,
        routes::brain::artifact_get,
        routes::brain::artifact_resolve,
        routes::brain::code_tracker_track,
        routes::brain::code_tracker_changed,
        routes::pvdsl::compile,
        routes::pvdsl::execute,
        routes::pvdsl::eval,
        routes::pvdsl::functions,
        // Community
        routes::community::list_posts,
        routes::community::create_post,
        // Circles
        routes::circles::list_circles,
        routes::circles::join_circle,
        // Messages
        routes::messages::list_messages,
        routes::messages::send_message,
        // Ventures
        routes::ventures::list_inquiries,
        routes::ventures::submit_inquiry,
        routes::ventures::update_status,
        routes::signal::detect,
        routes::signal::batch,
        routes::signal::thresholds,
        routes::reporting::generate_report,
        routes::reporting::list_reports,
        routes::reporting::timeline,
        routes::reporting::reportability,
        routes::benefit_risk::qbri_compute,
        routes::mesh::mesh_health,
        routes::mesh::mesh_topology,
        routes::mesh::mesh_simulate,
        routes::mesh::mesh_route_quality,
        routes::mesh::mesh_grounding,
        routes::mesh::mesh_node_info,
        routes::core_api::analyze_handler,
        routes::health::health,
        routes::health::ready,
        routes::sos::create_machine,
        routes::sos::execute_transition,
        routes::sos::get_state,
        routes::sos::get_history,
        routes::sos::validate_spec,
        routes::sos::list_machines,
        // Compliance
        routes::compliance::record_audit_event,
        routes::compliance::query_audit_trail,
        routes::compliance::list_gdpr_requests,
        routes::compliance::create_gdpr_request,
        routes::compliance::get_consent_records,
        routes::compliance::update_consent,
        routes::compliance::screen_export,
        routes::compliance::get_soc2_scorecard,
        // Platform ML
        routes::platform_ml::predict,
        routes::platform_ml::list_models,
        routes::platform_ml::get_model_benchmark,
        routes::platform_ml::trigger_training,
        routes::platform_ml::get_training_status,
        routes::platform_ml::get_al_suggestions,
        routes::platform_ml::get_aggregation_stats,
        // Admin
        routes::admin::list_users,
        routes::admin::get_user,
        routes::admin::update_user_role,
        routes::admin::get_stats,
        routes::admin::list_content,
        routes::admin::delete_content,
        routes::admin::list_flagged_posts,
        routes::admin::moderate_post,
        // Benchmarks
        routes::benchmarks::get_benchmarks,
        routes::benchmarks::get_platform_aggregates,
        // Career
        routes::career::transitions,
        // FAERS
        routes::faers::search,
        routes::faers::drug_events,
        routes::faers::signal_check,
        routes::faers::signal_graph,
        // Regulatory Intelligence (FDA Guidance + ICH Glossary)
        routes::regulatory_intelligence::guidance_search,
        routes::regulatory_intelligence::guidance_get,
        routes::regulatory_intelligence::ich_glossary_search,
        routes::regulatory_intelligence::ich_glossary_lookup,
        // ICSR (Individual Case Safety Report)
        routes::icsr::icsr_build,
        routes::icsr::icsr_validate,
        // Graph Layout
        routes::graph_layout::converge_layout,
        // Learning
        routes::learning::resolve_dag,
        // Marketplace
        routes::marketplace::search_experts,
        routes::marketplace::recommend_experts,
        routes::marketplace::create_engagement,
        // MCP Bridge
        routes::mcp::call_mcp_tool,
        routes::mcp::get_usage_stats,
        // Telemetry
        routes::telemetry::ingest_event,
        routes::telemetry::list_events,
        routes::telemetry::get_summary,
        // Tenant
        routes::tenant::get_current_tenant,
        routes::tenant::get_tenant_limits,
        routes::tenant::list_tiers,
        routes::tenant::provision_tenant,
    ),
    components(schemas(
        routes::core_api::AnalyzeRequest,
        routes::core_api::AnalyzeResponse,
        /*
        // Academy
        routes::academy::Course,
        routes::academy::Enrollment,
        routes::academy::EnrollRequest,
        routes::academy::KsbDomainSummary,
        routes::academy::LearningPathway,
        routes::academy::PathwayNode,
        // Community
        routes::community::Post,
        routes::community::CreatePostRequest,
        // Circles
        routes::circles::Circle,
        routes::circles::JoinRequest,
        // Messages
        routes::messages::Message,
        routes::messages::SendMessageRequest,
        // Ventures
        routes::ventures::PartnershipInquiry,
        routes::ventures::PartnershipRequest,
        routes::ventures::UpdateInquiryStatusRequest,
        routes::foundation::LevenshteinRequest,
        routes::foundation::LevenshteinResponse,
        routes::foundation::LevenshteinBoundedRequest,
        routes::foundation::LevenshteinBoundedResponse,
        routes::foundation::FuzzySearchRequest,
        routes::foundation::FuzzySearchResponse,
        routes::foundation::Sha256Request,
        routes::foundation::Sha256Response,
        routes::foundation::YamlParseRequest,
        routes::foundation::GraphTopsortRequest,
        routes::foundation::GraphLevelsRequest,
        routes::foundation::FsrsReviewRequest,
        routes::foundation::FsrsReviewResponse,
        routes::pv::ContingencyTableRequest,
        routes::pv::SignalCompleteResponse,
        routes::pv::SignalMetricResponse,
        routes::pv::NaranjoRequest,
        routes::pv::NaranjoResponse,
        routes::pv::WhoUmcRequest,
        routes::pv::WhoUmcResponse,
        routes::pv::SeriousnessRequest,
        routes::pv::SeriousnessResponse,
        routes::pv::ExpectednessRequest,
        routes::pv::ExpectednessResponse,
        routes::pv::CombinedRequest,
        routes::pv::CombinedResponse,
        routes::pv::RucamRequest,
        routes::pv::RucamResponse,
        routes::pv::RucamReactionType,
        routes::pv::RucamConcomitantDrugs,
        routes::pv::RucamAlternativeCauses,
        routes::pv::RucamPreviousHepatotoxicity,
        routes::pv::RucamSerologyResult,
        routes::pv::RucamYesNoNa,
        routes::pv::RucamRechallengeResult,
        routes::pv::RucamBreakdownResponse,
        routes::pv::NaranjoFullRequest,
        routes::pv::NaranjoFullResponse,
        routes::pv::WhoUmcFullRequest,
        routes::pv::WhoUmcFullResponse,
        routes::pv::WhoUmcTemporalStrengthDto,
        routes::pv::ChallengeResultDto,
        routes::pv::AlternativesLikelihoodDto,
        routes::pv::PlausibilityStrengthDto,
        routes::pv::WhoUmcCriteriaResponse,
        routes::vigilance::SafetyMarginRequest,
        routes::vigilance::SafetyMarginResponse,
        routes::vigilance::RiskScoreRequest,
        routes::vigilance::RiskScoreResponse,
        routes::vigilance::HarmType,
        routes::vigilance::MapToTovRequest,
        routes::vigilance::MapToTovResponse,
        routes::skills::SkillSummary,
        routes::skills::SkillDetail,
        routes::skills::ValidateRequest,
        routes::skills::ValidateResponse,
        routes::skills::ScanRequest,
        routes::skills::ScanResponse,
        routes::skills::TaxonomyNode,
        routes::skills::TaxonomyQueryRequest,
        routes::skills::ExecuteRequest,
        routes::skills::ExecuteResponse,
        routes::skills::SkillSchema,
        routes::brain::SessionCreateRequest,
        routes::brain::SessionResponse,
        routes::brain::ArtifactSaveRequest,
        routes::brain::ArtifactResponse,
        routes::brain::CodeTrackerRequest,
        routes::brain::CodeTrackerResponse,
        routes::pvdsl::PvdslCompileRequest,
        routes::pvdsl::PvdslCompileResponse,
        routes::pvdsl::PvdslExecuteRequest,
        routes::pvdsl::PvdslExecuteResponse,
        routes::pvdsl::PvdslEvalRequest,
        routes::pvdsl::PvdslFunctionInfo,
        routes::pvdsl::PvdslFunctionCategory,
        routes::pvdsl::PvdslFunctionsResponse,
        routes::guardian::TickResponse,
        routes::guardian::ActuatorResultSummary,
        routes::guardian::EvaluatePvRequest,
        routes::guardian::EvaluatePvResponse,
        routes::guardian::RiskScoreDetails,
        routes::guardian::ActionSummary,
        routes::guardian::StatusResponse,
        routes::guardian::SensorInfo,
        routes::guardian::ActuatorInfo,
        routes::guardian::ResetResponse,
        routes::vigil::StatusResponse,
        routes::vigil::ProcessInfo,
        routes::vigil::EndpointInfo,
        routes::vigil::ComponentsInfo,
        routes::vigil::SourceInfo,
        routes::vigil::ExecutorInfo,
        routes::vigil::HealthResponse,
        routes::vigil::HealthChecks,
        routes::vigil::CheckResult,
        routes::vigil::HealthSummary,
        routes::vigil::EmitEventRequest,
        routes::vigil::EmitEventResponse,
        routes::vigil::EventSummary,
        routes::vigil::MemorySearchRequest,
        routes::vigil::MemorySearchResponse,
        routes::vigil::MemoryPoint,
        routes::vigil::MemoryStatsResponse,
        routes::vigil::CollectionStats,
        routes::vigil::CollectionConfig,
        routes::vigil::LlmStatsResponse,
        routes::vigil::LlmStats,
        routes::signal::SignalDetectRequest,
        routes::signal::SignalDetectResponse,
        routes::signal::SignalBatchRequest,
        routes::signal::SignalBatchResponse,
        routes::signal::SignalThresholdsResponse,
        routes::signal::ThresholdSummary,
        routes::reporting::ReportRequest,
        routes::reporting::ReportResponse,
        routes::reporting::ReportType,
        routes::benefit_risk::QbriComputeRequest,
        routes::benefit_risk::QbriComputeResponse,
        routes::benefit_risk::QbriDeriveRequest,
        routes::benefit_risk::QbriDeriveResponse,
        routes::benefit_risk::QbriEquationResponse,
        routes::benefit_risk::ThresholdInfo,
        routes::benefit_risk::DecisionBoundaries,
        routes::benefit_risk::VariableDescriptions,
        // QBR types
        routes::benefit_risk::QbrContingencyTable,
        routes::benefit_risk::QbrMeasured,
        routes::benefit_risk::QbrHillCurveParams,
        routes::benefit_risk::QbrIntegrationBounds,
        routes::benefit_risk::QbrComputeRequest,
        routes::benefit_risk::QbrComputeResponse,
        routes::benefit_risk::QbrMethodDetailsResponse,
        routes::benefit_risk::QbrSimpleRequest,
        routes::benefit_risk::QbrSimpleResponse,
        routes::benefit_risk::QbrTherapeuticWindowRequest,
        routes::benefit_risk::QbrTherapeuticWindowResponse,
        routes::mesh::MeshHealthResponse,
        routes::mesh::TopologyNode,
        routes::mesh::TopologyEdge,
        routes::mesh::TopologyResponse,
        routes::mesh::SimulateRequest,
        routes::mesh::SimulateEdge,
        routes::mesh::SimulateResponse,
        routes::mesh::SimulateRouteResult,
        routes::mesh::RouteQualityRequest,
        routes::mesh::RouteQualityResponse,
        routes::mesh::GroundingEntry,
        routes::mesh::GroundingResponse,
        routes::mesh::TierDistribution,
        routes::mesh::NodeInfoRequest,
        routes::mesh::NodeInfoResponse,
        routes::common::ApiError,
        routes::sos::StateSpecRequest,
        routes::sos::TransitionSpecRequest,
        routes::sos::CreateMachineRequest,
        routes::sos::CreateMachineResponse,
        routes::sos::TransitionRequest,
        routes::sos::TransitionResponse,
        routes::sos::StateQueryRequest,
        routes::sos::AvailableTransition,
        routes::sos::StateResponse,
        routes::sos::HistoryRequest,
        routes::sos::HistoryEntry,
        routes::sos::HistoryResponse,
        routes::sos::HistoryMetrics,
        routes::sos::MachineSummary,
        routes::sos::ListMachinesResponse,
        routes::sos::AggregateStats,
        routes::sos::ValidateResponse,
        */
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "foundation", description = "Foundation algorithms - string distance, hashing, graph ops"),
        (name = "academy", description = "Competency-based learning and certification"),
        (name = "pv", description = "Pharmacovigilance signal detection and causality assessment"),
        (name = "vigilance", description = "Theory of Vigilance - safety margins, risk scoring, harm types"),
        (name = "guardian", description = "Guardian homeostasis control loop - sensing, decision, response"),
        (name = "vigil", description = "Vigil orchestrator - event bus, memory, LLM stats"),
        (name = "skills", description = "Skill registry, validation, and taxonomy"),
        (name = "community", description = "Social networking and knowledge sharing"),
        (name = "ventures", description = "Strategic partnerships and investments"),
        (name = "brain", description = "Working memory - sessions, artifacts, code tracking"),
        (name = "pvdsl", description = "PVDSL - Pharmacovigilance Domain-Specific Language compiler and runtime"),
        (name = "signal", description = "Signal detection pipeline - detect, batch, and threshold endpoints using signal-* crates"),
        (name = "reporting", description = "Automated safety report generation"),
        (name = "benefit-risk", description = "Benefit-risk assessment - QBRI (expert-judgment) and QBR (statistical-evidence, 4 forms)"),
        (name = "core", description = "Core PV-OS high-level operations"),
        (name = "mesh", description = "Mesh networking - topology simulation, route quality, grounding coverage"),
        (name = "SOS", description = "State Operating System - 15-layer state machine runtime"),
        (name = "compliance", description = "Compliance infrastructure - audit trails, GDPR, export controls, SOC 2"),
        (name = "ml", description = "Platform ML engine - model catalog, inference, training, active learning"),
        (name = "admin", description = "Administration - user management, content moderation, system stats"),
        (name = "benchmarks", description = "Performance benchmarks and platform aggregates"),
        (name = "career", description = "Career pathways - role transitions and progression"),
        (name = "faers", description = "FDA Adverse Event Reporting System - search, drug events, signal detection"),
        (name = "regulatory-intelligence", description = "Regulatory Intelligence - FDA Guidance Documents (2,794+) and ICH/CIOMS pharmacovigilance glossary (904 terms)"),
        (name = "icsr", description = "ICSR - E2B(R3) Individual Case Safety Report construction and validation"),
        (name = "graph-layout", description = "Graph visualization - force-directed layout convergence"),
        (name = "learning", description = "Learning DAG resolution and progress tracking"),
        (name = "marketplace", description = "Expert marketplace - search, recommend, engage"),
        (name = "mcp-bridge", description = "Model Context Protocol bridge - in-process tool execution"),
        (name = "telemetry", description = "System telemetry - event ingestion, querying, summaries"),
        (name = "tenant", description = "Multi-tenant management - provisioning, limits, tiers"),
    )
)]
pub struct ApiDoc;
