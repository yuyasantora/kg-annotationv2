use axum::{extract::Path, http::StatusCode, response::Json, extract::State};
use uuid::Uuid;
use crate::{
    models::{Annotation, CreateAnnotationRequest, UpdateAnnotationRequest, CreateAnnotationResponse},
    AppState,
    utils::json::JsonExtractor,
};

// 新しいアノテーション作成
pub async fn create_annotation(
    State(state): State<AppState>,
    JsonExtractor(payload): JsonExtractor<CreateAnnotationRequest>,
) -> Result<Json<CreateAnnotationResponse>, StatusCode> {
    // 画像が存在するか確認
    let image_exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM images WHERE id = $1)",
        payload.image_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !image_exists.unwrap_or(false) {
        eprintln!("Image not found: {}", payload.image_id);
        return Err(StatusCode::NOT_FOUND);
    }

    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    
    // TODO: 認証からuser_idを取得（現在は仮のUUID）
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users LIMIT 1")
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Failed to fetch a user from the database. Is it seeded? Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Bboxをリクエストのx,y,width,heightから作成
    let bbox = vec![payload.x, payload.y, payload.width, payload.height];

    sqlx::query(
        r#"
        INSERT INTO annotations (id, image_id, user_id, annotation_type, x, y, width, height, label, source, confidence, created_at, updated_at, bbox, points)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        "#,
    )
    .bind(id)
    .bind(payload.image_id)
    .bind(user_id)
    .bind(payload.annotation_type)
    .bind(payload.x)
    .bind(payload.y)
    .bind(payload.width)
    .bind(payload.height)
    .bind(payload.label)
    .bind(payload.source)
    .bind(payload.confidence)
    .bind(now)
    .bind(now)
    .bind(&bbox) // payload.bboxの代わりに作成したbboxをバインド
    .bind(payload.points)
    .execute(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to create annotation: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(CreateAnnotationResponse { id }))
}

// 画像に紐づくアノテーション全取得
pub async fn get_annotations_for_image(
    State(state): State<AppState>,
    Path(image_id): Path<Uuid>,
) -> Result<Json<Vec<Annotation>>, StatusCode> {
    sqlx::query_as!(
        Annotation,
        r#"
        SELECT 
            id, image_id, user_id, 
            annotation_type as "annotation_type: _",
            x, y, width, height, points, bbox, label,
            source as "source: _",
            confidence, 
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM annotations WHERE image_id = $1
        "#,
        image_id
    )
    .fetch_all(&state.db)
    .await
    .map(Json)
    .map_err(|e| {
        eprintln!("Failed to fetch annotations for image: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

// アノテーション取得
pub async fn get_annotation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Annotation>, StatusCode> {
    sqlx::query_as!(
        Annotation,
        r#"
        SELECT 
            id, image_id, user_id, 
            annotation_type as "annotation_type: _",
            x, y, width, height, points, bbox, label,
            source as "source: _",
            confidence, 
            created_at as "created_at!", 
            updated_at as "updated_at!"
        FROM annotations WHERE id = $1
        "#,
        id
    )
    .fetch_one(&state.db)
    .await
    .map(Json)
    .map_err(|_| StatusCode::NOT_FOUND)
}

// アノテーション更新
pub async fn update_annotation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    JsonExtractor(payload): JsonExtractor<UpdateAnnotationRequest>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        r#"
        UPDATE annotations
        SET 
            annotation_type = $1, x = $2, y = $3, width = $4, height = $5, 
            points = $6, bbox = $7, label = $8, confidence = $9, updated_at = $10
        WHERE id = $11
        "#,
    )
    .bind(payload.annotation_type)
    .bind(payload.x)
    .bind(payload.y)
    .bind(payload.width)
    .bind(payload.height)
    .bind(payload.points)
    .bind(payload.bbox)
    .bind(payload.label)
    .bind(payload.confidence)
    .bind(chrono::Utc::now())
    .bind(id)
    .execute(&state.db)
    .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => Ok(StatusCode::OK),
        Ok(_) => Ok(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to update annotation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// アノテーション削除
pub async fn delete_annotation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM annotations WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => Ok(StatusCode::NO_CONTENT),
        Ok(_) => Ok(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to delete annotation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(serde::Serialize)]
pub struct LabelsResponse {
    labels: Vec<String>,
}

pub async fn get_available_labels(
    State(state): State<AppState>,
) -> Result<Json<LabelsResponse>, StatusCode> {
    let labels = sqlx::query_scalar("SELECT DISTINCT label FROM annotations ORDER BY label")
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Failed to fetch distinct labels: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(LabelsResponse { labels }))
}
