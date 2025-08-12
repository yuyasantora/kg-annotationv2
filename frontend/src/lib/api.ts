// API基底URL設定
const AI_API_BASE_URL = process.env.NEXT_PUBLIC_AI_API_URL || 'http://localhost:8000';
const BACKEND_API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

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
