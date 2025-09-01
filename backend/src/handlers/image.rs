use axum::{
    extract::{State, Multipart, Path},
    http::StatusCode,
    response::Json,
    response::Response,
    body::Body,
};
use aws_sdk_s3::Client as S3Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{postgres::PgRow, FromRow, Row};
use uuid::Uuid;
use chrono::Utc;
use reqwest;
use image::GenericImageView;
use crate::{
    models::{ImageResponse, ImageSearchRequest},
    AppState,
};
use axum::http::header; // axumのheaderを使用
use std::time::Duration;
use aws_sdk_s3::presigning::PresigningConfig;

// S3事前署名URL作成ハンドラ
#[derive(Deserialize)]
pub struct PresignedUrlRequest {
    filename: String
}

#[derive(Serialize)]
pub struct PresignedUrlResponse {
    url: String,
    s3_key: String,
}

pub async fn generate_presigned_url (
    State(state): State<AppState>,
    Json(payload): Json<PresignedUrlRequest>,
) -> Result<Json<PresignedUrlResponse>, StatusCode> {
    let uuid = Uuid::new_v4();
    let s3_key = format!("images/{}_{}", uuid, payload.filename);

    let presigning_config = PresigningConfig::expires_in(Duration::from_secs(300))
        .map_err(|e| {
            eprintln!("Failed to create presigning config: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR

        })?;

    let presigned_request = state.s3_client
        .put_object()
        .bucket(&state.s3_bucket)
        .key(&s3_key)
        .presigned(presigning_config)
        .await
        .map_err(|e| {
            eprintln!("Failed to generate presigned URL: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(PresignedUrlResponse {
        url: presigned_request.uri().to_string(),
        s3_key,
    }))
}

// 画像アップロードハンドラ
#[derive(Deserialize)]
pub struct RegisterImageRequest {
    s3_key: String,
    original_filename: String,
    file_size: i64,
    width: i32,
    height: i32,
    format: String,
}

#[derive(Serialize)]
pub struct RegisterImageResponse {
    id: Uuid,
}

pub async fn register_uploaded_image(
    State(state): State<AppState>,
    Json(payload): Json<RegisterImageRequest>,
) -> Result<Json<RegisterImageResponse>, StatusCode> {
    let id = Uuid::new_v4();
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users LIMIT 1")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query!(
        r#"
        INSERT INTO images 
            (id, user_id, s3_bucket, s3_key, width, height, format, 
            original_filename, filename, file_size, created_at, updated_at)
        VALUES 
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
        "#,
        id,
        user_id,
        &state.s3_bucket,
        payload.s3_key,
        payload.width,
        payload.height,
        payload.format,
        payload.original_filename,
        payload.original_filename, // filename にも同じ値を使用
        payload.file_size
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to register image in DB: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(RegisterImageResponse { id }))
}


    
// upload_image ハンドラ
pub async fn upload_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ImageResponse>, StatusCode> {
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        if field.name() == Some("image") {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

            let image_format = image::guess_format(&data)
                .map_err(|_| StatusCode::BAD_REQUEST)?;

            let uuid = Uuid::new_v4();
            let s3_key = format!("images/{}_{}", uuid, &filename);

            // AIサービスで画像をベクトル化
            let client = reqwest::Client::new();
            let ai_service_url = "http://localhost:8001";
            
            // バイトデータをVec<u8>に変換
            let data_vec = data.to_vec();
            
            // multipart/form-dataフォームを作成
            let part = reqwest::multipart::Part::bytes(data_vec)
                .file_name(filename.clone())
                .mime_str(&image_format.to_mime_type())
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let form = reqwest::multipart::Form::new().part("image", part);

            let vectorize_res = client
                .post(format!("{}/vectorize_image", ai_service_url))
                .multipart(form)
                .send()
                .await;

            let image_vector_value: Option<Value> = match vectorize_res {
                Ok(res) if res.status().is_success() => res.json::<Value>().await.ok(),
                _ => None,
            };
            
            let image_vector: Option<Vec<f32>> = image_vector_value
                .and_then(|val| val.get("vector").cloned())
                .and_then(|vec_val| serde_json::from_value(vec_val).ok());


            let mut transaction = state.db.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let s3_upload_result = state.s3_client
                .put_object()
                .bucket(&state.s3_bucket)
                .key(&s3_key)
                .body(data.clone().into())
                .content_type(image_format.to_mime_type())
                .send()
                .await;
            
            if let Err(e) = s3_upload_result {
                eprintln!("S3 upload failed: {}", e);
                transaction.rollback().await.ok();
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            
            let image_obj = image::load_from_memory(&data).map_err(|_| StatusCode::BAD_REQUEST)?;
            let (width, height) = image_obj.dimensions();
            let created_at = Utc::now();

            // TODO: 認証機能が実装されるまで、仮のユーザーIDを使用
            // データベースから最初のユーザーのIDを取得
            let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users LIMIT 1")
                .fetch_one(&state.db)
                .await
                .map_err(|e| {
                    eprintln!("Failed to fetch a user from the database. Is it seeded? Error: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            let new_image_id: Uuid = sqlx::query(
                r#"
                INSERT INTO images (id, user_id, s3_bucket, s3_key, width, height, format, classification_label, created_at, vector, filename, original_filename, file_size)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                RETURNING id
                "#,
            )
            .bind(uuid)
            .bind(user_id)
            .bind(&state.s3_bucket)
            .bind(&s3_key)
            .bind(width as i32)
            .bind(height as i32)
            .bind(image_format.to_mime_type().to_string())
            .bind(None::<String>)
            .bind(created_at)
            .bind(image_vector.map(serde_json::to_value).transpose().map_err(|e| {
                eprintln!("Failed to serialize vector: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?)
            .bind(&filename) // $11
            .bind(&filename) // $12 original_filename
            .bind(data.len() as i64) // $13 file_size
            .map(|row: PgRow| row.get("id"))
            .fetch_one(&mut *transaction)
            .await
            .map_err(|e| {
                eprintln!("DB insert failed: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            transaction.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            return Ok(Json(ImageResponse {
                id: new_image_id,
                s3_key,
                created_at,
            }));
        }
    }

    Err(StatusCode::BAD_REQUEST)
}


// --- search_images ハンドラ ---

#[derive(FromRow, Debug)]
struct ImageVector {
    id: Uuid,
    vector: Option<serde_json::Value>,
}

// タイプミスを修正: VectorTextResponse -> VectorizeTextResponse
#[derive(Deserialize, Debug)]
struct VectorizeTextResponse {
    vectors: Vec<Vec<f32>>,
}

#[derive(Serialize)]
struct SearchSimilarRequest {
    query_vector: Vec<f32>,
    vectors: Vec<Vec<f32>>,
    ids: Vec<String>, // UuidからStringに変更
    top_k: usize,
}

#[derive(Deserialize, Debug)]
struct SearchResultItem {
    id: String, // UuidからStringに変更
    similarity: f32,
}

#[derive(Deserialize, Debug)]
struct SearchSimilarResponse {
    results: Vec<SearchResultItem>,
}

// 検索結果の型を定義
#[derive(Serialize)]
pub struct SearchResultWithSimilarity {
    pub id: Uuid,
    pub similarity: f32,
}

pub async fn search_images(
    State(state): State<AppState>,
    Json(payload): Json<ImageSearchRequest>,
) -> Result<Json<Vec<SearchResultWithSimilarity>>, StatusCode> {
    // 1. DBから全画像ベクトルを取得
    let image_vectors = sqlx::query_as::<_, ImageVector>(
        "SELECT id, vector FROM images WHERE vector IS NOT NULL"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch image vectors: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if image_vectors.is_empty() {
        return Ok(Json(vec![]));
    }

    // 2. 検索テキストをベクトル化
    let client = reqwest::Client::new();
    let ai_service_url = "http://localhost:8001";
    
    // リクエストの形式を修正
    let vectorize_req = vec![payload.query];  // 直接文字列の配列を送信

    let vectorize_res = client
        .post(format!("{}/vectorize_text", ai_service_url))
        .json(&vectorize_req)  // VectorizeTextRequestではなく、直接配列を送信
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .json::<VectorizeTextResponse>()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let query_vector = match vectorize_res.vectors.into_iter().next() {
        Some(vec) => vec,
        None => return Ok(Json(vec![])),
    };

    // 3. 類似画像を検索
    let (ids, vectors): (Vec<Uuid>, Vec<Vec<f32>>) = image_vectors
        .into_iter()
        .filter_map(|iv| {
            iv.vector
                .and_then(|v| serde_json::from_value(v).ok())
                .map(|vec_f32| (iv.id, vec_f32))
        })
        .unzip();
        
    let search_req = SearchSimilarRequest {
        query_vector,
        vectors,
        // UuidをStringに変換
        ids: ids.into_iter().map(|id| id.to_string()).collect(),
        top_k: 10,
    };

    let search_res = client
        .post(format!("{}/search_similar_images", ai_service_url))
        .json(&search_req)
        .send()
        .await
        .map_err(|e| {
            eprintln!("Failed to call search API: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .json::<SearchSimilarResponse>()
        .await
        .map_err(|e| {
            eprintln!("Failed to parse search API response: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 4. 結果を返す (StringをUuidにパースし直す)
    let result_items = search_res.results.into_iter()
        .filter_map(|item| Uuid::parse_str(&item.id).ok().map(|uuid| SearchResultWithSimilarity {
            id: uuid,
            similarity: item.similarity,
        }))
        .collect();

    Ok(Json(result_items))
}

// 画像取得ハンドラを追加
pub async fn get_image(
    State(state): State<AppState>,
    Path(image_id): Path<Uuid>,
) -> Result<Response<Body>, StatusCode> {
    // 1. データベースから画像情報を取得
    let image = sqlx::query!(
        r#"
        SELECT s3_bucket, s3_key, format
        FROM images
        WHERE id = $1
        "#,
        image_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch image info: {}", e);
        StatusCode::NOT_FOUND
    })?;

    // 2. S3から画像データを取得
    let get_object_output = state.s3_client
        .get_object()
        .bucket(&image.s3_bucket)
        .key(&image.s3_key)
        .send()
        .await
        .map_err(|e| {
            eprintln!("Failed to get object from S3: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 3. レスポンスを構築
    let data = get_object_output
        .body
        .collect()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_bytes();

    // axumのResponseBuilderを使用
    let response = Response::builder()
        .header(header::CONTENT_TYPE.as_str(), image.format)
        .header(header::CACHE_CONTROL.as_str(), "public, max-age=31536000")
        .body(Body::from(data))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

