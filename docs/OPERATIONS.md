# Operations - Wiki

## 1. Runtime Components

- Backend API.
- PostgreSQL.
- Attachment storage: filesystem in MVP, MinIO/S3-compatible adapter later if needed.
- Frontend static app.
- Optional in-process maintenance loop for expired staged uploads and idempotency records.

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
- Confirm maintenance logs do not show repeated staged attachment file delete failures.

## 4. Deployment Checklist

- `WIKI_ENVIRONMENT=production` set for shared/production deployments.
- `WIKI_JWT_SECRET` or `WIKI_AUTH__JWT_SECRET` set.
- `WIKI_DATABASE__URL` points to production PostgreSQL.
- Storage backend configured and writable.
- `WIKI_SERVER__CORS_ALLOWED_ORIGINS` contains only the HTTPS browser origins that may call the API.
- Backups enabled before first production traffic.

## 5. Readiness Gates

| Gate | Required before main development | Required before production |
| ---- | -------------------------------- | -------------------------- |
| API health/readiness documented | yes | yes |
| Docker compose config renders | yes | yes |
| WSL PostgreSQL smoke on this host | yes, accepted fallback | no, Docker or target env smoke preferred |
| Docker PostgreSQL smoke | only where Docker is available | yes |
| Backup/restore procedure documented | yes | yes |
| WSL backup restore drill on this host | yes, accepted fallback | no, target env drill preferred |
| Backup restore drill executed on target env | no | yes |
| TLS/CORS/secrets reviewed for target host | no | yes |
| Security scanner and dependency audit | no | yes |

## 6. Common Incidents

| Incident | First Action |
|---|---|
| API unavailable | Check `/api/v1/health`, target readiness, DB pool and logs |
| Search stale | Check PostgreSQL FTS update path |
| Upload failure | Check storage credentials/quota |
| Duplicate evidence | Check idempotency keys and source refs |
| Growing staged upload storage | Check `WIKI_MAINTENANCE__*`, backend logs and storage write/delete permissions |
| Permission leak suspicion | Disable affected user/session, inspect audit |

## 7. References

- `docs/TROUBLESHOOTING.md`
- `docs/DISASTER_RECOVERY.md`
- `docs/INCIDENT_RESPONSE.md`
- `docs/METRICS.md`
- `docs/MVP_READINESS.md`
