CREATE TABLE main.users (
  id UUID PRIMARY KEY DEFAULT main.gen_random_uuid(),
  username VARCHAR(128) NOT NULL UNIQUE,
  email VARCHAR(128) NOT NULL,
  active BOOLEAN NOT NULL DEFAULT FALSE
);
