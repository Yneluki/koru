-- Add migration script here
alter table koru_group_members add column joined_at timestamptz NOT NULL DEFAULT now();
-- green by default
alter table koru_group_members add column color TEXT NOT NULL DEFAULT '0,255,0';