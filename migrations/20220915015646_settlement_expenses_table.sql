-- Add migration script here
CREATE TABLE koru_settlement_expenses
(
    settlement_id          uuid NOT NULL,
    expense_id   uuid        NOT NULL
);

ALTER TABLE koru_settlement_expenses
    ADD CONSTRAINT fk_stl_exp_stl FOREIGN KEY (settlement_id)
        REFERENCES koru_settlement (id) ON DELETE CASCADE;
ALTER TABLE koru_settlement_expenses
    ADD CONSTRAINT fk_stl_exp_exp FOREIGN KEY (expense_id)
        REFERENCES koru_expense (id) ON DELETE CASCADE;