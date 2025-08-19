use axum::{
    routing::{get, post, put},
    Router,
    response::Json,
    http::StatusCode,
    extract::{Path, Query, State},
    Json as JsonExtractor,
};
use tower_http::cors::CorsLayer;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, DateTime};
use sqlx::{PgPool, Row};
use std::env;

mod models;
use models::{Annotation, CreateAnnotationRequest, UpdateAnnotationRequest};

// アプリケーションの状態
#[derive(Clone)]
struct AppState {
    db: PgPool,
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

    let state = AppState { db: pool };

    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/api/auth/login", post(login_placeholder))
        // 実際のDB接続API
        .route("/api/annotations", get(list_annotations).post(create_annotation))
        .route("/api/annotations/:id", put(update_annotation))
        .route("/api/images/:image_id/annotations", get(get_image_annotations))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")
        .await
        .unwrap();
        
    println!("🚀 Server running on http://0.0.0.0:3002");
    println!("📡 Database APIs available:");
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
