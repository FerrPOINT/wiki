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
- frontend MVP pages read from the public Wiki API; create document, edit/publish/archive/move document, create user, evidence, settings/admin overview and search flows call the same API;
- production server runtime stores users, sessions, spaces, documents, revisions, task/phase links, evidence, attachments, templates, audit and search in PostgreSQL and refuses to start without `WIKI_DATABASE__URL`;
- PostgreSQL runtime persistence is behind `shared::wiki_contract::WikiBackendPort`; public Wiki DTOs, `WikiSettingsSnapshot` and the port live in `shared::wiki_contract`; the concrete SQLx adapter is private in `infra::wiki_postgres`, with connection/bootstrap, SQL constants, row mapping, identity/auth/users/settings and spaces/space_members/tree operations separated from the remaining operation methods, while the main Wiki route module keeps HTTP/OpenAPI responsibilities and an explicit memory test/dev backend;
- API/server runtime uses `app::WikiAppContext` and no longer constructs the inherited task-tracker `AppContext`, repository bundle or report/notification/issue service graph;
- inherited task-tracker domain modules are excluded from the default `domain` crate build and quarantined behind feature `legacy-tracker`;
- inherited task-tracker app modules are excluded from the default `app` crate build and quarantined behind feature `legacy-tracker`;
- inherited task-tracker infra modules are excluded from the default `infra` crate build and quarantined behind feature `legacy-tracker`;
- public registration is guarded by `WIKI_AUTH__REGISTRATION_ENABLED` in both explicit memory test/dev backend and PostgreSQL runtime;
- PostgreSQL runtime enforces the basic global-admin, space-role and attachment-download boundaries for core read/write paths;
- attachment bytes are behind `domain::wiki::WikiAttachmentStorage`, with `infra::LocalWikiAttachmentStorage` wired by `server`;
- shared Wiki normalization, access predicates, content helpers, storage-name helpers, password hashing, Wiki JWT/session token helpers and access/refresh token-pair TTL assembly are in `app::wiki`; safe runtime settings snapshot is in `shared::wiki_contract`;
- search q/filter/limit normalization is in `app::wiki`; the PostgreSQL adapter still owns the SQL query and ranking details;
- the API crate no longer declares direct Wiki auth crypto dependencies or production SQLx adapter code after the helper and persistence-boundary extractions;
- CLI has mocked HTTP smoke coverage for auth, spaces, documents, task/phase dossiers, templates, settings, search, URL/file evidence request flows and API error envelopes; compiled-binary smoke verifies non-zero exit for API errors;
- deferred areas are documented as reference only.

The remaining work is hardening and architecture cleanup, not product-scope expansion.

## 1. Backend Domain Migration

The public API/router is now a Wiki MVP runtime with memory test fallback and SQLx/PostgreSQL persistence owned by `infra`. Identity/auth/users/settings and spaces/space_members/tree have been isolated into transition adapter modules; split the remaining operation methods into Wiki-owned application use cases/repositories:

- `documents`, `document_drafts`, `document_revisions`;
- document tree and breadcrumbs;
- task links by external task key;
- phase links by workflow phase key;
- evidence items and attachments;
- templates;
- audit log;
- PostgreSQL full-text search for document title/body.

Keep the inherited tracker compatibility surface outside default builds, then remove it after Wiki repositories no longer need transitional scaffolding:

- issues, boards, sprints and worklogs;
- watchers, votes, labels and issue links;
- custom fields, components and versions;
- reports and notifications legacy modules outside default builds.

Current status: runtime router, OpenAPI, API route files and default API tests are reduced to Wiki MVP; a Wiki domain baseline exists; SQLx runtime persistence is implemented as a transition adapter behind `shared::wiki_contract::WikiBackendPort` in `infra::wiki_postgres`; connection/bootstrap, SQL constants, row mapping, identity/auth/users/settings and spaces/space_members/tree operations are split into submodules, while the remaining operations still need to move into dedicated app use cases and infra repositories. Production `server::run` is PostgreSQL-only and memory mode is explicit test/dev composition; attachment bytes now use a dedicated storage port; shared Wiki validation/auth helpers and the Wiki runtime context live in the app layer; public Wiki DTOs/settings/port live in `shared::wiki_contract`; inherited task-tracker domain/app/infra modules are feature-gated as compatibility code.

## 2. Database And Migrations

- Extend the clean SQLx baseline only through new SQLx migrations.
- Treat Wiki as a fresh schema; inherited tracker migrations are compatibility/quarantine only.
- Review indexes with `EXPLAIN` when repositories are implemented.
- Keep audit writes transactional with the command that caused them.
- Update `docs/MIGRATIONS.md` after the schema decision.

## 3. API And OpenAPI

- Split the remaining non-identity `infra::wiki_postgres::mod` operation methods into application use cases backed by Wiki repositories.
- Keep inherited tracker routes out of the runtime router.
- Regenerate `openapi/openapi.json` after any handler DTO/route change.
- Keep generated frontend DTO types in sync with OpenAPI; replace handwritten endpoint wrappers with a generated operation client after the app/infra boundary stabilizes.
- Keep UI and CLI as ordinary clients of the same public API.

## 4. CLI Parity

- Keep `backend/cli/src/main.rs` aligned with `docs/CLI.md`.
- Expand CLI tests for command-specific validation failures as the command surface grows.
- Keep idempotency key coverage for repeated write commands as the command surface grows.

## 5. Frontend Integration

- Add broader permission denied and validation error coverage on API-backed pages.
- Extract the current page-level document editor, document tree, revision panel and evidence feed into reusable widgets/features as the UI hardens.
- Keep visible UI text Russian by default.
- Keep deferred integrations, reports and notifications pages out of MVP routes.

## 6. Search, Storage And Audit

- Tune current PostgreSQL FTS with ranking, query plans and language decisions; q/filter/limit normalization already lives in `app::wiki`, so the remaining work is SQL/repository behavior.
- Expand current local filesystem storage coverage behind the dedicated Wiki storage port; add S3/MinIO later behind the same abstraction.
- Expand attachment tests beyond the current staged upload, claim, download and missing-file smoke for less common storage edge cases.
- Expand audit tests for document publish/archive, evidence writes, user changes and permission changes.

## 7. Tests And Release Readiness

- Add backend unit tests for domain invariants.
- Add repository/API tests beyond the current persistence, permission and file-evidence smoke for spaces, documents, revisions, task/phase links, evidence, attachments, search and audit.
- Rerun PostgreSQL-backed API smoke with `WIKI_TEST_DATABASE_URL` set, including production backend construction, persistence across router rebuilds and disabled public registration.
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

1. Extract document, task/phase, evidence, template, audit and search operation methods from `infra::wiki_postgres::mod` into app/infra repository interfaces/implementations using `domain::wiki`; the identity/auth/users/settings and spaces/space_members/tree transition modules are already separated.
2. Introduce Wiki repository traits/use cases and wire them behind the existing shared API contract.
3. Add focused repository/API tests for document draft/publish/history, task/phase links, evidence and attachments.
4. Tune PostgreSQL FTS ranking/search filters and capture query-plan evidence for the expected MVP dataset size.
5. Bring CLI smoke tests to parity with the public API.
6. Remove remaining inherited tracker compatibility modules and the SeaORM migration compatibility layer.
7. Replace handwritten frontend endpoint wrappers with a generated operation client after the PostgreSQL-backed contract stabilizes.

## 10. Done Criteria For Backend Start

- `cargo check` and backend tests run on a host with MSVC Build Tools or another configured linker.
- Clean Wiki SQLx migrations create an empty database without tracker tables and include auth session storage.
- OpenAPI exposes only Wiki MVP endpoints.
- UI and CLI use the same public API operations.
- API/server runtime uses `app::WikiAppContext` instead of the inherited task-tracker `AppContext`.
- Production server refuses to start without `WIKI_DATABASE__URL`; memory runtime is available only through the explicit test/dev builder.
- Route handlers do not depend on the concrete PostgreSQL implementation.
- Postgres persistence smoke passes across router rebuilds.
- Non-member/viewer/editor/admin access boundaries are enforced by the PostgreSQL runtime.
- Static frontend data is limited to deterministic test/screenshot fixtures.
