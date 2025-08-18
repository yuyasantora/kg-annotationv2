"use client";

import { useState, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { AnnotationCanvas } from "@/components/annotation/AnnotationCanvas";
import { detectObjects, DetectionResult, getAnnotations, createAnnotation } from "@/lib/api";
import {
  Upload,
  Image as ImageIcon,
  Brain,
  Download,
  Settings,
  Search,
  BarChart3,
  User,
  LogOut,
  Play,
  Save,
  Trash2,
  Eye,
  EyeOff,
  ArrowLeft,
  ArrowRight,
  Loader2,
  CheckCircle,
  AlertCircle,
} from "lucide-react";

interface Annotation {
  id: number;
  type: 'bbox';
  x: number;
  y: number;
  width: number;
  height: number;
  label: string;
  confidence?: number;
  source?: 'manual' | 'ai';
}

export default function KGAnnotationApp() {
  const [currentPage, setCurrentPage] = useState("画像を登録する");
  const [uploadedFiles, setUploadedFiles] = useState<File[]>([]);
  const [selectedImageIndex, setSelectedImageIndex] = useState<number | null>(null);
  const [autoAnnotateMethod, setAutoAnnotateMethod] = useState("プリセットモデルを使用");
  const [confidenceThreshold, setConfidenceThreshold] = useState(0.3);
  const [previewOpen, setPreviewOpen] = useState(true);
  const [annotations, setAnnotations] = useState<Record<number, Annotation[]>>({});
  const [currentStep, setCurrentStep] = useState<"upload" | "annotation" | "labeling">("upload");
  const [isDetecting, setIsDetecting] = useState(false);
  const [detectionResults, setDetectionResults] = useState<Record<number, DetectionResult[]>>({});
  const [selectedImageForPreview, setSelectedImageForPreview] = useState<number | null>(null);
  const [backendTestResult, setBackendTestResult] = useState<string>("");
  const [saveStatus, setSaveStatus] = useState<Record<number, 'saving' | 'saved' | 'error'>>({});
  

  const handleFileUpload = useCallback((files: FileList | null) => {
    if (files) {
      const newFiles = Array.from(files).filter(file => 
        file.type.includes('image/')
      );
      setUploadedFiles(prev => [...prev, ...newFiles]);
    }
  }, []);

  // AI自動検出の実行
  const runAutoDetection = async () => {
    if (uploadedFiles.length === 0) return;

    setIsDetecting(true);
    try {
      const results: Record<number, DetectionResult[]> = {};
      const newAnnotations: Record<number, Annotation[]> = {};

      for (let i = 0; i < uploadedFiles.length; i++) {
        const file = uploadedFiles[i];
        
        try {
          const detectionResult = await detectObjects(file);
          results[i] = detectionResult.detections;

          // 検出結果をアノテーション形式に変換
          const aiAnnotations: Annotation[] = detectionResult.detections
            .filter(detection => detection.confidence >= confidenceThreshold)
            .map((detection, index) => ({
              id: Date.now() + index,
              type: 'bbox' as const,
              x: detection.bbox.x1,
              y: detection.bbox.y1,
              width: detection.bbox.x2 - detection.bbox.x1,
              height: detection.bbox.y2 - detection.bbox.y1,
              label: detection.class_name,
              confidence: detection.confidence,
              source: 'ai' as const,
            }));

          newAnnotations[i] = aiAnnotations;
        } catch (error) {
          console.error(`画像 ${file.name} の検出に失敗:`, error);
        }
      }

      setDetectionResults(results);
      setAnnotations(newAnnotations);
      
    } catch (error) {
      console.error('自動検出中にエラー:', error);
      alert('自動検出中にエラーが発生しました。');
    } finally {
      setIsDetecting(false);
    }
  };

  const startAnnotation = () => {
    if (uploadedFiles.length > 0) {
      setSelectedImageIndex(0);
      setCurrentStep("annotation");
    }
  };

  const nextImage = () => {
    if (selectedImageIndex !== null && selectedImageIndex < uploadedFiles.length - 1) {
      setSelectedImageIndex(selectedImageIndex + 1);
    }
  };

  const prevImage = () => {
    if (selectedImageIndex !== null && selectedImageIndex > 0) {
      setSelectedImageIndex(selectedImageIndex - 1);
    }
  };

  const handleAnnotationsChange = async (newAnnotations: Annotation[]) => {
    if (selectedImageIndex !== null) {
      // ローカルstateを更新
      setAnnotations(prev => ({
        ...prev,
        [selectedImageIndex]: newAnnotations
      }));
      
      // 前回のアノテーション数と比較して新しいものだけ保存
      const previousAnnotations = annotations[selectedImageIndex] || [];
      const newlyAdded = newAnnotations.filter(
        newAnnotation => !previousAnnotations.some(prev => prev.id === newAnnotation.id)
      );
      
      // 新しく追加されたアノテーションをバックエンドに保存
      for (const annotation of newlyAdded) {
        await saveAnnotationToBackend(annotation, selectedImageIndex);
      }
    }
  };

  const finishAnnotation = () => {
    setCurrentStep("labeling");
    setSelectedImageIndex(null);
  };

  const backToUpload = () => {
    setCurrentStep("upload");
    setSelectedImageIndex(null);
  };

  const testBackendConnection = async () => {
    try {
      console.log("🔗 Testing backend connection...");
      const result = await getAnnotations();
      console.log("✅ Backend response:", result);
      setBackendTestResult(`✅ Success: Found ${result.total} annotations`);
    } catch (error) {
      console.error("❌ Backend test failed:", error);
      setBackendTestResult(`❌ Error: ${error}`);
    }
  };

  const testCreateAnnotation = async () => {
    try {
      console.log("➕ Testing annotation creation...");
      const testAnnotation = {
        image_id: "550e8400-e29b-41d4-a716-446655440000",
        annotation_type: "bbox",
        x: 100,
        y: 50,
        width: 200,
        height: 150,
        label: "test_object",
        confidence: 0.95,
        source: "manual"
      };
      
      const result = await createAnnotation(testAnnotation);
      console.log("✅ Create response:", result);
      setBackendTestResult(`✅ Created: ${result.message}`);
    } catch (error) {
      console.error("❌ Create test failed:", error);
      setBackendTestResult(`❌ Create Error: ${error}`);
    }
  };

  const saveAnnotationToBackend = async (annotation: Annotation, imageIndex: number) => {
    try {
      setSaveStatus(prev => ({ ...prev, [imageIndex]: 'saving' }));
      
      // 仮の画像IDを生成（実際は画像アップロード時に生成）
      const imageId = "550e8400-e29b-41d4-a716-446655440000";
      
      const annotationData = {
        image_id: imageId,
        annotation_type: annotation.type, // 'bbox'
        x: annotation.x,
        y: annotation.y,
        width: annotation.width,
        height: annotation.height,
        label: annotation.label,
        confidence: 0.60,
        source: 'manual'
      };

      const result = await createAnnotation(annotationData);
      console.log('✅ Annotation saved to backend:', result);
      
      setSaveStatus(prev => ({ ...prev, [imageIndex]: 'saved' }));
      
      // 3秒後にステータスをクリア
      setTimeout(() => {
        setSaveStatus(prev => {
          const newStatus = { ...prev };
          delete newStatus[imageIndex];
          return newStatus;
        });
      }, 3000);
      
    } catch (error) {
      console.error('❌ Failed to save annotation:', error);
      setSaveStatus(prev => ({ ...prev, [imageIndex]: 'error' }));
    }
  };

  const saveAIDetectionsToBackend = async () => {
    for (const [imageIndex, detections] of Object.entries(detectionResults)) {
      const imageIndexNum = parseInt(imageIndex);
      
      for (const detection of detections) {
        const annotation = {
          image_id: `image-${imageIndexNum}-${Date.now()}`,
          annotation_type: 'bbox',
          x: detection.bbox.x1,
          y: detection.bbox.y1,
          width: detection.bbox.x2 - detection.bbox.x1,
          height: detection.bbox.y2 - detection.bbox.y1,
          label: detection.class_name,
          confidence: detection.confidence,
          source: 'ai'
        };
        
        try {
          await createAnnotation(annotation);
          console.log(`✅ AI detection saved: ${detection.class_name}`);
        } catch (error) {
          console.error(`❌ Failed to save AI detection:`, error);
        }
      }
    }
  };

  return (
    <div className="min-h-screen bg-gray-50 flex">
      {/* サイドバー */}
      <div className="w-80 bg-white border-r border-gray-200 flex flex-col">
        {/* ロゴエリア */}
        <div className="p-6 border-b border-gray-200">
          <div className="text-center">
            <div className="w-32 h-16 bg-blue-600 rounded-lg mx-auto mb-3 flex items-center justify-center">
              <span className="text-white font-bold text-lg">KG</span>
            </div>
            <h1 className="text-xl font-bold text-gray-900">
              KG画像アノテーションシステム
            </h1>
          </div>
        </div>

        {/* ユーザー情報 */}
        <div className="p-4 border-b border-gray-200 bg-blue-50">
          <div className="flex items-center space-x-3">
            <div className="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center">
              <User className="w-4 h-4 text-white" />
            </div>
            <div>
              <p className="text-sm font-medium text-gray-900">ようこそ、<strong>ユーザー名</strong> さん</p>
            </div>
          </div>
        </div>

        {/* ナビゲーション */}
        <div className="flex-1 p-4">
          <h3 className="text-sm font-semibold text-gray-700 mb-3">ナビゲーション</h3>
          <div className="space-y-2">
            {[
              { id: "画像を登録する", label: "画像を登録する", icon: Upload },
              { id: "画像を検索する", label: "画像を検索する", icon: Search },
              { id: "データセット作成", label: "データセット作成", icon: Download },
              { id: "ランキング", label: "ランキング", icon: BarChart3 },
            ].map((item) => {
              const Icon = item.icon;
              return (
                <Button
                  key={item.id}
                  variant={currentPage === item.id ? "default" : "ghost"}
                  className="w-full justify-start"
                  onClick={() => setCurrentPage(item.id)}
                >
                  <Icon className="mr-2 h-4 w-4" />
                  {item.label}
                </Button>
              );
            })}
          </div>
        </div>

        {/* ログアウト */}
        <div className="p-4 border-t border-gray-200">
          <Button variant="outline" className="w-full justify-start">
            <LogOut className="mr-2 h-4 w-4" />
            ログアウト
          </Button>
        </div>
      </div>

      {/* メインコンテンツ */}
      <div className="flex-1 overflow-auto">
        {currentPage === "画像を登録する" && (
          <>
            {/* アップロード画面 */}
            {currentStep === "upload" && (
              <div className="p-6 max-w-4xl mx-auto">
                <h1 className="text-2xl font-bold text-gray-900 mb-6">KG画像登録システム</h1>

                {/* 1. ファイルアップロード */}
                <div className="bg-white rounded-lg border border-gray-200 p-6 mb-6">
                  <h2 className="text-lg font-semibold mb-4">1. 画像をアップロード</h2>
                  <div 
                    className="border-2 border-dashed border-gray-300 rounded-lg p-8 text-center hover:border-blue-400 transition-colors cursor-pointer"
                    onDrop={(e) => {
                      e.preventDefault();
                      handleFileUpload(e.dataTransfer.files);
                    }}
                    onDragOver={(e) => e.preventDefault()}
                    onClick={() => document.getElementById('file-input')?.click()}
                  >
                    <Upload className="h-12 w-12 mx-auto text-gray-400 mb-4" />
                    <p className="text-gray-600 mb-2">画像ファイルを選択</p>
                    <p className="text-sm text-gray-500">PNG, JPG, JPEG ファイルをドラッグ&ドロップまたは</p>
                    <Button variant="outline" className="mt-3">
                      ファイル選択
                    </Button>
                    <input
                      id="file-input"
                      type="file"
                      multiple
                      accept="image/*"
                      className="hidden"
                      onChange={(e) => handleFileUpload(e.target.files)}
                    />
                  </div>

                  {uploadedFiles.length > 0 && (
                    <div className="mt-4">
                      <div className="flex items-center justify-between mb-3">
                        <h3 className="font-semibold">{uploadedFiles.length} 件の画像がステージング中</h3>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => setPreviewOpen(!previewOpen)}
                        >
                          {previewOpen ? <EyeOff className="h-4 w-4 mr-1" /> : <Eye className="h-4 w-4 mr-1" />}
                          プレビュー
                        </Button>
                      </div>
                      
                      {previewOpen && (
                        <>
                          <div className="grid grid-cols-3 gap-4">
                            {uploadedFiles.map((file, index) => (
                              <div key={index} className="relative group">
                                <div className="aspect-video bg-gray-100 rounded-lg overflow-hidden border-2 border-gray-200 hover:border-blue-400 transition-colors">
                                  <img
                                    src={URL.createObjectURL(file)}
                                    alt={file.name}
                                    className="w-full h-full object-contain cursor-pointer"
                                    onClick={() => setSelectedImageForPreview(index)}
                                  />
                                </div>
                                
                                {/* 画像情報 */}
                                <div className="mt-2 space-y-1">
                                  <p className="text-sm font-medium text-gray-900 truncate" title={file.name}>
                                    {file.name}
                                  </p>
                                  <div className="flex items-center justify-between text-xs text-gray-500">
                                    <span>{(file.size / 1024 / 1024).toFixed(1)} MB</span>
                                    <span>{file.type.split('/')[1].toUpperCase()}</span>
                                  </div>
                                </div>

                                {/* AI検出状態 */}
                                {detectionResults[index] && (
                                  <div className="absolute top-2 right-2 bg-green-500 text-white rounded-full p-1">
                                    <CheckCircle className="w-4 h-4" />
                                  </div>
                                )}

                                {/* 削除ボタン */}
                                <button
                                  onClick={() => {
                                    const newFiles = uploadedFiles.filter((_, i) => i !== index);
                                    setUploadedFiles(newFiles);
                                    // 検出結果も削除
                                    const newResults = { ...detectionResults };
                                    delete newResults[index];
                                    setDetectionResults(newResults);
                                  }}
                                  className="absolute top-2 left-2 bg-red-500 text-white rounded-full p-1 opacity-0 group-hover:opacity-100 transition-opacity"
                                >
                                  <Trash2 className="w-4 h-4" />
                                </button>
                              </div>
                            ))}
                          </div>

                          {/* モーダル表示 */}
                          {selectedImageForPreview !== null && (
                            <div 
                              className="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50"
                              onClick={() => setSelectedImageForPreview(null)}
                            >
                              <div className="max-w-4xl max-h-4xl p-4">
                                <img
                                  src={URL.createObjectURL(uploadedFiles[selectedImageForPreview])}
                                  alt={uploadedFiles[selectedImageForPreview].name}
                                  className="max-w-full max-h-full object-contain rounded-lg"
                                />
                                <div className="text-white text-center mt-4">
                                  <p className="text-lg font-medium">{uploadedFiles[selectedImageForPreview].name}</p>
                                  <p className="text-sm opacity-75">
                                    {(uploadedFiles[selectedImageForPreview].size / 1024 / 1024).toFixed(1)} MB • 
                                    {uploadedFiles[selectedImageForPreview].type}
                                  </p>
                                </div>
                              </div>
                            </div>
                          )}
                        </>
                      )}
                    </div>
                  )}
                </div>

                {/* 2. 自動アノテーション */}
                <div className="bg-white rounded-lg border border-gray-200 p-6 mb-6">
                  <h2 className="text-lg font-semibold mb-4">2. AI自動アノテーション</h2>
                  
                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">方法を選択</label>
                      <div className="space-y-2">
                        <label className="flex items-center">
                          <input
                            type="radio"
                            value="プリセットモデルを使用"
                            checked={autoAnnotateMethod === "プリセットモデルを使用"}
                            onChange={(e) => setAutoAnnotateMethod(e.target.value)}
                            className="mr-2"
                          />
                          プリセットモデルを使用（YOLOX）
                        </label>
                        <label className="flex items-center">
                          <input
                            type="radio"
                            value="カスタムモデルをアップロード"
                            checked={autoAnnotateMethod === "カスタムモデルをアップロード"}
                            onChange={(e) => setAutoAnnotateMethod(e.target.value)}
                            className="mr-2"
                          />
                          カスタムモデルをアップロード
                        </label>
                      </div>
                    </div>

                    {autoAnnotateMethod === "プリセットモデルを使用" && (
                      <div className="bg-blue-50 border border-blue-200 rounded-lg p-3">
                        <p className="text-sm text-blue-800">
                          <Brain className="inline w-4 h-4 mr-1" />
                          YOLOXモデル（一般物体検出）を使用します。80クラスの物体を検出できます。
                        </p>
                      </div>
                    )}

                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        信頼度のしきい値: {confidenceThreshold}
                      </label>
                      <input
                        type="range"
                        min="0"
                        max="1"
                        step="0.05"
                        value={confidenceThreshold}
                        onChange={(e) => setConfidenceThreshold(parseFloat(e.target.value))}
                        className="w-full"
                      />
                      <div className="flex justify-between text-xs text-gray-500 mt-1">
                        <span>低精度</span>
                        <span>高精度</span>
                      </div>
                    </div>

                    <Button 
                      className="w-full" 
                      onClick={runAutoDetection}
                      disabled={uploadedFiles.length === 0 || isDetecting}
                    >
                      {isDetecting ? (
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      ) : (
                        <Brain className="mr-2 h-4 w-4" />
                      )}
                      {isDetecting ? 'AI検出実行中...' : 'AI自動検出を実行'}
                    </Button>

                    {/* 検出結果サマリー */}
                    {Object.keys(detectionResults).length > 0 && (
                      <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                        <div className="flex items-center mb-2">
                          <CheckCircle className="w-5 h-5 text-green-600 mr-2" />
                          <h3 className="font-semibold text-green-800">AI検出完了</h3>
                        </div>
                        <div className="grid grid-cols-2 gap-4 text-sm">
                          <div>
                            <span className="text-green-700">処理済み画像:</span>
                            <span className="font-semibold ml-1">
                              {Object.keys(detectionResults).length}/{uploadedFiles.length}
                            </span>
                          </div>
                          <div>
                            <span className="text-green-700">検出オブジェクト数:</span>
                            <span className="font-semibold ml-1">
                              {Object.values(detectionResults).reduce((sum, results) => 
                                sum + results.filter(r => r.confidence >= confidenceThreshold).length, 0
                              )}
                            </span>
                          </div>
                        </div>
                      </div>
                    )}
                  </div>
                </div>

                {/* 3. 手動アノテーション開始 */}
                <div className="bg-white rounded-lg border border-gray-200 p-6">
                  <h2 className="text-lg font-semibold mb-4">3. 手動アノテーション</h2>
                  <p className="text-gray-600 mb-4">
                    AI検出結果の確認・修正、または手動でのアノテーション追加を行います。
                  </p>
                  
                  <Button 
                    className="w-full" 
                    disabled={uploadedFiles.length === 0}
                    onClick={startAnnotation}
                  >
                    <Play className="mr-2 h-4 w-4" />
                    アノテーション画面へ
                  </Button>
                </div>

                {/* 4. バックエンドAPIテスト */}
                <div className="bg-white rounded-lg border border-gray-200 p-6 mt-6">
                  <h2 className="text-lg font-semibold mb-4">4. バックエンドAPIテスト</h2>
                  <p className="text-gray-600 mb-4">
                    Rustバックエンドとの接続をテストします。
                  </p>
                  
                  <div className="space-y-3 mb-4">
                    <Button 
                      onClick={testBackendConnection}
                      variant="outline"
                      className="w-full"
                    >
                      📡 アノテーション一覧取得テスト
                    </Button>
                    
                    <Button 
                      onClick={testCreateAnnotation}
                      variant="outline"
                      className="w-full"
                    >
                      ➕ アノテーション作成テスト
                    </Button>
                  </div>

                  {backendTestResult && (
                    <div className={`p-3 rounded border text-sm ${
                      backendTestResult.includes('✅') 
                        ? 'bg-green-50 border-green-200 text-green-800'
                        : 'bg-red-50 border-red-200 text-red-800'
                    }`}>
                      {backendTestResult}
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* アノテーション画面 */}
            {currentStep === "annotation" && selectedImageIndex !== null && (
              <div className="h-full flex flex-col">
                {/* ヘッダー */}
                <div className="bg-white border-b border-gray-200 p-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-4">
                      <Button variant="outline" onClick={backToUpload}>
                        <ArrowLeft className="h-4 w-4 mr-2" />
                        戻る
                      </Button>
                      <h1 className="text-xl font-semibold">
                        画像アノテーション ({selectedImageIndex + 1}/{uploadedFiles.length})
                      </h1>
                      {detectionResults[selectedImageIndex] && (
                        <div className="bg-green-100 text-green-800 px-2 py-1 rounded-full text-xs">
                          AI検出済み: {detectionResults[selectedImageIndex].filter(r => r.confidence >= confidenceThreshold).length}件
                        </div>
                      )}
                      
                      {/* 保存状況表示 */}
                      {saveStatus[selectedImageIndex] && (
                        <div className={`px-2 py-1 rounded-full text-xs ${
                          saveStatus[selectedImageIndex] === 'saving' 
                            ? 'bg-blue-100 text-blue-800'
                            : saveStatus[selectedImageIndex] === 'saved'
                            ? 'bg-green-100 text-green-800'
                            : 'bg-red-100 text-red-800'
                        }`}>
                          {saveStatus[selectedImageIndex] === 'saving' && '💾 保存中...'}
                          {saveStatus[selectedImageIndex] === 'saved' && '✅ 保存済み'}
                          {saveStatus[selectedImageIndex] === 'error' && '❌ 保存エラー'}
                        </div>
                      )}
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      {/* AI検出結果を一括保存ボタン */}
                      {detectionResults[selectedImageIndex] && (
                        <Button 
                          variant="outline" 
                          onClick={saveAIDetectionsToBackend}
                          size="sm"
                        >
                          💾 AI検出結果を保存
                        </Button>
                      )}
                      
                      <Button 
                        variant="outline" 
                        onClick={prevImage}
                        disabled={selectedImageIndex === 0}
                      >
                        <ArrowLeft className="h-4 w-4" />
                        前へ
                      </Button>
                      <Button 
                        variant="outline" 
                        onClick={nextImage}
                        disabled={selectedImageIndex === uploadedFiles.length - 1}
                      >
                        次へ
                        <ArrowRight className="h-4 w-4 ml-1" />
                      </Button>
                      <Button onClick={finishAnnotation}>
                        <Save className="h-4 w-4 mr-2" />
                        完了
                      </Button>
                    </div>
                  </div>
                </div>

                {/* アノテーションキャンバス */}
                <div className="flex-1 p-6">
                  <AnnotationCanvas
                    imageUrl={URL.createObjectURL(uploadedFiles[selectedImageIndex])}
                    onAnnotationsChange={handleAnnotationsChange}
                    initialAnnotations={annotations[selectedImageIndex] || []}
                  />
                </div>
              </div>
            )}

            {/* ラベル入力画面 */}
            {currentStep === "labeling" && (
              <div className="p-6 max-w-4xl mx-auto">
                <div className="flex items-center mb-6">
                  <Button variant="outline" onClick={() => setCurrentStep("annotation")}>
                    <ArrowLeft className="h-4 w-4 mr-2" />
                    アノテーションに戻る
                  </Button>
                  <h1 className="text-2xl font-bold text-gray-900 ml-4">分類ラベル入力とDBへの保存</h1>
                </div>

                <div className="bg-white rounded-lg border border-gray-200 p-6">
                  <p className="text-gray-600 mb-6">
                    アノテーション作業が完了しました。各画像の分類ラベルを入力し、すべてをデータベースに保存します。
                  </p>

                  <div className="space-y-4 mb-6">
                    {uploadedFiles.map((file, index) => (
                      <div key={index} className="border border-gray-200 rounded-lg p-4">
                        <div className="flex items-start space-x-4">
                          <img
                            src={URL.createObjectURL(file)}
                            alt={file.name}
                            className="w-16 h-16 object-cover rounded border"
                          />
                          <div className="flex-1">
                            <h3 className="font-medium mb-2">{file.name}</h3>
                            <div className="space-y-2">
                              <div>
                                <label className="block text-sm font-medium text-gray-700 mb-1">
                                  分類ラベル:
                                </label>
                                <input
                                  type="text"
                                  className="w-full border border-gray-300 rounded-md px-3 py-2"
                                  placeholder="分類ラベルを入力"
                                />
                              </div>
                              <div className="text-sm text-gray-500 space-y-1">
                                <div>アノテーション数: {annotations[index]?.length || 0}</div>
                                {detectionResults[index] && (
                                  <div className="text-green-600">
                                    AI検出: {detectionResults[index].filter(r => r.confidence >= confidenceThreshold).length}件
                                  </div>
                                )}
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>

                  <Button className="w-full" size="lg">
                    <Save className="mr-2 h-4 w-4" />
                    全てをS3とデータベースに保存
                  </Button>
                </div>
              </div>
            )}
          </>
        )}

        {/* 他のページ */}
        {currentPage !== "画像を登録する" && (
          <div className="p-6 text-center">
            <h2 className="text-xl font-semibold text-gray-700">
              {currentPage}
            </h2>
            <p className="text-gray-500 mt-2">このページは開発中です</p>
          </div>
        )}
      </div>
    </div>
  );
}
