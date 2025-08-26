-- データセットのフォーマット型
CREATE TYPE dataset_format AS ENUM ('yolo', 'coco', 'voc');

-- データセットテーブル
CREATE TABLE datasets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    description TEXT,
    format dataset_format NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- データセットと画像の関連テーブル
CREATE TABLE dataset_images (
    dataset_id UUID REFERENCES datasets(id) ON DELETE CASCADE,
    image_id UUID REFERENCES images(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (dataset_id, image_id)
);
