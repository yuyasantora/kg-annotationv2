use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// sqlx::FromRow を追加
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Annotation {
    pub id: Uuid,
    pub image_id: Uuid,
    pub user_id: Uuid,
    // 専用のEnum
    pub annotation_type: AnnotationType,
    // f64 から f32 に戻す
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub label: String,
    // f64 から f32 に戻す
    pub confidence: Option<f32>,
    // 専用のEnum
    pub source: AnnotationSource,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAnnotationRequest {
    pub image_id: Uuid,
    pub annotation_type: AnnotationType,
    // f64 から f32 に戻す
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub label: String,
    // f64 から f32 に戻す
    pub confidence: Option<f32>,
    pub source: AnnotationSource,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateAnnotationRequest {
    // f64 から f32 に戻す
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub label: Option<String>,
    // f64 から f32 に戻す
    pub confidence: Option<f32>,
}

// データベースのENUM型に対応するRustのenumを定義
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "annotation_type", rename_all = "lowercase")]
pub enum AnnotationType {
    BoundingBox,
    Polygon,
    Point,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "annotation_source", rename_all = "lowercase")]
pub enum AnnotationSource {
    Manual,
    Ai,
}
