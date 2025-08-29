use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Image {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub s3_bucket: String,
    pub s3_key: String,
    pub file_size: i64,
    pub width: i32,
    pub height: i32,
    pub format: String,
    pub classification_label: Option<String>,
    pub created_at: DateTime<Utc>,
    pub vector: Option<serde_json::Value>,
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

// フロントエンドに返すレスポンス用の構造体を修正
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageResponse {
    pub id: Uuid,
    pub s3_key: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ImageSearchRequest {
    pub query: String,
}
