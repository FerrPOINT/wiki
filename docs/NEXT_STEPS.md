# Wiki MVP Remaining Work

## 0. Current Baseline

The documentation, screenshots, API-backed frontend MVP pages and SQLx-backed MVP API runtime are ready enough to continue hardening:

- MVP page set is fixed in routing, README and screenshot manifest;
- integrations, reports and notifications screens are out of frontend scope;
- README screenshot gallery points to 22 existing screenshots;
- CI/CD-style documentation filename parity is preserved;
- env/local setup/deployment docs use current `WIKI_*__*` variables and service names;
- migration docs direct new Wiki persistence work toward SQLx and quarantine inherited SeaORM task-tracker migrations;
- Wiki-owned domain value objects/invariants exist in `domain::wiki`;
- a fresh SQLx MVP schema baseline exists in `backend/migrations/202608310001_create_wiki_mvp.*.sql`;
- frontend MVP pages read from the public Wiki API; create document, edit/publish/archive/move document, create user, evidence and search flows call the same API;
- runtime API persistence stores users, sessions, spaces, documents, revisions, task/phase links, evidence, attachments, templates, audit and search in PostgreSQL when `WIKI_DATABASE__URL` is set;
- PostgreSQL runtime persistence is isolated in `api::routes::wiki::postgres`, while the main Wiki route module keeps router/DTO/memory fallback responsibilities;
- public registration is guarded by `WIKI_AUTH__REGISTRATION_ENABLED` in both memory fallback and PostgreSQL runtime;
- PostgreSQL runtime enforces the basic global-admin, space-role and attachment-download boundaries for core read/write paths;
- attachment bytes are behind `domain::wiki::WikiAttachmentStorage`, with `infra::LocalWikiAttachmentStorage` wired by `server`;
- shared Wiki normalization, access predicates, content helpers, storage-name helpers, password hashing, Wiki JWT/session token helpers and access/refresh token-pair TTL assembly are in `app::wiki`;
- the API crate no longer declares direct Wiki auth crypto dependencies after the helper extraction;
- CLI has mocked HTTP smoke coverage for auth, spaces, documents, task/phase dossiers, templates, search, URL/file evidence request flows and API error envelopes; compiled-binary smoke verifies non-zero exit for API errors;
- deferred areas are documented as reference only.

The remaining work is hardening and architecture cleanup, not product-scope expansion.

## 1. Backend Domain Migration

The public API/router is now a Wiki MVP runtime with memory test fallback and SQLx/PostgreSQL persistence. Move the pragmatic route-level persistence into Wiki-owned application use cases/repositories:

- `spaces` and `space_members`;
- `documents`, `document_drafts`, `document_revisions`;
- document tree and breadcrumbs;
- task links by external task key;
- phase links by workflow phase key;
- evidence items and attachments;
- templates;
- audit log;
- PostgreSQL full-text search for document title/body.

Remove or quarantine remaining inherited tracker concepts from backend internals:

- issues, boards, sprints and worklogs;
- watchers, votes, labels and issue links;
- custom fields, components and versions;
- reports and notifications runtime services.

Current status: runtime router, OpenAPI, API route files and default API tests are reduced to Wiki MVP; a Wiki domain baseline exists; SQLx runtime persistence is implemented as a transition adapter under `api::routes::wiki::postgres`; attachment bytes now use a dedicated storage port; shared Wiki validation/auth helpers live in the app layer; app/repository runtime wiring still needs replacement.

## 2. Database And Migrations

- Extend the clean SQLx baseline only through new SQLx migrations.
- Treat Wiki as a fresh schema; inherited tracker migrations are compatibility/quarantine only.
- Review indexes with `EXPLAIN` when repositories are implemented.
- Keep audit writes transactional with the command that caused them.
- Update `docs/MIGRATIONS.md` after the schema decision.

## 3. API And OpenAPI

- Extract current SQLx-backed API behavior into application use cases backed by Wiki repositories.
- Keep inherited tracker routes out of the runtime router.
- Regenerate `openapi/openapi.json` after any handler DTO/route change.
- Add generated frontend API types after OpenAPI is stable.
- Keep UI and CLI as ordinary clients of the same public API.

## 4. CLI Parity

- Keep `backend/cli/src/main.rs` aligned with `docs/CLI.md`.
- Expand CLI tests for command-specific validation failures as the command surface grows.
- Keep idempotency key coverage for repeated write commands as the command surface grows.

## 5. Frontend Integration

- Replace the remaining static settings/admin readiness values with API-backed data after settings endpoints are approved.
- Add broader permission denied and validation error coverage on API-backed pages.
- Extract the current page-level document editor, document tree, revision panel and evidence feed into reusable widgets/features as the UI hardens.
- Keep visible UI text Russian by default.
- Keep deferred integrations, reports and notifications pages out of MVP routes.

## 6. Search, Storage And Audit

- Tune current PostgreSQL FTS with ranking, query plans and language decisions.
- Expand current local filesystem storage coverage behind the dedicated Wiki storage port; add S3/MinIO later behind the same abstraction.
- Expand attachment tests beyond the current staged upload, claim, download and missing-file smoke for less common storage edge cases.
- Expand audit tests for document publish/archive, evidence writes, user changes and permission changes.

## 7. Tests And Release Readiness

- Add backend unit tests for domain invariants.
- Add repository/API tests beyond the current persistence, permission and file-evidence smoke for spaces, documents, revisions, task/phase links, evidence, attachments, search and audit.
- Rerun PostgreSQL-backed API smoke with `WIKI_TEST_DATABASE_URL` set, including persistence across router rebuilds and disabled public registration.
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

1. Extract SQLx route-level persistence into repository interfaces/implementations for identity, spaces and documents using `domain::wiki`.
2. Wire server composition to those repositories behind feature-compatible app use cases.
3. Add focused repository/API tests for document draft/publish/history, task/phase links, evidence and attachments.
4. Tune PostgreSQL FTS ranking/search filters and capture query-plan evidence for the expected MVP dataset size.
5. Bring CLI smoke tests to parity with the public API.
6. Remove remaining inherited tracker backend internals and SeaORM migration compatibility layer.
7. Generate frontend API client after the PostgreSQL-backed contract stabilizes.

## 10. Done Criteria For Backend Start

- `cargo check` and backend tests run on a host with MSVC Build Tools or another configured linker.
- Clean Wiki SQLx migrations create an empty database without tracker tables and include auth session storage.
- OpenAPI exposes only Wiki MVP endpoints.
- UI and CLI use the same public API operations.
- Postgres persistence smoke passes across router rebuilds.
- Non-member/viewer/editor/admin access boundaries are enforced by the PostgreSQL runtime.
- Static frontend data is limited to settings/admin readiness copy and deterministic test/screenshot fixtures.
