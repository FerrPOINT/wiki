# Wiki MVP Remaining Work

## 0. Current Baseline

The documentation and static frontend page design are ready enough to start backend implementation:

- MVP page set is fixed in routing, README and screenshot manifest;
- integrations, reports and notifications screens are out of frontend scope;
- README screenshot gallery points to 22 existing screenshots;
- CI/CD-style documentation filename parity is preserved;
- env/local setup/deployment docs use current `WIKI_*__*` variables and service names;
- migration docs direct new Wiki persistence work toward SQLx and quarantine inherited SeaORM task-tracker migrations;
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

Current status: runtime router, OpenAPI, API route files and default API tests are reduced to Wiki MVP; persistence/domain internals still need replacement.

## 2. Database And Migrations

- Create a clean Wiki schema migration set.
- Decide whether this project starts from a fresh Wiki schema or a compatibility migration from inherited tracker tables.
- Add indexes for document tree, revisions, task key, phase key, evidence owner and search.
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

1. Decide fresh Wiki schema vs compatibility migration from inherited tables.
2. Implement domain value objects and invariants without database dependencies.
3. Add PostgreSQL migrations and repositories for identity, spaces and documents.
4. Implement document draft/publish/history API and OpenAPI generation.
5. Wire frontend document/spaces pages to API with loading/error/empty states.
6. Add task and phase link tables/endpoints, then connect task/phase pages.
7. Implement evidence and attachment storage.
8. Add templates, search and audit coverage.
9. Bring CLI to parity with the public API.
10. Remove remaining inherited tracker backend internals and regenerate clients/specs.

## 10. Done Criteria For Backend Start

- `cargo check` and backend tests run on a host with MSVC Build Tools or another configured linker.
- Clean Wiki migrations create an empty database without tracker tables.
- OpenAPI exposes only Wiki MVP endpoints.
- UI and CLI use the same public API operations.
- Static frontend data is replaced or isolated behind mock fixtures used only in tests/screenshots.
