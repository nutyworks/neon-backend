-- This file should undo anything in up.sql

-- Drop statements for tables with foreign key relationships (order matters)
DROP TABLE IF EXISTS goods_character;
DROP TABLE IF EXISTS goods_in_bundle;
DROP TABLE IF EXISTS circle_goods;
DROP TABLE IF EXISTS circle_bundles;
DROP TABLE IF EXISTS circle_artists;
DROP TABLE IF EXISTS circle_links;

-- Drop statements for other tables
DROP TABLE IF EXISTS circles;
DROP TABLE IF EXISTS artists;
DROP TABLE IF EXISTS goods;
DROP TABLE IF EXISTS categories;
DROP TABLE IF EXISTS links;
DROP TABLE IF EXISTS characters;
DROP TABLE IF EXISTS refs;
DROP TABLE IF EXISTS bundles;