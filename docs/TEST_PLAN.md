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
| Authz          | viewer/editor/admin boundaries                                           |

## 3. Frontend Tests

- App shell navigation.
- Dashboard widgets.
- Spaces page.
- Document create/view.
- Task dossier and phase dossier pages.
- Search page states.
- Settings/admin API-backed states.
- Account menu and logout.

## 4. E2E Smoke

1. Login.
2. Open dashboard.
3. Open spaces.
4. Create document draft.
5. Open task dossier.
6. Open phase dossier.
7. Search.

## 5. Exit Criteria

- All P0/P1 REQs mapped in `docs/TRACEABILITY.md`.
- Backend unit/integration tests green.
- Frontend unit and E2E smoke green.
- Current generated frontend OpenAPI DTO types and thin endpoint wrappers typecheck.
- Full generated operation client is enabled after backend domain migration.
- Security test covers cross-space data isolation.
