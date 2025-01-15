-- Your SQL goes here

CREATE TYPE private.transaction_type AS ENUM ('debit', 'credit');


CREATE TABLE private.wallets (
    id SERIAL PRIMARY KEY,
    user_uuid UUID NOT NULL REFERENCES private.users(uuid),
    balance INT4 NOT NULL DEFAULT 0.00,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_uuid)
);

CREATE TABLE private.transactions (
    id SERIAL PRIMARY KEY,
    wallet_id INTEGER NOT NULL REFERENCES private.wallets(id),
    amount INT4 NOT NULL,
    transaction_type private.transaction_type NOT NULL,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_wallet_user_uuid ON private.wallets(user_uuid);
CREATE INDEX idx_transaction_wallet ON private.transactions(wallet_id);