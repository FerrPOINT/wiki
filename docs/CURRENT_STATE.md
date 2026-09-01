# Current State - Wiki

> Snapshot date: 2026-09-01. Authority is repository code and tests; update this file whenever capability state changes.

## Current Verified

| Capability | Status | Notes |
| ---------- | ------ | ----- |
| Product scope | Current | `docs/PRODUCT_REQUIREMENTS.md` defines the base Wiki MVP: auth/users/roles, spaces, documents/tree/revisions, task/phase links, evidence, search, templates, audit, runtime health/readiness, API, UI and CLI. |
| Frontend page set | Current | The router, README gallery and screenshot manifest cover only the approved MVP routes: `/login`, `/register`, `/`, `/spaces`, `/documents/new`, `/documents/:documentId`, `/tasks`, `/tasks/:taskKey`, `/phases`, `/phases/:phaseId`, `/evidence`, `/templates`, `/audit-log`, `/users`, `/settings`, `/search`, `/admin`. |
| Removed non-MVP screens | Current | `/integrations`, `/reports` and `/notifications` are absent from frontend routing, navigation, screenshot generation, README gallery and OpenAPI mocks. |
| Removed copied tracker backend | Current | Copied task-tracker domain/app/infra modules for issues, boards, sprints, worklogs, reports, notifications, email, cache, old repositories and old ORM entities have been removed from the backend workspace. |
| API runtime | Current | Runtime router and OpenAPI expose Wiki MVP endpoints plus operational `/api/v1/health` and `/api/v1/health/ready`. Production server requires `WIKI_DATABASE__URL` and wires SQLx/PostgreSQL persistence through `shared::wiki_contract::WikiBackendPort`; memory backend remains an explicit test/dev composition. |
| Runtime composition | Current | API/server state uses `app::WikiAppContext` with runtime config only. Docker Compose and init/deploy scripts contain backend, frontend, PostgreSQL and an uploads volume; extra cache/worker/migrator services are not part of the base Wiki runtime. The real Wiki server does not construct task-tracker service graphs. |
| Test runtime composition | Current | Docker test compose files and coverage runners use only the disposable Wiki PostgreSQL service. Legacy task-tracker Redis and `TT_DB_PASS` coverage setup have been removed from the active dev/ops commands. |
| Domain baseline | Current | `domain::wiki` defines Wiki value objects, roles, documents, revisions, evidence, attachments and core invariants. Domain tests cover route-safe keys, required names/titles, revision checksum, evidence payload shape and attachment metadata. |
| Application layer | Current | `app::wiki` owns normalization, access predicates, auth/session helpers, user/space/document/dossier/evidence/search/template/audit use cases and repository ports. |
| PostgreSQL persistence | Current | `infra::wiki_postgres` owns SQLx connection/bootstrap, SQL constants, row mapping and repository implementations for users, auth sessions, spaces, members, documents, drafts, revisions, task/phase links, evidence, attachments, templates, audit and search. Published document revisions store canonical Markdown, searchable plain text and sanitized HTML rendered from Markdown. |
| Migrations | Current | `backend/migrations` contains the canonical SQLx migration files; `backend/migration` is a thin SQLx runner over that directory. The schema is a fresh Wiki MVP schema without task-tracker tables. |
| PostgreSQL smoke | Current | Env-gated tests with the `wiki_postgres_` prefix cover disabled registration, access/refresh token rotation and logout invalidation, viewer/editor/space-admin/global-admin permission boundaries, archived document and archived space write rejection including task/phase link commands, membership revocation, staged attachment reuse rejection, staged attachment owner mismatch rejection, persistence across backend/router rebuilds and FTS index-plan evidence. `scripts/postgres-smoke.ps1` runs them against Docker Postgres; `scripts/postgres-smoke-wsl.ps1` runs the same suite against an isolated temporary WSL PostgreSQL database. |
| Attachments | Current | Attachment uploads reject empty files, unsafe names, runtime size-limit violations, reuse after evidence claim and claims by a non-owner; bytes are behind `domain::wiki::WikiAttachmentStorage`, and server wires `infra::LocalWikiAttachmentStorage`, which rejects unsafe or platform-ambiguous storage keys. PostgreSQL claims staged attachments atomically in the evidence transaction. |
| CLI | Current | `wiki` CLI covers the MVP public API groups for auth, users, spaces/member management, documents/revisions, task/phase dossiers, evidence, attachments, templates, audit, search and settings. |
| Frontend API-backed pages | Current | Dashboard, spaces, documents, tasks, phases, evidence, templates, users, settings, admin overview, audit and search read from the public Wiki API; create/update/archive space, assign/remove space members, create/edit/publish/archive/move document, link existing documents to task/phase dossiers, create template, create/update user, URL evidence and file evidence forms, evidence deep-link detail, attachment metadata and attachment download call the same API. Document reading renders backend-sanitized `body_html` for published content while draft forms keep Markdown as the editing source. Document/evidence write hooks invalidate audit cache so `/audit-log` refreshes after user actions. Production create forms start empty except safe runtime defaults such as the default space key; example values live in placeholders, not submitted defaults. Production login fields are empty by default and no frontend route or screenshot advertises memory-test demo credentials. |
| Screenshot evidence | Current | 17 desktop and 5 mobile screenshots exist for the MVP page set; README renders the gallery inline and `docs/assets/screens/manifest.md` references the same files. Latest capture reflects empty production create forms. |
| Documentation set | Current | CI/CD-style documentation filename parity is preserved. User guide, test plan and threat model now explicitly cover the MVP page map, API/CLI parity, screenshot evidence and security controls without adding deferred task-tracker scope. |
| Pre-development readiness | Current | `docs/MVP_READINESS.md` defines the 100% gate for starting main development, with capability coverage, design freeze, API/CLI freeze, negative cases, developer handoff and go/no-go checklist. |

## Functional Coverage

- Auth: register/login/refresh/logout/current user, disabled registration policy, safe runtime settings, refresh rotation and logout invalidation for access/refresh token paths.
- Users and roles: global admin user list/create/update, space-admin member management and viewer/editor permission boundaries.
- Spaces: list/create/update/archive, member list/upsert/delete and page tree.
- Documents: create/get/draft/publish/archive/move, immutable revision detail and latest-first revision history; archived documents and documents inside archived spaces reject write commands.
- Task/phase dossiers: list/detail, linked documents and linked evidence by external keys; archived documents and archived spaces are rejected on link commands.
- Evidence and attachments: URL evidence, staged file upload, owner-bound file evidence claim, visible file checksum/metadata and authorized attachment download.
- Search: document/evidence search with MVP filters and permission boundaries.
- Search performance: document search uses PostgreSQL `tsvector`/GIN with title/body weighting and an env-gated `EXPLAIN` smoke for the filtered MVP query shape.
- Templates: list/create/client template selection flow using Markdown body from template data.
- Audit: append-only write action trail for core MVP commands.

## Verified Checks

- Latest backend WSL regression after readiness, auth session hardening, FTS query, attachment upload hardening, compose cleanup, shared tracker cleanup, sanitized revision HTML and public `body_html` API exposure: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace -- --test-threads=1`, `cargo clippy --workspace --all-targets -- -D warnings`.
- Focused PostgreSQL test group passes against an isolated temporary WSL PostgreSQL database through `pwsh -File scripts/postgres-smoke-wsl.ps1`; the same test group also compiles and safely skips without `WIKI_TEST_DATABASE_URL`. Latest run covered 9 `wiki_postgres_` tests including refresh rotation, logout invalidation for access/refresh tokens, viewer/editor/space-admin/global-admin permission boundaries, archived document and archived space draft/publish/move/archive/task-link/phase-link rejection plus staged attachment reuse and owner mismatch rejection.
- Published revision storage is covered by an env-gated PostgreSQL persistence smoke that verifies sanitized HTML generated from Markdown does not retain unsafe script content.
- Latest frontend regression after attachment metadata/download, search filter UI, task/phase document link UI, evidence deep-link detail, production form cleanup and audit cache invalidation: `npm run typecheck`, `npm run test`, `npm run lint`, `npm run format:check`, `npm run build`, `npm run test:e2e -- --project=chromium`.
- Docker PostgreSQL smoke runner syntax check passed; `pwsh -File scripts/postgres-smoke.ps1` currently stops with a clear Docker-daemon unavailable message on this host because `com.docker.service` cannot be started from this process.
- `docker compose config`, `docker compose -f backend/docker-compose.test.yml config` and `docker compose -f docker-compose.test.yml config` render the MVP service set without extra cache/worker services.
- Dev/ops scripts parse cleanly through `bash -n scripts/init.sh scripts/run-e2e-tests.sh scripts/deploy-production.sh scripts/deploy-staging.sh backend/scripts/run-e2e-tests.sh scripts/backup.sh scripts/restore.sh scripts/cleanup_old_backups.sh`; static search confirms no active Redis, `migrator` service or `TT_DB_PASS` dependency remains in Wiki init/test/deploy commands.
- Backup/restore scripts now use the current Docker Compose runtime: `postgres`, `backend`, `POSTGRES_*` with legacy `WIKI_DB_*` fallback, and the `uploads` volume through `WIKI_STORAGE__DIR`.
- Screenshot script passed against `vite preview`: `npm run shoot:evidence` captured 17 desktop and 5 mobile MVP screenshots, including an empty production login form without demo credentials.
- Static checks confirmed no active references to removed `/integrations`, `/reports` or `/notifications` routes, no unresolved task markers, no short Markdown docs, README/manifest screenshot refs with `missing=0`, API/PRD documentation parity with all 42 OpenAPI paths and CI/CD docs parity `missing_from_wiki=0`.

## Known Local Environment Limits

- Native Windows Rust linking currently requires MSVC `link.exe`; backend checks are run through WSL on this host.
- `pnpm add` is blocked on this host by Corepack/Node `ERR_VM_DYNAMIC_IMPORT_CALLBACK_MISSING`; existing package binaries under `frontend/node_modules/.bin` can still be used for TypeScript/tests/build/lint verification.
- Docker PostgreSQL smoke can be run through `scripts/postgres-smoke.ps1` once Docker Desktop is available. In the last setup check, Docker CLI was installed, but the Docker daemon/service was stopped and could not be started from this process; the test Postgres port `3458` was closed.

## Remaining Gaps

No known blockers remain for starting main Wiki development.

Production/release gates still remain:

- Run the Docker-backed PostgreSQL smoke on a host where Docker Desktop is available, in addition to the already passing WSL PostgreSQL smoke.
- Keep expanding repository/API coverage for less common permission edge cases after the DB smoke can run.
- Execute a backup/restore drill and target-host TLS/CORS/secrets review.
- Replace handwritten frontend endpoint wrappers with a generated operation client after the API contract stabilizes.
- Keep screenshots regenerated after any UI or route change.
