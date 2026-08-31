# API Contract - Wiki

## 1. Base

- Base path: `/api/v1`.
- Transport: HTTP JSON.
- Auth: bearer token or session where enabled.
- Errors: stable envelope from `docs/API_STANDARDS.md`.

## 2. Resource Groups

| Group | Paths |
|---|---|
| Auth | `/auth/*`, `/users/me` |
| Spaces | `/spaces`, `/spaces/{space_key}` |
| Documents | `/documents/*`, `/spaces/{space_key}/documents` |
| Task dossiers | `/spaces/{space_key}/tasks/*` |
| Phase dossiers | `/spaces/{space_key}/phases/*` |
| Evidence | `/evidence`, `/spaces/{space_key}/tasks/{task_key}/evidence`, `/spaces/{space_key}/phases/{phase_key}/evidence` |
| Attachments | `/attachments/*` |
| Search | `/search` |
| Admin | `/users`, `/settings`, `/audit-log` |

## 3. Rules

- Create endpoints return `201 Created`.
- Commands return `200` or `202` with explicit result state.
- `Idempotency-Key` is required for mutating commands that need deduplication.
- OpenAPI is generated from code and committed.
- Generated frontend client must match OpenAPI.
