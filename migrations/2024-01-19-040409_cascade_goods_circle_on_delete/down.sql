-- This file should undo anything in `up.sql`
ALTER TABLE circle_goods
DROP CONSTRAINT IF EXISTS circle_goods_goods_id_fkey,
ADD CONSTRAINT circle_goods_goods_id_fkey
FOREIGN KEY (goods_id) REFERENCES goods(id);

ALTER TABLE goods_in_bundle
DROP CONSTRAINT IF EXISTS goods_in_bundle_goods_id_fkey,
ADD CONSTRAINT goods_in_bundle_goods_id_fkey
FOREIGN KEY (goods_id) REFERENCES goods(id);

ALTER TABLE goods_character
DROP CONSTRAINT IF EXISTS goods_character_goods_id_fkey,
ADD CONSTRAINT goods_character_goods_id_fkey
FOREIGN KEY (goods_id) REFERENCES goods(id);
