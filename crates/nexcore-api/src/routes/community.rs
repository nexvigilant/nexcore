//! Community module — social networking and knowledge sharing

use crate::ApiState;
use crate::persistence::PostRecord;
use axum::extract::{Json, State};
use axum::routing::get;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Community post
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Post {
    pub id: String,
    pub author: String,
    pub role: String,
    pub content: String,
    pub likes: u32,
    pub replies: u32,
    pub created_at: DateTime<Utc>,
}

/// Request to create a new post
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreatePostRequest {
    pub author: String,
    pub role: String,
    pub content: String,
}

/// List all community posts
#[utoipa::path(
    get,
    path = "/api/v1/community/posts",
    responses(
        (status = 200, description = "List of community posts", body = Vec<Post>),
    ),
    tag = "community"
)]
pub async fn list_posts(
    State(state): State<ApiState>,
) -> Result<Json<Vec<Post>>, crate::routes::common::ApiError> {
    let records = state
        .persistence
        .list_posts()
        .await
        .map_err(|e| crate::routes::common::ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let mut posts: Vec<Post> = records
        .into_iter()
        .map(|r| Post {
            id: r.id,
            author: r.author,
            role: r.role,
            content: r.content,
            likes: r.likes,
            replies: r.replies,
            created_at: r.created_at,
        })
        .collect();

    // If empty, add some seed posts for better DX
    if posts.is_empty() {
        posts = vec![
            Post {
                id: "1".to_string(),
                author: "Dr. Sarah Chen".to_string(),
                role: "Signal Detection Lead".to_string(),
                content: "Interesting pattern in the latest FAERS Q4 data: unusual clustering of hepatotoxicity reports for a newly approved kinase inhibitor. Has anyone else noticed this?".to_string(),
                likes: 12,
                replies: 5,
                created_at: Utc::now(),
            },
            Post {
                id: "2".to_string(),
                author: "James Wilson".to_string(),
                role: "PV Specialist".to_string(),
                content: "Just completed the D08 Signal Detection pathway on Academy! The PRR/ROR exercises were incredibly practical. Highly recommend.".to_string(),
                likes: 24,
                replies: 8,
                created_at: Utc::now(),
            },
        ];
    }

    Ok(Json(posts))
}

/// Create a new community post
#[utoipa::path(
    post,
    path = "/api/v1/community/posts",
    request_body = CreatePostRequest,
    responses(
        (status = 201, description = "Post created successfully", body = Post),
    ),
    tag = "community"
)]
pub async fn create_post(
    State(state): State<ApiState>,
    Json(req): Json<CreatePostRequest>,
) -> Result<Json<Post>, crate::routes::common::ApiError> {
    let post = Post {
        id: uuid::Uuid::new_v4().to_string(),
        author: req.author,
        role: req.role,
        content: req.content,
        likes: 0,
        replies: 0,
        created_at: Utc::now(),
    };

    let record = PostRecord {
        id: post.id.clone(),
        author: post.author.clone(),
        role: post.role.clone(),
        content: post.content.clone(),
        likes: post.likes,
        replies: post.replies,
        created_at: post.created_at,
    };

    state
        .persistence
        .save_post(&record)
        .await
        .map_err(|e| crate::routes::common::ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(post))
}

pub fn router() -> axum::Router<ApiState> {
    axum::Router::new().route("/posts", get(list_posts).post(create_post))
}
