-- Your SQL goes here

-- Add the 'count' column to the goods_in_bundle table
ALTER TABLE goods_in_bundle
ADD COLUMN count INT NOT NULL DEFAULT 1;

-- Update existing rows to set a default count value (e.g., 1)
UPDATE goods_in_bundle
SET count = 1;