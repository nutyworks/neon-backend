-- Your SQL goes here
ALTER TABLE circle_goods
DROP CONSTRAINT IF EXISTS circle_goods_goods_id_fkey,
ADD CONSTRAINT circle_goods_goods_id_fkey
FOREIGN KEY (goods_id) REFERENCES goods(id) ON DELETE CASCADE;

ALTER TABLE goods_in_bundle
DROP CONSTRAINT IF EXISTS goods_in_bundle_goods_id_fkey,
ADD CONSTRAINT goods_in_bundle_goods_id_fkey
FOREIGN KEY (goods_id) REFERENCES goods(id) ON DELETE CASCADE;
