use axum::{
    routing::{ get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;
use sqlx::PgPool;
use std::sync::Arc;
use aws_sdk_s3::Client as S3Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod models;
mod handlers;

// models と handlers から必要なものをすべてインポート
use handlers::{
    annotation::{
        list_annotations,
        create_annotation,
        update_annotation,
        delete_annotation,
        get_image_annotations,
        get_distinct_labels, // この行を追加
    },
    dataset::create_dataset,
    export::export_dataset,
    image::{upload_image, search_images},
};

// AppStateの定義
#[derive(Clone)]
pub struct AppState {
    db: PgPool,
    s3_client: Arc<S3Client>,
    s3_bucket: String,
}

#[derive(Serialize)]
pub struct LabelsResponse {
    pub labels: Vec<String>,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    println!("🦀 KG Annotation Backend starting...");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to PostgreSQL");
    println!("✅ Connected to PostgreSQL");

    let aws_config = aws_config::from_env().load().await;
    let s3_client = Arc::new(S3Client::new(&aws_config));
    let s3_bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "kgbacket".to_string());
    println!("✅ AWS S3 client configured");

    let state = AppState { 
        db: pool, 
        s3_client, 
        s3_bucket,
    };

    let cors = CorsLayer::permissive();

    // ルーターの定義
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/images", post(upload_image))
        .route("/api/images/search", post(search_images))
        .route("/api/images/:image_id/annotations", get(get_image_annotations))
        .route("/api/annotations", get(list_annotations).post(create_annotation))
        .route("/api/annotations/labels", get(get_distinct_labels)) // この行を追加
        .route("/api/annotations/:id", put(update_annotation).delete(delete_annotation))
        .route("/api/datasets", post(create_dataset))
        .route("/api/export", post(export_dataset))
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await.unwrap();
    println!("🚀 Server running on http://0.0.0.0:3002");
    
    axum::serve(listener, app).await.unwrap();
}
