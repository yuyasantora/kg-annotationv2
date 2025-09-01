// API基底URL設定
const AI_API_BASE_URL = process.env.NEXT_PUBLIC_AI_API_URL || 'http://localhost:8001';
const BACKEND_API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3002';

export interface DetectionResult {
  id: number;
  class_name: string;
  confidence: number;
  bbox: {
    x1: number;
    y1: number;
    x2: number;
    y2: number;
  };
}

export interface AIDetectionResponse {
  success: boolean;
  detections: DetectionResult[];
  total_objects: number;
  image_info: {
    filename: string;
    size: [number, number];
    format: string;
  };
}

export async function detectObjects(imageFile: File): Promise<AIDetectionResponse> {
  const formData = new FormData();
  formData.append('image', imageFile);

  const response = await fetch(`${AI_API_BASE_URL}/detect`, {
    method: 'POST',
    body: formData,
  });

  if (!response.ok) {
    throw new Error(`AI検出API呼び出しに失敗: ${response.status}`);
  }

  return response.json();
}

export async function vectorizeText(texts: string[]): Promise<any> {
  const response = await fetch(`${AI_API_BASE_URL}/vectorize`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(texts),
  });

  if (!response.ok) {
    throw new Error(`テキストベクトル化API呼び出しに失敗: ${response.status}`);
  }

  return response.json();
}

export async function searchSimilarImages(queryVector: number[], topK: number = 5): Promise<any> {
  const response = await fetch(`${AI_API_BASE_URL}/search_similar`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      query_vector: queryVector,
      top_k: topK,
    }),
  });

  if (!response.ok) {
    throw new Error(`類似画像検索API呼び出しに失敗: ${response.status}`);
  }

  return response.json();
}

// アノテーション関連の型定義
export interface AnnotationData {
  id: string;
  image_id: string;
  user_id: string;
  annotation_type: string;
  x: number;
  y: number;
  width: number;
  height: number;
  label: string;
  confidence?: number;
  source: string;
  created_at: string;
  updated_at: string;
}

export interface CreateAnnotationRequest {
  image_id: string;
  annotation_type: string;
  x: number;
  y: number;
  width: number;
  height: number;
  label: string;
  confidence?: number;
  source: string;
  bbox?: number[] | null;
  points?: any | null;
}

export interface AnnotationListResponse {
  annotations: AnnotationData[];
  total: number;
}

// アノテーションAPI関数
export async function getAnnotations(imageId?: string): Promise<AnnotationListResponse> {
  const url = imageId 
    ? `${BACKEND_API_BASE_URL}/api/images/${imageId}/annotations`
    : `${BACKEND_API_BASE_URL}/api/annotations`;

  const response = await fetch(url);
  
  if (!response.ok) {
    throw new Error(`アノテーション取得に失敗: ${response.status}`);
  }

  return response.json();
}

export async function createAnnotation(annotation: CreateAnnotationRequest): Promise<any> {
  const response = await fetch(`${BACKEND_API_BASE_URL}/api/annotations`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(annotation),
  });

  if (!response.ok) {
    // レスポンスの本文を読み取ってエラーメッセージに追加する
    const errorText = await response.text();
    throw new Error(`アノテーション作成に失敗: ${response.status} - ${errorText}`);
  }

  return response.json();
}

export async function updateAnnotation(id: string, updates: Partial<CreateAnnotationRequest>): Promise<any> {
  const response = await fetch(`${BACKEND_API_BASE_URL}/api/annotations/${id}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(updates),
  });

  if (!response.ok) {
    throw new Error(`アノテーション更新に失敗: ${response.status}`);
  }

  return response.json();
}

export async function deleteAnnotation(id: string): Promise<any> {
  const response = await fetch(`${BACKEND_API_BASE_URL}/api/annotations/${id}`, {
    method: 'DELETE',
  });

  if (!response.ok) {
    throw new Error(`アノテーション削除に失敗: ${response.status}`);
  }

  return response.json();
}

// 画像アップロード関連の型定義
export interface ImageUploadResponse {
  id: string;
  filename: string;
  original_filename: string;
  file_size: number;
  width: number;
  height: number;
  format: string;
  classification_label?: string;
  created_at: string;
  annotation_count: number;
}

// 画像アップロード
export async function uploadImage(imageFile: File): Promise<ImageUploadResponse> {
  const formData = new FormData();
  formData.append('image', imageFile);

  try {
    console.log('📤 Uploading to:', `${BACKEND_API_BASE_URL}/api/images`);
    
    const response = await fetch(`${BACKEND_API_BASE_URL}/api/images`, {
      method: 'POST',
      // CORSの設定を追加
      mode: 'cors',
      credentials: 'same-origin',
      body: formData,
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`画像アップロードに失敗: ${response.status} - ${errorText}`);
    }

    return response.json();
  } catch (error) {
    console.error('❌ Upload error:', error);
    throw error;
  }
}

// 検索関数
interface SearchResult {
  id: string;
  similarity: number;
}

export async function searchImages(query: string): Promise<SearchResult[]> {
  const response = await fetch(`${BACKEND_API_BASE_URL}/api/images/search`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query }),
  });

  if (!response.ok) {
    throw new Error(`画像検索に失敗: ${response.status}`);
  }

  return response.json();
}

export enum DatasetFormat {
  Yolo = 'yolo',
  Coco = 'coco',
  Voc = 'voc',
}

// データセットエクスポートのリクエスト型
export interface ExportDatasetRequest {
  name: string;
  format: DatasetFormat;
  filter: {
    type: "detection" | "classification";
    labels: string[];
  };
}

// データセットエクスポートAPI関数
export async function exportDataset(request: ExportDatasetRequest): Promise<Blob> {
  const response = await fetch(`${BACKEND_API_BASE_URL}/api/export`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`データセットのエクスポートに失敗: ${response.status} - ${errorText}`);
  }

  return response.blob(); // ZIPファイルをBlobとして受け取る
}

// 利用可能なアノテーションラベル一覧を取得する関数
export async function getAvailableLabels(): Promise<string[]> {
  const response = await fetch(`${BACKEND_API_BASE_URL}/api/annotations/labels`);
  
  if (!response.ok) {
    throw new Error(`ラベル一覧の取得に失敗: ${response.status}`);
  }

  const data = await response.json();
  return data.labels; // { labels: ["car", "person"] } のような形式を想定
}

// 事前署名URL取得のレスポンス型
export interface PresignedUrlResponse {
  url: string;
  s3_key: string;
}

// 事前署名URLを取得する関数
export async function getPresignedUrl(filename: string): Promise<PresignedUrlResponse> {
  const response = await fetch(`${BACKEND_API_BASE_URL}/api/images/presigned-url`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ filename }),
  });

  if (!response.ok) {
    throw new Error(`事前署名URLの取得に失敗: ${response.status}`);
  }

  return response.json();
}

export interface RegisterImageRequest {
  s3_key: string;
  original_filename: string;
  file_size: number;
  width: number;
  height: number;
  format: string;
}

export interface RegisterImageResponse {
  id: string;
}

export async function registerImage(imageData: RegisterImageRequest): Promise<RegisterImageResponse> {
  const response = await fetch(`${BACKEND_API_BASE_URL}/api/images/register`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(imageData),
  });

  if (!response.ok) {
    throw new Error(`画像の登録に失敗: ${response.status}`);
  }

  return response.json();
}
