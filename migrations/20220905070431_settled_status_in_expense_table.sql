-- Add migration script here
ALTER TABLE koru_expense ADD COLUMN settled bool default false not null;