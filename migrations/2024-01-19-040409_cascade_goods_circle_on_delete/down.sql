-- This file should undo anything in `up.sql`
ALTER TABLE circle_goods
DROP CONSTRAINT IF EXISTS circle_goods_goods_id_fkey,
ADD CONSTRAINT circle_goods_goods_id_fkey
FOREIGN KEY (goods_id) REFERENCES goods(id);