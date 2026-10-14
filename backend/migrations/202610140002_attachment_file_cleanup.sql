-- Keep file cleanup retryable after the staged attachment is no longer claimable.
CREATE TABLE attachment_file_cleanup (
    attachment_id uuid PRIMARY KEY,
    storage_key text NOT NULL UNIQUE CHECK (btrim(storage_key) <> ''),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX attachment_file_cleanup_created_idx
    ON attachment_file_cleanup (created_at, attachment_id);
