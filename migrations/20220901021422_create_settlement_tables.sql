-- Add migration script here
CREATE TABLE koru_settlement
(
    id          uuid NOT NULL,
    PRIMARY KEY (id),
    group_id   uuid        NOT NULL,
    end_date timestamptz NOT NULL
);

CREATE TABLE koru_transaction
(
    settlement_id          uuid NOT NULL,
    from_user_id   uuid        NOT NULL,
    to_user_id   uuid        NOT NULL,
    amount real NOT NULL
);

ALTER TABLE koru_settlement
    ADD CONSTRAINT fk_settlement_date_group FOREIGN KEY (group_id)
        REFERENCES koru_group (id) ON DELETE CASCADE;
ALTER TABLE koru_transaction
    ADD CONSTRAINT fk_transaction_from_user FOREIGN KEY (from_user_id)
        REFERENCES koru_user (id) ON DELETE CASCADE;
ALTER TABLE koru_transaction
    ADD CONSTRAINT fk_transaction_to_user FOREIGN KEY (to_user_id)
        REFERENCES koru_user (id) ON DELETE CASCADE;
ALTER TABLE koru_transaction
    ADD CONSTRAINT fk_transaction_stl FOREIGN KEY (settlement_id)
        REFERENCES koru_settlement (id) ON DELETE CASCADE;