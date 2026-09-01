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
- File commands cover Markdown input from a path, Markdown input from stdin via `--from-file -` and attachment download to a requested path.
- CLI parity tests must keep `/api/v1/auth/register`, `/api/v1/auth/refresh`, health/readiness, metrics and OpenAPI listed as documented non-CLI exceptions rather than missing command coverage.

## 5. Permission Matrix

| Scenario | Anonymous | Viewer | Editor | Space admin | System admin |
| -------- | --------- | ------ | ------ | ----------- | ------------ |
| Read public auth/register pages | allow | allow | allow | allow | allow |
| Read published document in own space | deny | allow | allow | allow | allow |
| Create/edit/publish document | deny | deny | allow | allow | allow |
| Archive or move document | deny | deny | allow | allow | allow |
| Add URL/file evidence | deny | deny | allow | allow | allow |
| Manage space members | deny | deny | deny | allow | allow |
| Manage global users/settings/audit | deny | deny | deny | deny | allow |
| Read another space without membership | deny | deny | deny | deny unless system admin | allow |

Every permission test must check API behavior first. Frontend role states are supporting evidence only.

## 6. Data And Migration Tests

- Empty PostgreSQL database applies every migration from `backend/migrations`.
- Migration schema contains every table listed in `docs/DATA_MODEL.md`.
- Same-space constraints reject cross-space document/task/phase/evidence/attachment relations.
- Soft-delete/archive behavior is consistent for spaces and documents.
- Published revisions remain immutable after draft updates and later publishes.
- Search plan uses `document_revisions_search_idx` for the MVP filtered query shape.
- Audit writes are committed with the write command or rolled back with it.

## 7. Frontend Tests

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

## 8. E2E Smoke

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

## 9. Screenshot And Page Evidence

- `frontend/scripts/shoot-evidence.mjs` covers the approved MVP route set from `docs/ROUTING.md`.
- `README.md` renders a visible screenshot gallery for every desktop route.
- `docs/assets/screens/manifest.md` references the same screenshot files as README.
- Mobile smoke screenshots cover dashboard, spaces, document view, task dossier and search.
- Operational `/api/v1/health` and `/api/v1/health/ready` probes are API-only and do not require screenshots.

## 10. Ops And Readiness Checks

- `/api/v1/health` returns liveness for a started API process.
- `/api/v1/health/ready` reflects runtime dependency readiness.
- `/metrics` renders Prometheus text outside the versioned API contract.
- Backup and restore instructions cover PostgreSQL plus attachments.
- Docker PostgreSQL smoke is run on Docker hosts; WSL PostgreSQL smoke is the accepted Windows fallback when Docker daemon is unavailable.

## 11. Exit Criteria

- All P0/P1 REQs mapped in `docs/TRACEABILITY.md`.
- Backend unit/integration tests green.
- Frontend unit and E2E smoke green.
- Current generated frontend OpenAPI DTO types and thin endpoint wrappers typecheck.
- Generated operation-client migration is tracked as hardening, not as an MVP release blocker.
- Security test covers cross-space data isolation.
- Screenshot README/manifest/script parity check passes.
- Documentation has no unresolved task markers or short placeholder pages.
- `docs/MVP_READINESS.md` has no blocking item for main development.
