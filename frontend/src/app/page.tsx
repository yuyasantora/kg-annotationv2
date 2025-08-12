"use client";

import { useState, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { AnnotationCanvas } from "@/components/annotation/AnnotationCanvas";
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
} from "lucide-react";

export default function KGAnnotationApp() {
  const [currentPage, setCurrentPage] = useState("画像を登録する");
  const [uploadedFiles, setUploadedFiles] = useState<File[]>([]);
  const [selectedImageIndex, setSelectedImageIndex] = useState<number | null>(null);
  const [autoAnnotateMethod, setAutoAnnotateMethod] = useState("プリセットモデルを使用");
  const [confidenceThreshold, setConfidenceThreshold] = useState(0.3);
  const [previewOpen, setPreviewOpen] = useState(false);
  const [annotations, setAnnotations] = useState<Record<number, any[]>>({});
  const [currentStep, setCurrentStep] = useState<"upload" | "annotation" | "labeling">("upload");

  const handleFileUpload = useCallback((files: FileList | null) => {
    if (files) {
      const newFiles = Array.from(files).filter(file => 
        file.type.includes('image/')
      );
      setUploadedFiles(prev => [...prev, ...newFiles]);
    }
  }, []);

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

  const handleAnnotationsChange = (newAnnotations: any[]) => {
    if (selectedImageIndex !== null) {
      setAnnotations(prev => ({
        ...prev,
        [selectedImageIndex]: newAnnotations
      }));
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
                        <div className="grid grid-cols-4 gap-3">
                          {uploadedFiles.map((file, index) => (
                            <div key={index} className="relative">
                              <img
                                src={URL.createObjectURL(file)}
                                alt={file.name}
                                className="w-full h-20 object-cover rounded border"
                              />
                              <p className="text-xs text-gray-600 mt-1 truncate">{file.name}</p>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  )}
                </div>

                {/* 2. 自動アノテーション */}
                <div className="bg-white rounded-lg border border-gray-200 p-6 mb-6">
                  <h2 className="text-lg font-semibold mb-4">2. 自動アノテーション (任意)</h2>
                  
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
                          プリセットモデルを使用
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
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">モデルを選択</label>
                        <select className="w-full border border-gray-300 rounded-md px-3 py-2">
                          <option>信号検出</option>
                        </select>
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
                    </div>

                    <Button className="w-full">
                      <Brain className="mr-2 h-4 w-4" />
                      自動アノテーションを実行
                    </Button>
                  </div>
                </div>

                {/* 3. 手動アノテーション開始 */}
                <div className="bg-white rounded-lg border border-gray-200 p-6">
                  <h2 className="text-lg font-semibold mb-4">3. 手動アノテーション</h2>
                  <p className="text-gray-600 mb-4">
                    Webブラウザ上で直接アノテーションを行います。バウンディングボックスの描画、ラベル付けが可能です。
                  </p>
                  
                  <Button 
                    className="w-full" 
                    disabled={uploadedFiles.length === 0}
                    onClick={startAnnotation}
                  >
                    <Play className="mr-2 h-4 w-4" />
                    アノテーション開始
                  </Button>
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
                    </div>
                    <div className="flex items-center space-x-2">
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
                              <div className="text-sm text-gray-500">
                                アノテーション数: {annotations[index]?.length || 0}
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
