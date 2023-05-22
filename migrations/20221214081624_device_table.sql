-- Add migration script here
CREATE TABLE koru_user_device
(
    user_id uuid NOT NULL,
    PRIMARY KEY (user_id),
    device  text NULL DEFAULT NULL
);

ALTER TABLE koru_user_device
    ADD CONSTRAINT fk_user_device FOREIGN KEY (user_id)
        REFERENCES koru_user (id) ON DELETE CASCADE;