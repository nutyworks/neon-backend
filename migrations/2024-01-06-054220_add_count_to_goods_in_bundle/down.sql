-- This file should undo anything in `up.sql`

-- Drop the 'count' column from the goods_in_bundle table
ALTER TABLE goods_in_bundle
DROP COLUMN count;