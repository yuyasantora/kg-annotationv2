use axum::{
    routing::{get, post, put},
    Router,
    response::Json,
    http::{StatusCode, Method, header},
    extract::{Path, Query, State, Multipart},
    Json as JsonExtractor,
};
use tower_http::cors::{CorsLayer, Any};
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, DateTime};
use sqlx::{PgPool, Row};
use std::env;
use aws_sdk_s3::Client as S3Client;
use aws_config::{load_defaults, BehaviorVersion};

mod models;
use models::{Annotation, CreateAnnotationRequest, UpdateAnnotationRequest, ImageResponse};

// アプリケーションの状態
#[derive(Clone)]
struct AppState {
    db: PgPool,
    s3_client: S3Client,
}

// クエリパラメータ用
#[derive(Deserialize)]
pub struct AnnotationQuery {
    pub image_id: Option<Uuid>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Serialize)]
pub struct AnnotationListResponse {
    pub annotations: Vec<Annotation>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct CreateAnnotationResponse {
    pub id: Uuid,
    pub message: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    println!("🦀 KG Annotation Backend starting...");

    // データベース接続
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    println!("✅ Connected to PostgreSQL");

    // AWS S3クライアントの設定
    let aws_config = load_defaults(BehaviorVersion::latest()).await;
    let s3_client = S3Client::new(&aws_config);

    println!("✅ AWS S3 client configured");

    let state = AppState { db: pool, s3_client };

    // CORSの設定を修正
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<header::HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT])
        .allow_headers([
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::AUTHORIZATION,
        ])
        .allow_credentials(true);

    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/api/auth/login", post(login_placeholder))
        // 画像関連API
        .route("/api/images", post(upload_image).get(list_images))
        .route("/api/images/:id", get(get_image))
        // アノテーション関連API
        .route("/api/annotations", get(list_annotations).post(create_annotation))
        .route("/api/annotations/:id", put(update_annotation))
        .route("/api/images/:image_id/annotations", get(get_image_annotations))
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")
        .await
        .unwrap();
        
    println!("🚀 Server running on http://0.0.0.0:3002");
    println!("📡 APIs available:");
    println!("   POST /api/images - 画像アップロード");
    println!("   GET  /api/images - 画像一覧");
    println!("   GET  /api/annotations - アノテーション一覧");
    println!("   POST /api/annotations - アノテーション作成");
    println!("   PUT  /api/annotations/:id - アノテーション更新");
    
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn login_placeholder() -> Json<Value> {
    Json(json!({
        "message": "Login endpoint placeholder",
        "status": "not_implemented"
    }))
}

// アノテーション一覧取得
async fn list_annotations(
    State(state): State<AppState>,
    Query(params): Query<AnnotationQuery>,
) -> Result<Json<AnnotationListResponse>, StatusCode> {
    println!("📋 Getting annotations list from database");

    let limit = params.limit.unwrap_or(10) as i64;
    let offset = params.offset.unwrap_or(0) as i64;

    // 条件分岐を避けて別々に実装
    let rows = match params.image_id {
        Some(image_id) => {
            sqlx::query(
                "SELECT id, image_id, user_id, annotation_type::text as annotation_type, 
                 x, y, width, height, label, confidence, source::text as source, 
                 created_at, updated_at 
                 FROM annotations 
                 WHERE image_id = $1 
                 ORDER BY created_at DESC 
                 LIMIT $2 OFFSET $3"
            )
            .bind(image_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(|e| {
                println!("❌ Database error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
        }
        None => {
            sqlx::query(
                "SELECT id, image_id, user_id, annotation_type::text as annotation_type, 
                 x, y, width, height, label, confidence, source::text as source, 
                 created_at, updated_at 
                 FROM annotations 
                 ORDER BY created_at DESC 
                 LIMIT $1 OFFSET $2"
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(|e| {
                println!("❌ Database error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
        }
    };

    // 手動でAnnotation構造体に変換
    let annotations: Vec<Annotation> = rows.into_iter().map(|row| {
        Annotation {
            id: row.get("id"),
            image_id: row.get("image_id"),
            user_id: row.get("user_id"),
            annotation_type: row.get::<Option<String>, _>("annotation_type")
                .unwrap_or_else(|| "boundingbox".to_string()),
            x: row.get("x"),
            y: row.get("y"),
            width: row.get("width"),
            height: row.get("height"),
            label: row.get("label"),
            confidence: row.get("confidence"),
            source: row.get::<Option<String>, _>("source")
                .unwrap_or_else(|| "manual".to_string()),
            created_at: row.get::<Option<DateTime<Utc>>, _>("created_at")
                .unwrap_or_else(|| Utc::now()),
            updated_at: row.get::<Option<DateTime<Utc>>, _>("updated_at")
                .unwrap_or_else(|| Utc::now()),
        }
    }).collect();

    // 総数取得も同様の分岐を作成
    let total = match params.image_id {
        Some(image_id) => {
            sqlx::query("SELECT COUNT(*) as count FROM annotations WHERE image_id = $1")
                .bind(image_id)
                .fetch_one(&state.db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .get::<i64, _>("count") as usize
        }
        None => {
            sqlx::query("SELECT COUNT(*) as count FROM annotations")
                .fetch_one(&state.db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .get::<i64, _>("count") as usize
        }
    };

    Ok(Json(AnnotationListResponse {
        annotations,
        total,
    }))
}

// アノテーション作成
async fn create_annotation(
    State(state): State<AppState>,
    JsonExtractor(payload): JsonExtractor<CreateAnnotationRequest>,
) -> Result<Json<CreateAnnotationResponse>, StatusCode> {
    println!("➕ Creating annotation in database for image_id: {}", payload.image_id);
    println!("   Label: {}, Position: ({}, {}), Size: {}x{}", 
        payload.label, payload.x, payload.y, payload.width, payload.height);

    // 実際に存在するuser_idを取得
    let user_result = sqlx::query("SELECT id FROM users LIMIT 1")
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            println!("❌ Failed to get user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let user_id: Uuid = user_result.get("id");
    
    // labelを先に保存しておく
    let label_for_message = payload.label.clone();

    // ENUMを文字列として挿入
    let result = sqlx::query(
        "INSERT INTO annotations (image_id, user_id, annotation_type, x, y, width, height, label, confidence, source)
         VALUES ($1, $2, 'boundingbox'::annotation_type, $3, $4, $5, $6, $7, $8, 'manual'::annotation_source)
         RETURNING id"
    )
    .bind(payload.image_id)
    .bind(user_id)
    .bind(payload.x)
    .bind(payload.y)
    .bind(payload.width)
    .bind(payload.height)
    .bind(payload.label)
    .bind(payload.confidence)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        println!("❌ Database error: {}", e);
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    Ok(Json(CreateAnnotationResponse {
        id: result.get("id"),
        message: format!("Annotation '{}' created successfully", label_for_message),
    }))
}

// アノテーション更新
async fn update_annotation(
    State(state): State<AppState>,
    Path(annotation_id): Path<Uuid>,
    JsonExtractor(payload): JsonExtractor<UpdateAnnotationRequest>,
) -> Result<Json<Value>, StatusCode> {
    println!("✏️ Updating annotation {} in database", annotation_id);

    let result = sqlx::query(
        "UPDATE annotations 
         SET x = COALESCE($1, x),
             y = COALESCE($2, y),
             width = COALESCE($3, width),
             height = COALESCE($4, height),
             label = COALESCE($5, label),
             confidence = COALESCE($6, confidence),
             updated_at = NOW()
         WHERE id = $7
         RETURNING id"
    )
    .bind(payload.x)
    .bind(payload.y)
    .bind(payload.width)
    .bind(payload.height)
    .bind(payload.label)
    .bind(payload.confidence)
    .bind(annotation_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        println!("❌ Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match result {
        Some(_) => Ok(Json(json!({
            "message": format!("Annotation {} updated successfully", annotation_id)
        }))),
        None => Err(StatusCode::NOT_FOUND)
    }
}

// 特定画像のアノテーション取得
async fn get_image_annotations(
    State(state): State<AppState>,
    Path(image_id): Path<Uuid>,
) -> Result<Json<AnnotationListResponse>, StatusCode> {
    println!("🖼️ Getting annotations for image {} from database", image_id);

    let rows = sqlx::query(
        "SELECT id, image_id, user_id, annotation_type::text as annotation_type, 
         x, y, width, height, label, confidence, source::text as source, 
         created_at, updated_at 
         FROM annotations 
         WHERE image_id = $1 
         ORDER BY created_at DESC"
    )
    .bind(image_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        println!("❌ Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let annotations: Vec<Annotation> = rows.into_iter().map(|row| {
        Annotation {
            id: row.get("id"),
            image_id: row.get("image_id"),
            user_id: row.get("user_id"),
            annotation_type: row.get::<Option<String>, _>("annotation_type")
                .unwrap_or_else(|| "boundingbox".to_string()),
            x: row.get("x"),
            y: row.get("y"),
            width: row.get("width"),
            height: row.get("height"),
            label: row.get("label"),
            confidence: row.get("confidence"),
            source: row.get::<Option<String>, _>("source")
                .unwrap_or_else(|| "manual".to_string()),
            created_at: row.get::<Option<DateTime<Utc>>, _>("created_at")
                .unwrap_or_else(|| Utc::now()),
            updated_at: row.get::<Option<DateTime<Utc>>, _>("updated_at")
                .unwrap_or_else(|| Utc::now()),
        }
    }).collect();

    let total = annotations.len();

    Ok(Json(AnnotationListResponse {
        annotations,
        total,
    }))
}

// 画像アップロード
async fn upload_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ImageResponse>, StatusCode> {
    println!("📤 Processing image upload...");

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("");
        
        if name == "image" {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let content_type = field.content_type().unwrap_or("").to_string();
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            
            println!("📁 Uploading file: {} ({} bytes)", filename, data.len());
            
            // データサイズを先に保存
            let file_size = data.len() as i64;
            
            // 画像サイズを取得
            let image = image::load_from_memory(&data)
                .map_err(|_| StatusCode::BAD_REQUEST)?;
            let (width, height) = (image.width() as i32, image.height() as i32);
            
            // S3にアップロード
            let s3_key = format!("images/{}/{}", Uuid::new_v4(), &filename);
            let s3_bucket = env::var("S3_BUCKET").unwrap_or_else(|_| "kgbacket".to_string());
            
            // S3アップロード実行
            let upload_result = state.s3_client
                .put_object()
                .bucket(&s3_bucket)
                .key(&s3_key)
                .body(data.clone().into())  // dataをクローン
                .content_type(&content_type)
                .send()
                .await;
            
            match upload_result {
                Ok(_) => {
                    println!("☁️ Successfully uploaded to S3: s3://{}/{}", s3_bucket, s3_key);
                }
                Err(e) => {
                    println!("❌ Failed to upload to S3: {}", e);
                    println!("❌ Error details: {:#?}", e);  // より詳細なエラー情報
                    println!("🔍 AWS Environment:");
                    println!("  Bucket: {}", s3_bucket);
                    println!("  Region: {}", std::env::var("AWS_REGION").unwrap_or_else(|_| "not set".to_string()));
                    println!("  Access Key: {}", std::env::var("AWS_ACCESS_KEY_ID").map(|k| "set".to_string()).unwrap_or_else(|_| "not set".to_string()));
                    println!("  Secret Key: {}", std::env::var("AWS_SECRET_ACCESS_KEY").map(|_| "set".to_string()).unwrap_or_else(|_| "not set".to_string()));
                    
                    // バケットの存在確認を試みる
                    match state.s3_client.head_bucket()
                        .bucket(&s3_bucket)
                        .send()
                        .await
                    {
                        Ok(_) => println!("✅ Bucket exists and is accessible"),
                        Err(e) => println!("❌ Bucket error: {}", e),
                    }
                }
            }
            
            // データベースに保存
            let user_result = sqlx::query("SELECT id FROM users LIMIT 1")
                .fetch_one(&state.db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let user_id: Uuid = user_result.get("id");
            
            let result = sqlx::query(
                "INSERT INTO images (user_id, filename, original_filename, s3_key, s3_bucket, file_size, width, height, format)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 RETURNING id, created_at"
            )
            .bind(user_id)
            .bind(&filename)  // filenameをリファレンスとして渡す
            .bind(&filename)  // 同様
            .bind(&s3_key)
            .bind(&s3_bucket)
            .bind(file_size)  // 保存しておいたサイズを使用
            .bind(width)
            .bind(height)
            .bind(content_type.split('/').last().unwrap_or("unknown"))
            .fetch_one(&state.db)
            .await
            .map_err(|e| {
                println!("❌ Database error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            
            let image_id: Uuid = result.get("id");
            let created_at: DateTime<Utc> = result.get("created_at");
            
            println!("✅ Image uploaded successfully: {}", image_id);
            
            return Ok(Json(ImageResponse {
                id: image_id,
                filename: filename.clone(),  // クローン
                original_filename: filename,  // 所有権を移動
                file_size,
                width,
                height,
                format: content_type.split('/').last().unwrap_or("unknown").to_string(),
                classification_label: None,
                created_at,
                annotation_count: 0,
            }));
        }
    }
    
    Err(StatusCode::BAD_REQUEST)
}

// 画像一覧取得
async fn list_images(
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    println!("📋 Getting images list from database");

    let rows = sqlx::query(
        "SELECT i.id, i.filename, i.original_filename, i.file_size, i.width, i.height, 
         i.format, i.classification_label, i.created_at,
         COUNT(a.id) as annotation_count
         FROM images i
         LEFT JOIN annotations a ON i.id = a.image_id
         GROUP BY i.id, i.filename, i.original_filename, i.file_size, i.width, i.height, 
         i.format, i.classification_label, i.created_at
         ORDER BY i.created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        println!("❌ Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let images: Vec<ImageResponse> = rows.into_iter().map(|row| {
        ImageResponse {
            id: row.get("id"),
            filename: row.get("filename"),
            original_filename: row.get("original_filename"),
            file_size: row.get("file_size"),
            width: row.get("width"),
            height: row.get("height"),
            format: row.get("format"),
            classification_label: row.get("classification_label"),
            created_at: row.get("created_at"),
            annotation_count: row.get::<i64, _>("annotation_count"),
        }
    }).collect();

    Ok(Json(json!({
        "images": images,
        "total": images.len()
    })))
}

// 特定画像の詳細取得
async fn get_image(
    State(state): State<AppState>,
    Path(image_id): Path<Uuid>,
) -> Result<Json<ImageResponse>, StatusCode> {
    println!("🖼️ Getting image {} details", image_id);

    let row = sqlx::query(
        "SELECT i.id, i.filename, i.original_filename, i.file_size, i.width, i.height, 
         i.format, i.classification_label, i.created_at,
         COUNT(a.id) as annotation_count
         FROM images i
         LEFT JOIN annotations a ON i.id = a.image_id
         WHERE i.id = $1
         GROUP BY i.id, i.filename, i.original_filename, i.file_size, i.width, i.height, 
         i.format, i.classification_label, i.created_at"
    )
    .bind(image_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        println!("❌ Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let image = ImageResponse {
                id: row.get("id"),
                filename: row.get("filename"),
                original_filename: row.get("original_filename"),
                file_size: row.get("file_size"),
                width: row.get("width"),
                height: row.get("height"),
                format: row.get("format"),
                classification_label: row.get("classification_label"),
                created_at: row.get("created_at"),
                annotation_count: row.get::<i64, _>("annotation_count"),
            };
            Ok(Json(image))
        }
        None => Err(StatusCode::NOT_FOUND)
    }
}
