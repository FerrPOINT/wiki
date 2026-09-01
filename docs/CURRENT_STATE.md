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
| Runtime composition | Current | API/server state uses `app::WikiAppContext` with runtime config only. Docker Compose contains backend, frontend, PostgreSQL and an uploads volume; extra cache/worker services are not part of the base Wiki runtime. The real Wiki server does not construct task-tracker service graphs. |
| Domain baseline | Current | `domain::wiki` defines Wiki value objects, roles, documents, revisions, evidence, attachments and core invariants. Domain tests cover route-safe keys, required names/titles, revision checksum, evidence payload shape and attachment metadata. |
| Application layer | Current | `app::wiki` owns normalization, access predicates, auth/session helpers, user/space/document/dossier/evidence/search/template/audit use cases and repository ports. |
| PostgreSQL persistence | Current | `infra::wiki_postgres` owns SQLx connection/bootstrap, SQL constants, row mapping and repository implementations for users, auth sessions, spaces, members, documents, drafts, revisions, task/phase links, evidence, attachments, templates, audit and search. |
| Migrations | Current | `backend/migrations` contains the canonical SQLx migration files; `backend/migration` is a thin SQLx runner over that directory. The schema is a fresh Wiki MVP schema without task-tracker tables. |
| PostgreSQL smoke | Current | Env-gated tests with the `wiki_postgres_` prefix cover disabled registration, membership revocation, persistence across backend/router rebuilds and FTS index-plan evidence. `scripts/postgres-smoke.ps1` runs them against Docker Postgres; `scripts/postgres-smoke-wsl.ps1` runs the same suite against an isolated temporary WSL PostgreSQL database. |
| Attachments | Current | Attachment uploads reject empty files, unsafe names and runtime size-limit violations; bytes are behind `domain::wiki::WikiAttachmentStorage`, and server wires `infra::LocalWikiAttachmentStorage`, which rejects unsafe or platform-ambiguous storage keys. |
| CLI | Current | `wiki` CLI covers the MVP public API groups for auth, users, spaces/member management, documents/revisions, task/phase dossiers, evidence, attachments, templates, audit, search and settings. |
| Frontend API-backed pages | Current | Dashboard, spaces, documents, tasks, phases, evidence, templates, users, settings, admin overview, audit and search read from the public Wiki API; create/update/archive space, assign/remove space members, create/edit/publish/archive/move document, link existing documents to task/phase dossiers, create template, create/update user, URL evidence and file evidence forms, attachment metadata and attachment download call the same API. |
| Screenshot evidence | Current | 17 desktop and 5 mobile screenshots exist for the MVP page set; README renders the gallery inline and `docs/assets/screens/manifest.md` references the same files. |
| Documentation set | Current | CI/CD-style documentation filename parity is preserved. User guide, test plan and threat model now explicitly cover the MVP page map, API/CLI parity, screenshot evidence and security controls without adding deferred task-tracker scope. |
| Pre-development readiness | Current | `docs/MVP_READINESS.md` defines the 100% gate for starting main development, with capability coverage, design freeze, API/CLI freeze, negative cases, developer handoff and go/no-go checklist. |

## Functional Coverage

- Auth: register/login/refresh/logout/current user, disabled registration policy and safe runtime settings.
- Users and roles: admin user list/create/update plus space member role management.
- Spaces: list/create/update/archive, member list/upsert/delete and page tree.
- Documents: create/get/draft/publish/archive/move, immutable revision detail and latest-first revision history.
- Task/phase dossiers: list/detail, linked documents and linked evidence by external keys.
- Evidence and attachments: URL evidence, staged file upload, file evidence claim, visible file checksum/metadata and authorized attachment download.
- Search: document/evidence search with MVP filters and permission boundaries.
- Search performance: document search uses PostgreSQL `tsvector`/GIN with title/body weighting and an env-gated `EXPLAIN` smoke for the filtered MVP query shape.
- Templates: list/create/client template selection flow using Markdown body from template data.
- Audit: append-only write action trail for core MVP commands.

## Verified Checks

- Latest backend WSL regression after readiness, FTS query, attachment upload hardening, compose cleanup and shared tracker cleanup: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace -- --test-threads=1 --nocapture`, `cargo clippy --workspace --all-targets -- -D warnings`.
- Focused PostgreSQL test group passes against an isolated temporary WSL PostgreSQL database through `pwsh -File scripts/postgres-smoke-wsl.ps1`; the same test group also compiles and safely skips without `WIKI_TEST_DATABASE_URL`.
- Latest frontend regression after attachment metadata/download, search filter UI and task/phase document link UI: `npm run typecheck`, `npm run test`, `npm run lint`, `npm run format:check`, `npm run build`, `npm run test:e2e -- --project=chromium`.
- Docker PostgreSQL smoke runner syntax check passed; `pwsh -File scripts/postgres-smoke.ps1` currently stops with a clear Docker-daemon unavailable message on this host because `com.docker.service` cannot be started from this process.
- `docker compose config` and `docker compose -f backend/docker-compose.test.yml config` render the MVP service set without extra cache/worker services.
- Screenshot script passed against `vite preview`: `npm run shoot:evidence` captured 17 desktop and 5 mobile MVP screenshots.
- Static checks confirmed no active references to removed `/integrations`, `/reports` or `/notifications` routes, no unresolved task markers, no short Markdown docs, README/manifest screenshot refs with `missing=0`, API/PRD documentation parity with all 42 OpenAPI paths and CI/CD docs parity `missing_from_wiki=0`.

## Known Local Environment Limits

- Native Windows Rust linking currently requires MSVC `link.exe`; backend checks are run through WSL on this host.
- `pnpm add` is blocked on this host by Corepack/Node `ERR_VM_DYNAMIC_IMPORT_CALLBACK_MISSING`; existing package binaries under `frontend/node_modules/.bin` can still be used for TypeScript/tests/build/lint verification.
- Docker PostgreSQL smoke can be run through `scripts/postgres-smoke.ps1` once Docker Desktop is available. In the last setup check, Docker CLI was installed, but the Docker daemon/service was stopped and could not be started from this process; Postgres ports `3458` and `15432` were closed.

## Remaining Gaps

No known blockers remain for starting main Wiki development.

Production/release gates still remain:

- Run the Docker-backed PostgreSQL smoke on a host where Docker Desktop is available, in addition to the already passing WSL PostgreSQL smoke.
- Keep expanding repository/API coverage for less common permission edge cases after the DB smoke can run.
- Execute a backup/restore drill and target-host TLS/CORS/secrets review.
- Replace handwritten frontend endpoint wrappers with a generated operation client after the API contract stabilizes.
- Keep screenshots regenerated after any UI or route change.
