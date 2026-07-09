CREATE TABLE IF NOT EXISTS accounts (
    id              UUID            PRIMARY KEY,
    email           TEXT            NOT NULL,
    created_at      TIMESTAMPTZ     NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS profiles (
    id                  UUID            PRIMARY KEY,
    account_id          UUID            NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name                TEXT            NOT NULL,
    kind                TEXT            NOT NULL CHECK (kind IN ('adult', 'kids')),
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT now(),

    UNIQUE (account_id, name)
);