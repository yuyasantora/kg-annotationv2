use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, Method},
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tower_http::cors::{Any, CorsLayer};

mod models;
mod handlers;
mod utils; // utilsモジュールを宣言


use crate::handlers::{
    annotation::{
        create_annotation, delete_annotation, get_annotation, get_annotations_for_image,
        update_annotation, get_available_labels,
    },
    dataset::create_dataset,
    export::export_dataset,
    image::{upload_image, search_images, get_image, generate_presigned_url, register_uploaded_image},
};
use aws_sdk_s3::Client as S3Client;

#[derive(Clone)]
pub struct AppState {
    db: sqlx::PgPool,
    s3_client: S3Client,
    s3_bucket: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    println!("🦀 KG Annotation Backend starting...");

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create pool.");
    println!("✅ Connected to PostgreSQL");

    let aws_config = aws_config::load_from_env().await;
    let s3_client = S3Client::new(&aws_config);
    let s3_bucket = env::var("S3_BUCKET_NAME").expect("S3_BUCKET_NAME must be set");
    println!("✅ AWS S3 client configured");

    let state = AppState {
        db: pool,
        s3_client,
        s3_bucket,
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE])
        .allow_origin(Any);

    let app = Router::new()
        .route("/api/annotations", post(create_annotation).get(get_annotations_for_image))
        .route("/api/annotations/labels", get(get_available_labels))
        .route("/api/annotations/image/:image_id", get(get_annotations_for_image))
        .route("/api/annotations/:id", get(get_annotation).put(update_annotation).delete(delete_annotation))
        .route("/api/datasets", post(create_dataset))
        .route("/api/images/presigned-url", post(generate_presigned_url))
        .route("/api/images/register", post(register_uploaded_image))
        .route("/api/images", post(upload_image))
        .route("/api/images/:id", get(get_image))
        .route("/api/images/search", post(search_images))
        .route("/api/export", post(export_dataset))
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await.unwrap();
    println!("🚀 Server running on http://0.0.0.0:3002");
    axum::serve(listener, app).await.unwrap();
}
