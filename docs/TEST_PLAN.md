# Test Plan - Wiki

## 1. Scope

The test plan covers document lifecycle, task/phase links, evidence ingestion, permissions, search and operational behavior.

## 2. Backend Tests

| Area           | Scenarios                                                                |
| -------------- | ------------------------------------------------------------------------ |
| Spaces         | create/update/archive, membership, duplicate key                         |
| Documents      | draft, publish, conflict, archive, move cycle rejection                  |
| Revisions      | immutable content, diff, history                                         |
| Task dossiers  | idempotent link creation, external task key lookup, permission filtering |
| Phase dossiers | phase key grouping, linked documents/evidence, permission filtering      |
| Evidence       | add file/url evidence, checksum, dedup                                   |
| Attachments    | upload/download, filename/content-type validation, quota                  |
| Search         | indexing, permission filtering, archived filters                         |
| Settings       | admin-only safe runtime snapshot                                         |
| Health         | liveness, readiness before/after runtime dependency initialization       |
| Authz          | viewer/editor/admin boundaries                                           |

## 3. API Contract Tests

- `openapi/openapi.json` must expose every route implemented by the public API router.
- `docs/API.md` and `docs/PRODUCT_REQUIREMENTS.md` must document the same endpoint paths as OpenAPI.
- API error responses must keep the standard `{ error: { code, message } }` envelope.
- List endpoints must keep bounded `limit` behavior and stable ordering.
- Upload endpoints must reject empty files, unsafe filenames and payloads over the configured size limit.
- Search endpoints must verify permission filtering before returning document, task, phase or evidence results.

## 4. CLI Tests

- CLI commands call only `/api/v1` HTTP endpoints and never access PostgreSQL, filesystem storage or backend internals directly.
- JSON output is the default for every command group.
- Non-success API responses produce a non-zero exit code and preserve the API error code/message.
- Write commands send an idempotency key where retries are expected.
- File commands cover Markdown input from file/stdin and attachment download to a requested path.

## 5. Frontend Tests

- App shell navigation.
- Dashboard widgets.
- Spaces page.
- Document create/view.
- Task dossier and phase dossier pages.
- Evidence list and add-link/add-file forms.
- Templates list and apply flow.
- Search page states.
- Settings/admin API-backed states.
- Account menu and logout.
- Loading, empty, validation-error, permission-error and retry states for API-backed MVP pages.

## 6. E2E Smoke

1. Login.
2. Open dashboard.
3. Open spaces.
4. Create document draft.
5. Save draft and publish a revision.
6. Open task dossier.
7. Open phase dossier.
8. Add URL evidence and file evidence.
9. Search with document type, task key and phase key filters.
10. Open admin settings/audit pages as system admin.

## 7. Screenshot And Page Evidence

- `frontend/scripts/shoot-evidence.mjs` covers the approved MVP route set from `docs/ROUTING.md`.
- `README.md` renders a visible screenshot gallery for every desktop route.
- `docs/assets/screens/manifest.md` references the same screenshot files as README.
- Mobile smoke screenshots cover dashboard, spaces, document view, task dossier and search.
- Operational `/api/v1/health` and `/api/v1/health/ready` probes are API-only and do not require screenshots.

## 8. Exit Criteria

- All P0/P1 REQs mapped in `docs/TRACEABILITY.md`.
- Backend unit/integration tests green.
- Frontend unit and E2E smoke green.
- Current generated frontend OpenAPI DTO types and thin endpoint wrappers typecheck.
- Generated operation-client migration is tracked as hardening, not as an MVP release blocker.
- Security test covers cross-space data isolation.
- Screenshot README/manifest/script parity check passes.
- Documentation has no unresolved task markers or short placeholder pages.
