from fastapi import FastAPI, UploadFile, File, HTTPException
from fastapi.responses import JSONResponse
from fastapi.middleware.cors import CORSMiddleware
import uvicorn
from PIL import Image
import io
import numpy as np
import cv2
import torch
from sentence_transformers import SentenceTransformer
import faiss
from typing import List, Dict, Any
from pydantic import BaseModel
import os
from datetime import datetime
import urllib.request

# YOLOXのカスタム実装をインポート
from yolox.onnx_predictor import YOLOXONNXPredictor

app = FastAPI(title="KG Annotation AI Service", version="0.1.0")

# CORS設定
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# グローバル変数でモデルを保持
yolox_predictor = None
sentence_model = None
vector_index = None

@app.on_event("startup")
async def startup_event():
    """アプリケーション起動時にモデルを読み込み"""
    global yolox_predictor, sentence_model, vector_index
    
    print("🚀 Loading AI models...")
    
    # YOLOXモデルの読み込み
    try:
        # YOLOXのONNXモデルをダウンロード
        model_path = "yolox_s.onnx"
        if not os.path.exists(model_path):
            print("📥 Downloading YOLOX ONNX model...")
            model_url = "https://github.com/Megvii-BaseDetection/YOLOX/releases/download/0.1.1rc0/yolox_s.onnx"
            urllib.request.urlretrieve(model_url, model_path)
            print("✅ YOLOX ONNX model downloaded")
        
        # モデルをバイト形式で読み込み
        with open(model_path, 'rb') as f:
            model_bytes = f.read()
        
        # YOLOXプレディクターを初期化
        yolox_predictor = YOLOXONNXPredictor(
            model_bytes=model_bytes,
            input_shape_str="640,640"
        )
        print("✅ YOLOX model loaded")
    except Exception as e:
        print(f"❌ Failed to load YOLOX model: {e}")
    
    # Sentence Transformersモデルの読み込み
    try:
        sentence_model = SentenceTransformer('all-MiniLM-L6-v2')
        print("✅ Sentence Transformer model loaded")
    except Exception as e:
        print(f"❌ Failed to load Sentence Transformer: {e}")
    
    # ベクトル検索インデックス初期化
    vector_index = faiss.IndexFlatIP(384)  # all-MiniLM-L6-v2の次元数
    print("✅ Vector index initialized")

@app.get("/")
async def root():
    return {
        "message": "🐍 KG Annotation AI Service",
        "status": "healthy",
        "version": "0.1.0",
        "models_loaded": {
            "yolox": yolox_predictor is not None,
            "sentence_transformer": sentence_model is not None,
            "vector_index": vector_index is not None
        }
    }

@app.get("/health")
async def health_check():
    return {
        "status": "healthy",
        "timestamp": datetime.now().isoformat(),
        "models_status": {
            "yolox": "loaded" if yolox_predictor else "not_loaded",
            "sentence_transformer": "loaded" if sentence_model else "not_loaded",
            "vector_index": "initialized" if vector_index else "not_initialized"
        }
    }

@app.post("/detect")
async def detect_objects(image: UploadFile = File(...)):
    """YOLOX物体検出API"""
    if yolox_predictor is None:
        raise HTTPException(status_code=503, detail="YOLOX model not loaded")
    
    try:
        # 画像読み込み
        image_bytes = await image.read()
        pil_image = Image.open(io.BytesIO(image_bytes))
        
        # RGB→BGRに変換（OpenCV形式）
        if pil_image.mode != 'RGB':
            pil_image = pil_image.convert('RGB')
        
        # PIL → numpy → OpenCV BGR
        img_array = np.array(pil_image)
        img_bgr = cv2.cvtColor(img_array, cv2.COLOR_RGB2BGR)
        
        # YOLOX推論実行
        detections_raw = yolox_predictor.predict(
            origin_img_bgr=img_bgr,
            score_thr=0.3,
            nms_thr=0.45
        )
        
        # フロントエンド用の形式に変換
        detections = []
        for i, det in enumerate(detections_raw):
            detections.append({
                "id": i,
                "class_name": det["label_name"],
                "confidence": round(det["score"], 3),
                "bbox": {
                    "x1": float(det["xmin"]),
                    "y1": float(det["ymin"]),
                    "x2": float(det["xmax"]),
                    "y2": float(det["ymax"])
                }
            })
        
        return {
            "success": True,
            "image_info": {
                "filename": image.filename,
                "size": pil_image.size,
                "format": pil_image.format
            },
            "detections": detections,
            "total_objects": len(detections)
        }
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Detection failed: {str(e)}")

@app.post("/vectorize")
async def vectorize_text(texts: List[str]):
    """テキストのベクトル化API"""
    if sentence_model is None:
        raise HTTPException(status_code=503, detail="Sentence Transformer model not loaded")
    
    try:
        # テキストをベクトル化
        embeddings = sentence_model.encode(texts, convert_to_tensor=False)
        
        return {
            "success": True,
            "vectors": embeddings.tolist(),
            "dimension": embeddings.shape[1],
            "total_texts": len(texts)
        }
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Vectorization failed: {str(e)}")

# リクエストの型を定義
class SearchRequest(BaseModel):
    query_vector: List[float]
    vectors: List[List[float]]  # 複数のベクトルを受け取る
    top_k: int = 5

@app.post("/search_similar")
async def search_similar_images(request: SearchRequest):
    """ベクトル類似検索API"""
    try:
        # 検索用のインデックスを作成
        dimension = len(request.query_vector)
        index = faiss.IndexFlatIP(dimension)  # コサイン類似度用のインデックス
        
        # ベクトルをインデックスに追加
        vectors = np.array(request.vectors).astype(np.float32)
        index.add(vectors)
        
        # 検索実行
        query_vector = np.array([request.query_vector]).astype(np.float32)
        scores, indices = index.search(query_vector, min(request.top_k, len(request.vectors)))
        
        results = []
        for i, (score, idx) in enumerate(zip(scores[0], indices[0])):
            if idx != -1:  # 有効なインデックス
                results.append({
                    "rank": i + 1,
                    "index": int(idx),
                    "similarity": float(score)
                })
        
        return {
            "success": True,
            "query_dimension": dimension,
            "results": results,
            "total_found": len(results)
        }
        
    except Exception as e:
        print(f"❌ Search error: {e}")
        raise HTTPException(status_code=500, detail=f"Search failed: {str(e)}")

if __name__ == "__main__":
    print("🐍 KG Annotation AI Service starting...")
    uvicorn.run(app, host="0.0.0.0", port=8001)  # ポートを8001に変更