-- Add migration script here
CREATE TABLE koru_user_credentials
(
    email      TEXT        NOT NULL,
    PRIMARY KEY (email),
    password   TEXT        NOT NULL
);

INSERT INTO koru_user_credentials (email, password) SELECT email, password FROM koru_user;

ALTER TABLE koru_user DROP COLUMN password;
