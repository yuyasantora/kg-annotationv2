use axum::{
    routing::{get, post},
    Router,
    response::Json,
    http::StatusCode,
};
use tower_http::cors::CorsLayer;
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    println!("🦀 KG Annotation Backend starting...");

    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/api/auth/login", post(login_placeholder))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .unwrap();
        
    println!("🚀 Server running on http://0.0.0.0:3001");
    
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "kg-annotation-backend",
        "version": "0.1.0"
    }))
}

async fn login_placeholder() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "message": "Login endpoint - coming soon",
        "status": "placeholder"
    })))
}
