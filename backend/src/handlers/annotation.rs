use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Json as JsonExtractor,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// modelsから必要なものをインポート
use crate::models::{
    Annotation, CreateAnnotationRequest, UpdateAnnotationRequest, AnnotationSource, AnnotationType,
};
// AppStateをインポート
use crate::AppState;

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
    State(state): State<AppState>, // State<PgPool> から State<AppState> に変更
) -> Result<Json<AnnotationResponse>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    // FromRowを実装したので、query_as!が使えるようになり、安全になる
    // ここではまずコンパイルを通すために、簡単なクエリで修正
    let annotations = sqlx::query_as!(
        Annotation,
        r#"
        SELECT 
            id, image_id, user_id, 
            annotation_type as "annotation_type: _", 
            x, y, width, height, label, confidence, 
            source as "source: _", 
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM annotations
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(&state.db) // pool を state.db に変更
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch annotations: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total = annotations.len();

    Ok(Json(AnnotationResponse {
        annotations,
        total,
    }))
}

// 新しいアノテーション作成
pub async fn create_annotation(
    State(state): State<AppState>, // State<PgPool> から State<AppState> に変更
    JsonExtractor(payload): JsonExtractor<CreateAnnotationRequest>,
) -> Result<Json<CreateAnnotationResponse>, StatusCode> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let user_id = Uuid::new_v4(); // TODO: 認証からuser_idを取得

    sqlx::query!(
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
    .execute(&state.db) // pool を state.db に変更
    .await
    .map(|_| {
        Json(CreateAnnotationResponse {
            id,
            message: "Annotation created successfully".to_string(),
        })
    })
    .map_err(|e| {
        eprintln!("Failed to create annotation: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

// アノテーション更新
pub async fn update_annotation(
    Path(annotation_id): Path<Uuid>,
    State(state): State<AppState>,
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
    .execute(&state.db) // pool を state.db に変更
    .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => {
            Ok(Json(serde_json::json!({
                "message": "Annotation updated successfully"
            })))
        }
        Ok(_) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to update annotation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// アノテーション削除
pub async fn delete_annotation(
    Path(annotation_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = sqlx::query!(
        "DELETE FROM annotations WHERE id = $1",
        annotation_id
    )
    .execute(&state.db) // pool を state.db に変更
    .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => {
            Ok(Json(serde_json::json!({
                "message": "Annotation deleted successfully"
            })))
        }
        Ok(_) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to delete annotation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 特定画像のアノテーション取得
pub async fn get_image_annotations(
    Path(image_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<AnnotationResponse>, StatusCode> {
    let annotations = sqlx::query_as!(
        Annotation,
        r#"
        SELECT 
            id, image_id, user_id, 
            annotation_type as "annotation_type: _", 
            x, y, width, height, label, confidence, 
            source as "source: _", 
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM annotations 
        WHERE image_id = $1 
        ORDER BY created_at DESC
        "#,
        image_id
    )
    .fetch_all(&state.db) // pool を state.db に変更
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch image annotations: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total = annotations.len();

    Ok(Json(AnnotationResponse {
        annotations,
        total,
    }))
}

#[derive(Serialize)]
pub struct LabelsResponse {
    pub labels: Vec<String>,
}

// 利用可能なアノテーションラベル一覧を取得する関数
pub async fn get_distinct_labels(
    State(state): State<AppState>,
) -> Result<Json<LabelsResponse>, StatusCode> {
    let labels = sqlx::query!(
        r#"
        SELECT DISTINCT label FROM annotations ORDER BY label
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to query distinct labels: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .into_iter()
    .map(|rec| rec.label)
    .collect();

    Ok(Json(LabelsResponse { labels }))
}
