-- Add migration script here
alter table koru_group_members add primary key (group_id,user_id)
