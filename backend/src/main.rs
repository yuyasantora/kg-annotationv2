use axum::{
    http::{header::CONTENT_TYPE, Method},
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tower::ServiceExt;
use tower_http::cors::{Any, CorsLayer};

// lambda_httpから必要なものだけをインポートするように修正
use lambda_http::{run, service_fn, Error, Request, RequestExt};
use lambda_http::http::Uri;
use lambda_http::request::RequestContext;


mod models;
mod handlers;
mod utils;

use crate::handlers::{
    annotation::{
        create_annotation, delete_annotation, get_annotation, get_annotations_for_image,
        update_annotation, get_available_labels,
    },
    dataset::create_dataset,
    export::export_dataset,
    image::{
        search_images, get_image, generate_presigned_url, register_uploaded_image,
    },
};
use aws_sdk_s3::Client as S3Client;

#[derive(Clone)]
pub struct AppState {
    db: sqlx::PgPool,
    s3_client: S3Client,
    s3_bucket: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Main function started."); // <-- デバッグログ1
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create pool.");

    let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let s3_client = S3Client::new(&aws_config);
    let s3_bucket = env::var("S3_BUCKET_NAME").expect("S3_BUCKET_NAME must be set");

    let state = AppState {
        db: pool.clone(),
        s3_client,
        s3_bucket,
    };

    let app = Router::new()
        .route("/api/annotations", post(create_annotation).get(get_annotations_for_image))
        .route("/api/annotations/labels", get(get_available_labels))
        .route("/api/annotations/image/:image_id", get(get_annotations_for_image))
        .route("/api/annotations/:id", get(get_annotation).put(update_annotation).delete(delete_annotation))
        .route("/api/datasets", post(create_dataset))
        .route("/api/images/register", post(register_uploaded_image))
        .route("/api/images/presigned-url", post(generate_presigned_url))
        .route("/api/images/:id", get(get_image))
        .route("/api/images/search", post(search_images))
        .route("/api/export", post(export_dataset))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([CONTENT_TYPE])
                .allow_origin(Any),
        )
        .with_state(state);

    println!("Router created. Starting lambda handler."); // <-- デバッグログ2

    run(service_fn(move |mut event: Request| {
        // REST(API GW v1) の stage を取得（/v1 を除去するため）
        let stage: Option<&str> = event
            .request_context_ref()
            .and_then(|rc| match rc {
                RequestContext::ApiGatewayV1(ctx) => ctx.stage.as_deref(), // ← ここを修正
                _ => None,
            });

        if let Some(stage) = stage {
            let orig = event.uri().clone();
            let path = orig.path();
            let prefix = format!("/{stage}");
            if let Some(stripped) = path.strip_prefix(&prefix) {
                let new_path = if let Some(q) = orig.query() {
                    format!("{stripped}?{q}")
                } else {
                    stripped.to_string()
                };
                let mut parts = orig.into_parts();
                parts.path_and_query = Some(new_path.parse().unwrap());
                *event.uri_mut() = Uri::from_parts(parts).unwrap();
            }
        }

        app.clone().oneshot(event)
    }))
    .await
}

fn is_running_on_lambda() -> bool {
    env::var("AWS_LAMETADATA_AWS_REQUEST_ID").is_ok() || env::var("AWS_LAMBDA_RUNTIME_API").is_ok()
}
