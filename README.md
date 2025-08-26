# KG Annotation v2

Knowledge Graph 画像アノテーションシステム

## プロジェクトのセットアップ

### 必要要件

- Node.js (v18以上)
- Rust (最新版)
- PostgreSQL
- sqlx-cli (`cargo install sqlx-cli`)
- AWS アカウント (S3バケット用)

### バックエンドのセットアップ

1. PostgreSQLのセットアップ
   ```bash
   # psqlでpostgresユーザーとして接続
   sudo -u postgres psql

   # データベースと専用ユーザーを作成 (your_passwordは適宜変更)
   CREATE USER kg_user WITH SUPERUSER PASSWORD 'your_password';
   CREATE DATABASE kg_annotation OWNER kg_user;
   \q
   ```
   > **Note:** `createdb kg_annotation` でも作成可能ですが、OSユーザーが存在しないエラーが出る場合は上記の方法を推奨します。

2. 環境変数の設定
   ```bash
   # backend/.envファイルを作成し、内容を記述
   # ユーザー名とパスワードはステップ1で設定したものに合わせる
   DATABASE_URL=postgresql://kg_user:your_password@localhost:5432/kg_annotation
   AWS_ACCESS_KEY_ID=your_access_key
   AWS_SECRET_ACCESS_KEY=your_secret_key
   S3_BUCKET=your_bucket_name
   ```
   > **Note:** `DATABASE_URL`のポート番号はデフォルトの`5432`を想定しています。環境に合わせて変更してください。

3. データベースマイグレーション
   ```bash
   # sqlx-cliが未インストールの場合はインストール
   cargo install sqlx-cli

   # backendディレクトリに移動してマイグレーションを実行
   cd backend
   sqlx migrate run
   ```
   > **Note:** `psql`コマンドで手動マイグレーションは行わないでください。`sqlx`が実行履歴を管理しているため、エラーの原因となります。

4. バックエンドの起動
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
