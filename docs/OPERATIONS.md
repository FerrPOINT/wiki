# Operations - Wiki

## 1. Runtime Components

- Backend API.
- PostgreSQL.
- Attachment storage: filesystem in MVP, MinIO/S3-compatible adapter later if needed.
- Frontend static app.
- Optional in-process maintenance jobs for cleanup after backend domain migration.

## 2. Health Checks

| Endpoint | Meaning |
|---|---|
| `/api/v1/health` | current process liveness |
| `/api/v1/health/ready` | persistent backend readiness |
| `/metrics` | Prometheus metrics |

## 3. Routine Tasks

- Verify backups and restore drills.
- Monitor search freshness and upload failures.
- Rotate JWT/refresh secrets; rotate future API tokens only if that deferred scope is enabled.
- Review audit logs for permission changes.
- Clean expired temporary files.

## 4. Deployment Checklist

- `WIKI_JWT_SECRET` or `WIKI_AUTH__JWT_SECRET` set.
- `WIKI_DATABASE__URL` points to production PostgreSQL.
- Storage backend configured and writable.
- CORS and public URL match deployment host.
- Backups enabled before first production traffic.

## 5. Common Incidents

| Incident | First Action |
|---|---|
| API unavailable | Check `/api/v1/health`, target readiness, DB pool and logs |
| Search stale | Check PostgreSQL FTS update path |
| Upload failure | Check storage credentials/quota |
| Duplicate evidence | Check idempotency keys and source refs |
| Permission leak suspicion | Disable affected user/session, inspect audit |

## 6. References

- `docs/TROUBLESHOOTING.md`
- `docs/DISASTER_RECOVERY.md`
- `docs/INCIDENT_RESPONSE.md`
- `docs/METRICS.md`
