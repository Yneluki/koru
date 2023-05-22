-- Add migration script here
CREATE TABLE koru_event
(
    id         uuid        NOT NULL,
    PRIMARY KEY (id),
    event_date   timestamptz NOT NULL,
    process_date timestamptz NULL DEFAULT NULL,
    event_data   json        NOT NULL
);