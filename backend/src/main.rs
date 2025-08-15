use axum::{
    routing::{get, post, put, delete},
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
use chrono::Utc;
use sqlx::{PgPool, Row};
use std::env;

mod models;
use models::{Annotation, CreateAnnotationRequest, UpdateAnnotationRequest};

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
    println!("🦀 KG Annotation Backend starting...");

    // 環境変数の読み込み
    dotenv::dotenv().ok();
    
    // データベース接続
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    println!("🔗 Connecting to database...");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create pool");

    // データベース接続テスト
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => println!("✅ Database connection successful"),
        Err(e) => {
            println!("❌ Database connection failed: {}", e);
            std::process::exit(1);
        }
    }

    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/api/auth/login", post(login_placeholder))
        // 実際のDB接続版のAPI
        .route("/api/annotations", get(list_annotations).post(create_annotation))
        .route("/api/annotations/:id", put(update_annotation).delete(delete_annotation))
        .route("/api/images/:image_id/annotations", get(get_image_annotations))
        .with_state(pool)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")
        .await
        .unwrap();
        
    println!("🚀 Server running on http://0.0.0.0:3002");
    println!("🗄️ Database APIs available:");
    println!("   GET  /api/annotations - アノテーション一覧");
    println!("   POST /api/annotations - アノテーション作成");
    println!("   PUT  /api/annotations/:id - アノテーション更新"); 
    println!("   DELETE /api/annotations/:id - アノテーション削除");
    println!("   GET  /api/images/:image_id/annotations - 画像のアノテーション");
    
    axum::serve(listener, app).await.unwrap();
}

// モック用のヘルパー関数
fn create_mock_annotation(id: Uuid, image_id: Uuid, label: &str, x: f32, y: f32, w: f32, h: f32) -> Annotation {
    Annotation {
        id,
        image_id,
        user_id: Uuid::new_v4(),
        annotation_type: "bbox".to_string(),
        x,
        y,
        width: w,
        height: h,
        label: label.to_string(),
        confidence: Some(0.85),
        source: "manual".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

// 実際のDB版のハンドラー関数
async fn list_annotations(
    Query(params): Query<AnnotationQuery>,
    State(pool): State<PgPool>,
) -> Result<Json<AnnotationListResponse>, StatusCode> {
    println!("📋 Getting annotations from database");

    // SQLクエリの構築
    let mut query_str = "SELECT * FROM annotations WHERE 1=1".to_string();
    let mut query_params = Vec::new();

    if let Some(image_id) = params.image_id {
        query_str.push_str(&format!(" AND image_id = ${}", query_params.len() + 1));
        query_params.push(image_id.to_string());
    }

    let limit = params.limit.unwrap_or(50) as i64;
    let offset = params.offset.unwrap_or(0) as i64;

    query_str.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", 
        query_params.len() + 1, query_params.len() + 2));

    // 実際のクエリ実行
    let rows = sqlx::query(&query_str)
        .bind(limit)
        .bind(offset)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            println!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let annotations: Vec<Annotation> = rows.into_iter().map(|row| {
        Annotation {
            id: row.get("id"),
            image_id: row.get("image_id"),
            user_id: row.get("user_id"),
            annotation_type: row.get("annotation_type"),
            x: row.get("x"),
            y: row.get("y"),
            width: row.get("width"),
            height: row.get("height"),
            label: row.get("label"),
            confidence: row.get("confidence"),
            source: row.get("source"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }).collect();

    println!("Found {} annotations", annotations.len());

    Ok(Json(AnnotationListResponse {
        annotations: annotations.clone(),
        total: annotations.len(),
    }))
}

// アノテーション作成（モック）
async fn create_annotation_mock(
    JsonExtractor(payload): JsonExtractor<CreateAnnotationRequest>,
) -> Result<Json<CreateAnnotationResponse>, StatusCode> {
    println!("➕ Mock: Creating annotation for image_id: {}", payload.image_id);
    println!("   Label: {}, Position: ({}, {}), Size: {}x{}", 
        payload.label, payload.x, payload.y, payload.width, payload.height);

    let new_id = Uuid::new_v4();

    Ok(Json(CreateAnnotationResponse {
        id: new_id,
        message: format!("Annotation '{}' created successfully", payload.label),
    }))
}

// アノテーション更新（モック）
async fn update_annotation_mock(
    Path(annotation_id): Path<Uuid>,
    JsonExtractor(payload): JsonExtractor<UpdateAnnotationRequest>,
) -> Result<Json<Value>, StatusCode> {
    println!("✏️ Mock: Updating annotation {}", annotation_id);
    
    if let Some(label) = &payload.label {
        println!("   New label: {}", label);
    }
    if let Some(x) = payload.x {
        println!("   New position: ({}, {})", x, payload.y.unwrap_or(0.0));
    }

    Ok(Json(json!({
        "message": format!("Annotation {} updated successfully", annotation_id)
    })))
}

// アノテーション削除（モック）
async fn delete_annotation_mock(
    Path(annotation_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    println!("🗑️ Mock: Deleting annotation {}", annotation_id);

    Ok(Json(json!({
        "message": format!("Annotation {} deleted successfully", annotation_id)
    })))
}

// 特定画像のアノテーション取得（モック）
async fn get_image_annotations_mock(
    Path(image_id): Path<Uuid>,
) -> Result<Json<AnnotationListResponse>, StatusCode> {
    println!("🖼️ Mock: Getting annotations for image {}", image_id);

    // その画像用のモックアノテーション
    let mock_annotations = vec![
        create_mock_annotation(
            Uuid::new_v4(),
            image_id,
            "detected_object_1",
            120.0, 80.0, 180.0, 160.0
        ),
        create_mock_annotation(
            Uuid::new_v4(),
            image_id,
            "detected_object_2",
            350.0, 220.0, 100.0, 80.0
        ),
    ];

    Ok(Json(AnnotationListResponse {
        annotations: mock_annotations.clone(),
        total: mock_annotations.len(),
    }))
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "kg-annotation-backend",
        "version": "0.1.0",
        "features": ["mock_annotations"]
    }))
}

async fn login_placeholder() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "message": "Login endpoint - coming soon",
        "status": "placeholder"
    })))
}
