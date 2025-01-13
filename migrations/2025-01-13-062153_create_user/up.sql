-- Your SQL goes here
CREATE TYPE private.account_type AS ENUM ('user', 'admin', 'super_admin');

CREATE TABLE private.users (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL,
    name TEXT NOT NULL,
    phone TEXT NOT NULL,
    password TEXT NOT NULL,
    email TEXT NOT NULL,
    role private.account_type NOT NULL,
    active BOOL NOT NULL
);

CREATE UNIQUE INDEX idx_users_username ON private.users (username);