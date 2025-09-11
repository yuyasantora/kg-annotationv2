# STAGE 1: onnxruntime-builder
# onnxruntimeをLambdaと互換性のある形でビルドするためだけのステージ
# Python 3.7がインストールされるamazonlinux:2から、Python 3.9がプリインストールされている公式イメージに変更
# onnxruntimeのビルド要件を満たすため、より新しいDebian "Bookworm"をベースにします
FROM python:3.10-bookworm as onnxruntime-builder

# ビルドに必要なツールをインストール (Bookwormのデフォルトg++はバージョン12なので要件を満たします)
RUN apt-get update && apt-get install -y --no-install-recommends g++ git wget ca-certificates

# onnxruntimeが新しいバージョン(3.28+)を要求するため、最新のCMakeをインストールします
ENV CMAKE_VERSION=3.30.0
RUN wget -q https://github.com/Kitware/CMake/releases/download/v${CMAKE_VERSION}/cmake-${CMAKE_VERSION}-linux-x86_64.tar.gz && \
    tar -xzvf cmake-${CMAKE_VERSION}-linux-x86_64.tar.gz --strip-components=1 -C /usr/local && \
    rm cmake-${CMAKE_VERSION}-linux-x86_64.tar.gz

# onnxruntimeのソースコードをクローン
RUN git clone --recursive https://github.com/microsoft/onnxruntime.git

WORKDIR /onnxruntime

# onnxruntimeをビルドして、wheelパッケージを作成
# Lambda環境に合わせて、不要なプロバイダを無効化し、最適化を行う
# onnxruntimeのビルドプロセスがNumPyと、wheel作成にpackagingを要求するため、ここでインストールします
RUN python3 -m pip install -U pip setuptools wheel numpy packaging
# ビルド後のテストをスキップして安定化させる
RUN ./build.sh --config Release --build_wheel --parallel --allow_running_as_root --skip_tests

# STAGE 2: final-image
# 実際にLambdaで実行される最終的なイメージ
FROM python:3.10-slim-bookworm

WORKDIR /app

# AWS Lambda Runtime Interface Client (RIC) をインストール
RUN pip install \
        --no-cache-dir \
        --upgrade \
        awslambdaric

# STAGE 1でビルドしたonnxruntimeのwheelパッケージをコピー
COPY --from=onnxruntime-builder /onnxruntime/build/Linux/Release/dist/*.whl /tmp/

# ビルドしたonnxruntimeと、その他の依存ライブラリをインストール
RUN pip install /tmp/*.whl
COPY requirements.txt .
RUN pip install \
        --no-cache-dir \
        -r requirements.txt

# アプリケーションコードをコピー
COPY . .

# Lambdaが実行するコマンドを指定
ENTRYPOINT [ "/usr/local/bin/python", "-m", "awslambdaric" ]
CMD [ "main.handler" ]