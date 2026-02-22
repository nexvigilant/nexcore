//! Messaging module — private member communications

use crate::ApiState;
use crate::persistence::MessageRecord;
use axum::extract::{Json, Path, State};
use axum::http::HeaderMap;
use axum::routing::{get, post};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Private message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Message {
    pub id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub read: bool,
}

/// Request to send a message
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SendMessageRequest {
    pub sender_id: String,
    pub recipient_id: String,
    pub content: String,
}

/// List messages for current user
#[utoipa::path(
    get,
    path = "/api/v1/community/messages",
    responses(
        (status = 200, description = "List of messages", body = Vec<Message>),
    ),
    tag = "community"
)]
pub async fn list_messages(
    headers: HeaderMap,
    State(state): State<ApiState>,
) -> Result<Json<Vec<Message>>, crate::routes::common::ApiError> {
    let user_id = crate::tenant::extract_user_id_from_headers(&headers)
        .unwrap_or_else(|| "anonymous".to_string());
    let records = state
        .persistence
        .list_messages(&user_id)
        .await
        .map_err(|e| crate::routes::common::ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let responses = records
        .into_iter()
        .map(|r| Message {
            id: r.id,
            sender_id: r.sender_id,
            recipient_id: r.recipient_id,
            content: r.content,
            created_at: r.created_at,
            read: r.read,
        })
        .collect();

    Ok(Json(responses))
}

/// Send a new message
#[utoipa::path(
    post,
    path = "/api/v1/community/messages",
    request_body = SendMessageRequest,
    responses(
        (status = 201, description = "Message sent successfully", body = Message),
    ),
    tag = "community"
)]
pub async fn send_message(
    State(state): State<ApiState>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<Message>, crate::routes::common::ApiError> {
    let message = Message {
        id: uuid::Uuid::new_v4().to_string(),
        sender_id: req.sender_id,
        recipient_id: req.recipient_id,
        content: req.content,
        created_at: Utc::now(),
        read: false,
    };

    let record = MessageRecord {
        id: message.id.clone(),
        sender_id: message.sender_id.clone(),
        recipient_id: message.recipient_id.clone(),
        content: message.content.clone(),
        created_at: message.created_at,
        read: message.read,
    };

    state
        .persistence
        .save_message(&record)
        .await
        .map_err(|e| crate::routes::common::ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(message))
}

pub fn router() -> axum::Router<ApiState> {
    axum::Router::new().route("/", get(list_messages).post(send_message))
}
