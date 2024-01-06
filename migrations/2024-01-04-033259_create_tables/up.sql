-- Your SQL goes here

CREATE TABLE circles (
  id SERIAL PRIMARY KEY,
  name varchar(255) NOT NULL,
  description text,
  location varchar(255)
);

CREATE TABLE artists (
  id SERIAL PRIMARY KEY,
  name varchar(255) NOT NULL,
  account_url varchar(255)
);

CREATE TABLE goods (
  id SERIAL PRIMARY KEY,
  name varchar(255),
  description text,
  price int,
  category_id SERIAL
);

CREATE TABLE categories (
  id SERIAL PRIMARY KEY,
  name varchar(255) NOT NULL
);

CREATE TYPE link_type AS ENUM ('prepayment', 'netorder', 'info', 'notice', 'other');

CREATE TABLE links (
  id SERIAL PRIMARY KEY,
  type link_type NOT NULL,
  url varchar(255) NOT NULL
);

CREATE TABLE characters (
  id SERIAL PRIMARY KEY,
  name varchar(255) NOT NULL,
  reference_id SERIAL
);

CREATE TABLE refs (
  id SERIAL PRIMARY KEY,
  name varchar(255) NOT NULL
);

CREATE TABLE goods_character (
  goods_id SERIAL,
  character_id SERIAL,
  PRIMARY KEY (goods_id, character_id)
);

CREATE TYPE bundle_type AS ENUM ('select', 'random');

CREATE TABLE bundles (
  id SERIAL PRIMARY KEY,
  name varchar(255),
  price int,
  description varchar(255),
  type bundle_type NOT NULL,
  count int NOT NULL
);

CREATE TABLE goods_in_bundle (
  bundle_id SERIAL,
  goods_id SERIAL,
  PRIMARY KEY (bundle_id, goods_id)
);

CREATE TABLE circle_goods (
  circle_id SERIAL,
  goods_id SERIAL,
  PRIMARY KEY (circle_id, goods_id)
);

CREATE TABLE circle_bundles (
  circle_id SERIAL,
  bundle_id SERIAL,
  PRIMARY KEY (circle_id, bundle_id)
);

CREATE TABLE circle_artists (
  circle_id SERIAL,
  artist_id SERIAL,
  PRIMARY KEY (circle_id, artist_id)
);

CREATE TABLE circle_links (
  link_id SERIAL,
  circle_id SERIAL,
  PRIMARY KEY (link_id, circle_id)
);

ALTER TABLE goods ADD FOREIGN KEY (category_id) REFERENCES categories (id);

ALTER TABLE characters ADD FOREIGN KEY (reference_id) REFERENCES refs (id);

ALTER TABLE goods_character ADD FOREIGN KEY (goods_id) REFERENCES goods (id);

ALTER TABLE goods_character ADD FOREIGN KEY (character_id) REFERENCES characters (id);

ALTER TABLE goods_in_bundle ADD FOREIGN KEY (bundle_id) REFERENCES bundles (id);

ALTER TABLE goods_in_bundle ADD FOREIGN KEY (goods_id) REFERENCES goods (id);

ALTER TABLE circle_goods ADD FOREIGN KEY (circle_id) REFERENCES circles (id);

ALTER TABLE circle_goods ADD FOREIGN KEY (goods_id) REFERENCES goods (id);

ALTER TABLE circle_bundles ADD FOREIGN KEY (circle_id) REFERENCES circles (id);

ALTER TABLE circle_bundles ADD FOREIGN KEY (bundle_id) REFERENCES bundles (id);

ALTER TABLE circle_artists ADD FOREIGN KEY (circle_id) REFERENCES circles (id);

ALTER TABLE circle_artists ADD FOREIGN KEY (artist_id) REFERENCES artists (id);

ALTER TABLE circle_links ADD FOREIGN KEY (link_id) REFERENCES links (id);

ALTER TABLE circle_links ADD FOREIGN KEY (circle_id) REFERENCES circles (id);