-- Your SQL goes here

ALTER TABLE users
ADD COLUMN code_verifier CHAR(128);

ALTER TABLE users
ADD COLUMN oauth_state CHAR(16);