-- This file should undo anything in `up.sql`

ALTER TABLE tokens
DROP CONSTRAINT IF EXISTS tokens_user_id_fkey,
ADD CONSTRAINT tokens_user_id_fkey
FOREIGN KEY (user_id) REFERENCES users(id);

ALTER TABLE user_circles
DROP CONSTRAINT IF EXISTS user_circles_user_id_fkey,
DROP CONSTRAINT IF EXISTS user_circles_circle_id_fkey,
ADD CONSTRAINT user_circles_user_id_fkey
FOREIGN KEY (user_id) REFERENCES users(id),
ADD CONSTRAINT user_circles_circle_id_fkey
FOREIGN KEY (circle_id) REFERENCES circles(id);