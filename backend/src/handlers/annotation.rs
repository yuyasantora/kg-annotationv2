use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Json as JsonExtractor,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use std::collections::HashMap;

use crate::models::{
    Annotation, CreateAnnotationRequest, UpdateAnnotationRequest,
    AnnotationType, AnnotationSource,
};

// レスポンス用の構造体
#[derive(Serialize)]
pub struct AnnotationResponse {
    pub annotations: Vec<Annotation>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct CreateAnnotationResponse {
    pub id: Uuid,
    pub message: String,
}

// クエリパラメータ用
#[derive(Deserialize)]
pub struct AnnotationQuery {
    pub image_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// アノテーション一覧取得
pub async fn list_annotations(
    Query(params): Query<AnnotationQuery>,
    State(pool): State<PgPool>,
) -> Result<Json<AnnotationResponse>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let mut query = "SELECT * FROM annotations WHERE 1=1".to_string();
    let mut query_params: Vec<String> = Vec::new();

    // 動的クエリ構築
    if let Some(image_id) = params.image_id {
        query.push_str(&format!(" AND image_id = ${}", query_params.len() + 1));
        query_params.push(image_id.to_string());
    }

    if let Some(user_id) = params.user_id {
        query.push_str(&format!(" AND user_id = ${}", query_params.len() + 1));
        query_params.push(user_id.to_string());
    }

    query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", 
        query_params.len() + 1, query_params.len() + 2));
    query_params.push(limit.to_string());
    query_params.push(offset.to_string());

    // 実際のクエリ実行（簡単な例）
    let annotations = sqlx::query_as::<_, Annotation>(&query)
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = annotations.len();

    Ok(Json(AnnotationResponse {
        annotations,
        total,
    }))
}

// 新しいアノテーション作成
pub async fn create_annotation(
    State(pool): State<PgPool>,
    JsonExtractor(payload): JsonExtractor<CreateAnnotationRequest>,
) -> Result<Json<CreateAnnotationResponse>, StatusCode> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    // TODO: 認証からuser_idを取得（現在は仮のUUID）
    let user_id = Uuid::new_v4();

    let result = sqlx::query!(
        r#"
        INSERT INTO annotations (
            id, image_id, user_id, annotation_type, x, y, width, height, 
            label, confidence, source, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#,
        id,
        payload.image_id,
        user_id,
        payload.annotation_type as AnnotationType,
        payload.x,
        payload.y,
        payload.width,
        payload.height,
        payload.label,
        payload.confidence,
        payload.source as AnnotationSource,
        now,
        now
    )
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Ok(Json(CreateAnnotationResponse {
            id,
            message: "Annotation created successfully".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// アノテーション更新
pub async fn update_annotation(
    Path(annotation_id): Path<Uuid>,
    State(pool): State<PgPool>,
    JsonExtractor(payload): JsonExtractor<UpdateAnnotationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let now = chrono::Utc::now();

    let result = sqlx::query!(
        r#"
        UPDATE annotations 
        SET 
            x = COALESCE($2, x),
            y = COALESCE($3, y),
            width = COALESCE($4, width),
            height = COALESCE($5, height),
            label = COALESCE($6, label),
            confidence = COALESCE($7, confidence),
            updated_at = $8
        WHERE id = $1
        "#,
        annotation_id,
        payload.x,
        payload.y,
        payload.width,
        payload.height,
        payload.label,
        payload.confidence,
        now
    )
    .execute(&pool)
    .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => {
            Ok(Json(serde_json::json!({
                "message": "Annotation updated successfully"
            })))
        }
        Ok(_) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// アノテーション削除
pub async fn delete_annotation(
    Path(annotation_id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query!(
        "DELETE FROM annotations WHERE id = $1",
        annotation_id
    )
    .execute(&pool)
    .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => {
            Ok(Json(serde_json::json!({
                "message": "Annotation deleted successfully"
            })))
        }
        Ok(_) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// 特定画像のアノテーション取得
pub async fn get_image_annotations(
    Path(image_id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<AnnotationResponse>, StatusCode> {
    let annotations = sqlx::query_as::<_, Annotation>(
        "SELECT * FROM annotations WHERE image_id = $1 ORDER BY created_at DESC"
    )
    .bind(image_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = annotations.len();

    Ok(Json(AnnotationResponse {
        annotations,
        total,
    }))
}
