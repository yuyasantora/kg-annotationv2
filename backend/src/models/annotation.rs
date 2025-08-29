use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;
use validator::Validate; // validatorをインポート

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "annotation_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AnnotationType {
    BoundingBox,
    Polygon,
    Point,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "annotation_source", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AnnotationSource {
    Manual,
    Ai,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct Annotation {
    pub id: Uuid,
    pub image_id: Uuid,
    pub user_id: Uuid,
    pub annotation_type: AnnotationType,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub points: Option<serde_json::Value>,
    pub bbox: Option<Vec<f32>>,
    pub label: String,
    pub source: AnnotationSource,
    pub confidence: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// アノテーション作成リクエスト
#[derive(Debug, Serialize, Deserialize, Clone, Validate)] // Validateを追加
pub struct CreateAnnotationRequest {
    pub image_id: Uuid,
    pub annotation_type: AnnotationType,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub label: String,
    pub confidence: Option<f32>,
    pub source: AnnotationSource,
    // 以下の2つのフィールドを追加
    pub bbox: Option<Vec<f32>>,
    pub points: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Validate)] // Validateを追加
pub struct UpdateAnnotationRequest {
    pub annotation_type: AnnotationType,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub points: Option<serde_json::Value>,
    pub bbox: Option<Vec<f32>>,
    pub label: String,
    pub confidence: Option<f32>,
}

// アノテーション作成時のレスポンス
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAnnotationResponse {
    pub id: Uuid,
}
