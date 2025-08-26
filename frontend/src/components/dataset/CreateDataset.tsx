import { useState, useEffect } from "react";
import { DatasetFormat, exportDataset, getAvailableLabels } from "@/lib/api";

export function CreateDataset() {
  const [name, setName] = useState("");
  const [format, setFormat] = useState<DatasetFormat>(DatasetFormat.Yolo);
  const [filterType, setFilterType] = useState<"detection" | "classification">("detection");
  const [selectedLabels, setSelectedLabels] = useState<string[]>([]);
  const [availableLabels, setAvailableLabels] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  // 利用可能なラベルを取得
  useEffect(() => {
    const fetchLabels = async () => {
      try {
        const labels = await getAvailableLabels();
        setAvailableLabels(labels);
      } catch (error) {
        console.error("Failed to fetch labels:", error);
      }
    };
    fetchLabels();
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name) return;

    setIsLoading(true);
    try {
      const response = await exportDataset({
        name,
        format,
        filter: {
          type: filterType,
          labels: selectedLabels,
        },
      });

      // Blobからファイルをダウンロード
      const url = window.URL.createObjectURL(response);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${name}_${format}.zip`;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);

    } catch (error) {
      console.error("Failed to export dataset:", error);
      alert("データセットの作成に失敗しました。");
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
          <label className="block text-sm font-medium text-gray-700">
            フィルタータイプ
            <select
              value={filterType}
              onChange={(e) => setFilterType(e.target.value as "detection" | "classification")}
              className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
            >
              <option value="detection">物体検出ラベル</option>
              <option value="classification">分類ラベル</option>
            </select>
          </label>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700">
            含めるラベル（未選択の場合は全て）
            <select
              multiple
              value={selectedLabels}
              onChange={(e) => setSelectedLabels(Array.from(e.target.selectedOptions, option => option.value))}
              className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
              size={5}
            >
              {availableLabels.map(label => (
                <option key={label} value={label}>{label}</option>
              ))}
            </select>
          </label>
          <p className="mt-1 text-sm text-gray-500">
            Ctrlキーを押しながらクリックで複数選択できます
          </p>
        </div>

        <button
          type="submit"
          disabled={isLoading || !name}
          className={`w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 ${
            isLoading || !name ? 'opacity-50 cursor-not-allowed' : ''
          }`}
        >
          {isLoading ? '作成中...' : 'データセットを作成'}
        </button>
      </form>
    </div>
  );
}
