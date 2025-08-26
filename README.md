# KG Annotation v2

Knowledge Graph 画像アノテーションシステム

## プロジェクトのセットアップ

### 必要要件

- Node.js (v18以上)
- Rust (最新版)
- PostgreSQL
- AWS アカウント (S3バケット用)

### バックエンドのセットアップ

1. PostgreSQLのセットアップ
```bash
# PostgreSQLデータベースの作成
createdb kg_annotation

# マイグレーションの実行
cd backend
psql postgresql://postgres:password@localhost:5433/kg_annotation -f migrations/001_initial.sql
psql postgresql://postgres:password@localhost:5433/kg_annotation -f migrations/002_add_datasets.sql
```

2. 環境変数の設定
```bash
# backend/.envファイルを作成
DATABASE_URL=postgresql://postgres:password@localhost:5433/kg_annotation
AWS_ACCESS_KEY_ID=your_access_key
AWS_SECRET_ACCESS_KEY=your_secret_key
S3_BUCKET=your_bucket_name
```

3. バックエンドの起動
```bash
cd backend
cargo run
```

### フロントエンドのセットアップ

1. 依存関係のインストール
```bash
cd frontend
npm install
```

2. 環境変数の設定
```bash
# frontend/.env.localファイルを作成
NEXT_PUBLIC_API_URL=http://localhost:3002
NEXT_PUBLIC_AI_API_URL=http://localhost:8001
```

3. 開発サーバーの起動
```bash
npm run dev
```

### AIサービスのセットアップ

1. 依存関係のインストール
```bash
cd ai-service
pip install -r requirements.txt
```

2. サービスの起動
```bash
python main.py
```

## アクセス方法

- フロントエンド: http://localhost:3000
- バックエンドAPI: http://localhost:3002
- AIサービス: http://localhost:8001

## 主な機能

- 画像アップロード
- AIによる自動アノテーション
- 手動アノテーション
- 画像検索
- データセット作成（YOLO, COCO, VOC形式）

## 技術スタック

- フロントエンド
  - Next.js
  - TypeScript
  - Tailwind CSS
  - shadcn/ui

- バックエンド
  - Rust
  - Axum
  - SQLx
  - PostgreSQL

- AIサービス
  - Python
  - FastAPI
  - YOLOX
  - sentence-transformers

## ライセンス

[ライセンス情報を追加]
