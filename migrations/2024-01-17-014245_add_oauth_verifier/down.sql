-- This file should undo anything in `up.sql`

ALTER TABLE users
DROP COLUMN code_verifier;

ALTER TABLE users
DROP COLUMN oauth_state;