DROP INDEX audit_time_id_idx;

CREATE INDEX audit_time_idx ON audit_log (created_at DESC);
