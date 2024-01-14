-- Your SQL goes here

CREATE TYPE role_type AS ENUM ('admin', 'moderator', 'user');

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    handle VARCHAR(20) UNIQUE NOT NULL,
    nickname VARCHAR(100) NOT NULL,
    password CHAR(97) NOT NULL,
    twitter_id VARCHAR(16) DEFAULT NULL,
    role role_type NOT NULL DEFAULT 'user'
);

CREATE TABLE tokens (
    id SERIAL PRIMARY KEY,
    selector CHAR(12) NOT NULL,
    hashed_validator char(64) NOT NULL,
    user_id SERIAL REFERENCES users(id),
    expires TIMESTAMP
);

CREATE TABLE user_circles (
    id SERIAL PRIMARY KEY,
    user_id SERIAL REFERENCES users(id),
    circle_id SERIAL REFERENCES circles(id),
    UNIQUE (user_id, circle_id)
)