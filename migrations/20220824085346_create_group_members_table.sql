-- Add migration script here
CREATE TABLE koru_group_members
(
    group_id   uuid        NOT NULL,
    user_id   uuid        NOT NULL
);
