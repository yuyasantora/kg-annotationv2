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
  const [isEditing, setIsEditing] = useState(false);
  const [editMode, setEditMode] = useState<'move' | 'resize' | null>(null);
  const [resizeHandle, setResizeHandle] = useState<'nw' | 'ne' | 'sw' | 'se' | 'n' | 's' | 'w' | 'e' | null>(null);
  const [dragStart, setDragStart] = useState<{ x: number; y: number } | null>(null);
  const [originalAnnotation, setOriginalAnnotation] = useState<Annotation | null>(null);
  const [editingLabel, setEditingLabel] = useState<number | null>(null);
  const [tempLabel, setTempLabel] = useState<string>("");

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

    // 描画中のボックス（currentRect）
    if (currentRect && isDrawing) {
      // currentRectは既に画像座標系なので、キャンバス座標系に変換
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

    // リサイズハンドルの描画
    annotations.forEach((annotation, index) => {
      drawResizeHandles(ctx, annotation, index);
    });
  }, [image, annotations, currentRect, isDrawing, selectedAnnotation, scale, imageOffset]);

  // キャンバス描画の更新
  useEffect(() => {
    drawCanvas();
  }, [drawCanvas]);

  // deleteSelected関数をuseCallbackで定義（他のuseCallbackと一緒に、useEffectより前に配置）
  const deleteSelected = useCallback(() => {
    if (selectedAnnotation === null) return;

    console.log('🗑️ Deleting annotation:', selectedAnnotation);
    const newAnnotations = annotations.filter((_, index) => index !== selectedAnnotation);
    setAnnotations(newAnnotations);
    onAnnotationsChange(newAnnotations);
    setSelectedAnnotation(null);
  }, [selectedAnnotation, annotations, onAnnotationsChange]);

  // キーボードイベントハンドラー
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // フォーカスされている要素がinputやtextareaの場合は無視
      const activeElement = document.activeElement;
      if (activeElement?.tagName === 'INPUT' || activeElement?.tagName === 'TEXTAREA') {
        return;
      }

      switch (e.key) {
        case 'Delete':
        case 'Backspace':
          e.preventDefault();
          if (selectedAnnotation !== null) {
            console.log('🗑️ Deleting annotation via keyboard:', selectedAnnotation);
            deleteSelected();
          }
          break;
          
        case 'Escape':
          e.preventDefault();
          console.log('🚫 Escaping selection');
          setSelectedAnnotation(null);
          setIsEditing(false);
          setEditMode(null);
          setResizeHandle(null);
          setDragStart(null);
          setOriginalAnnotation(null);
          break;
          
        case ' ':
          e.preventDefault();
          console.log('🔄 Toggling tools');
          setSelectedTool(prev => prev === 'select' ? 'bbox' : 'select');
          break;
      }
    };

    // イベントリスナーを追加
    document.addEventListener('keydown', handleKeyDown);
    
    // クリーンアップ
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [selectedAnnotation, deleteSelected]);

  // マウスイベントハンドラーの前に追加
  // マウスポジションを取得（キャンバス座標系）
  const getCanvasMousePos = (e: React.MouseEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0 };

    const rect = canvas.getBoundingClientRect();
    return {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top
    };
  };

  // 画像座標系に変換
  const getImageMousePos = (e: React.MouseEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0 };

    const rect = canvas.getBoundingClientRect();
    const x = ((e.clientX - rect.left - imageOffset.x) / scale);
    const y = ((e.clientY - rect.top - imageOffset.y) / scale);
    
    return { x: Math.max(0, Math.min(image?.width || 0, x)), y: Math.max(0, Math.min(image?.height || 0, y)) };
  };

  const handleMouseDown = (e: React.MouseEvent) => {
    const canvasPos = getCanvasMousePos(e); // キャンバス座標系
    const imagePos = getImageMousePos(e);   // 画像座標系
    
    console.log('🖱️ Mouse down:', { 
      canvasPos, 
      imagePos,
      selectedTool, 
      selectedAnnotation, 
      annotationsCount: annotations.length 
    });

    if (selectedTool === "select") {
      // まず全てのアノテーションから選択対象を探す（キャンバス座標系で判定）
      let clickedAnnotationIndex = -1;
      for (let i = annotations.length - 1; i >= 0; i--) {
        const annotation = annotations[i];
        const x = imageOffset.x + annotation.x * scale;
        const y = imageOffset.y + annotation.y * scale;
        const width = annotation.width * scale;
        const height = annotation.height * scale;

        console.log('🔍 Checking annotation:', {
          index: i,
          canvasPos,
          annotationBounds: { x, y, width, height },
          hit: canvasPos.x >= x && canvasPos.x <= x + width && canvasPos.y >= y && canvasPos.y <= y + height
        });

        if (canvasPos.x >= x && canvasPos.x <= x + width && canvasPos.y >= y && canvasPos.y <= y + height) {
          clickedAnnotationIndex = i;
          console.log('🎯 Found annotation:', { index: i, annotation });
          break;
        }
      }
      
      // クリックされたアノテーションがある場合
      if (clickedAnnotationIndex !== -1) {
        const annotation = annotations[clickedAnnotationIndex];
        
        // 既に選択されているアノテーションの場合、リサイズハンドルをチェック
        if (selectedAnnotation === clickedAnnotationIndex) {
          console.log('🔄 Already selected, checking handles...');
          const handle = getResizeHandle(canvasPos.x, canvasPos.y, annotation, clickedAnnotationIndex);
          console.log('🔧 Handle found:', handle);
          
          if (handle) {
            console.log('🎛️ Starting resize mode:', handle);
            setIsEditing(true);
            setEditMode('resize');
            setResizeHandle(handle);
            setDragStart(canvasPos); // キャンバス座標系で保存
            setOriginalAnnotation({ ...annotation });
            return;
          }
          
          // リサイズハンドルでない場合は移動モード
          console.log('📦 Starting move mode');
          setIsEditing(true);
          setEditMode('move');
          setDragStart(canvasPos); // キャンバス座標系で保存
          setOriginalAnnotation({ ...annotation });
          return;
        } else {
          // 新しいアノテーションを選択
          console.log('✨ Selecting new annotation:', clickedAnnotationIndex);
          setSelectedAnnotation(clickedAnnotationIndex);
          return;
        }
      } else {
        // 何もクリックされていない場合、選択を解除
        console.log('❌ Deselecting annotation');
        setSelectedAnnotation(null);
      }
    } else if (selectedTool === "bbox") {
      console.log('📐 Starting bbox drawing');
      setIsDrawing(true);
      setStartPoint(canvasPos); // キャンバス座標系で保存
      setCurrentRect({ x: canvasPos.x, y: canvasPos.y, width: 0, height: 0 });
      setSelectedAnnotation(null);
    }
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    const canvasPos = getCanvasMousePos(e);
    
    // 編集モード
    if (isEditing && selectedAnnotation !== null && dragStart && originalAnnotation) {
      const deltaX = (canvasPos.x - dragStart.x) / scale;
      const deltaY = (canvasPos.y - dragStart.y) / scale;
      
      const newAnnotations = [...annotations];
      const annotation = { ...originalAnnotation };
      
      if (editMode === 'move') {
        // 移動
        annotation.x = originalAnnotation.x + deltaX;
        annotation.y = originalAnnotation.y + deltaY;
      } else if (editMode === 'resize' && resizeHandle) {
        // リサイズ
        let newX = originalAnnotation.x;
        let newY = originalAnnotation.y;
        let newWidth = originalAnnotation.width;
        let newHeight = originalAnnotation.height;
        
        switch (resizeHandle) {
          case 'nw':
            newX = originalAnnotation.x + deltaX;
            newY = originalAnnotation.y + deltaY;
            newWidth = originalAnnotation.width - deltaX;
            newHeight = originalAnnotation.height - deltaY;
            break;
          case 'ne':
            newY = originalAnnotation.y + deltaY;
            newWidth = originalAnnotation.width + deltaX;
            newHeight = originalAnnotation.height - deltaY;
            break;
          case 'sw':
            newX = originalAnnotation.x + deltaX;
            newWidth = originalAnnotation.width - deltaX;
            newHeight = originalAnnotation.height + deltaY;
            break;
          case 'se':
            newWidth = originalAnnotation.width + deltaX;
            newHeight = originalAnnotation.height + deltaY;
            break;
          case 'n':
            newY = originalAnnotation.y + deltaY;
            newHeight = originalAnnotation.height - deltaY;
            break;
          case 's':
            newHeight = originalAnnotation.height + deltaY;
            break;
          case 'w':
            newX = originalAnnotation.x + deltaX;
            newWidth = originalAnnotation.width - deltaX;
            break;
          case 'e':
            newWidth = originalAnnotation.width + deltaX;
            break;
        }
        
        // 最小サイズ制限
        if (newWidth > 10 && newHeight > 10) {
          annotation.x = newX;
          annotation.y = newY;
          annotation.width = newWidth;
          annotation.height = newHeight;
        }
      }
      
      newAnnotations[selectedAnnotation] = annotation;
      setAnnotations(newAnnotations);
      onAnnotationsChange(newAnnotations);
      return;
    }
    
    // 通常の描画モード - 画像座標系で保存
    if (!isDrawing || !startPoint || selectedTool !== "bbox") return;

    // 画像座標系に変換
    const imagePos = getImageMousePos(e);
    const startImagePos = {
      x: (startPoint.x - imageOffset.x) / scale,
      y: (startPoint.y - imageOffset.y) / scale
    };

    const width = Math.abs(imagePos.x - startImagePos.x);
    const height = Math.abs(imagePos.y - startImagePos.y);
    const x = Math.min(imagePos.x, startImagePos.x);
    const y = Math.min(imagePos.y, startImagePos.y);

    // currentRectを画像座標系で保存
    setCurrentRect({ x, y, width, height });
  };

  const handleMouseUp = () => {
    // 編集モード終了
    if (isEditing) {
      setIsEditing(false);
      setEditMode(null);
      setResizeHandle(null);
      setDragStart(null);
      setOriginalAnnotation(null);
      return;
    }
    
    // 通常の描画モード終了
    if (!isDrawing || !currentRect || selectedTool !== "bbox") return;

    setIsDrawing(false);

    // 最小サイズチェック（画像座標系で）
    if (currentRect.width > 10 && currentRect.height > 10) {
      const newAnnotation = {
        id: Date.now(),
        type: 'bbox' as const,
        x: currentRect.x,        // 既に画像座標系
        y: currentRect.y,        // 既に画像座標系
        width: currentRect.width,   // 既に画像座標系
        height: currentRect.height, // 既に画像座標系
        label: `Object ${annotations.length + 1}`,
      };

      const newAnnotations = [...annotations, newAnnotation];
      setAnnotations(newAnnotations);
      onAnnotationsChange(newAnnotations);
    }

    setCurrentRect(null);
    setStartPoint(null);
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

  // ラベル編集開始
  const startLabelEdit = (index: number) => {
    setEditingLabel(index);
    setTempLabel(annotations[index].label);
  };

  // ラベル編集確定
  const saveLabelEdit = (index: number) => {
    if (tempLabel.trim()) {
      updateAnnotationLabel(index, tempLabel.trim());
    }
    setEditingLabel(null);
    setTempLabel("");
  };

  // ラベル編集キャンセル
  const cancelLabelEdit = () => {
    setEditingLabel(null);
    setTempLabel("");
  };

  // リサイズハンドルの判定
  const getResizeHandle = (mouseX: number, mouseY: number, annotation: Annotation, index: number) => {
    if (selectedAnnotation !== index) return null;
    
    const x = imageOffset.x + annotation.x * scale;
    const y = imageOffset.y + annotation.y * scale;
    const width = annotation.width * scale;
    const height = annotation.height * scale;
    const handleSize = 12;
    
    console.log('🔍 Checking resize handles:', {
      mouseX, mouseY,
      boundingBox: { x, y, width, height },
      handleSize
    });
    
    const handles = [
      { type: 'nw', x: x - handleSize/2, y: y - handleSize/2 },
      { type: 'ne', x: x + width - handleSize/2, y: y - handleSize/2 },
      { type: 'sw', x: x - handleSize/2, y: y + height - handleSize/2 },
      { type: 'se', x: x + width - handleSize/2, y: y + height - handleSize/2 },
      { type: 'n', x: x + width/2 - handleSize/2, y: y - handleSize/2 },
      { type: 's', x: x + width/2 - handleSize/2, y: y + height - handleSize/2 },
      { type: 'w', x: x - handleSize/2, y: y + height/2 - handleSize/2 },
      { type: 'e', x: x + width - handleSize/2, y: y + height/2 - handleSize/2 },
    ];
    
    for (const handle of handles) {
      const inHandle = mouseX >= handle.x && mouseX <= handle.x + handleSize &&
                       mouseY >= handle.y && mouseY <= handle.y + handleSize;
      if (inHandle) {
        console.log('✅ Handle hit:', handle.type);
        return handle.type as typeof resizeHandle;
      }
    }
    
    console.log('❌ No handle hit');
    return null;
  };

  // 移動可能エリアの判定
  const isInMoveArea = (mouseX: number, mouseY: number, annotation: Annotation) => {
    const x = imageOffset.x + annotation.x * scale;
    const y = imageOffset.y + annotation.y * scale;
    const width = annotation.width * scale;
    const height = annotation.height * scale;
    
    return mouseX >= x && mouseX <= x + width && 
           mouseY >= y && mouseY <= y + height;
  };

  // リサイズハンドルの描画
  const drawResizeHandles = (ctx: CanvasRenderingContext2D, annotation: Annotation, index: number) => {
    if (selectedAnnotation !== index) return;
    
    const x = imageOffset.x + annotation.x * scale;
    const y = imageOffset.y + annotation.y * scale;
    const width = annotation.width * scale;
    const height = annotation.height * scale;
    
    const handleSize = 8;
    ctx.fillStyle = '#ef4444';
    ctx.strokeStyle = '#ffffff';
    ctx.lineWidth = 2;
    
    const handles = [
      { x: x - handleSize/2, y: y - handleSize/2 },
      { x: x + width - handleSize/2, y: y - handleSize/2 },
      { x: x - handleSize/2, y: y + height - handleSize/2 },
      { x: x + width - handleSize/2, y: y + height - handleSize/2 },
      { x: x + width/2 - handleSize/2, y: y - handleSize/2 },
      { x: x + width/2 - handleSize/2, y: y + height - handleSize/2 },
      { x: x - handleSize/2, y: y + height/2 - handleSize/2 },
      { x: x + width - handleSize/2, y: y + height/2 - handleSize/2 },
    ];
    
    handles.forEach(handle => {
      ctx.fillRect(handle.x, handle.y, handleSize, handleSize);
      ctx.strokeRect(handle.x, handle.y, handleSize, handleSize);
    });
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
                className={`p-3 border rounded-lg cursor-pointer transition-colors ${
                  selectedAnnotation === index ? 'bg-blue-50 border-blue-300' : 'hover:bg-gray-50'
                }`}
                onClick={() => setSelectedAnnotation(index)}
              >
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    {editingLabel === index ? (
                      <input
                        type="text"
                        value={tempLabel}
                        onChange={(e) => setTempLabel(e.target.value)}
                        onBlur={() => saveLabelEdit(index)}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') {
                            saveLabelEdit(index);
                          } else if (e.key === 'Escape') {
                            cancelLabelEdit();
                          }
                        }}
                        className="w-full px-2 py-1 text-sm border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                        autoFocus
                      />
                    ) : (
                      <span 
                        className="font-medium cursor-pointer hover:text-blue-600"
                        onDoubleClick={() => startLabelEdit(index)}
                      >
                        {annotation.label}
                      </span>
                    )}
                  </div>
                  <div className="text-xs text-gray-500 ml-2">
                    {Math.round(annotation.x)}, {Math.round(annotation.y)}
                  </div>
                </div>
                <div className="text-xs text-gray-400 mt-1">
                  {Math.round(annotation.width)} × {Math.round(annotation.height)}
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
