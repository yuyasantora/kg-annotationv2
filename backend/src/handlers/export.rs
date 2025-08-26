use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use zip::{write::FileOptions, ZipWriter};
use std::io::Cursor;
use futures::stream::{StreamExt, FutureUnordered};

// modelsからDatasetFormatをインポート
use crate::models::DatasetFormat;
// AppStateをインポート
use crate::AppState;


#[derive(Deserialize)]
pub struct FilterOptions {
    #[serde(rename = "type")]
    pub filter_type: String, // "detection" or "classification"
    pub labels: Vec<String>,
}

#[derive(Deserialize)]
pub struct ExportRequest {
    pub name: String,
    pub format: DatasetFormat,
    pub filter: FilterOptions,
}

pub async fn export_dataset(
    State(state): State<AppState>,
    Json(payload): Json<ExportRequest>,
) -> Result<Response, StatusCode> {
    // filter.labels に基づいて、関連する image_id を取得する
    // labelsが空の場合は、アノテーションを持つ全ての画像を対象とする
    let image_ids: Vec<Uuid> = if payload.filter.labels.is_empty() {
        sqlx::query!(r#" SELECT DISTINCT image_id FROM annotations "#)
        .fetch_all(&state.db) // poolからstate.dbに変更
        .await
        .map_err(|e| {
            eprintln!("Failed to query all annotated image IDs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_iter()
        .map(|rec| rec.image_id)
        .collect()
    } else {
        sqlx::query!(
            r#" SELECT DISTINCT image_id FROM annotations WHERE label = ANY($1) "#,
            &payload.filter.labels,
        )
        .fetch_all(&state.db) // poolからstate.dbに変更
        .await
        .map_err(|e| {
            eprintln!("Failed to query image IDs by labels: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_iter()
        .map(|rec| rec.image_id)
        .collect()
    };

    if image_ids.is_empty() {
        return Ok((StatusCode::NOT_FOUND, "No images found for the given labels").into_response());
    }

    // TODO: ZIPファイル生成ロジック
    let response_body = format!(
        "Found {} images for dataset '{}' with format {}. ZIP generation pending.",
        image_ids.len(),
        payload.name,
        payload.format
    );

    Ok((StatusCode::OK, response_body).into_response())
}
