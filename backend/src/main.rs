use axum::{
    routing::{get, post},
    Router,
    response::Json,
    http::StatusCode,
    extract::{State, Multipart},
};
use tower_http::cors::CorsLayer;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, DateTime};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use aws_sdk_s3::Client as S3Client;

mod models;
use models::ImageResponse;

// 検索用のリクエスト型
#[derive(Debug, Deserialize)]
pub struct ImageSearchRequest {
    pub query: String,
    pub top_k: Option<i32>,
}

// アノテーション作成のリクエスト型
#[derive(Debug, Deserialize)]
pub struct CreateAnnotationRequest {
    pub image_id: Uuid,
    pub annotation_type: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub label: String,
    pub confidence: f64,
    pub source: String,
}

// アノテーション作成のレスポンス型
#[derive(Debug, Serialize)]
pub struct CreateAnnotationResponse {
    pub id: Uuid,
    pub message: String,
}

// AppStateにS3の設定を追加
#[derive(Clone)]
struct AppState {
    db: PgPool,
    s3_client: Arc<S3Client>,
    s3_bucket: String,
}

// アップロード処理
async fn upload_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ImageResponse>, StatusCode> {
    println!("📤 Processing image upload...");

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        if field.name() == Some("image") {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let content_type = field.content_type().unwrap_or("").to_string();
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            
            println!("📁 Uploading file: {} ({} bytes)", filename, data.len());
            
            // 画像の検証とメタデータ取得
            let image = image::load_from_memory(&data)
                .map_err(|_| StatusCode::BAD_REQUEST)?;
            let (width, height) = (image.width() as i32, image.height() as i32);
            let file_size = data.len() as i64;
            
            // S3キーの生成
            let uuid = Uuid::new_v4();
            let s3_key = format!("images/{}_{}", uuid, &filename);
            
            // トランザクション開始
            let mut transaction = state.db.begin().await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // ユーザーID取得
            let user_result = sqlx::query("SELECT id FROM users LIMIT 1")
                .fetch_one(&mut *transaction)
        .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let user_id: Uuid = user_result.get("id");

            // S3アップロード
            let upload_result = state.s3_client
                .put_object()
                .bucket(&state.s3_bucket)
                .key(&s3_key)
                .body(data.into())
                .content_type(&content_type)
                .send()
                .await;

            match upload_result {
                Ok(_) => {
                    println!("✅ S3 upload successful: {}", s3_key);
                }
                Err(e) => {
                    println!("❌ S3 upload failed: {}", e);
                    transaction.rollback().await
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }

            // データベースに保存
            let result = sqlx::query(
                "INSERT INTO images (user_id, filename, original_filename, s3_key, s3_bucket, file_size, width, height, format)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 RETURNING id, created_at"
            )
            .bind(user_id)
            .bind(&filename)
            .bind(&filename)
            .bind(&s3_key)
            .bind(&state.s3_bucket)
            .bind(file_size)
            .bind(width)
            .bind(height)
            .bind(content_type.split('/').last().unwrap_or("unknown"))
            .fetch_one(&mut *transaction)
            .await
            .map_err(|e| {
                println!("❌ Database error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // トランザクションをコミット
            transaction.commit().await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let image_id: Uuid = result.get("id");
            let created_at: DateTime<Utc> = result.get("created_at");
            
            println!("✅ Image uploaded successfully: {}", image_id);
            
            return Ok(Json(ImageResponse {
                id: image_id,
                filename: filename.clone(),
                original_filename: filename,
                file_size,
                width,
                height,
                format: content_type.split('/').last().unwrap_or("unknown").to_string(),
                classification_label: None,
                created_at,
                annotation_count: 0,
                url: format!("https://s3.ap-northeast-1.amazonaws.com/{}/{}", state.s3_bucket, s3_key),
            }));
        }
    }
    
    Err(StatusCode::BAD_REQUEST)
}

// 検索ハンドラー
async fn search_images(
    State(state): State<AppState>,
    Json(payload): Json<ImageSearchRequest>,
) -> Result<Json<Value>, StatusCode> {
    println!("🔍 Searching images with query: {}", payload.query);
    
    // 1. まず全ての画像のラベルを取得
    let image_labels = sqlx::query!(
        "SELECT DISTINCT label FROM annotations"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        println!("❌ Failed to fetch labels: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let labels: Vec<String> = image_labels.iter()
        .map(|row| row.label.clone())
        .collect();

    println!("📝 Found {} unique labels", labels.len());

    // 2. ラベルをベクトル化
    let client = reqwest::Client::new();
    let vectorize_response = client
        .post("http://localhost:8001/vectorize")
        .json(&labels)
        .send()
        .await
        .map_err(|e| {
            println!("❌ Failed to vectorize labels: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let vector_data: Value = vectorize_response.json().await.map_err(|e| {
        println!("❌ Failed to parse vectorize response: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 3. クエリをベクトル化
    let query_response = client
        .post("http://localhost:8001/vectorize")
        .json(&vec![payload.query.clone()])
        .send()
        .await
        .map_err(|e| {
            println!("❌ Failed to vectorize query: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let query_vector: Value = query_response.json().await.map_err(|e| {
        println!("❌ Failed to parse query vector: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 4. 類似度計算
    let search_request = json!({
        "query_vector": query_vector["vectors"][0],
        "vectors": vector_data["vectors"],
        "top_k": payload.top_k.unwrap_or(5)
    });

    let search_response = client
        .post("http://localhost:8001/search_similar")
        .json(&search_request)
        .send()
        .await
        .map_err(|e| {
            println!("❌ Failed to search similar: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let search_results: Value = search_response.json().await.map_err(|e| {
        println!("❌ Failed to parse search response: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 5. 検索結果の画像情報を取得
    let mut images = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    if let Some(results) = search_results["results"].as_array() {
        for result in results {
            let index = result["index"].as_i64().unwrap_or_default() as i32;
            let similarity = result["similarity"].as_f64().unwrap_or_default();

            let label = &labels[index as usize];
            let image_rows = sqlx::query!(
                "SELECT DISTINCT i.* 
                 FROM images i 
                 JOIN annotations a ON i.id = a.image_id 
                 WHERE a.label = $1 
                 LIMIT 1",
                label
            )
            .fetch_all(&state.db)
            .await
            .map_err(|e| {
                println!("❌ Database error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            for row in image_rows {
                if seen_ids.insert(row.id) {
                    let s3_url = format!("https://s3.ap-northeast-1.amazonaws.com/{}/{}", 
                        state.s3_bucket, row.s3_key);
                    
                    images.push(json!({
                        "id": row.id,
                        "filename": row.filename,
                        "original_filename": row.original_filename,
                        "s3_key": row.s3_key,
                        "s3_bucket": row.s3_bucket,
                        "file_size": row.file_size,
                        "width": row.width,
                        "height": row.height,
                        "format": row.format,
                        "similarity_score": similarity,
                        "url": s3_url
                    }));
                }
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "query": payload.query,
        "images": images
    })))
}

// アノテーション作成ハンドラー
async fn create_annotation(
    State(state): State<AppState>,
    Json(payload): Json<CreateAnnotationRequest>,
) -> Result<Json<CreateAnnotationResponse>, StatusCode> {
    println!("➕ Creating annotation for image_id: {}", payload.image_id);
    println!("   Label: {}, Position: ({}, {}), Size: {}x{}", 
        payload.label, payload.x, payload.y, payload.width, payload.height);

    // ユーザーID取得
    let user_result = sqlx::query("SELECT id FROM users LIMIT 1")
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            println!("❌ Failed to get user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let user_id: Uuid = user_result.get("id");

    // アノテーションを保存
    let result = sqlx::query(
        "INSERT INTO annotations (image_id, user_id, annotation_type, x, y, width, height, label, confidence, source)
         VALUES ($1, $2, $3::annotation_type, $4, $5, $6, $7, $8, $9, $10::annotation_source)
         RETURNING id"
    )
    .bind(payload.image_id)
    .bind(user_id)
    .bind(&payload.annotation_type)
    .bind(payload.x)
    .bind(payload.y)
    .bind(payload.width)
    .bind(payload.height)
    .bind(&payload.label)
    .bind(payload.confidence)
    .bind(&payload.source)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        println!("❌ Failed to save annotation: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let annotation_id = result.get::<Uuid, _>("id");
    println!("✅ Annotation created: {}", annotation_id);

    Ok(Json(CreateAnnotationResponse {
        id: annotation_id,
        message: format!("Annotation '{}' created successfully", payload.label),
    }))
}

// データセットのフォーマット型
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DatasetFormat {
    Yolo,
    Coco,
    Voc,
}

// SQLxのType実装を追加
impl sqlx::Type<sqlx::Postgres> for DatasetFormat {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("dataset_format")
    }
}

// Encodeトレイトの実装を追加
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for DatasetFormat {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        let s = match self {
            DatasetFormat::Yolo => "yolo",
            DatasetFormat::Coco => "coco",
            DatasetFormat::Voc => "voc",
        };
        <&str as sqlx::encode::Encode<sqlx::Postgres>>::encode(s, buf)


    }

}
// Decodeトレイトの実装も追加
impl<'r> sqlx::decode::Decode<'r, sqlx::Postgres> for DatasetFormat {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <String as sqlx::decode::Decode<sqlx::Postgres>>::decode(value)?;
        match s.as_str() {
            "yolo" => Ok(DatasetFormat::Yolo),
            "coco" => Ok(DatasetFormat::Coco),
            "voc" => Ok(DatasetFormat::Voc),
            _ => Err("Invalid dataset format".into()),
        }
    }
}
// データセット作成のリクエスト型
#[derive(Debug, Deserialize)]
pub struct CreateDatasetRequest {
    pub name: String,
    pub description: Option<String>,
    pub format: DatasetFormat,
    pub image_ids: Vec<Uuid>,
}

// データセット作成のレスポンス型
#[derive(Debug, Serialize)]
pub struct CreateDatasetResponse {
    pub id: Uuid,
    pub name: String,
    pub format: DatasetFormat,
    pub download_url: String,
}

// データセット作成のハンドラ
async fn create_dataset(
    State(state): State<AppState>,
    Json(payload): Json<CreateDatasetRequest>,
) -> Result<Json<CreateDatasetResponse>, StatusCode> {
    println!("📦 Creating dataset: {}", payload.name);
    println!("   Format: {:?}", payload.format);
    println!("   Images: {} selected", payload.image_ids.len());

    // トランザクション開始
    let mut transaction = state.db.begin().await
        .map_err(|e| {
            println!("❌ Failed to start transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // データセットの基本情報を保存
    let dataset_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO datasets (id, name, description, format)
         VALUES ($1, $2, $3, $4::dataset_format)",
        dataset_id,
        payload.name,
        payload.description,
        &payload.format as &DatasetFormat  // 型キャストを明示的に指定
    )
    .execute(&mut *transaction)
    .await
    .map_err(|e| {
        println!("❌ Failed to create dataset: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // 画像とアノテーションを取得
    let mut dataset_content = Vec::new();
    for image_id in &payload.image_ids {
        // 画像情報を取得
        let image = sqlx::query(
            "SELECT * FROM images WHERE id = $1"
        )
        .bind(image_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|e| {
            println!("❌ Failed to fetch image {}: {}", image_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // アノテーションを取得
        let annotations = sqlx::query(
            "SELECT * FROM annotations WHERE image_id = $1"
        )
        .bind(image_id)
        .fetch_all(&mut *transaction)
    .await
    .map_err(|e| {
            println!("❌ Failed to fetch annotations for image {}: {}", image_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        dataset_content.push((image, annotations));
    }

    // フォーマットに応じてデータセットを生成
    let (s3_key, content) = match payload.format {
        DatasetFormat::Yolo => export_dataset_yolo(&dataset_content),
        DatasetFormat::Coco => export_dataset_coco(&dataset_content),
        DatasetFormat::Voc => export_dataset_voc(&dataset_content),
    };

    // S3にアップロード
    let upload_result = state.s3_client
        .put_object()
        .bucket(&state.s3_bucket)
        .key(&s3_key)
        .body(content.into())
        .content_type("application/zip")
        .send()
        .await;

    match upload_result {
        Ok(_) => {
            println!("✅ Dataset uploaded to S3: {}", s3_key);
            transaction.commit().await.map_err(|e| {
                println!("❌ Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

            Ok(Json(CreateDatasetResponse {
                id: dataset_id,
                name: payload.name,
                format: payload.format,
                download_url: format!("https://s3.ap-northeast-1.amazonaws.com/{}/{}", 
                    state.s3_bucket, s3_key),
            }))
        }
        Err(e) => {
            println!("❌ Failed to upload dataset: {}", e);
            transaction.rollback().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// エクスポート関数の修正
fn export_dataset_yolo(_dataset_content: &[(sqlx::postgres::PgRow, Vec<sqlx::postgres::PgRow>)]) -> (String, Vec<u8>) {
    let s3_key = format!("datasets/{}/yolo.zip", Uuid::new_v4());
    (s3_key, Vec::new())
}

fn export_dataset_coco(_dataset_content: &[(sqlx::postgres::PgRow, Vec<sqlx::postgres::PgRow>)]) -> (String, Vec<u8>) {
    let s3_key = format!("datasets/{}/coco.zip", Uuid::new_v4());
    (s3_key, Vec::new())
}

fn export_dataset_voc(_dataset_content: &[(sqlx::postgres::PgRow, Vec<sqlx::postgres::PgRow>)]) -> (String, Vec<u8>) {
    let s3_key = format!("datasets/{}/voc.zip", Uuid::new_v4());
    (s3_key, Vec::new())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    println!("🦀 KG Annotation Backend starting...");

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

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

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/images", post(upload_image))
        .route("/api/images/search", post(search_images))
        .route("/api/annotations", post(create_annotation))  // アノテーション作成エンドポイントを追加
        .route("/api/datasets", post(create_dataset))  // データセット作成エンドポイントを追加
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")
        .await
        .unwrap();
        
    println!("🚀 Server running on http://0.0.0.0:3002");
    
    axum::serve(listener, app).await.unwrap();
}
