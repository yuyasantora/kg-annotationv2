# 1. ビルド環境のステージ
FROM rust:1.70 AS builder

# 作業ディレクトリを作成
WORKDIR /usr/src/app

# 依存関係のファイルをコピー
COPY Cargo.toml Cargo.lock ./

# 空のmain.rsを作成して、依存関係のみをビルド（キャッシュのため）
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# ソースコードをコピー
COPY src ./src

# アプリケーションをビルド
RUN cargo build --release

# 2. 本番環境のステージ
FROM debian:buster-slim

# ビルドステージからコンパイル済みのバイナリをコピー
COPY --from=builder /usr/src/app/target/release/kg-annotation-backend /usr/local/bin/

# アプリケーションを起動
CMD ["kg-annotation-backend"]
