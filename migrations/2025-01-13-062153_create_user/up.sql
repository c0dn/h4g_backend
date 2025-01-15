-- Your SQL goes here
CREATE TYPE private.account_type AS ENUM ('user', 'admin');

CREATE TABLE private.users (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resident_id TEXT NOT NULL,
    name TEXT NOT NULL,
    phone TEXT NOT NULL,
    password TEXT NOT NULL,
    email TEXT NOT NULL,
    role private.account_type NOT NULL,
    active BOOL NOT NULL,
    dob TEXT,
    address JSONB,
    school TEXT,
    force_pw_change BOOL NOT NULL
);

CREATE UNIQUE INDEX idx_users_resident_id ON private.users (resident_id);