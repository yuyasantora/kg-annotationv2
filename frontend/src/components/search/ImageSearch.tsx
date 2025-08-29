"use client";

import { useState } from "react";
import { Search, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { searchImages } from "@/lib/api";
import Image from 'next/image';

interface SearchResult {
  id: string;
  similarity: number;  // 必須フィールドに変更
  imageUrl: string;
}

// Next.jsの設定でAPIのドメインを取得
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3002';

export function ImageSearch() {
  const [query, setQuery] = useState("");
  const [isSearching, setIsSearching] = useState(false);
  const [results, setResults] = useState<SearchResult[]>([]);

  const handleSearch = async () => {
    if (!query.trim()) return;

    setIsSearching(true);
    try {
      const searchResults = await searchImages(query);  // searchImagesの戻り値の型が変更される
      const results: SearchResult[] = searchResults.map(result => ({
        id: result.id,
        similarity: result.similarity,
        imageUrl: `${API_BASE_URL}/api/images/${result.id}`
      }));
      setResults(results);
    } catch (error) {
      console.error("検索エラー:", error);
      setResults([]);
    } finally {
      setIsSearching(false);
    }
  };

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h1 className="text-2xl font-bold text-gray-900 mb-6">画像検索</h1>

      {/* 検索フォーム (変更なし) */}
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

      {/* 検索結果の表示を改善 */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h2 className="text-lg font-semibold mb-4">検索結果 ({results.length}件)</h2>
        
        {isSearching ? (
          <p className="text-gray-500 text-center py-8">検索中...</p>
        ) : results.length === 0 ? (
          <p className="text-gray-500 text-center py-8">
            検索結果がありません
          </p>
        ) : (
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
            {results.map((result) => (
              <div key={result.id} className="border rounded-lg p-2">
                <div className="aspect-square relative mb-2">
                  {/* img要素を使用してNext.jsのImage制限を回避 */}
                  <img
                    src={result.imageUrl}
                    alt="検索結果"
                    className="w-full h-full object-cover rounded"
                  />
                </div>
                <div className="text-sm">
                  <p className="text-gray-600">
                    類似度: {(result.similarity * 100).toFixed(1)}%
                  </p>
                  <p className="font-mono text-xs text-gray-500 truncate">
                    {result.id}
                  </p>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}