CREATE TABLE bus_outbox_messages (
    id BIGSERIAL PRIMARY KEY,
    topic VARCHAR(256) NOT NULL,
    payload BYTEA DEFAULT NULL,
    key BYTEA DEFAULT NULL,
    headers JSONB DEFAULT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
