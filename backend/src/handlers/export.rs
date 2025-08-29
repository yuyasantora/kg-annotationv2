use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use zip::{write::FileOptions, ZipWriter};
use std::io::{Cursor, Write};
use futures::stream::{StreamExt, FuturesUnordered};
use sqlx::PgPool;
use aws_sdk_s3::Client as S3Client;

use crate::{
    models::{Annotation, DatasetFormat, Image},
    AppState,
};

#[derive(Deserialize)]
pub struct FilterOptions {
    #[serde(rename = "type")]
    pub filter_type: String,
    pub labels: Vec<String>,
}

#[derive(Deserialize)]
pub struct ExportRequest {
    pub name: String,
    pub format: DatasetFormat,
    pub filter: FilterOptions,
}

struct ImageData {
    image: Image,
    s3_data: Vec<u8>,
    annotations: Vec<Annotation>,
}

pub async fn export_dataset(
    State(state): State<AppState>,
    Json(payload): Json<ExportRequest>,
) -> Result<Response, StatusCode> {
    let image_ids_result = if payload.filter.labels.is_empty() {
        sqlx::query_scalar!(r#"SELECT DISTINCT image_id FROM annotations"#)
            .fetch_all(&state.db)
            .await
    } else {
        sqlx::query_scalar!(
            r#"SELECT DISTINCT image_id FROM annotations WHERE label = ANY($1)"#,
            &payload.filter.labels
        )
        .fetch_all(&state.db)
        .await
    };

    let image_ids: Vec<Uuid> = image_ids_result
        .map_err(|e| {
            eprintln!("Failed to query image IDs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if image_ids.is_empty() {
        return Ok((StatusCode::NOT_FOUND, "No images found for the given labels").into_response());
    }

    let image_data = get_image_data_from_s3(&state.s3_client, &state.db, image_ids).await?;
    
    let all_labels: Vec<String> = sqlx::query_scalar("SELECT DISTINCT label FROM annotations")
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Failed to fetch labels: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if all_labels.is_empty() {
        return Ok((StatusCode::NOT_FOUND, "No labels found in the database").into_response());
    }

    let zip_data = match payload.format {
        DatasetFormat::Yolo => generate_yolo_zip(&image_data, &all_labels, &payload.name).map_err(|e| {
            eprintln!("Failed to generate YOLO zip: {:?}", e);
            e
        })?,
        _ => {
            eprintln!("Unsupported format: {:?}", payload.format);
            return Err(StatusCode::NOT_IMPLEMENTED);
        }
    };

    let headers = [
        (header::CONTENT_TYPE, "application/zip".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}.zip\"", payload.name),
        ),
    ];

    Ok((headers, zip_data).into_response())
}

async fn get_image_data_from_s3(
    s3_client: &S3Client,
    pool: &PgPool,
    image_ids: Vec<Uuid>,
) -> Result<Vec<ImageData>, StatusCode> {
    let mut image_data_futures = FuturesUnordered::new();

    for image_id in image_ids {
        let s3_client = s3_client.clone();
        let pool = pool.clone();

        image_data_futures.push(tokio::spawn(async move {
            let image = sqlx::query_as!(
                Image,
                r#"
                SELECT 
                    id, user_id, filename, original_filename, s3_bucket, s3_key, file_size, 
                    width, height, format, classification_label, created_at as "created_at!", vector
                FROM images WHERE id = $1
                "#,
                image_id
            )
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                eprintln!("Failed to fetch image data for ID {}: {}", image_id, e);
                e
            })?;

            let s3_object = s3_client
                .get_object()
                .bucket(&image.s3_bucket)
                .key(&image.s3_key)
                .send()
                .await
                .map_err(|e| {
                    eprintln!("Failed to get object from S3 for image {}: {}", image_id, e);
                    sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                })?;

            let s3_data = s3_object
                .body
                .collect()
                .await
                .map_err(|e| {
                    eprintln!("Failed to read S3 body for image {}: {}", image_id, e);
                    sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                })?
                .into_bytes()
                .to_vec();
            
            let annotations = sqlx::query_as!(
                Annotation,
                r#"
                SELECT 
                    id, image_id, user_id, 
                    annotation_type as "annotation_type: _",
                    x, y, width, height, points, bbox, label,
                    source as "source: _",
                    confidence,
                    created_at as "created_at!",
                    updated_at as "updated_at!"
                FROM annotations WHERE image_id = $1
                "#,
                image_id
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| {
                eprintln!("Failed to fetch annotations for image {}: {}", image_id, e);
                e
            })?;

            Ok::<_, sqlx::Error>(ImageData { image, s3_data, annotations })
        }));
    }

    let mut image_data = Vec::new();
    while let Some(result) = image_data_futures.next().await {
        match result {
            Ok(Ok(data)) => image_data.push(data),
            Ok(Err(e)) => {
                eprintln!("Database or S3 error: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            Err(e) => {
                eprintln!("Task execution error: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    if image_data.is_empty() {
        eprintln!("No image data could be retrieved");
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(image_data)
}

fn generate_yolo_zip(
    image_data: &[ImageData],
    all_labels: &[String],
    dataset_name: &str,
) -> Result<Vec<u8>, StatusCode> {
    let mut buf = Vec::new();
    let cursor = Cursor::new(&mut buf);
    let mut zip = ZipWriter::new(cursor);
    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    // YOLOv5/v8形式のディレクトリ構造を作成
    let directories = [
        format!("{}/images/train", dataset_name),
        format!("{}/images/val", dataset_name),
        format!("{}/labels/train", dataset_name),
        format!("{}/labels/val", dataset_name),
    ];

    // ディレクトリを作成
    for dir in &directories {
        if let Err(e) = zip.add_directory(dir, options) {
            eprintln!("Failed to create directory {}: {}", dir, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // data.yamlファイルを作成
    if let Err(e) = zip.start_file(format!("{}/data.yaml", dataset_name), options) {
        eprintln!("Failed to create data.yaml: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // YOLOv5/v8形式のdata.yaml内容を書き込み
    let yaml_content = format!(
        "path: ..\ntrain: images/train\nval: images/val\nnc: {}\nnames:\n{}\n",
        all_labels.len(),
        all_labels.iter()
            .map(|label| format!("  - '{}'", label))
            .collect::<Vec<_>>()
            .join("\n")
    );

    if let Err(e) = zip.write_all(yaml_content.as_bytes()) {
        eprintln!("Failed to write data.yaml content: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // 全データの80%をトレーニング、20%を検証用に使用
    let train_count = (image_data.len() * 8) / 10;

    for (i, data) in image_data.iter().enumerate() {
        let is_train = i < train_count;
        let subset = if is_train { "train" } else { "val" };
        let original_filename = &data.image.original_filename;
        
        // 一意のファイル名を生成（画像IDを使用）
        let unique_base_name = format!("{}_{}", 
            data.image.id.simple().to_string(),  // UUIDの短い形式を使用
            original_filename.rsplit_once('.').map_or(original_filename.as_str(), |(base, _)| base)
        );

        // ラベルファイルを作成（一意の名前を使用）
        let label_path = format!("{}/labels/{}/{}.txt", dataset_name, subset, unique_base_name);
        if let Err(e) = zip.start_file(&label_path, options) {
            eprintln!("Failed to create label file {}: {}", label_path, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        // アノテーションをYOLO形式で書き込み
        let mut label_content = String::new();
        for ann in &data.annotations {
            if let Some(bbox) = &ann.bbox {
                if let Some(label_index) = all_labels.iter().position(|l| l == &ann.label) {
                    // ゼロ除算を避けるためのチェック
                    if data.image.width <= 0 || data.image.height <= 0 {
                        eprintln!(
                            "Image {} has invalid dimensions (width: {}, height: {}), skipping annotation.",
                            data.image.id, data.image.width, data.image.height
                        );
                        continue;
                    }

                    // YOLOフォーマットに変換（中心座標とサイズを0-1の範囲に正規化）
                    let x_center = (bbox[0] + bbox[2] / 2.0) / data.image.width as f32;
                    let y_center = (bbox[1] + bbox[3] / 2.0) / data.image.height as f32;
                    let width = bbox[2] / data.image.width as f32;
                    let height = bbox[3] / data.image.height as f32;

                    // 値が0-1の範囲内にあることを確認
                    if x_center >= 0.0 && x_center <= 1.0 &&
                       y_center >= 0.0 && y_center <= 1.0 &&
                       width >= 0.0 && width <= 1.0 &&
                       height >= 0.0 && height <= 1.0 {
                        label_content.push_str(&format!("{} {:.6} {:.6} {:.6} {:.6}\n", 
                            label_index, x_center, y_center, width, height
                        ));
                    } else {
                        eprintln!(
                            "Invalid normalized coordinates for image {}: ({}, {}, {}, {})",
                            data.image.id, x_center, y_center, width, height
                        );
                    }
                }
            }
        }

        if let Err(e) = zip.write_all(label_content.as_bytes()) {
            eprintln!("Failed to write label content: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        // 画像ファイルを追加（一意の名前を使用）
        let ext = original_filename.rsplit_once('.').map_or("", |(_, ext)| ext);
        let ext_with_dot = if ext.is_empty() { "" } else { "." };
        let image_path = format!("{}/images/{}/{}{}{}", 
            dataset_name,
            subset,
            unique_base_name,
            ext_with_dot,
            ext
        );
        
        if let Err(e) = zip.start_file(&image_path, options) {
            eprintln!("Failed to create image file {}: {}", image_path, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        
        if let Err(e) = zip.write_all(&data.s3_data) {
            eprintln!("Failed to write image data for {}: {}", image_path, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    match zip.finish() {
        Ok(_) => Ok(buf),  // () の代わりに _ を使用してCursorを無視
        Err(e) => {
            eprintln!("Failed to finish zip file: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
