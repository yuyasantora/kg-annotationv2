# ビルド環境のステージ
FROM node:18-alpine AS builder

# 作業ディレクトリ
WORKDIR /app

# 依存関係のインストール
COPY package.json package-lock.json ./
RUN npm install

# ソースコードのコピー
COPY . .

# ビルド
RUN npm run build

# 本番環境のステージ
FROM node:18-alpine AS runner

# 作業ディレクトリ
WORKDIR /app

# ビルド済みの成果物をコピー
COPY --from=builder /app/public ./public
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./package.json

# アプリケーションの起動
CMD ["npm", "start"]