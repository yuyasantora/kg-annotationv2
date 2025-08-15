use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
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
}
