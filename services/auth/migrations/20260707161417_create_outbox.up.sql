CREATE TABLE IF NOT EXISTS outbox(
    id              UUID        PRIMARY KEY,
    subject         TEXT        NOT NULL,
    payload         JSONB       NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at    TIMESTAMPTZ    
);

CREATE INDEX idx_outbox_pending ON outbox (created_at) WHERE published_at IS NULL;