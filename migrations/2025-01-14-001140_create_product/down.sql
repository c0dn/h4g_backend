-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS private.products;

DROP INDEX IF EXISTS private.products_search_idx;