//! wksp-api-client — Typed client for NexCore API
//!
//! Extracted from ncos for workspace-wide sharing.
//! Server-side: reqwest (SSR feature)
//! Client-side: gloo-net (hydrate/WASM feature)

use serde::{ Deserialize, Serialize };

// Re-export all the DTOs from the original ncos client
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HealthResponse {
    pub status: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuardianStatus {
    pub iteration: u64,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub sensors: u32,
    #[serde(default)]
    pub actuators: u32,
    #[serde(default)]
    pub amplification: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VigilStatus {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub sources: u32,
    #[serde(default)]
    pub executors: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmStats {
    #[serde(default)]
    pub total_calls: u64,
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub avg_tokens_per_call: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignalRequest {
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignalResult {
    #[serde(default)]
    pub prr: f64,
    #[serde(default)]
    pub prr_signal: bool,
    #[serde(default)]
    pub ror: f64,
    #[serde(default)]
    pub ror_lower_ci: f64,
    #[serde(default)]
    pub ror_signal: bool,
    #[serde(default)]
    pub ic: f64,
    #[serde(default)]
    pub ic025: f64,
    #[serde(default)]
    pub ic_signal: bool,
    #[serde(default)]
    pub ebgm: f64,
    #[serde(default)]
    pub eb05: f64,
    #[serde(default)]
    pub ebgm_signal: bool,
    #[serde(default)]
    pub chi_square: f64,
    #[serde(default)]
    pub chi_signal: bool,
    #[serde(default)]
    pub any_signal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NaranjoCausality {
    pub score: i32,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhoUmcCausality {
    pub category: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrainSession {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub artifact_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrainArtifact {
    pub id: String,
    #[serde(default)]
    pub artifact_type: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillInfo {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub compliance_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Course {
    pub id: String,
    pub code: String,
    pub title: String,
    pub description: String,
    pub tier: String,
    pub level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Enrollment {
    pub id: String,
    pub user_id: String,
    pub course_id: String,
    pub progress: f64,
    pub enrolled_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnrollRequest {
    pub course_id: String,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KsbDomainSummary {
    pub code: String,
    pub name: String,
    pub ksb_count: u32,
    pub dominant_primitive: String,
    pub cognitive_primitive: String,
    pub transfer_confidence: f64,
    pub pvos_layer: Option<String>,
    pub example_ksbs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Post {
    pub id: String,
    pub author: String,
    pub role: String,
    pub content: String,
    pub likes: u32,
    pub replies: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreatePostRequest {
    pub author: String,
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Circle {
    pub id: String,
    pub name: String,
    pub description: String,
    pub member_count: u32,
    pub post_count: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JoinRequest {
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Message {
    pub id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SendMessageRequest {
    pub sender_id: String,
    pub recipient_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartnershipInquiry {
    pub id: String,
    pub name: String,
    pub email: String,
    pub organization: String,
    pub interest: String,
    pub message: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartnershipRequest {
    pub name: String,
    pub email: String,
    pub organization: String,
    pub interest: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateInquiryStatusRequest {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PvdslResult {
    #[serde(default)]
    pub output: String,
    #[serde(default)]
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    #[default]
    SignalSummary,
    AuditTrail,
    GuardianPerformance,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReportRequest {
    pub report_type: ReportType,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReportResponse {
    pub id: String,
    pub report_type: ReportType,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub content: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QbriRequest {
    #[serde(default)]
    pub benefits: Vec<BenefitInput>,
    #[serde(default)]
    pub risks: Vec<RiskInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenefitInput {
    pub name: String,
    pub weight: f64,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskInput {
    pub name: String,
    pub weight: f64,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QbriResult {
    #[serde(default)]
    pub qbri: f64,
    #[serde(default)]
    pub interpretation: String,
}

/// A node in the learning pathway DAG.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathwayNode {
    pub course_id: String,
    pub code: String,
    pub title: String,
    #[serde(default)]
    pub tier: String,
    #[serde(default)]
    pub level: u8,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub unlocked: bool,
}

/// A complete learning pathway.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearningPathway {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub nodes: Vec<PathwayNode>,
}

/// Learner journey state for the state machine widget.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LearnerState {
    #[default]
    Onboarding,
    Exploring,
    Assessed,
    Learning,
    Certified,
}

impl LearnerState {
    /// Display label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Onboarding => "Onboarding",
            Self::Exploring => "Exploring",
            Self::Assessed => "Assessed",
            Self::Learning => "Learning",
            Self::Certified => "Certified",
        }
    }

    /// Lex Primitiva symbol.
    #[must_use]
    pub const fn symbol(&self) -> &'static str {
        match self {
            Self::Onboarding => "∃",
            Self::Exploring => "λ",
            Self::Assessed => "κ",
            Self::Learning => "σ",
            Self::Certified => "π",
        }
    }

    /// Progress percentage (0-100).
    #[must_use]
    pub const fn progress(&self) -> u8 {
        match self {
            Self::Onboarding => 0,
            Self::Exploring => 20,
            Self::Assessed => 40,
            Self::Learning => 70,
            Self::Certified => 100,
        }
    }

    /// All states in order.
    pub const ALL: [LearnerState; 5] = [
        Self::Onboarding,
        Self::Exploring,
        Self::Assessed,
        Self::Learning,
        Self::Certified,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuardianEvalRequest {
    pub drug_name: String,
    pub event_name: String,
    #[serde(default)]
    pub case_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuardianEvalResult {
    #[serde(default)]
    pub risk_level: String,
    #[serde(default)]
    pub recommended_actions: Vec<String>,
    #[serde(default)]
    pub risk_score: f64,
}

/// Server-side API client using reqwest
#[cfg(feature = "ssr")]
pub mod server {
    use super::*;

    pub struct ApiClient {
        client: reqwest::Client,
        base_url: String,
        api_key: Option<String>,
    }

    impl ApiClient {
        pub fn new(base_url: String, api_key: Option<String>) -> Self {
            Self {
                client: reqwest::Client::new(),
                base_url,
                api_key,
            }
        }

        fn url(&self, path: &str) -> String {
            format!("{}{}", self.base_url, path)
        }

        fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
            let mut req = self.client.request(method, self.url(path));
            if let Some(ref key) = self.api_key {
                req = req.header("X-API-Key", key);
            }
            req
        }

        pub async fn health(&self) -> Result<HealthResponse, String> {
            self.request(reqwest::Method::GET, "/health/ready")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn guardian_status(&self) -> Result<GuardianStatus, String> {
            self.request(reqwest::Method::GET, "/api/v1/guardian/status")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn vigil_status(&self) -> Result<VigilStatus, String> {
            self.request(reqwest::Method::GET, "/api/v1/vigil/status")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn llm_stats(&self) -> Result<LlmStats, String> {
            self.request(reqwest::Method::GET, "/api/v1/vigil/llm/stats")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn signal_complete(&self, req: &SignalRequest) -> Result<SignalResult, String> {
            self.request(reqwest::Method::POST, "/api/v1/pv/signal/complete")
                .json(req)
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn guardian_tick(&self) -> Result<serde_json::Value, String> {
            self.request(reqwest::Method::POST, "/api/v1/guardian/tick")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn guardian_evaluate(
            &self,
            req: &GuardianEvalRequest
        ) -> Result<GuardianEvalResult, String> {
            self.request(reqwest::Method::POST, "/api/v1/guardian/evaluate")
                .json(req)
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn brain_sessions(&self) -> Result<Vec<BrainSession>, String> {
            self.request(reqwest::Method::GET, "/api/v1/brain/sessions")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn brain_session_load(&self, session_id: &str) -> Result<BrainSession, String> {
            self.request(reqwest::Method::GET, &format!("/api/v1/brain/sessions/{session_id}"))
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn brain_artifact_get(
            &self,
            session_id: &str,
            name: &str
        ) -> Result<BrainArtifact, String> {
            self.request(
                reqwest::Method::GET,
                &format!("/api/v1/brain/artifacts/{session_id}/{name}")
            )
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn skills_list(&self) -> Result<Vec<SkillInfo>, String> {
            self.request(reqwest::Method::GET, "/api/v1/skills")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn pvdsl_execute(&self, code: &str) -> Result<PvdslResult, String> {
            self.request(reqwest::Method::POST, "/api/v1/pvdsl/execute")
                .json(&serde_json::json!({ "code": code }))
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn naranjo(&self, answers: &[i32]) -> Result<NaranjoCausality, String> {
            self.request(reqwest::Method::POST, "/api/v1/pv/naranjo")
                .json(&serde_json::json!({ "answers": answers }))
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn who_umc(
            &self,
            answers: &serde_json::Value
        ) -> Result<WhoUmcCausality, String> {
            self.request(reqwest::Method::POST, "/api/v1/pv/who-umc")
                .json(answers)
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn qbri_compute(&self, req: &QbriRequest) -> Result<QbriResult, String> {
            self.request(reqwest::Method::POST, "/api/v1/benefit-risk/qbri/compute")
                .json(req)
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn guardian_pause(&self) -> Result<serde_json::Value, String> {
            self.request(reqwest::Method::POST, "/api/v1/guardian/pause")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn guardian_resume(&self) -> Result<serde_json::Value, String> {
            self.request(reqwest::Method::POST, "/api/v1/guardian/resume")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn guardian_reset(&self) -> Result<serde_json::Value, String> {
            self.request(reqwest::Method::POST, "/api/v1/guardian/reset")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn signal_thresholds(&self) -> Result<serde_json::Value, String> {
            self.request(reqwest::Method::GET, "/api/v1/signal/thresholds")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn pvdsl_functions(&self) -> Result<serde_json::Value, String> {
            self.request(reqwest::Method::GET, "/api/v1/pvdsl/functions")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn pvdsl_compile(&self, code: &str) -> Result<serde_json::Value, String> {
            self.request(reqwest::Method::POST, "/api/v1/pvdsl/compile")
                .json(&serde_json::json!({ "source": code }))
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn reporting_generate(&self, req: &ReportRequest) -> Result<ReportResponse, String> {
            self.request(reqwest::Method::POST, "/api/v1/reporting/generate")
                .json(req)
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn reporting_list(&self) -> Result<Vec<ReportResponse>, String> {
            self.request(reqwest::Method::GET, "/api/v1/reporting/list")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn community_list_posts(&self) -> Result<Vec<Post>, String> {
            self.request(reqwest::Method::GET, "/api/v1/community/posts")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn community_create_post(&self, req: &CreatePostRequest) -> Result<Post, String> {
            self.request(reqwest::Method::POST, "/api/v1/community/posts")
                .json(req)
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn community_list_circles(&self) -> Result<Vec<Circle>, String> {
            self.request(reqwest::Method::GET, "/api/v1/community/circles")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn community_join_circle(&self, circle_id: &str, req: &JoinRequest) -> Result<serde_json::Value, String> {
            self.request(reqwest::Method::POST, &format!("/api/v1/community/circles/{circle_id}/join"))
                .json(req)
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn community_list_messages(&self) -> Result<Vec<Message>, String> {
            self.request(reqwest::Method::GET, "/api/v1/community/messages")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn community_send_message(&self, req: &SendMessageRequest) -> Result<Message, String> {
            self.request(reqwest::Method::POST, "/api/v1/community/messages")
                .json(req)
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn academy_list_courses(&self) -> Result<Vec<Course>, String> {
            self.request(reqwest::Method::GET, "/api/v1/academy/courses")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn academy_ksb_domains(&self) -> Result<Vec<KsbDomainSummary>, String> {
            self.request(reqwest::Method::GET, "/api/v1/academy/ksb/domains")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn academy_pathways(&self) -> Result<Vec<LearningPathway>, String> {
            self.request(reqwest::Method::GET, "/api/v1/academy/pathways")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn academy_list_enrollments(&self) -> Result<Vec<Enrollment>, String> {
            self.request(reqwest::Method::GET, "/api/v1/academy/enrollments")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn academy_enroll(&self, req: &EnrollRequest) -> Result<Enrollment, String> {
            self.request(reqwest::Method::POST, "/api/v1/academy/enroll")
                .json(req)
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn ventures_list_inquiries(&self) -> Result<Vec<PartnershipInquiry>, String> {
            self.request(reqwest::Method::GET, "/api/v1/ventures/inquiries")
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn ventures_submit_inquiry(&self, req: &PartnershipRequest) -> Result<PartnershipInquiry, String> {
            self.request(reqwest::Method::POST, "/api/v1/ventures/inquiries")
                .json(req)
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }

        pub async fn ventures_update_status(&self, id: &str, req: &UpdateInquiryStatusRequest) -> Result<serde_json::Value, String> {
            self.request(reqwest::Method::PATCH, &format!("/api/v1/ventures/inquiries/{id}/status"))
                .json(req)
                .send().await
                .map_err(|e| format!("Request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Parse failed: {e}"))
        }
    }
}

// Client-side implementation for hydration (placeholders for now)
#[cfg(feature = "hydrate")]
pub mod client {
    // This will eventually use gloo-net or another WASM fetch wrapper
}
