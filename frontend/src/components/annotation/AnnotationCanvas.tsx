"use client";

import { useEffect, useRef, useState } from "react";
import { fabric } from "fabric";
import { Button } from "@/components/ui/button";
import {
  Square,
  Circle,
  MousePointer,
  Trash2,
  Save,
  Undo,
  Redo,
  ZoomIn,
  ZoomOut,
} from "lucide-react";

interface AnnotationCanvasProps {
  imageUrl: string;
  onAnnotationsChange: (annotations: any[]) => void;
}

export function AnnotationCanvas({ imageUrl, onAnnotationsChange }: AnnotationCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [canvas, setCanvas] = useState<fabric.Canvas | null>(null);
  const [selectedTool, setSelectedTool] = useState<"select" | "bbox" | "circle">("select");
  const [annotations, setAnnotations] = useState<any[]>([]);
  const [isDrawing, setIsDrawing] = useState(false);

  useEffect(() => {
    if (!canvasRef.current) return;

    const fabricCanvas = new fabric.Canvas(canvasRef.current, {
      width: 800,
      height: 600,
      backgroundColor: '#f3f4f6',
    });

    // 画像を読み込み
    fabric.Image.fromURL(imageUrl, (img) => {
      // キャンバスサイズに合わせて画像をスケール
      const scale = Math.min(800 / img.width!, 600 / img.height!);
      img.scale(scale);
      img.set({
        left: (800 - img.getScaledWidth()) / 2,
        top: (600 - img.getScaledHeight()) / 2,
        selectable: false,
        evented: false,
      });
      fabricCanvas.add(img);
      fabricCanvas.sendToBack(img);
    });

    setCanvas(fabricCanvas);

    return () => {
      fabricCanvas.dispose();
    };
  }, [imageUrl]);

  // バウンディングボックス描画
  const startDrawing = (pointer: fabric.Point) => {
    if (selectedTool !== "bbox") return;
    
    setIsDrawing(true);
    const rect = new fabric.Rect({
      left: pointer.x,
      top: pointer.y,
      width: 0,
      height: 0,
      fill: 'transparent',
      stroke: '#3b82f6',
      strokeWidth: 2,
      selectable: true,
    });

    canvas?.add(rect);
    canvas?.setActiveObject(rect);
  };

  const finishDrawing = () => {
    if (!isDrawing) return;
    setIsDrawing(false);
    
    // アノテーションデータを更新
    const objects = canvas?.getObjects().filter(obj => obj.type === 'rect') || [];
    const newAnnotations = objects.map((obj, index) => ({
      id: index,
      type: 'bbox',
      x: obj.left,
      y: obj.top,
      width: obj.getScaledWidth(),
      height: obj.getScaledHeight(),
      label: 'object', // デフォルトラベル
    }));
    
    setAnnotations(newAnnotations);
    onAnnotationsChange(newAnnotations);
  };

  // マウスイベント処理
  useEffect(() => {
    if (!canvas) return;

    canvas.on('mouse:down', (e) => {
      if (selectedTool === "bbox" && e.pointer) {
        startDrawing(e.pointer);
      }
    });

    canvas.on('mouse:up', () => {
      finishDrawing();
    });

    canvas.on('mouse:move', (e) => {
      if (!isDrawing || !e.pointer) return;
      
      const activeObj = canvas.getActiveObject();
      if (activeObj && activeObj.type === 'rect') {
        const pointer = canvas.getPointer(e.e);
        const rect = activeObj as fabric.Rect;
        const startX = rect.left!;
        const startY = rect.top!;
        
        rect.set({
          width: Math.abs(pointer.x - startX),
          height: Math.abs(pointer.y - startY),
        });
        
        if (pointer.x < startX) rect.set({ left: pointer.x });
        if (pointer.y < startY) rect.set({ top: pointer.y });
        
        canvas.renderAll();
      }
    });

    return () => {
      canvas.off('mouse:down');
      canvas.off('mouse:up');
      canvas.off('mouse:move');
    };
  }, [canvas, selectedTool, isDrawing]);

  const deleteSelected = () => {
    const activeObj = canvas?.getActiveObject();
    if (activeObj) {
      canvas?.remove(activeObj);
      canvas?.discardActiveObject();
    }
  };

  const clearAll = () => {
    const objects = canvas?.getObjects().filter(obj => obj.type !== 'image') || [];
    objects.forEach(obj => canvas?.remove(obj));
    setAnnotations([]);
    onAnnotationsChange([]);
  };

  return (
    <div className="flex flex-col space-y-4">
      {/* ツールバー */}
      <div className="flex items-center space-x-2 p-4 bg-white rounded-lg border">
        <Button
          variant={selectedTool === "select" ? "default" : "outline"}
          size="sm"
          onClick={() => setSelectedTool("select")}
        >
          <MousePointer className="h-4 w-4" />
        </Button>
        <Button
          variant={selectedTool === "bbox" ? "default" : "outline"}
          size="sm"
          onClick={() => setSelectedTool("bbox")}
        >
          <Square className="h-4 w-4" />
        </Button>
        <Button
          variant={selectedTool === "circle" ? "default" : "outline"}
          size="sm"
          onClick={() => setSelectedTool("circle")}
        >
          <Circle className="h-4 w-4" />
        </Button>
        
        <div className="w-px h-6 bg-gray-300 mx-2" />
        
        <Button variant="outline" size="sm" onClick={deleteSelected}>
          <Trash2 className="h-4 w-4" />
        </Button>
        <Button variant="outline" size="sm" onClick={clearAll}>
          クリア
        </Button>
        
        <div className="w-px h-6 bg-gray-300 mx-2" />
        
        <Button variant="outline" size="sm">
          <Undo className="h-4 w-4" />
        </Button>
        <Button variant="outline" size="sm">
          <Redo className="h-4 w-4" />
        </Button>
        
        <div className="flex-1" />
        
        <Button size="sm">
          <Save className="h-4 w-4 mr-2" />
          保存
        </Button>
      </div>

      {/* キャンバス */}
      <div className="bg-white rounded-lg border p-4">
        <canvas
          ref={canvasRef}
          className="border border-gray-300 rounded-lg"
        />
      </div>

      {/* アノテーション一覧 */}
      <div className="bg-white rounded-lg border p-4">
        <h3 className="font-semibold mb-3">アノテーション一覧 ({annotations.length})</h3>
        <div className="space-y-2 max-h-32 overflow-y-auto">
          {annotations.length === 0 ? (
            <p className="text-gray-500 text-sm">アノテーションがありません</p>
          ) : (
            annotations.map((annotation, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-2 bg-gray-50 rounded border"
              >
                <span className="text-sm">
                  {annotation.type} - {annotation.label}
                </span>
                <input
                  type="text"
                  value={annotation.label}
                  onChange={(e) => {
                    const newAnnotations = [...annotations];
                    newAnnotations[index].label = e.target.value;
                    setAnnotations(newAnnotations);
                    onAnnotationsChange(newAnnotations);
                  }}
                  className="text-xs border rounded px-2 py-1 w-20"
                />
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
