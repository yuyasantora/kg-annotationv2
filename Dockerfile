# 1. ベースイメージとして公式のPython 3.9イメージを使用
FROM python:3.9-slim

# 作業ディレクトリを設定
WORKDIR /app

# 2. AWS Lambda Runtime Interface Client (RIC) をインストール
# これにより、コンテナがLambdaのイベントを受け取れるようになります
RUN pip install \
        --no-cache-dir \
        --upgrade \
        awslambdaric

# onnxruntimeをビルドして、wheelパッケージを作成
# Lambda環境に合わせて、不要なプロバイダを無効化し、最適化を行う
RUN python3 -m pip install -U pip setuptools wheel
RUN ./build.sh --config Release --build_wheel \
    --parallel --use_coreml=OFF --use_cuda=OFF --use_tensorrt=OFF \
    --use_migraphx=OFF --use_xnnpack=OFF --use_qnn=OFF

# 3. 必要なライブラリをインストール
# まずrequirements.txtだけをコピーしてインストールすることで、
# コード変更時にもライブラリの再インストールをスキップでき、ビルドが高速になります
COPY requirements.txt .
RUN pip install \
        --no-cache-dir \
        -r requirements.txt

# 4. アプリケーションコードをコピー
COPY . .

# 5. Lambdaが実行するコマンドを指定
# CMD ["python", "main.py"] ではなく、Lambdaのハンドラを指定します
# main.py の中の `app` というFastAPIインスタンスをUvicornで起動します
ENTRYPOINT [ "/usr/local/bin/python", "-m", "awslambdaric" ]
CMD [ "main.app" ]
