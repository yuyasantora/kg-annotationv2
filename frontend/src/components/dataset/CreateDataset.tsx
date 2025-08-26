import { useState } from "react";
import {DatasetFormat, createDataset, type CreateDatasetRequest} from "@/lib/api";

interface CreateDatasetProps {
    selectedImages: string[]; // 選択された画像のIDリスト
    onSuccess?: (downloadUrl: string) => void; // データセット作成成功時のコールバック
    onError?: (error: Error) => void;
}

export function CreateDataset({ selectedImages, onSuccess, onError}: CreateDatasetProps) {
    const [name, setName] = useState("");
    const [format, setFormat] = useState<DatasetFormat>(DatasetFormat.Yolo);
    const [description, setDescription] = useState("");
    const [isLoading, setIsLoading] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!name || selectedImages.length === 0) return;

        setIsLoading(true);
        try {
            const request: CreateDatasetRequest = {
                name,
                description: description || undefined,
                format,
                image_ids: selectedImages,
            };
            
            const response = await createDataset(request);
            onSuccess?.(response.download_url);
        } catch (error) {
            console.error("Failed to create dataset:", error);
            onError?.(error as Error);
        } finally {
            setIsLoading(false);
        }


    };
    return (
        <div className="p-4 bg-white rounded-lg shadow">
          <h2 className="text-xl font-bold mb-4">データセット作成</h2>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700">
                データセット名
                <input
                  type="text"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                  required
                />
              </label>
            </div>
    
            <div>
              <label className="block text-sm font-medium text-gray-700">
                説明
                <textarea
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                  rows={3}
                />
              </label>
            </div>
    
            <div>
              <label className="block text-sm font-medium text-gray-700">
                フォーマット
                <select
                  value={format}
                  onChange={(e) => setFormat(e.target.value as DatasetFormat)}
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
                >
                  <option value={DatasetFormat.Yolo}>YOLO</option>
                  <option value={DatasetFormat.Coco}>COCO</option>
                  <option value={DatasetFormat.Voc}>VOC</option>
                </select>
              </label>
            </div>
    
            <div>
              <p className="text-sm text-gray-600">
                選択された画像: {selectedImages.length}枚
              </p>
            </div>
    
            <button
              type="submit"
              disabled={isLoading || selectedImages.length === 0}
              className={`w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 ${
                isLoading || selectedImages.length === 0 ? 'opacity-50 cursor-not-allowed' : ''
              }`}
            >
              {isLoading ? '作成中...' : 'データセットを作成'}
            </button>
          </form>
        </div>
      );
    
}
