# Wiki MVP Remaining Work

## 0. Current Baseline

The documentation, screenshots, API-backed frontend MVP pages and SQLx-backed MVP API runtime are ready enough to continue hardening:

- MVP page set is fixed in routing, README and screenshot manifest;
- integrations, reports and notifications screens are out of frontend scope;
- README screenshot gallery points to 22 existing screenshots;
- CI/CD-style documentation filename parity is preserved;
- env/local setup/deployment docs use current `WIKI_*__*` variables and service names;
- migration docs and CI use canonical SQLx migrations through `backend/migrations` and the thin `backend/migration` SQLx runner;
- Wiki-owned domain value objects/invariants exist in `domain::wiki`;
- a fresh SQLx MVP schema baseline exists in `backend/migrations/202608310001_create_wiki_mvp.*.sql`;
- frontend MVP pages read from the public Wiki API; create document, edit/publish/archive/move document, create user, evidence, settings/admin overview and search flows call the same API;
- production server runtime stores users, sessions, spaces, documents, revisions, task/phase links, evidence, attachments, templates, audit and search in PostgreSQL and refuses to start without `WIKI_DATABASE__URL`;
- PostgreSQL runtime persistence is behind `shared::wiki_contract::WikiBackendPort`; public Wiki DTOs, `WikiSettingsSnapshot` and the port live in `shared::wiki_contract`; the concrete SQLx adapter is private in `infra::wiki_postgres`, with connection/bootstrap, SQL constants, row mapping and all operation slices separated into focused modules; auth/session/current-user, users/settings, spaces/members/tree, documents/revisions, task/phase dossiers, evidence/attachments, search, templates and pool-backed audit list/write have app-level use case/repository ports, while the main Wiki route module keeps HTTP/OpenAPI responsibilities and an explicit memory test/dev backend;
- API/server runtime uses `app::WikiAppContext` and no longer constructs the inherited task-tracker `AppContext`, repository bundle or report/notification/issue service graph;
- inherited task-tracker domain modules are excluded from the default `domain` crate build and quarantined behind feature `legacy-tracker`;
- inherited task-tracker app modules are excluded from the default `app` crate build and quarantined behind feature `legacy-tracker`;
- inherited task-tracker infra modules are excluded from the default `infra` crate build and quarantined behind feature `legacy-tracker`;
- public registration is guarded by `WIKI_AUTH__REGISTRATION_ENABLED` in both explicit memory test/dev backend and PostgreSQL runtime;
- PostgreSQL runtime enforces global-admin, space-role, archived-space write and attachment-download boundaries for core read/write paths; the explicit memory test/dev backend mirrors the same MVP boundaries for smoke coverage;
- archived documents are read-only for draft/publish/move write commands in both memory and SQLx-backed runtime paths;
- focused API regressions cover space membership delete/revocation semantics, latest-first immutable document revision history, search filters by document type/task/phase/archive state, search permission boundaries, published-revision search behavior, task/phase document link space boundaries, evidence document-space inference, document/space mismatch rejection, staged file evidence claim/download and reused attachment rejection; the space membership delete/revocation path also has an env-gated PostgreSQL API regression that must be run when `WIKI_TEST_DATABASE_URL` is available;
- attachment bytes are behind `domain::wiki::WikiAttachmentStorage`, with `infra::LocalWikiAttachmentStorage` wired by `server`;
- shared Wiki normalization, access predicates, content helpers, storage-name helpers, password hashing, Wiki JWT/session token helpers and access/refresh token-pair TTL assembly are in `app::wiki`; safe runtime settings snapshot is in `shared::wiki_contract`;
- auth/session flow validation, spaces/members/tree command validation, document create/draft/publish/archive/move command validation, task/phase dossier normalization/link command assembly, evidence/list/upload payload validation, user create/update validation and password hashing are in `app::wiki`; search q/filter/limit normalization and merge/sort/limit behavior are in `app::wiki`; template create validation/normalization and pool-backed audit command/list boundaries are in `app::wiki`; the PostgreSQL adapter owns SQL/storage details behind repository ports;
- the API crate no longer declares direct Wiki auth crypto dependencies or production SQLx adapter code after the helper and persistence-boundary extractions;
- CLI covers the MVP public API groups for auth, users, spaces/member management, documents/revisions, task/phase dossiers, evidence, attachments, templates, audit, search and settings; mocked HTTP smoke coverage verifies path encoding, read/write idempotency behavior, admin/lifecycle commands, URL/file evidence, attachment downloads and API error envelopes; compiled-binary smoke verifies non-zero exit for API errors, Markdown stdin input for `doc create --from-file -`, local missing-file fail-fast behavior before HTTP, env option handling and compact/table output formats;
- focused frontend component tests cover spaces tree preview/empty state, document editor/revision/linked dossier/evidence/archive read-only states and evidence registry/filter/URL/file submission;
- frontend API errors are formatted into safe Russian user-facing messages across MVP query/form states, with focused coverage for validation details, permission denied errors, retry actions and document-compose space-key normalization;
- domain unit tests cover the first Wiki-owned invariants for route-safe keys, required space/document names, revision publish payload, evidence payload shape and attachment metadata;
- deferred areas are documented as reference only.

The remaining work is hardening and architecture cleanup, not product-scope expansion.

## 1. Backend Domain Migration

The public API/router is now a Wiki MVP runtime with memory test fallback and SQLx/PostgreSQL persistence owned by `infra`. The transition SQLx adapter has been split into focused modules, and the MVP operation slices now have Wiki-owned application use cases/repository ports.

Keep the inherited tracker compatibility surface outside default builds, then remove it after Wiki repositories no longer need transitional scaffolding:

- issues, boards, sprints and worklogs;
- watchers, votes, labels and issue links;
- custom fields, components and versions;
- reports and notifications legacy modules outside default builds.

Current status: runtime router, OpenAPI, API route files and default API tests are reduced to Wiki MVP; a Wiki domain baseline exists; SQLx runtime persistence is implemented behind `shared::wiki_contract::WikiBackendPort` in `infra::wiki_postgres`; connection/bootstrap, SQL constants, row mapping and all operation slices are split into submodules. Auth/session/current-user, users/settings, spaces/members/tree, documents/revisions, task/phase dossiers, evidence/attachments, search, templates and pool-backed audit list/write have app-level use cases and repository ports. Production `server::run` is PostgreSQL-only and memory mode is explicit test/dev composition; shared Wiki validation/auth/users/settings/spaces/documents/dossiers/evidence/search/audit helpers and the Wiki runtime context live in the app layer; attachment bytes use a dedicated storage port; public Wiki DTOs/settings/port live in `shared::wiki_contract`; inherited task-tracker domain/app/infra modules are feature-gated as compatibility code. User create/update, auth register/login/logout, space/member writes, document draft/publish/archive/move writes, task/phase document link writes, evidence create and attachment upload writes are transactional in the repository adapter; the generic SQLx audit helper remains as a shared persistence helper for repository transactions.

## 2. Database And Migrations

- Extend the clean SQLx baseline only through new SQLx migrations.
- Treat Wiki as a fresh schema; do not reintroduce inherited tracker migrations.
- Review indexes with `EXPLAIN` when repositories are implemented.
- Keep audit writes transactional with the command that caused them.
- Update `docs/MIGRATIONS.md` after the schema decision.

## 3. API And OpenAPI

- Keep application use cases/repository ports as the API/infra boundary for all MVP operations.
- Keep inherited tracker routes out of the runtime router.
- Regenerate `openapi/openapi.json` after any handler DTO/route change.
- Keep generated frontend DTO types in sync with OpenAPI; replace handwritten endpoint wrappers with a generated operation client after the app/infra boundary stabilizes.
- Keep UI and CLI as ordinary clients of the same public API.

## 4. CLI Parity

Current status: the CLI now exposes the MVP public API command groups for humans, scripts and automation without a separate agent model.

- Keep `backend/cli/src/main.rs` aligned with `docs/CLI.md`.
- Keep CLI edge parity current when adding or changing public API commands.
- Keep idempotency key coverage for repeated write commands as the command surface grows.

## 5. Frontend Integration

- Keep permission denied and validation error coverage current when API-backed page states change.
- Keep the current spaces/document/evidence component tests aligned with visible MVP page states when the UI changes.
- Extract the current page-level document editor, document tree, revision panel and evidence feed into reusable widgets/features as the UI hardens.
- Keep visible UI text Russian by default.
- Keep deferred integrations, reports and notifications pages out of MVP routes.

## 6. Search, Storage And Audit

- Tune current PostgreSQL FTS with ranking, query plans and language decisions; q/filter/limit normalization and response merge/limit already live in `app::wiki`, so the remaining work is SQL/repository behavior.
- Expand current local filesystem storage coverage behind the dedicated Wiki storage port; add S3/MinIO later behind the same abstraction.
- Expand attachment tests beyond the current staged upload, claim, download, reuse rejection and missing-file smoke for less common storage edge cases.
- Keep audit writes inside the same transaction as the command that caused them.
- Continue expanding audit tests beyond the current memory smoke for document archive, user updates and PostgreSQL-backed permission changes.

## 7. Tests And Release Readiness

- Continue expanding backend unit tests for less common domain invariant combinations.
- Rerun PostgreSQL-backed API smoke with `WIKI_TEST_DATABASE_URL` set, including production backend construction, persistence across router rebuilds, disabled public registration and the env-gated space membership delete/revocation regression.
- Add repository/API tests beyond the current persistence, permission, audit, archived-document, space membership, revision/search, task/phase boundary and file-evidence smoke for the remaining PostgreSQL-backed permission edge-case combinations.
- Keep frontend component tests current for changed MVP page states and add permission/validation error cases.
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

1. Rerun fresh PostgreSQL smoke with `WIKI_TEST_DATABASE_URL`, including router rebuild persistence, disabled-registration coverage and the space membership delete/revocation regression.
2. Continue focused repository/API hardening for the remaining PostgreSQL-backed permission edge cases.
3. Tune PostgreSQL FTS ranking/search filters and capture query-plan evidence for the expected MVP dataset size.
4. Remove remaining inherited tracker compatibility modules.
5. Replace handwritten frontend endpoint wrappers with a generated operation client after the PostgreSQL-backed contract stabilizes.

## 10. Done Criteria For Backend Start

- `cargo check` and backend tests run on a host with MSVC Build Tools or another configured linker.
- Clean Wiki SQLx migrations create an empty database without tracker tables and include auth session storage.
- OpenAPI exposes only Wiki MVP endpoints.
- UI and CLI use the same public API operations.
- API/server runtime uses `app::WikiAppContext` instead of the inherited task-tracker `AppContext`.
- MVP operation slices use app-level use cases/repository ports before SQLx/storage details.
- Production server refuses to start without `WIKI_DATABASE__URL`; memory runtime is available only through the explicit test/dev builder.
- Route handlers do not depend on the concrete PostgreSQL implementation.
- Postgres persistence smoke passes across router rebuilds.
- Non-member/viewer/editor/admin access boundaries are enforced by the PostgreSQL runtime and mirrored by explicit memory smoke tests.
- Static frontend data is limited to deterministic test/screenshot fixtures.
