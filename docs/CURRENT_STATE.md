# Current State - Wiki

> Snapshot date: 2026-09-01. Authority is repository code and tests; update this file whenever capability state changes.

## Current Verified

| Capability | Status | Notes |
| ---------- | ------ | ----- |
| Product scope | Current | `docs/PRODUCT_REQUIREMENTS.md` defines the base Wiki MVP: auth/users/roles, spaces, documents/tree/revisions, task/phase links, evidence, search, templates, audit, API, UI and CLI. |
| Frontend page set | Current | The router, README gallery and screenshot manifest cover only the approved MVP routes: `/login`, `/register`, `/`, `/spaces`, `/documents/new`, `/documents/:documentId`, `/tasks`, `/tasks/:taskKey`, `/phases`, `/phases/:phaseId`, `/evidence`, `/templates`, `/audit-log`, `/users`, `/settings`, `/search`, `/admin`. |
| Removed non-MVP screens | Current | `/integrations`, `/reports` and `/notifications` are absent from frontend routing, navigation, screenshot generation, README gallery and OpenAPI mocks. |
| Removed copied tracker backend | Current | Copied task-tracker domain/app/infra modules for issues, boards, sprints, worklogs, reports, notifications, email, cache, old repositories and old ORM entities have been removed from the backend workspace. |
| API runtime | Current | Runtime router and OpenAPI expose Wiki MVP endpoints only. Production server requires `WIKI_DATABASE__URL` and wires SQLx/PostgreSQL persistence through `shared::wiki_contract::WikiBackendPort`; memory backend remains an explicit test/dev composition. |
| Runtime composition | Current | API/server state uses `app::WikiAppContext` with runtime config only. The real Wiki server does not construct task-tracker service graphs. |
| Domain baseline | Current | `domain::wiki` defines Wiki value objects, roles, documents, revisions, evidence, attachments and core invariants. Domain tests cover route-safe keys, required names/titles, revision checksum, evidence payload shape and attachment metadata. |
| Application layer | Current | `app::wiki` owns normalization, access predicates, auth/session helpers, user/space/document/dossier/evidence/search/template/audit use cases and repository ports. |
| PostgreSQL persistence | Current | `infra::wiki_postgres` owns SQLx connection/bootstrap, SQL constants, row mapping and repository implementations for users, auth sessions, spaces, members, documents, drafts, revisions, task/phase links, evidence, attachments, templates, audit and search. |
| Migrations | Current | `backend/migrations` contains the canonical SQLx migration files; `backend/migration` is a thin SQLx runner over that directory. The schema is a fresh Wiki MVP schema without task-tracker tables. |
| Attachments | Current | Attachment bytes are behind `domain::wiki::WikiAttachmentStorage`; server wires `infra::LocalWikiAttachmentStorage`, which rejects unsafe or platform-ambiguous storage keys. |
| CLI | Current | `wiki` CLI covers the MVP public API groups for auth, users, spaces/member management, documents/revisions, task/phase dossiers, evidence, attachments, templates, audit, search and settings. |
| Frontend API-backed pages | Current | Dashboard, spaces, documents, tasks, phases, evidence, templates, users, settings, admin overview, audit and search read from the public Wiki API; create/edit/publish/archive/move document, create user, URL evidence and file evidence forms call the same API. |
| Screenshot evidence | Current | 17 desktop and 5 mobile screenshots exist for the MVP page set; README and `docs/assets/screens/manifest.md` reference the same files. |
| Documentation set | Current | CI/CD-style documentation filename parity is preserved and docs describe the Wiki MVP rather than task-tracker behavior. |

## Functional Coverage

- Auth: register/login/refresh/logout/current user, disabled registration policy and safe runtime settings.
- Users and roles: admin user list/create/update plus space member role management.
- Spaces: list/create/update/archive, member list/upsert/delete and page tree.
- Documents: create/get/draft/publish/archive/move, immutable revisions and latest-first revision history.
- Task/phase dossiers: list/detail, linked documents and linked evidence by external keys.
- Evidence and attachments: URL evidence, staged file upload, file evidence claim, attachment metadata and download.
- Search: document/evidence search with MVP filters and permission boundaries.
- Templates: list/create/client template selection flow using Markdown body from template data.
- Audit: append-only write action trail for core MVP commands.

## Verified Checks

- Latest backend WSL compile check after removing copied tracker modules: `cargo check --workspace`.
- Previous full backend regression passed before this cleanup: `cargo fmt --all -- --check`, `cargo test --workspace -- --test-threads=1 --nocapture`, `cargo clippy --workspace --all-targets -- -D warnings`.
- Previous frontend regression passed before this cleanup: `npm run typecheck`, `npm run test`, `npm run lint`, `npm run format:check`, `npm run build`, `npm run test:e2e -- --project=chromium`, `npm run shoot:evidence`.
- Static checks previously confirmed no active references to removed `/integrations`, `/reports` or `/notifications` routes, no unresolved placeholder markers, no short Markdown docs and screenshot refs with `missing=0`.

## Known Local Environment Limits

- Native Windows Rust linking currently requires MSVC `link.exe`; backend checks are run through WSL on this host.
- `pnpm add` is blocked on this host by Corepack/Node `ERR_VM_DYNAMIC_IMPORT_CALLBACK_MISSING`; existing package binaries under `frontend/node_modules/.bin` can still be used for TypeScript/tests/build/lint verification.
- Fresh PostgreSQL API smoke requires an explicit disposable `WIKI_TEST_DATABASE_URL`. In the last setup check, local Docker PostgreSQL was not available from the current WSL distro.

## Remaining Gaps

- Run fresh PostgreSQL-backed API smoke with `WIKI_TEST_DATABASE_URL`: migrations, production backend construction, disabled registration, persistence across router rebuilds and membership revocation.
- Capture PostgreSQL FTS query-plan/ranking evidence for the expected MVP dataset size.
- Expand repository/API coverage for less common permission and attachment edge cases.
- Replace handwritten frontend endpoint wrappers with a generated operation client after the API contract stabilizes.
- Keep screenshots regenerated after any UI or route change.
