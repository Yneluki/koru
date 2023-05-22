-- Add migration script here
ALTER TABLE koru_group
    ADD CONSTRAINT fk_group_admin_user FOREIGN KEY (admin_id)
        REFERENCES koru_user (id) ON DELETE CASCADE;

ALTER TABLE koru_group_members
    ADD CONSTRAINT fk_group_members_user FOREIGN KEY (user_id)
        REFERENCES koru_user (id) ON DELETE CASCADE;

ALTER TABLE koru_group_members
    ADD CONSTRAINT fk_group_members_group FOREIGN KEY (group_id)
        REFERENCES koru_group (id) ON DELETE CASCADE;