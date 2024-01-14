-- This file should undo anything in `up.sql`

DROP TABLE user_circles;
DROP TABLE tokens;
DROP TABLE users;

DROP TYPE IF EXISTS role_type;
