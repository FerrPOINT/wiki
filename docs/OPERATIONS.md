# Operations - Wiki

## 1. Runtime Components

- Backend API.
- PostgreSQL.
- Redis.
- Object storage: filesystem, MinIO or S3.
- Frontend static app.
- Optional maintenance jobs for previews and cleanup after backend domain migration.

## 2. Health Checks

| Endpoint | Meaning |
|---|---|
| `/health` | process is alive |
| `/ready` | database/cache/storage reachable |
| `/metrics` | Prometheus metrics |

## 3. Routine Tasks

- Verify backups and restore drills.
- Monitor search freshness and upload failures.
- Rotate API tokens.
- Review audit logs for permission changes.
- Clean expired temporary files.

## 4. Deployment Checklist

- `WIKI_AUTH_SECRET` set.
- `WIKI_DATABASE_URL` points to production PostgreSQL.
- Storage backend configured and writable.
- CORS and public URL match deployment host.
- Backups enabled before first production traffic.

## 5. Common Incidents

| Incident | First Action |
|---|---|
| API unavailable | Check `/ready`, DB pool, logs |
| Search stale | Check PostgreSQL FTS update path |
| Upload failure | Check storage credentials/quota |
| Duplicate evidence | Check idempotency keys and source refs |
| Permission leak suspicion | Disable affected token, inspect audit |

## 6. References

- `docs/TROUBLESHOOTING.md`
- `docs/DISASTER_RECOVERY.md`
- `docs/INCIDENT_RESPONSE.md`
- `docs/METRICS.md`
