# Wiki MVP Remaining Work

## 0. Current Baseline

The documentation and static frontend page design are ready enough to start backend implementation:

- MVP page set is fixed in routing, README and screenshot manifest;
- integrations, reports and notifications screens are out of frontend scope;
- README screenshot gallery points to 22 existing screenshots;
- CI/CD-style documentation filename parity is preserved;
- env/local setup/deployment docs use current `WIKI_*__*` variables and service names;
- migration docs direct new Wiki persistence work toward SQLx and quarantine inherited SeaORM task-tracker migrations;
- Wiki-owned domain value objects/invariants exist in `domain::wiki`;
- a fresh SQLx MVP schema baseline exists in `backend/migrations/202608310001_create_wiki_mvp.*.sql`;
- deferred areas are documented as reference only.

The remaining work is implementation, not product-scope expansion.

## 1. Backend Domain Migration

The public API/router is now a Wiki MVP in-memory shell. Replace the inherited task-tracker domain behind it with Wiki-owned modules:

- `spaces` and `space_members`;
- `documents`, `document_drafts`, `document_revisions`;
- document tree and breadcrumbs;
- task links by external task key;
- phase links by workflow phase key;
- evidence items and attachments;
- templates;
- audit log;
- PostgreSQL full-text search.

Remove or quarantine remaining inherited tracker concepts from backend internals:

- issues, boards, sprints and worklogs;
- watchers, votes, labels and issue links;
- custom fields, components and versions;
- reports and notifications runtime services.

Current status: runtime router, OpenAPI, API route files and default API tests are reduced to Wiki MVP; a Wiki domain baseline exists; app/infra runtime wiring still needs replacement.

## 2. Database And Migrations

- Extend the clean SQLx baseline only through new SQLx migrations.
- Treat Wiki as a fresh schema; inherited tracker migrations are compatibility/quarantine only.
- Review indexes with `EXPLAIN` when repositories are implemented.
- Keep audit writes transactional with the command that caused them.
- Update `docs/MIGRATIONS.md` after the schema decision.

## 3. API And OpenAPI

- Replace in-memory API handlers with application use cases backed by Wiki repositories.
- Keep inherited tracker routes out of the runtime router.
- Regenerate `openapi/openapi.json` after any handler DTO/route change.
- Add generated frontend API types after OpenAPI is stable.
- Keep UI and CLI as ordinary clients of the same public API.

## 4. CLI Parity

- Keep `backend/cli/src/main.rs` aligned with `docs/CLI.md`.
- Add CLI smoke tests against mocked HTTP responses.
- Add idempotency keys for repeated write commands.

## 5. Frontend Integration

- Replace static page data with API queries and mutations.
- Add loading, empty, permission denied, validation error and retry states on API-backed pages.
- Implement document tree, document editor, revision panel and evidence feed as reusable widgets/features.
- Keep visible UI text Russian by default.
- Keep deferred integrations, reports and notifications pages out of MVP routes.

## 6. Search, Storage And Audit

- Implement PostgreSQL FTS over document title/body and evidence metadata.
- Implement local filesystem storage first, with S3/MinIO behind the same storage port.
- Store checksums and size metadata for uploaded files.
- Record audit entries for document publish/archive, evidence writes, user changes and permission changes.

## 7. Tests And Release Readiness

- Add backend unit tests for domain invariants.
- Add repository/API tests for spaces, documents, revisions, task/phase links, evidence, attachments, search and audit.
- Add frontend component tests for editor/tree/revision/evidence states.
- Keep screenshot evidence regenerated after route or UI changes.
- Fix local Rust toolchain by installing MSVC Build Tools so `cargo check/test` can run on this host.

## 8. Deferred Until After MVP

- Reports UI and report projections.
- Notification center, unread counters and delivery channels.
- Webhook ingestion and outbound delivery.
- Import/export bundles.
- Real-time collaborative editing.
- OCR and binary attachment indexing.

## 9. Recommended Implementation Order

1. Add SQLx repository interfaces/implementations for identity, spaces and documents using `domain::wiki`.
2. Wire server composition to SQLx repositories behind feature-compatible app use cases.
3. Implement document draft/publish/history API against PostgreSQL and regenerate OpenAPI.
4. Wire frontend document/spaces pages to API with loading/error/empty states.
5. Add task and phase link repository operations, then connect task/phase pages.
6. Implement evidence and attachment storage transactionally, including staged attachment claim.
7. Add templates, search and audit coverage.
8. Bring CLI smoke tests to parity with the public API.
9. Remove remaining inherited tracker backend internals and SeaORM migration compatibility layer.
10. Generate frontend API client after the PostgreSQL-backed contract stabilizes.

## 10. Done Criteria For Backend Start

- `cargo check` and backend tests run on a host with MSVC Build Tools or another configured linker.
- Clean Wiki SQLx migrations create an empty database without tracker tables.
- OpenAPI exposes only Wiki MVP endpoints.
- UI and CLI use the same public API operations.
- Static frontend data is replaced or isolated behind mock fixtures used only in tests/screenshots.
