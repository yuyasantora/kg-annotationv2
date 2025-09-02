# STAGE 1: onnxruntime-builder
# onnxruntimeをLambdaと互換性のある形でビルドするためだけのステージ
# Python 3.7がインストールされるamazonlinux:2から、Python 3.9がプリインストールされている公式イメージに変更
FROM python:3.9-buster as onnxruntime-builder

# ビルドに必要なツールをインストール (yumからapt-getに変更)
RUN apt-get update && apt-get install -y --no-install-recommends cmake g++ git

# onnxruntimeのソースコードをクローン
RUN git clone --recursive https://github.com/microsoft/onnxruntime.git

WORKDIR /onnxruntime

# onnxruntimeをビルドして、wheelパッケージを作成
# Lambda環境に合わせて、不要なプロバイダを無効化し、最適化を行う
RUN python3 -m pip install -U pip setuptools wheel
RUN ./build.sh --config Release --build_wheel \
    --parallel --use_coreml=OFF --use_cuda=OFF --use_tensorrt=OFF \
    --use_migraphx=OFF --use_xnnpack=OFF --use_qnn=OFF \
    --use_openvino=OFF --use_nnapi=OFF --use_dnnl=OFF

# STAGE 2: final-image
# 実際にLambdaで実行される最終的なイメージ
FROM python:3.9-slim

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
CMD [ "main.app" ]