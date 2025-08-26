use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use sqlx::postgres::PgRow;

use uuid::Uuid;

use crate::{
    models::{CreateDatasetRequest, CreateDatasetResponse, DatasetFormat},
    AppState,
};

// create_dataset ハンドラ
pub async fn create_dataset(
    State(state): State<AppState>,
    Json(payload): Json<CreateDatasetRequest>,
) -> Result<Json<CreateDatasetResponse>, StatusCode> {
    println!("📦 Creating dataset: {}", payload.name);
    let dataset_id = Uuid::new_v4();
    let s3_key = format!("datasets/{}/{}.zip", dataset_id, payload.name);
    
    Ok(Json(CreateDatasetResponse {
        id: dataset_id,
        name: payload.name,
        format: payload.format,
        download_url: format!("https://s3.{}.amazonaws.com/{}/{}", "ap-northeast-1", state.s3_bucket, s3_key),
    }))
}

// エクスポート用のヘルパー関数 (今は空)
fn export_dataset_yolo(_dataset_content: &[(PgRow, Vec<PgRow>)]) -> (String, Vec<u8>) {
    let s3_key = format!("datasets/{}/yolo.zip", Uuid::new_v4());
    (s3_key, Vec::new())
}

fn export_dataset_coco(_dataset_content: &[(PgRow, Vec<PgRow>)]) -> (String, Vec<u8>) {
    let s3_key = format!("datasets/{}/coco.zip", Uuid::new_v4());
    (s3_key, Vec::new())
}

fn export_dataset_voc(_dataset_content: &[(PgRow, Vec<PgRow>)]) -> (String, Vec<u8>) {
    let s3_key = format!("datasets/{}/voc.zip", Uuid::new_v4());
    (s3_key, Vec::new())
}