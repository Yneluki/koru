-- Add migration script here
CREATE TABLE koru_user
(
    id         uuid        NOT NULL,
    PRIMARY KEY (id),
    email      TEXT        NOT NULL UNIQUE,
    name       TEXT        NOT NULL,
    password   TEXT        NOT NULL,
    created_at timestamptz NOT NULL
);