-- imagesテーブルにベクトルを保存するためのカラムを追加します
-- jsonb型は、数値の配列のような複雑なデータを柔軟に格納するのに適しています
ALTER TABLE images
ADD COLUMN vector jsonb;
