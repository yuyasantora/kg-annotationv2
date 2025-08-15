use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Annotation {
    pub id: Uuid,
    pub image_id: Uuid,
    pub user_id: Uuid,
    pub annotation_type: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub label: String,
    pub confidence: Option<f32>,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAnnotationRequest {
    pub image_id: Uuid,
    pub annotation_type: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub label: String,
    pub confidence: Option<f32>,
    pub source: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateAnnotationRequest {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub label: Option<String>,
    pub confidence: Option<f32>,
}
