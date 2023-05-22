-- Add migration script here
ALTER TABLE koru_settlement ADD COLUMN start_date timestamptz NULL DEFAULT NULL;