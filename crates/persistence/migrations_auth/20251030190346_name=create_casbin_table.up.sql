-- Add up migration script here
CREATE TABLE IF NOT EXISTS casbin_rule (
    id SERIAL PRIMARY KEY,
    ptype VARCHAR NOT NULL,
    v0 VARCHAR NOT NULL,
    v1 VARCHAR NOT NULL,
    v2 VARCHAR,
    v3 VARCHAR,
    v4 VARCHAR,
    v5 VARCHAR,
    CONSTRAINT unique_key UNIQUE(ptype, v0, v1, v2, v3, v4, v5)
);
