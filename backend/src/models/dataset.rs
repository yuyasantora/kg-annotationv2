use serde::{Deserialize, Serialize};
use uuid::Uuid;

// main.rsからDatasetFormatを移動
#[derive(Debug, Serialize, Deserialize, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "dataset_format")]
#[serde(rename_all = "lowercase")]
pub enum DatasetFormat {
    Yolo,
    Coco,
    Voc
}

// to_stringの実装
impl std::fmt::Display for DatasetFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatasetFormat::Yolo => write!(f, "yolo"),
            DatasetFormat::Coco => write!(f, "coco"),
            DatasetFormat::Voc => write!(f, "voc"),
        }
    }
}

// CreateDatasetRequestのモデル
#[derive(Debug, Deserialize)]
pub struct CreateDatasetRequest {
    pub name: String,
    pub description: Option<String>,
    pub format: DatasetFormat,
    pub image_ids: Vec<Uuid>,
}

// CreateDatasetResponseのモデル
#[derive(Debug, Serialize)]
pub struct CreateDatasetResponse {
    pub id: Uuid,
    pub name: String,
    pub format: DatasetFormat,
    pub download_url: String,
}

