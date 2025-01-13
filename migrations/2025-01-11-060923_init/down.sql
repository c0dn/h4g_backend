-- This file should undo anything in `up.sql`
REVOKE ALL PRIVILEGES ON SCHEMA private FROM "testuser";
DROP SCHEMA IF EXISTS private CASCADE;