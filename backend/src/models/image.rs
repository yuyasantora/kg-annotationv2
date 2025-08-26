use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub s3_key: String,
    pub s3_bucket: String,
    pub file_size: i64,
    pub width: i32,
    pub height: i32,
    pub format: String,
    pub classification_label: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateImageRequest {
    pub filename: String,
    pub file_size: i64,
    pub width: i32,
    pub height: i32,
    pub format: String,
    pub classification_label: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ImageResponse {
    pub id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub file_size: i64,
    pub width: i32,
    pub height: i32,
    pub format: String,
    pub classification_label: Option<String>,
    pub created_at: DateTime<Utc>,
    pub annotation_count: i64,
    pub url: String,  // 追加
}

// main.rs から ImageSearchRequest を移動
#[derive(Debug, Deserialize)]
pub struct ImageSearchRequest {
    pub query: String,
    pub top_k: Option<i32>,
}
