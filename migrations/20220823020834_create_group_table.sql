-- Add migration script here
CREATE TABLE koru_group
(
    id         uuid        NOT NULL,
    PRIMARY KEY (id),
    name       TEXT        NOT NULL,
    admin_id   uuid        NOT NULL,
    created_at timestamptz NOT NULL
);
