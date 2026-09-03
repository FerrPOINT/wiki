CREATE TABLE idempotency_records (
    id uuid PRIMARY KEY,
    actor_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    idempotency_key text NOT NULL,
    method text NOT NULL,
    path text NOT NULL,
    request_hash text NOT NULL,
    state text NOT NULL,
    response_status integer,
    response_content_type text,
    response_body bytea,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL DEFAULT now() + interval '24 hours',
    CONSTRAINT idempotency_key_not_blank CHECK (btrim(idempotency_key) <> ''),
    CONSTRAINT idempotency_method_not_blank CHECK (btrim(method) <> ''),
    CONSTRAINT idempotency_path_not_blank CHECK (btrim(path) <> ''),
    CONSTRAINT idempotency_request_hash_not_blank CHECK (btrim(request_hash) <> ''),
    CONSTRAINT idempotency_state_check CHECK (state IN ('processing', 'completed')),
    CONSTRAINT idempotency_completed_response_check CHECK (
        (
            state = 'processing'
            AND response_status IS NULL
            AND response_content_type IS NULL
            AND response_body IS NULL
        )
        OR (
            state = 'completed'
            AND response_status IS NOT NULL
            AND response_body IS NOT NULL
        )
    )
);

CREATE UNIQUE INDEX idempotency_records_actor_key_idx
    ON idempotency_records (actor_id, idempotency_key);

CREATE INDEX idempotency_records_expires_idx
    ON idempotency_records (expires_at);
