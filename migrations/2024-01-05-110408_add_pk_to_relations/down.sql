-- This file should undo anything in `up.sql`

-- Revert changes for circle_artists
ALTER TABLE circle_artists
DROP CONSTRAINT IF EXISTS unique_circle_artist_ids,
DROP COLUMN IF EXISTS id;

-- Revert changes for circle_bundles
ALTER TABLE circle_bundles
DROP CONSTRAINT IF EXISTS unique_circle_bundle_ids,
DROP COLUMN IF EXISTS id;

-- Revert changes for circle_goods
ALTER TABLE circle_goods
DROP CONSTRAINT IF EXISTS unique_circle_goods_ids,
DROP COLUMN IF EXISTS id;

-- Revert changes for circle_links
ALTER TABLE circle_links
DROP CONSTRAINT IF EXISTS unique_circle_link_ids,
DROP COLUMN IF EXISTS id;

-- Revert changes for goods_character
ALTER TABLE goods_character
DROP CONSTRAINT IF EXISTS unique_goods_character_ids,
DROP COLUMN IF EXISTS id;

-- Revert changes for goods_in_bundle
ALTER TABLE goods_in_bundle
DROP CONSTRAINT IF EXISTS unique_goods_in_bundle_ids,
DROP COLUMN IF EXISTS id;

-- For circle_artists
ALTER TABLE circle_artists
ADD CONSTRAINT circle_artists_pkey PRIMARY KEY (circle_id, artist_id);

-- For circle_bundles
ALTER TABLE circle_bundles
ADD CONSTRAINT circle_bundles_pkey PRIMARY KEY (circle_id, bundle_id);

-- For circle_goods
ALTER TABLE circle_goods
ADD CONSTRAINT circle_goods_pkey PRIMARY KEY (circle_id, goods_id);

-- For circle_links
ALTER TABLE circle_links
ADD CONSTRAINT circle_links_pkey PRIMARY KEY (link_id, circle_id);

-- For goods_character
ALTER TABLE goods_character
ADD CONSTRAINT goods_character_pkey PRIMARY KEY (goods_id, character_id);

-- For goods_in_bundle
ALTER TABLE goods_in_bundle
ADD CONSTRAINT goods_in_bundle_pkey PRIMARY KEY (bundle_id, goods_id);
