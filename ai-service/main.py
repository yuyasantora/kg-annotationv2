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
import os
from datetime import datetime

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
yolo_model = None
sentence_model = None
vector_index = None

@app.on_event("startup")
async def startup_event():
    """アプリケーション起動時にモデルを読み込み"""
    global yolo_model, sentence_model, vector_index
    
    print("🚀 Loading AI models...")
    
    # YOLOXモデルの読み込み
    try:
        from ultralytics import YOLO
        yolo_model = YOLO('yolov8n.pt')  # 軽量版から開始
        print("✅ YOLO model loaded")
    except Exception as e:
        print(f"❌ Failed to load YOLO model: {e}")
    
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
            "yolo": yolo_model is not None,
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
            "yolo": "loaded" if yolo_model else "not_loaded",
            "sentence_transformer": "loaded" if sentence_model else "not_loaded",
            "vector_index": "initialized" if vector_index else "not_initialized"
        }
    }

@app.post("/detect")
async def detect_objects(image: UploadFile = File(...)):
    """YOLOX物体検出API"""
    if yolo_model is None:
        raise HTTPException(status_code=503, detail="YOLO model not loaded")
    
    try:
        # 画像読み込み
        image_bytes = await image.read()
        pil_image = Image.open(io.BytesIO(image_bytes))
        
        # RGB変換
        if pil_image.mode != 'RGB':
            pil_image = pil_image.convert('RGB')
        
        # YOLO推論
        results = yolo_model(pil_image)
        
        # 検出結果を処理
        detections = []
        for r in results:
            boxes = r.boxes
            if boxes is not None:
                for i, box in enumerate(boxes):
                    x1, y1, x2, y2 = box.xyxy[0].tolist()
                    conf = box.conf[0].item()
                    cls = int(box.cls[0].item())
                    class_name = yolo_model.names[cls]
                    
                    detections.append({
                        "id": i,
                        "class_name": class_name,
                        "confidence": round(conf, 3),
                        "bbox": {
                            "x1": round(x1, 2),
                            "y1": round(y1, 2),
                            "x2": round(x2, 2),
                            "y2": round(y2, 2)
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

@app.post("/search_similar")
async def search_similar_images(query_vector: List[float], top_k: int = 5):
    """ベクトル類似検索API"""
    if vector_index is None:
        raise HTTPException(status_code=503, detail="Vector index not initialized")
    
    try:
        query_array = np.array([query_vector], dtype=np.float32)
        
        # 検索実行
        scores, indices = vector_index.search(query_array, top_k)
        
        results = []
        for i, (score, idx) in enumerate(zip(scores[0], indices[0])):
            if idx != -1:  # 有効なインデックス
                results.append({
                    "rank": i + 1,
                    "index": int(idx),
                    "similarity_score": float(score)
                })
        
        return {
            "success": True,
            "query_dimension": len(query_vector),
            "results": results,
            "total_found": len(results)
        }
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Search failed: {str(e)}")

if __name__ == "__main__":
    print("🐍 KG Annotation AI Service starting...")
    uvicorn.run(app, host="0.0.0.0", port=8000)