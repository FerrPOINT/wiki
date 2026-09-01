# Disaster Recovery - Wiki

## 1. Objectives

- RPO: 24 hours by default.
- RTO: 4 hours for small self-hosted deployment.
- Backups cover PostgreSQL and object storage.

## 2. Backup Scope

| Data | Backup |
|---|---|
| PostgreSQL | logical dump or volume snapshot |
| Attachments/evidence artifacts | S3/MinIO versioning or filesystem sync |
| Config | encrypted `.env`/secret manager export |
| OpenAPI/docs | git repository |

## 3. Restore Procedure

1. Stop API and workers.
2. Restore PostgreSQL.
3. Restore object storage.
4. Run checksum consistency check for `attachments`.
5. Start API in read-only mode if available.
6. Validate `/api/v1/health/ready`, document open, search and attachment download.
7. Re-enable writes.

## 4. Drills

Run a restore drill before production and then monthly.
