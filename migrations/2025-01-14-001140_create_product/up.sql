-- Your SQL goes here
CREATE TABLE private.products (
    uuid UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    image_path TEXT NOT NULL,
    description TEXT NOT NULL,
    stock INT4 NOT NULL,
    cost INT4 NOT NULL,
    search_vector tsvector NOT NULL GENERATED ALWAYS AS (
        setweight(to_tsvector('english', coalesce(title,'')), 'A') ||
        setweight(to_tsvector('english', coalesce(description,'')), 'B')
    ) STORED
);

CREATE INDEX products_search_idx ON private.products USING GIN (search_vector);
