-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS private.idx_users_username;

DROP TABLE IF EXISTS private.users;

DROP TYPE IF EXISTS private.account_type;
