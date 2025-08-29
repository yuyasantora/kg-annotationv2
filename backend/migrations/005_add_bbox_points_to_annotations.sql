-- annotationsテーブルに、バウンディングボックスとポリゴン用のカラムを追加します。
ALTER TABLE annotations
ADD COLUMN bbox REAL[], -- バウンディングボックス座標 (例: [x, y, width, height])
ADD COLUMN points JSONB; -- ポリゴン座標 (例: [[x1, y1], [x2, y2], ...])
