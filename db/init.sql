SET CLIENT_MIN_MESSAGES TO WARNING;
SET CLIENT_ENCODING = 'UTF8';
DROP SCHEMA IF EXISTS main CASCADE;
SET CLIENT_MIN_MESSAGES TO DEBUG;
SET CLIENT_ENCODING = 'UTF8';
-- We create everything within a main schema, so that we have
-- an easy mean to setup / tear down by creating and dropping
-- the schema.
CREATE SCHEMA main;
SET SEARCH_PATH = main;
CREATE EXTENSION pg_trgm SCHEMA main;
CREATE EXTENSION pgcrypto SCHEMA main;
CREATE TABLE main.users (
  id UUID PRIMARY KEY DEFAULT main.gen_random_uuid(),
  username VARCHAR(128) NOT NULL UNIQUE,
  email VARCHAR(128) NOT NULL,
  active BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  update_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
