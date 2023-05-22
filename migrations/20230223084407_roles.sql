-- Add migration script here
CREATE TYPE role AS ENUM ('admin', 'user');
CREATE TABLE koru_user_roles
(
    user_id      uuid        NOT NULL,
    PRIMARY KEY (user_id),
    role    role       NOT NULL default 'user'
);
ALTER TABLE koru_user_roles
    ADD CONSTRAINT fk_user_role FOREIGN KEY (user_id)
        REFERENCES koru_user (id) ON DELETE CASCADE;
