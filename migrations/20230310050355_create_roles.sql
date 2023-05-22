-- Add migration script here

TRUNCATE koru_user_roles;
INSERT INTO koru_user_roles (user_id, role) SELECT id, 'user' as role FROM koru_user;