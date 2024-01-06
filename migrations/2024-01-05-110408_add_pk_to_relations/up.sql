-- Your SQL goes here

-- For circle_artists
ALTER TABLE circle_artists
DROP CONSTRAINT IF EXISTS circle_artists_pkey;

-- For circle_bundles
ALTER TABLE circle_bundles
DROP CONSTRAINT IF EXISTS circle_bundles_pkey;

-- For circle_goods
ALTER TABLE circle_goods
DROP CONSTRAINT IF EXISTS circle_goods_pkey;

-- For circle_links
ALTER TABLE circle_links
DROP CONSTRAINT IF EXISTS circle_links_pkey;

-- For goods_character
ALTER TABLE goods_character
DROP CONSTRAINT IF EXISTS goods_character_pkey;

-- For goods_in_bundle
ALTER TABLE goods_in_bundle
DROP CONSTRAINT IF EXISTS goods_in_bundle_pkey;

-- For circle_artists
ALTER TABLE circle_artists
ADD COLUMN id SERIAL PRIMARY KEY,
ADD CONSTRAINT unique_circle_artist_ids UNIQUE (circle_id, artist_id);

-- For circle_bundles
ALTER TABLE circle_bundles
ADD COLUMN id SERIAL PRIMARY KEY,
ADD CONSTRAINT unique_circle_bundle_ids UNIQUE (circle_id, bundle_id);

-- For circle_goods
ALTER TABLE circle_goods
ADD COLUMN id SERIAL PRIMARY KEY,
ADD CONSTRAINT unique_circle_goods_ids UNIQUE (circle_id, goods_id);

-- For circle_links
ALTER TABLE circle_links
ADD COLUMN id SERIAL PRIMARY KEY,
ADD CONSTRAINT unique_circle_link_ids UNIQUE (link_id, circle_id);

-- For goods_character
ALTER TABLE goods_character
ADD COLUMN id SERIAL PRIMARY KEY,
ADD CONSTRAINT unique_goods_character_ids UNIQUE (goods_id, character_id);

-- For goods_in_bundle
ALTER TABLE goods_in_bundle
ADD COLUMN id SERIAL PRIMARY KEY,
ADD CONSTRAINT unique_goods_in_bundle_ids UNIQUE (bundle_id, goods_id);
