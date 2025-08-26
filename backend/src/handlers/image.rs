use axum::{
    extract::{State, Multipart},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{
    models::{ImageResponse, ImageSearchRequest},
    AppState,
};

// upload_image ハンドラ
pub async fn upload_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ImageResponse>, StatusCode> {
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        if field.name() == Some("image") {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

            let image_format = image::guess_format(&data).map_err(|_| StatusCode::BAD_REQUEST)?;
            let image = image::load_from_memory(&data).map_err(|_| StatusCode::BAD_REQUEST)?;
            let (width, height) = (image.width() as i32, image.height() as i32);
            let file_size = data.len() as i64;

            let uuid = Uuid::new_v4();
            let s3_key = format!("images/{}_{}", uuid, &filename);

            let mut transaction = state.db.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let user_result = sqlx::query("SELECT id FROM users LIMIT 1")
                .fetch_one(&mut *transaction).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let user_id: Uuid = user_result.get("id");

            // S3アップロード部分を修正
            let upload_result = state.s3_client
                .put_object()
                .bucket(&state.s3_bucket)
                .key(&s3_key)
                .body(data.into())
                .content_type(image_format.to_mime_type())
                .send()
                .await;

            // エラーハンドリングを別の形に書き換え
            if let Err(_) = upload_result {
                // エラーが発生した場合、トランザクションをロールバック
                if let Err(_) = transaction.rollback().await {
                    eprintln!("Failed to rollback transaction after S3 upload error");
                }
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            let result = sqlx::query(
                "INSERT INTO images (user_id, filename, original_filename, s3_key, s3_bucket, file_size, width, height, format)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 RETURNING id, created_at"
            )
            .bind(user_id)
            .bind(&filename)
            .bind(&filename)
            .bind(&s3_key)
            .bind(&state.s3_bucket)
            .bind(file_size)
            .bind(width)
            .bind(height)
            .bind(image_format.to_mime_type())
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            transaction.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let image_id: Uuid = result.get("id");
            let created_at: DateTime<Utc> = result.get("created_at");

            return Ok(Json(ImageResponse {
                id: image_id,
                filename: filename.clone(),
                original_filename: filename,
                file_size,
                width,
                height,
                format: image_format.to_mime_type().to_string(),
                classification_label: None,
                created_at,
                annotation_count: 0,
                url: format!("https://s3.{}.amazonaws.com/{}/{}", "ap-northeast-1", state.s3_bucket, s3_key),
            }));
        }
    }
    Err(StatusCode::BAD_REQUEST)
}


// search_images ハンドラ
pub async fn search_images(
    State(_state): State<AppState>, // stateを_stateにリネーム
    Json(payload): Json<ImageSearchRequest>,
) -> Result<Json<Value>, StatusCode> {
    // この関数の実装は複雑で、AIサービスとの連携も含むため、
    // いったんモック（ダミー）のレスポンスを返すようにしてコンパイルを通します。
    // 後ほど、main.rsから元のロジックを移植します。
    println!("🔍 Searching images with query: {}", payload.query);
    Ok(Json(json!({
        "success": true,
        "query": payload.query,
        "images": []
    })))
}