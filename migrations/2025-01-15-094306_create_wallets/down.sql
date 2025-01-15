-- This file should undo anything in `up.sql`
DROP TABLE private.transactions;
DROP TABLE private.wallets;
DROP TYPE IF EXISTS private.transaction_type;