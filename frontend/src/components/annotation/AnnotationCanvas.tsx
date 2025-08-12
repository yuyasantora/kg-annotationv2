"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import { Button } from "@/components/ui/button";
import {
  Square,
  MousePointer,
  Trash2,
  ZoomIn,
  ZoomOut,
} from "lucide-react";

interface Annotation {
  id: number;
  type: 'bbox';
  x: number;
  y: number;
  width: number;
  height: number;
  label: string;
}

interface AnnotationCanvasProps {
  imageUrl: string;
  onAnnotationsChange: (annotations: Annotation[]) => void;
  initialAnnotations?: Annotation[];
}

export function AnnotationCanvas({ 
  imageUrl, 
  onAnnotationsChange, 
  initialAnnotations = [] 
}: AnnotationCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [selectedTool, setSelectedTool] = useState<"select" | "bbox">("select");
  const [annotations, setAnnotations] = useState<Annotation[]>(initialAnnotations);
  const [isDrawing, setIsDrawing] = useState(false);
  const [startPoint, setStartPoint] = useState<{ x: number; y: number } | null>(null);
  const [currentRect, setCurrentRect] = useState<{ x: number; y: number; width: number; height: number } | null>(null);
  const [selectedAnnotation, setSelectedAnnotation] = useState<number | null>(null);
  const [image, setImage] = useState<HTMLImageElement | null>(null);
  const [scale, setScale] = useState(1);
  const [imageOffset, setImageOffset] = useState({ x: 0, y: 0 });

  // 画像の読み込み
  useEffect(() => {
    const img = new Image();
    img.onload = () => {
      setImage(img);
      if (canvasRef.current) {
        const canvas = canvasRef.current;
        const containerWidth = 800;
        const containerHeight = 600;
        
        // 画像をキャンバスに収めるためのスケール計算
        const scaleX = (containerWidth - 40) / img.width;
        const scaleY = (containerHeight - 40) / img.height;
        const newScale = Math.min(scaleX, scaleY, 1);
        
        setScale(newScale);
        
        // 画像を中央に配置
        const scaledWidth = img.width * newScale;
        const scaledHeight = img.height * newScale;
        setImageOffset({
          x: (containerWidth - scaledWidth) / 2,
          y: (containerHeight - scaledHeight) / 2
        });
      }
    };
    img.src = imageUrl;
  }, [imageUrl]);

  // キャンバスの描画
  const drawCanvas = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas || !image) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // キャンバスをクリア
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // 背景を描画
    ctx.fillStyle = '#f3f4f6';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // 画像を描画
    ctx.drawImage(
      image,
      imageOffset.x,
      imageOffset.y,
      image.width * scale,
      image.height * scale
    );

    // アノテーションを描画
    annotations.forEach((annotation, index) => {
      const x = imageOffset.x + annotation.x * scale;
      const y = imageOffset.y + annotation.y * scale;
      const width = annotation.width * scale;
      const height = annotation.height * scale;

      // ボックスの描画
      ctx.strokeStyle = selectedAnnotation === index ? '#ef4444' : '#3b82f6';
      ctx.lineWidth = 2;
      ctx.setLineDash([]);
      ctx.strokeRect(x, y, width, height);

      // 半透明の塗りつぶし
      ctx.fillStyle = selectedAnnotation === index ? 'rgba(239, 68, 68, 0.1)' : 'rgba(59, 130, 246, 0.1)';
      ctx.fillRect(x, y, width, height);

      // ラベルの描画
      if (annotation.label) {
        ctx.fillStyle = selectedAnnotation === index ? '#ef4444' : '#3b82f6';
        ctx.font = '12px sans-serif';
        ctx.fillText(annotation.label, x, y - 5);
      }
    });

    // 現在描画中のボックス
    if (currentRect && isDrawing) {
      const x = imageOffset.x + currentRect.x * scale;
      const y = imageOffset.y + currentRect.y * scale;
      const width = currentRect.width * scale;
      const height = currentRect.height * scale;

      ctx.strokeStyle = '#10b981';
      ctx.lineWidth = 2;
      ctx.setLineDash([5, 5]);
      ctx.strokeRect(x, y, width, height);

      ctx.fillStyle = 'rgba(16, 185, 129, 0.1)';
      ctx.fillRect(x, y, width, height);
    }
  }, [image, annotations, currentRect, isDrawing, selectedAnnotation, scale, imageOffset]);

  // キャンバス描画の更新
  useEffect(() => {
    drawCanvas();
  }, [drawCanvas]);

  // マウスイベントハンドラー
  const getMousePos = (e: React.MouseEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0 };

    const rect = canvas.getBoundingClientRect();
    const x = ((e.clientX - rect.left - imageOffset.x) / scale);
    const y = ((e.clientY - rect.top - imageOffset.y) / scale);
    
    return { x: Math.max(0, Math.min(image?.width || 0, x)), y: Math.max(0, Math.min(image?.height || 0, y)) };
  };

  const handleMouseDown = (e: React.MouseEvent) => {
    const pos = getMousePos(e);

    if (selectedTool === "select") {
      // 既存のアノテーションを選択
      let found = false;
      for (let i = annotations.length - 1; i >= 0; i--) {
        const annotation = annotations[i];
        if (
          pos.x >= annotation.x &&
          pos.x <= annotation.x + annotation.width &&
          pos.y >= annotation.y &&
          pos.y <= annotation.y + annotation.height
        ) {
          setSelectedAnnotation(i);
          found = true;
          break;
        }
      }
      if (!found) {
        setSelectedAnnotation(null);
      }
    } else if (selectedTool === "bbox") {
      // 新しいボックスの描画開始
      setIsDrawing(true);
      setStartPoint(pos);
      setCurrentRect({ x: pos.x, y: pos.y, width: 0, height: 0 });
      setSelectedAnnotation(null);
    }
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isDrawing || !startPoint || selectedTool !== "bbox") return;

    const pos = getMousePos(e);
    const width = Math.abs(pos.x - startPoint.x);
    const height = Math.abs(pos.y - startPoint.y);
    const x = Math.min(pos.x, startPoint.x);
    const y = Math.min(pos.y, startPoint.y);

    setCurrentRect({ x, y, width, height });
  };

  const handleMouseUp = () => {
    if (!isDrawing || !currentRect || selectedTool !== "bbox") return;

    setIsDrawing(false);

    // 最小サイズチェック
    if (currentRect.width > 10 && currentRect.height > 10) {
      const newAnnotation: Annotation = {
        id: Date.now(),
        type: 'bbox',
        x: Math.round(currentRect.x),
        y: Math.round(currentRect.y),
        width: Math.round(currentRect.width),
        height: Math.round(currentRect.height),
        label: 'object'
      };

      const newAnnotations = [...annotations, newAnnotation];
      setAnnotations(newAnnotations);
      onAnnotationsChange(newAnnotations);
      setSelectedAnnotation(newAnnotations.length - 1);
    }

    setCurrentRect(null);
    setStartPoint(null);
  };

  const deleteSelected = () => {
    if (selectedAnnotation === null) return;

    const newAnnotations = annotations.filter((_, index) => index !== selectedAnnotation);
    setAnnotations(newAnnotations);
    onAnnotationsChange(newAnnotations);
    setSelectedAnnotation(null);
  };

  const clearAll = () => {
    setAnnotations([]);
    onAnnotationsChange([]);
    setSelectedAnnotation(null);
  };

  const updateAnnotationLabel = (index: number, newLabel: string) => {
    const newAnnotations = [...annotations];
    newAnnotations[index].label = newLabel;
    setAnnotations(newAnnotations);
    onAnnotationsChange(newAnnotations);
  };

  return (
    <div className="flex flex-col space-y-4">
      {/* ツールバー */}
      <div className="flex items-center space-x-2 p-4 bg-white rounded-lg border">
        <div className="text-sm font-medium text-gray-700 mr-4">ツール:</div>
        
        <Button
          variant={selectedTool === "select" ? "default" : "outline"}
          size="sm"
          onClick={() => setSelectedTool("select")}
        >
          <MousePointer className="h-4 w-4 mr-1" />
          選択
        </Button>
        
        <Button
          variant={selectedTool === "bbox" ? "default" : "outline"}
          size="sm"
          onClick={() => setSelectedTool("bbox")}
        >
          <Square className="h-4 w-4 mr-1" />
          ボックス
        </Button>
        
        <div className="w-px h-6 bg-gray-300 mx-2" />
        
        <Button 
          variant="outline" 
          size="sm" 
          onClick={deleteSelected}
          disabled={selectedAnnotation === null}
        >
          <Trash2 className="h-4 w-4 mr-1" />
          削除
        </Button>
        
        <Button variant="outline" size="sm" onClick={clearAll}>
          クリア
        </Button>

        <div className="flex-1" />
        
        <div className="text-sm text-gray-600">
          {selectedTool === 'bbox' 
            ? 'マウスでドラッグしてボックスを描画' 
            : selectedAnnotation !== null 
              ? `ボックス ${selectedAnnotation + 1} が選択中`
              : 'ボックスをクリックして選択'
          }
        </div>
      </div>

      {/* キャンバス */}
      <div className="bg-white rounded-lg border p-4" ref={containerRef}>
        <canvas
          ref={canvasRef}
          width={800}
          height={600}
          className="border border-gray-300 rounded-lg"
          style={{ 
            cursor: selectedTool === 'bbox' ? 'crosshair' : 'default',
            display: 'block'
          }}
          onMouseDown={handleMouseDown}
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
        />
      </div>

      {/* アノテーション一覧 */}
      <div className="bg-white rounded-lg border p-4">
        <h3 className="font-semibold mb-3">
          アノテーション一覧 ({annotations.length})
        </h3>
        <div className="space-y-2 max-h-40 overflow-y-auto">
          {annotations.length === 0 ? (
            <p className="text-gray-500 text-sm">
              アノテーションがありません。上のツールバーで「ボックス」を選択し、画像上でドラッグして描画してください。
            </p>
          ) : (
            annotations.map((annotation, index) => (
              <div
                key={annotation.id}
                className={`flex items-center justify-between p-3 rounded border cursor-pointer transition-colors ${
                  selectedAnnotation === index 
                    ? 'bg-blue-50 border-blue-200' 
                    : 'bg-gray-50 border-gray-200 hover:bg-gray-100'
                }`}
                onClick={() => setSelectedAnnotation(index)}
              >
                <div className="flex-1">
                  <span className="text-sm font-medium">
                    ボックス {index + 1}
                  </span>
                  <div className="text-xs text-gray-500">
                    位置: ({annotation.x}, {annotation.y}) 
                    サイズ: {annotation.width} × {annotation.height}
                  </div>
                </div>
                <div className="ml-3">
                  <input
                    type="text"
                    value={annotation.label}
                    onChange={(e) => updateAnnotationLabel(index, e.target.value)}
                    placeholder="ラベル"
                    className="text-sm border rounded px-2 py-1 w-24"
                    onClick={(e) => e.stopPropagation()}
                  />
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
