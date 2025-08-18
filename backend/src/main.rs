use axum::{
    routing::{get, post, put},
    Router,
    response::Json,
    http::StatusCode,
    extract::{Path, Query},
    Json as JsonExtractor,
};
use tower_http::cors::CorsLayer;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

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

    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/api/auth/login", post(login_placeholder))
        // アノテーション関連のモックAPI
        .route("/api/annotations", get(list_annotations_mock).post(create_annotation_mock))
        .route("/api/annotations/:id", put(update_annotation_mock))
        .route("/api/images/:image_id/annotations", get(get_image_annotations_mock))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")
        .await
        .unwrap();
        
    println!("🚀 Server running on http://0.0.0.0:3002");
    println!("📡 Mock APIs available:");
    println!("   GET  /api/annotations - アノテーション一覧");
    println!("   POST /api/annotations - アノテーション作成");
    println!("   PUT  /api/annotations/:id - アノテーション更新");
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

// アノテーション一覧取得（モック）
async fn list_annotations_mock(
    Query(params): Query<AnnotationQuery>,
) -> Result<Json<AnnotationListResponse>, StatusCode> {
    println!("📋 Mock: Getting annotations list");

    // モックデータを生成
    let mut mock_annotations = vec![
        create_mock_annotation(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "person",
            100.0, 50.0, 200.0, 300.0
        ),
        create_mock_annotation(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "car",
            300.0, 200.0, 150.0, 100.0
        ),
        create_mock_annotation(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "dog",
            50.0, 250.0, 80.0, 60.0
        ),
    ];

    // 画像IDでフィルタリング
    if let Some(image_id) = params.image_id {
        mock_annotations.retain(|ann| ann.image_id == image_id);
    }

    // ページネーション
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(10);
    let total = mock_annotations.len();
    
    let paginated: Vec<Annotation> = mock_annotations
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    Ok(Json(AnnotationListResponse {
        annotations: paginated,
        total,
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
