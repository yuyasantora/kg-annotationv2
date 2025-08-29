from fastapi import FastAPI, UploadFile, File, HTTPException, Request
from fastapi.responses import JSONResponse
from fastapi.exceptions import RequestValidationError
from fastapi.middleware.cors import CORSMiddleware
import uvicorn
from PIL import Image
import io
import numpy as np
import cv2
from sentence_transformers import SentenceTransformer
import faiss
from typing import List
from pydantic import BaseModel
import os
from datetime import datetime
import urllib.request
import traceback

# YOLOXのカスタム実装をインポート
from yolox.onnx_predictor import YOLOXONNXPredictor

app = FastAPI(title="KG Annotation AI Service", version="0.2.0")

# バリデーションエラーハンドラを追加
@app.exception_handler(RequestValidationError)
async def validation_exception_handler(request: Request, exc: RequestValidationError):
    print(f"❌ Validation error: {exc.errors()}")
    return JSONResponse(
        status_code=422,
        content={
            "detail": exc.errors(),
            # FormDataオブジェクトをそのまま含めるのではなく、必要な情報だけを抽出
            "error_summary": str(exc)
        },
    )

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
clip_model = None

@app.on_event("startup")
async def startup_event():
    """アプリケーション起動時にモデルを読み込み"""
    global yolox_predictor, clip_model
    
    print("🚀 Loading AI models...")
    
    # YOLOXモデルの読み込み (既存のまま)
    try:
        model_path = "yolox_s.onnx"
        if not os.path.exists(model_path):
            print("📥 Downloading YOLOX ONNX model...")
            model_url = "https://github.com/Megvii-BaseDetection/YOLOX/releases/download/0.1.1rc0/yolox_s.onnx"
            urllib.request.urlretrieve(model_url, model_path)
            print("✅ YOLOX ONNX model downloaded")
        
        with open(model_path, 'rb') as f:
            model_bytes = f.read()
        
        yolox_predictor = YOLOXONNXPredictor(model_bytes=model_bytes, input_shape_str="640,640")
        print("✅ YOLOX model loaded")
    except Exception as e:
        print(f"❌ Failed to load YOLOX model: {e}")
    
    # Sentence TransformersモデルをCLIPモデルに変更
    try:
        # モデルを'clip-ViT-B-32'に変更
        clip_model = SentenceTransformer('clip-ViT-B-32')
        print("✅ CLIP model (clip-ViT-B-32) loaded")
    except Exception as e:
        print(f"❌ Failed to load CLIP model: {e}")
    
@app.get("/")
async def root():
    return {
        "message": "🐍 KG Annotation AI Service",
        "status": "healthy",
        "version": "0.2.0",
        "models_loaded": {
            "yolox": yolox_predictor is not None,
            "clip_model": clip_model is not None,
        }
    }

# --- ここから新しいAPIと修正されたAPI ---

@app.post("/vectorize_image")
async def vectorize_image(
    image: UploadFile = File(..., description="The image file to vectorize")
):
    """画像をベクトル化するAPI"""
    if clip_model is None:
        raise HTTPException(status_code=503, detail="CLIP model not loaded")
    
    try:
        # 入力チェック
        if not image:
            raise HTTPException(status_code=422, detail="No image file provided")
        
        print(f"📥 Received image: {image.filename}, content_type: {image.content_type}")
        
        # content_typeチェック
        if not image.content_type or not image.content_type.startswith('image/'):
            raise HTTPException(
                status_code=422,
                detail=f"Invalid content type: {image.content_type}. Expected image/*"
            )
        
        # 画像データの読み込み
        try:
            image_bytes = await image.read()
            if not image_bytes:
                raise ValueError("Empty image data")
            print(f"📊 Image size: {len(image_bytes)} bytes")
        except Exception as e:
            print(f"❌ Failed to read image data: {e}")
            raise HTTPException(status_code=422, detail=f"Failed to read image data: {str(e)}")
        
        # PILで画像を開く
        try:
            pil_image = Image.open(io.BytesIO(image_bytes))
            print(f"🖼 Image opened: size={pil_image.size}, mode={pil_image.mode}")
        except Exception as e:
            print(f"❌ Failed to open image: {e}")
            raise HTTPException(status_code=422, detail=f"Failed to open image: {str(e)}")
        
        # RGBに変換
        try:
            if pil_image.mode != 'RGB':
                print(f"🔄 Converting image from {pil_image.mode} to RGB")
                pil_image = pil_image.convert('RGB')
        except Exception as e:
            print(f"❌ Failed to convert image to RGB: {e}")
            raise HTTPException(status_code=422, detail=f"Failed to convert image to RGB: {str(e)}")
        
        # 画像をベクトル化
        try:
            print("🧮 Starting image vectorization...")
            embedding = clip_model.encode(pil_image, convert_to_tensor=False)
            print(f"✅ Vectorization successful: shape={embedding.shape}")
        except Exception as e:
            print(f"❌ Vectorization failed: {e}")
            print(f"❌ Traceback: {traceback.format_exc()}")
            raise HTTPException(status_code=500, detail=f"Vectorization failed: {str(e)}")
        
        return {
            "success": True,
            "vector": embedding.tolist(),
            "dimension": embedding.shape[0]
        }
    except HTTPException:
        raise
    except Exception as e:
        error_detail = f"Unexpected error during image vectorization: {str(e)}"
        print(f"❌ Error: {error_detail}")
        print(f"❌ Error type: {type(e)}")
        print(f"❌ Traceback: {traceback.format_exc()}")
        raise HTTPException(status_code=500, detail=error_detail)


@app.post("/vectorize_text")
async def vectorize_text(texts: List[str]):
    """テキストをベクトル化するAPI (旧/vectorize)"""
    if clip_model is None:
        raise HTTPException(status_code=503, detail="CLIP model not loaded")
    
    try:
        # テキストをベクトル化
        embeddings = clip_model.encode(texts, convert_to_tensor=False)
        
        return {
            "success": True,
            "vectors": embeddings.tolist(),
            "dimension": embeddings.shape[1],
            "total_texts": len(texts)
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Text vectorization failed: {str(e)}")

class SearchRequest(BaseModel):
    query_vector: List[float]
    vectors: List[List[float]]
    ids: List[str] # 画像IDのリストを追加
    top_k: int = 10

@app.post("/search_similar_images")
async def search_similar_images(request: SearchRequest):
    """ベクトル類似検索API (旧/search_similar)"""
    if len(request.vectors) == 0 or len(request.ids) == 0:
        return {"success": True, "results": []}
    if len(request.vectors) != len(request.ids):
        raise HTTPException(status_code=400, detail="Length of vectors and ids must be the same")

    try:
        dimension = len(request.query_vector)
        # CLIPのベクトル長(512)と一致しているか確認
        if dimension != 512:
             print(f"⚠️ Warning: Query vector dimension is {dimension}, but CLIP model expects 512.")

        index = faiss.IndexFlatIP(dimension)
        
        vectors_np = np.array(request.vectors).astype('float32')
        faiss.normalize_L2(vectors_np) # コサイン類似度計算のために正規化
        index.add(vectors_np)
        
        query_vector_np = np.array([request.query_vector]).astype('float32')
        faiss.normalize_L2(query_vector_np) # 同様に正規化
        
        scores, indices = index.search(query_vector_np, min(request.top_k, len(request.vectors)))
        
        results = []
        for score, idx in zip(scores[0], indices[0]):
            if idx != -1:
                results.append({
                    "id": request.ids[idx], # 数値インデックスの代わりに画像IDを返す
                    "similarity": float(score)
                })
        
        return {"success": True, "results": results}
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Search failed: {str(e)}")

# --- YOLOXのAPI (既存のまま) ---
@app.post("/detect")
async def detect_objects(image: UploadFile = File(...)):
    if yolox_predictor is None:
        raise HTTPException(status_code=503, detail="YOLOX model not loaded")
    
    try:
        image_bytes = await image.read()
        pil_image = Image.open(io.BytesIO(image_bytes))
        
        if pil_image.mode != 'RGB':
            pil_image = pil_image.convert('RGB')
        
        img_array = np.array(pil_image)
        img_bgr = cv2.cvtColor(img_array, cv2.COLOR_RGB2BGR)
        
        detections_raw = yolox_predictor.predict(origin_img_bgr=img_bgr, score_thr=0.3, nms_thr=0.45)
        
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
        
        return {"success": True, "detections": detections}
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Detection failed: {str(e)}")


if __name__ == "__main__":
    print("🐍 KG Annotation AI Service starting...")
    # ポートを8001に変更
    uvicorn.run(app, host="0.0.0.0", port=8001)