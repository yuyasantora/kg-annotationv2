"use client";

import { useState } from "react";
import { Search, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { searchImages } from "@/lib/api";

export function ImageSearch() {
  const [query, setQuery] = useState("");
  const [isSearching, setIsSearching] = useState(false);
  const [results, setResults] = useState<any[]>([]);

  const handleSearch = async () => {
    if (!query.trim()) return;

    setIsSearching(true);
    try {
      const response = await searchImages(query);
      setResults(response.images);
    } catch (error) {
      console.error("検索エラー:", error);
    } finally {
      setIsSearching(false);
    }
  };

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h1 className="text-2xl font-bold text-gray-900 mb-6">画像検索</h1>

      {/* 検索フォーム */}
      <div className="bg-white rounded-lg border border-gray-200 p-6 mb-6">
        <div className="flex gap-2">
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="検索キーワードを入力..."
            className="flex-1 px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            onKeyDown={(e) => e.key === "Enter" && handleSearch()}
          />
          <Button
            onClick={handleSearch}
            disabled={isSearching || !query.trim()}
          >
            {isSearching ? (
              <Loader2 className="h-4 w-4 animate-spin mr-2" />
            ) : (
              <Search className="h-4 w-4 mr-2" />
            )}
            検索
          </Button>
        </div>
      </div>

      {/* 検索結果 */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h2 className="text-lg font-semibold mb-4">検索結果</h2>
        
        {results.length === 0 ? (
          <p className="text-gray-500 text-center py-8">
            検索結果がありません
          </p>
        ) : (
          <div className="grid grid-cols-3 gap-4">
            {results.map((image) => (
              <div
                key={image.id}
                className="relative group"
              >
                <div className="aspect-video bg-gray-100 rounded-lg overflow-hidden border-2 border-gray-200 hover:border-blue-400 transition-colors">
                  <img 
                    src={image.url}  // URLが正しく渡されているか
                    alt={image.filename}
                    onError={(e) => {
                      console.error('画像読み込みエラー:', image.url);  // エラーログを追加
                      // エラー時の処理
                    }}
                    className="w-full h-full object-contain"
                  />
                </div>
                
                {/* 画像情報 */}
                <div className="mt-2 space-y-1">
                  <p className="text-sm font-medium text-gray-900 truncate" title={image.filename}>
                    {image.filename}
                  </p>
                  <div className="flex items-center justify-between text-xs text-gray-500">
                    <span>{(image.file_size / 1024 / 1024).toFixed(1)} MB</span>
                    <span>{image.format.toUpperCase()}</span>
                  </div>
                  {image.similarity_score && (
                    <div className="text-xs text-blue-600">
                      類似度: {(image.similarity_score * 100).toFixed(1)}%
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}