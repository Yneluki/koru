-- Add migration script here
CREATE TABLE koru_expense
(
    id          uuid NOT NULL,
     PRIMARY KEY (id),
    group_id   uuid        NOT NULL,
    member_id   uuid        NOT NULL,
    description       TEXT        NOT NULL,
    amount real not null,
    created_at timestamptz NOT NULL,
    modified_at timestamptz NULL
);

ALTER TABLE koru_expense
    ADD CONSTRAINT fk_expenses_group FOREIGN KEY (group_id)
        REFERENCES koru_group (id) ON DELETE CASCADE;
ALTER TABLE koru_expense
    ADD CONSTRAINT fk_expenses_user FOREIGN KEY (member_id)
        REFERENCES koru_user (id) ON DELETE CASCADE;