//! Delegation API Routes - Flight Simulator for Gemini Training
//!
//! Exposes DelegationRouter via REST API for cross-model learning.

use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use nexcore_vigilance::primitives::delegation::{
    DelegationRouter, ErrorCost, Model, TaskCharacteristics,
};

/// Create the delegation router (stateless)
#[allow(dead_code)]
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/route", post(route_task))
        .route("/training", get(get_training_suite))
        .route("/validate", post(validate_answer))
}

// =============================================================================
// REQUEST/RESPONSE TYPES
// =============================================================================

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct RouteTaskRequest {
    /// Number of items to process
    pub item_count: usize,
    /// Is the task repetitive (same operation many times)?
    pub is_repetitive: bool,
    /// Does it have clear structure/patterns?
    pub has_structure: bool,
    /// Does it need deep reasoning?
    pub needs_reasoning: bool,
    /// Is it a novel/unprecedented problem?
    pub is_novel: bool,
    /// Contains sensitive/confidential data?
    pub is_sensitive: bool,
    /// Requires image/audio/video processing?
    pub is_multimodal: bool,
    /// Error cost level: "low", "medium", "high", "critical"
    pub error_cost: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct RouteTaskResponse {
    /// Recommended model
    pub model: String,
    /// Model's primary strength
    pub model_strength: String,
    /// Routing confidence (0-100%)
    pub confidence: f64,
    /// Human-readable rationale
    pub rationale: String,
    /// Model's error tolerance (0-1)
    pub error_tolerance: f64,
    /// Suggested prompt structure for this model
    pub prompt_hints: Vec<String>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct TrainingScenario {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub task: RouteTaskRequest,
    pub expected_model: String,
    pub difficulty: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct TrainingSuite {
    pub scenarios: Vec<TrainingScenario>,
    pub instructions: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ValidateAnswerRequest {
    pub scenario_id: u32,
    pub predicted_model: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ValidateAnswerResponse {
    pub correct: bool,
    pub expected_model: String,
    pub predicted_model: String,
    pub feedback: String,
    pub score: u32,
}

// =============================================================================
// HANDLERS
// =============================================================================

/// POST /api/v1/delegation/route
/// Route a task to the optimal model
#[allow(dead_code)]
async fn route_task(
    Json(req): Json<RouteTaskRequest>,
) -> Result<Json<RouteTaskResponse>, (StatusCode, String)> {
    let error_cost = match req.error_cost.to_lowercase().as_str() {
        "low" => ErrorCost::Low,
        "medium" => ErrorCost::Medium,
        "high" => ErrorCost::High,
        "critical" => ErrorCost::Critical,
        _ => return Err((StatusCode::BAD_REQUEST, "Invalid error_cost".to_string())),
    };

    let task = TaskCharacteristics {
        item_count: req.item_count,
        is_repetitive: req.is_repetitive,
        has_structure: req.has_structure,
        needs_reasoning: req.needs_reasoning,
        is_novel: req.is_novel,
        is_sensitive: req.is_sensitive,
        is_multimodal: req.is_multimodal,
        error_cost,
    };

    let decision = DelegationRouter::route(&task);
    let model = decision.model;

    let prompt_hints = match model {
        Model::GeminiFlash => vec![
            "Use structured output format (JSON/tables)".to_string(),
            "Batch items into groups of 10-20".to_string(),
            "Include validation regex patterns".to_string(),
            "Request explicit error markers".to_string(),
        ],
        Model::GeminiPro => vec![
            "Include image/document references".to_string(),
            "Request multi-modal analysis".to_string(),
            "Use longer context windows".to_string(),
        ],
        Model::ClaudeOpus => vec![
            "Break into reasoning steps".to_string(),
            "Request explicit assumptions".to_string(),
            "Ask for alternative approaches".to_string(),
            "Include edge case analysis".to_string(),
        ],
        Model::ClaudeSonnet => vec![
            "Balance detail with speed".to_string(),
            "Use code blocks for implementation".to_string(),
            "Request test cases".to_string(),
        ],
        Model::ClaudeHaiku => vec![
            "Keep prompts concise".to_string(),
            "Single-step tasks only".to_string(),
            "Use yes/no or classification format".to_string(),
        ],
    };

    Ok(Json(RouteTaskResponse {
        model: format!("{:?}", model),
        model_strength: format!("{:?}", model.primary_strength()),
        confidence: decision.confidence * 100.0,
        rationale: decision.rationale.to_string(),
        error_tolerance: model.error_tolerance(),
        prompt_hints,
    }))
}

/// GET /api/v1/delegation/training
/// Get training scenarios for Gemini
#[allow(dead_code)]
async fn get_training_suite() -> Json<TrainingSuite> {
    let scenarios = get_training_scenarios();

    let instructions = r#"
# Gemini Delegation Training - Flight Simulator

## Your Mission
Learn to route tasks to the optimal AI model. You will be tested on 10 scenarios.

## Decision Tree (Priority Order)
1. **Sensitive OR Critical Error Cost** → ClaudeOpus (safety first)
2. **Bulk (>10 items) + Repetitive + Structured** → GeminiFlash (speed)
3. **Novel + Needs Reasoning** → ClaudeOpus (depth)
4. **Multimodal** → GeminiPro (vision)
5. **High Volume (>50)** → GeminiFlash (throughput)
6. **Default** → ClaudeSonnet (balanced)

## Model Strengths
| Model | Best For |
|-------|----------|
| GeminiFlash | Bulk generation, pattern matching, templates |
| GeminiPro | Images, documents, long context |
| ClaudeOpus | Novel problems, architecture, security |
| ClaudeSonnet | Code generation, refactoring, review |
| ClaudeHaiku | Simple queries, classification |

## Training Protocol
1. Read each scenario
2. Analyze task characteristics
3. Apply decision tree
4. Submit your prediction
5. Learn from feedback

## Scoring
- Easy: 10 points
- Medium: 20 points
- Hard: 30 points
- Perfect score: 170 points
"#
    .to_string();

    Json(TrainingSuite {
        scenarios,
        instructions,
    })
}

/// POST /api/v1/delegation/validate
/// Validate Gemini's answer
#[allow(dead_code)]
async fn validate_answer(
    Json(req): Json<ValidateAnswerRequest>,
) -> Result<Json<ValidateAnswerResponse>, (StatusCode, String)> {
    let scenarios = get_training_scenarios();
    let scenario = scenarios
        .iter()
        .find(|s| s.id == req.scenario_id)
        .ok_or((StatusCode::NOT_FOUND, "Scenario not found".to_string()))?;

    let correct = req.predicted_model.to_lowercase() == scenario.expected_model.to_lowercase();

    let score = if correct {
        match scenario.difficulty.as_str() {
            "easy" => 10,
            "medium" => 20,
            "hard" => 30,
            _ => 10,
        }
    } else {
        0
    };

    let feedback = if correct {
        format!(
            "✓ Correct! {} is optimal for: {}",
            scenario.expected_model, scenario.description
        )
    } else {
        let hint = match scenario.expected_model.as_str() {
            "ClaudeOpus" => {
                "Look for: sensitive data, critical errors, novel problems, or deep reasoning needs"
            }
            "GeminiFlash" => "Look for: high volume (>10), repetitive, structured patterns",
            "GeminiPro" => "Look for: images, documents, or multimodal content",
            "ClaudeSonnet" => "This is the balanced default when no strong signals present",
            _ => "Review the decision tree",
        };
        format!("✗ Expected {}. Hint: {}", scenario.expected_model, hint)
    };

    Ok(Json(ValidateAnswerResponse {
        correct,
        expected_model: scenario.expected_model.clone(),
        predicted_model: req.predicted_model,
        feedback,
        score,
    }))
}

fn get_training_scenarios() -> Vec<TrainingScenario> {
    vec![
        TrainingScenario {
            id: 1,
            name: "Bulk Test Generation".to_string(),
            description: "Generate unit tests for 100+ API endpoints".to_string(),
            task: RouteTaskRequest {
                item_count: 112,
                is_repetitive: true,
                has_structure: true,
                needs_reasoning: false,
                is_novel: false,
                is_sensitive: false,
                is_multimodal: false,
                error_cost: "low".to_string(),
            },
            expected_model: "GeminiFlash".to_string(),
            difficulty: "easy".to_string(),
        },
        TrainingScenario {
            id: 2,
            name: "Architecture Design".to_string(),
            description: "Design a new microservice architecture".to_string(),
            task: RouteTaskRequest {
                item_count: 1,
                is_repetitive: false,
                has_structure: true,
                needs_reasoning: true,
                is_novel: true,
                is_sensitive: false,
                is_multimodal: false,
                error_cost: "high".to_string(),
            },
            expected_model: "ClaudeOpus".to_string(),
            difficulty: "easy".to_string(),
        },
        TrainingScenario {
            id: 3,
            name: "Security Audit".to_string(),
            description: "Review authentication code for vulnerabilities".to_string(),
            task: RouteTaskRequest {
                item_count: 5,
                is_repetitive: false,
                has_structure: true,
                needs_reasoning: true,
                is_novel: false,
                is_sensitive: true,
                is_multimodal: false,
                error_cost: "critical".to_string(),
            },
            expected_model: "ClaudeOpus".to_string(),
            difficulty: "medium".to_string(),
        },
        TrainingScenario {
            id: 4,
            name: "Image Analysis".to_string(),
            description: "Analyze screenshots for UI bugs".to_string(),
            task: RouteTaskRequest {
                item_count: 20,
                is_repetitive: true,
                has_structure: true,
                needs_reasoning: false,
                is_novel: false,
                is_sensitive: false,
                is_multimodal: true,
                error_cost: "medium".to_string(),
            },
            expected_model: "GeminiPro".to_string(),
            difficulty: "easy".to_string(),
        },
        TrainingScenario {
            id: 5,
            name: "Code Refactoring".to_string(),
            description: "Refactor 10 functions following new patterns".to_string(),
            task: RouteTaskRequest {
                item_count: 10,
                is_repetitive: true,
                has_structure: true,
                needs_reasoning: false,
                is_novel: false,
                is_sensitive: false,
                is_multimodal: false,
                error_cost: "medium".to_string(),
            },
            expected_model: "GeminiFlash".to_string(),
            difficulty: "medium".to_string(),
        },
        TrainingScenario {
            id: 6,
            name: "Novel Algorithm".to_string(),
            description: "Implement a custom signal detection algorithm".to_string(),
            task: RouteTaskRequest {
                item_count: 1,
                is_repetitive: false,
                has_structure: false,
                needs_reasoning: true,
                is_novel: true,
                is_sensitive: false,
                is_multimodal: false,
                error_cost: "high".to_string(),
            },
            expected_model: "ClaudeOpus".to_string(),
            difficulty: "easy".to_string(),
        },
        TrainingScenario {
            id: 7,
            name: "Documentation Update".to_string(),
            description: "Update docs for 50 functions".to_string(),
            task: RouteTaskRequest {
                item_count: 50,
                is_repetitive: true,
                has_structure: true,
                needs_reasoning: false,
                is_novel: false,
                is_sensitive: false,
                is_multimodal: false,
                error_cost: "low".to_string(),
            },
            expected_model: "GeminiFlash".to_string(),
            difficulty: "easy".to_string(),
        },
        TrainingScenario {
            id: 8,
            name: "Tricky: Small Sensitive".to_string(),
            description: "Review 3 credential handling functions".to_string(),
            task: RouteTaskRequest {
                item_count: 3,
                is_repetitive: false,
                has_structure: true,
                needs_reasoning: true,
                is_novel: false,
                is_sensitive: true,
                is_multimodal: false,
                error_cost: "critical".to_string(),
            },
            expected_model: "ClaudeOpus".to_string(),
            difficulty: "hard".to_string(),
        },
        TrainingScenario {
            id: 9,
            name: "Tricky: Large Novel".to_string(),
            description: "Generate 100 test cases for a new protocol".to_string(),
            task: RouteTaskRequest {
                item_count: 100,
                is_repetitive: true,
                has_structure: true,
                needs_reasoning: false,
                is_novel: true,
                is_sensitive: false,
                is_multimodal: false,
                error_cost: "medium".to_string(),
            },
            expected_model: "GeminiFlash".to_string(),
            difficulty: "hard".to_string(),
        },
        TrainingScenario {
            id: 10,
            name: "Tricky: Multimodal + Sensitive".to_string(),
            description: "Analyze medical images and write diagnostic report".to_string(),
            task: RouteTaskRequest {
                item_count: 5,
                is_repetitive: false,
                has_structure: false,
                needs_reasoning: true,
                is_novel: true,
                is_sensitive: true,
                is_multimodal: true,
                error_cost: "critical".to_string(),
            },
            expected_model: "ClaudeOpus".to_string(),
            difficulty: "hard".to_string(),
        },
    ]
}
