-- ユーザーテーブル
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE user_role AS ENUM ('admin', 'annotator', 'viewer');

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    cognito_sub VARCHAR NOT NULL UNIQUE,
    username VARCHAR NOT NULL UNIQUE,
    email VARCHAR NOT NULL UNIQUE,
    role user_role NOT NULL DEFAULT 'annotator',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 画像テーブル
CREATE TABLE images (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    filename VARCHAR NOT NULL,
    original_filename VARCHAR NOT NULL,
    s3_key VARCHAR NOT NULL,
    s3_bucket VARCHAR NOT NULL,
    file_size BIGINT NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    format VARCHAR NOT NULL,
    classification_label VARCHAR,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- アノテーションテーブル
CREATE TYPE annotation_type AS ENUM ('boundingbox', 'point', 'polygon');
CREATE TYPE annotation_source AS ENUM ('manual', 'ai');

CREATE TABLE annotations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    image_id UUID NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    annotation_type annotation_type NOT NULL DEFAULT 'boundingbox',
    x REAL NOT NULL,
    y REAL NOT NULL,
    width REAL NOT NULL,
    height REAL NOT NULL,
    label VARCHAR NOT NULL,
    confidence REAL,
    source annotation_source NOT NULL DEFAULT 'manual',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- インデックス
CREATE INDEX idx_annotations_image_id ON annotations(image_id);
CREATE INDEX idx_annotations_user_id ON annotations(user_id);
CREATE INDEX idx_images_user_id ON images(user_id);

-- テストデータ挿入
INSERT INTO users (cognito_sub, username, email, role) VALUES 
('test-user-1', 'testuser', 'test@example.com', 'annotator');
