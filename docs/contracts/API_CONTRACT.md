# API Contract - Wiki

## 1. Base

- Base path: `/api/v1`.
- Transport: HTTP JSON.
- Auth: bearer token or session where enabled.
- Errors: stable envelope from `docs/API_STANDARDS.md`.

## 2. Resource Groups

| Group | Paths |
|---|---|
| Health | `/health`, `/health/ready` |
| Auth | `/auth/*`, `/users/me` |
| Spaces | `/spaces`, `/spaces/{space_key}` |
| Documents | `/documents/*`, `/spaces/{space_key}/documents` |
| Task dossiers | `/spaces/{space_key}/tasks/*` |
| Phase dossiers | `/spaces/{space_key}/phases/*` |
| Evidence | `/evidence`, `/spaces/{space_key}/tasks/{task_key}/evidence`, `/spaces/{space_key}/phases/{phase_key}/evidence` |
| Attachments | `/attachments/*` |
| Search | `/search` |
| Admin | `/users`, `/settings`, `/audit-log` |

`/metrics` is an operational Prometheus endpoint outside versioned `/api/v1` and outside OpenAPI v1.

## 3. Rules

- Create endpoints return `201 Created`.
- Commands return `200` or `202` with explicit result state.
- `Idempotency-Key` is required for mutating domain/admin commands that need deduplication. The server scopes it by actor, method, path+query and request body hash, replays successful writes, and rejects changed-payload reuse with `409 CONFLICT`; auth/session endpoints are outside replay scope.
- OpenAPI is generated from code and committed.
- Generated frontend client must match OpenAPI.
- CLI command groups map to this public API or explicitly document an API-only exception.
