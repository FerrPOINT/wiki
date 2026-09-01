# Delivery Architecture - Wiki

## 1. Components

```text
Browser -> Frontend static app -> Backend API -> PostgreSQL
                                      |-> Attachment storage
                                      |-> PostgreSQL FTS
```

## 2. Deployment Modes

| Mode | Description |
|---|---|
| local dev | Vite + cargo run + local PostgreSQL |
| docker compose | frontend, backend, PostgreSQL and uploads volume |
| production single-node | reverse proxy + services + managed backup |

## 3. Release Flow

1. Build backend and frontend.
2. Run migrations.
3. Start API in readiness-gated mode.
4. Serve frontend static assets.
5. Run smoke tests.

## 4. Rollback

- Code rollback must be compatible with applied migrations.
- Data rollback uses backup/restore only for destructive incidents.
- Evidence and published revisions are append-only; rollback adds compensating records.

## 5. References

- `docs/DEPLOYMENT.md`
- `docs/RUNTIME.md`
- `docs/OPERATIONS.md`
