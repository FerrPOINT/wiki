# Current State - Wiki

> Snapshot date: 2026-08-31. Authority is repository code and tests; update this file whenever capabilities move from target to current.

## Current Verified

| Capability                 | Status  | Notes                                                                                                                                                                                                                                                                                                                                    |
| -------------------------- | ------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Repository scaffold        | Current | `task-tracker` code copied into `wiki`; `.git` from Wiki preserved                                                                                                                                                                                                                                                                       |
| Product requirements       | Current | `docs/PRODUCT_REQUIREMENTS.md` defines the reduced base Wiki scope                                                                                                                                                                                                                                                                       |
| Documentation set          | Current | CI/CD-style document set prepared for Wiki                                                                                                                                                                                                                                                                                               |
| CLI shape                  | Current | `wiki` CLI command surface drafted for public API operations; mocked HTTP smoke tests cover filtered search including `document_type`, document create and file-evidence upload/claim request flow                                                                                                                                       |
| API runtime                | Current | Runtime router and OpenAPI expose Wiki MVP endpoints only; memory fallback remains for fast tests, while real server config with `WIKI_DATABASE__URL` uses SQLx/PostgreSQL for MVP operations                                                                                                                                            |
| Registration policy        | Current | Public `/auth/register` is controlled by `WIKI_AUTH__REGISTRATION_ENABLED`; disabled registration returns `403` in memory and PostgreSQL runtime modes                                                                                                                                                                                   |
| Domain baseline            | Current | `domain::wiki` defines Wiki-owned value objects, roles, documents, revisions, evidence, attachments and core invariants                                                                                                                                                                                                                  |
| SQLx schema baseline       | Current | `backend/migrations/202608310001_create_wiki_mvp.*.sql` creates a fresh Wiki MVP schema without task-tracker tables; `202608310002_add_auth_runtime.*.sql` adds usernames and auth sessions                                                                                                                                              |
| SQLx runtime persistence   | Current | Users, auth sessions, spaces, members, documents, drafts, revisions, task/phase links, evidence, attachments, templates, audit and search are backed by PostgreSQL in `api::routes::wiki::postgres::PostgresWikiBackend`; the main `api::routes::wiki` module keeps router/DTO/memory fallback and delegates PostgreSQL runtime behavior |
| Attachment storage port    | Current | Attachment bytes are written/read through `domain::wiki::WikiAttachmentStorage`; server wires `infra::LocalWikiAttachmentStorage` for the PostgreSQL runtime, and local storage rejects unsafe or platform-ambiguous storage keys                                                                                                        |
| Wiki app helpers           | Current | Shared Wiki normalization, role/access predicates, Markdown text extraction, checksums, safe download names, password hashing, Wiki JWT/session token helpers and access/refresh token-pair TTL assembly live in `app::wiki` instead of private API route helpers; the API crate no longer declares direct Wiki auth crypto dependencies |
| Frontend route shell       | Current | Wiki MVP routes and screenshots exist for the approved page set only                                                                                                                                                                                                                                                                     |
| Frontend API-backed pages  | Current | Dashboard, spaces, documents, tasks, phases, evidence, templates, users, audit and search read from the public Wiki API; search sends the MVP `document_type` filter; create document, create user, URL evidence and file evidence forms call the same API                                                                               |
| Page design contract       | Current | `docs/PAGE_DESIGN.md` fixes page composition, states and deferred boundaries before backend work                                                                                                                                                                                                                                         |
| Refined MVP page design    | Current | Spaces, documents, tasks, phases, evidence and search pages include API-ready layouts and metadata blocks                                                                                                                                                                                                                                |
| Screenshot evidence        | Current | 17 desktop and 5 mobile screenshots regenerated for the MVP page set                                                                                                                                                                                                                                                                     |
| MVP documentation cleanup  | Current | Removed visible/technical integrations, reports and notifications scope from frontend routes, README gallery and screenshot manifest                                                                                                                                                                                                     |
| Development readiness docs | Current | README, local setup, env, migrations, storage, security, ops and runbooks are aligned with Wiki MVP/current-vs-target boundaries; host-side `cargo run` env is documented separately from Docker Compose `.env`                                                                                                                          |
| Frontend API shell         | Current | Thin handwritten auth and Wiki API client; old tracker generated client removed                                                                                                                                                                                                                                                          |
| Env/project identity       | Current | `WIKI_` prefix, docker names and frontend package identity                                                                                                                                                                                                                                                                               |

## Target MVP Product Surface

| Capability      | Target                                                                                                                |
| --------------- | --------------------------------------------------------------------------------------------------------------------- |
| Users and roles | login/logout, current user, admin/editor/viewer                                                                       |
| Spaces          | CRUD, members, permissions, archive                                                                                   |
| Documents       | Markdown draft, publish, revision history, archive                                                                    |
| Page tree       | parent/child, breadcrumbs, move within space                                                                          |
| Task links      | link documents/evidence by external task key                                                                          |
| Phase links     | link documents/evidence by phase key                                                                                  |
| Evidence        | `external_url` and `uploaded_file` evidence, checksum, lists by owner                                                 |
| Attachments     | local storage metadata and download                                                                                   |
| Search          | PostgreSQL FTS over published document title/body with basic filters; evidence metadata stays bounded metadata search |
| Templates       | requirements, research note, implementation note, test plan, release note                                             |
| Audit           | write actions and access/role changes                                                                                 |
| API/UI/CLI      | same MVP operations through public `/api/v1`                                                                          |

## Explicitly Deferred

- Comments, mentions and inline reviews.
- Advanced reports.
- Approval chains.
- Import/export bundles.
- Real-time collaborative editing.
- OCR and binary attachment indexing.

## Inherited To Replace

- Backend app/infra and old SeaORM migration crate still include task-tracker modules outside the active Wiki API shell.
- SQLx persistence currently lives in the API route subtree as a pragmatic MVP runtime adapter; it has been split out of the main router/DTO file into `api::routes::wiki::postgres`, but extracting dedicated app use cases/repositories is still target architecture work. Attachment bytes already cross a Wiki-specific storage port instead of direct route-level filesystem access, shared Wiki validation/auth helpers have moved into the application layer, and the API crate no longer depends directly on Argon2/JWT/SHA hashing crates for Wiki auth.
- Per-space permission checks are implemented in the PostgreSQL runtime for core read/write paths; staged attachment, claimed file evidence, attachment-download access, missing-file handling and FTS search semantics are covered by the PostgreSQL API smoke. Deeper repository/API coverage is still needed for less common edge-case combinations.
- Public registration enable/disable behavior is implemented through shared auth config and covered by the API memory smoke; PostgreSQL mode uses the same config gate before creating a user.
- Generated Wiki frontend OpenAPI client is pending backend domain/repository stabilization.

## Verification Commands

```bash
cd backend
cargo fmt --all -- --check
cargo test -p shared
cargo test -p domain
cargo test -p app wiki_helpers
cargo test -p app wiki_auth
cargo test -p infra local_wiki_attachment_storage
cargo test -p wiki-cli
cargo check -p api
cargo check -p wiki-cli

cd frontend
.\node_modules\.bin\tsc.cmd --noEmit
.\node_modules\.bin\eslint.cmd . --max-warnings=0
.\node_modules\.bin\prettier.cmd --check .
.\node_modules\.bin\vitest.cmd run
.\node_modules\.bin\vite.cmd build
.\node_modules\.bin\playwright.cmd test --project=chromium
node scripts/shoot-evidence.mjs
```

Latest verification on 2026-08-31:

- `docker run ... cargo check --workspace` passed on Linux container; direct Windows linking remains blocked by missing MSVC `link.exe`.
- `wsl bash -lc 'cargo check --workspace'` passed on Linux WSL.
- `docker run ... cargo check -p api` and `cargo check -p wiki-cli` passed on Linux container.
- `wsl bash -lc 'cargo check -p api -p server -p infra'` passed after introducing the Wiki attachment storage port and server-side infra wiring.
- `wsl bash -lc 'cargo test -p infra local_wiki_attachment_storage -- --nocapture'` passed: local Wiki attachment storage round-trip plus unsafe and platform-ambiguous key rejection.
- `wsl bash -lc 'cargo check --workspace'` and `wsl bash -lc 'cargo test --workspace -- --test-threads=1 --nocapture'` passed after storage-key hardening and the `app::wiki` helper extraction; API PostgreSQL smoke still skips when `WIKI_TEST_DATABASE_URL` is unset.
- `cargo tree -p api -e normal --depth 1` confirms active API runtime has no normal dependency on `infra`; `server` wires `infra::LocalWikiAttachmentStorage`.
- `wsl bash -lc 'cargo test -p app wiki_helpers -- --nocapture'` passed after moving shared Wiki helpers from API routes to `app::wiki`.
- `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p app wiki_auth -- --nocapture'` passed after moving Wiki password hashing, token hashing, JWT helpers and access/refresh token-pair TTL assembly from API routes to `app::wiki`.
- `wsl bash -lc 'cargo check --manifest-path backend/Cargo.toml -p app -p api -p server'` passed after the Wiki auth/session helper extraction.
- `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p api -- --test-threads=1 --nocapture'` passed after the Wiki auth/session helper extraction; API PostgreSQL smoke still skips when `WIKI_TEST_DATABASE_URL` is unset in WSL.
- `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml --workspace -- --test-threads=1 --nocapture'` passed after the Wiki auth/session helper extraction; API PostgreSQL smoke still skips when `WIKI_TEST_DATABASE_URL` is unset in WSL and infra Docker repository tests remain ignored.
- `wsl bash -lc 'cargo check --manifest-path backend/Cargo.toml -p api'` passed after removing stale direct `argon2`, `jsonwebtoken`, `sha2`, `hex` and `rand_core` dependencies from the API crate.
- `wsl bash -lc 'cargo check --manifest-path backend/Cargo.toml -p app -p api -p server'` passed after splitting the PostgreSQL Wiki runtime into `api::routes::wiki::postgres`.
- `wsl bash -lc 'cargo fmt --manifest-path backend/Cargo.toml --all -- --check'`, `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p api -- --test-threads=1 --nocapture'`, `wsl bash -lc 'cargo clippy --manifest-path backend/Cargo.toml -p app -p api --all-targets -- -D warnings'`, host `npm run typecheck`, `npm run test`, `npm run lint`, `npm run format:check`, `npm run build` and `npm run shoot:evidence` passed after the PostgreSQL module split; WSL PostgreSQL smoke still skips without `WIKI_TEST_DATABASE_URL`.
- `wsl bash -lc 'cargo fmt --manifest-path backend/Cargo.toml --all -- --check'`, `wsl bash -lc 'cargo check --manifest-path backend/Cargo.toml -p shared -p app -p api -p server'`, `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p shared -- --nocapture'`, `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p app wiki_ -- --nocapture'`, `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p api -- --test-threads=1 --nocapture'`, `wsl bash -lc 'cargo clippy --manifest-path backend/Cargo.toml -p shared -p app -p api --all-targets -- -D warnings'`, `wsl bash -lc 'cargo run --manifest-path backend/Cargo.toml -p api --bin openapi-gen -- openapi/openapi.json'`, host `npm run typecheck`, `npm run test`, `npm run lint`, `npm run format:check` and `npm run build` passed after adding the `WIKI_AUTH__REGISTRATION_ENABLED` gate; WSL PostgreSQL smoke still skips without `WIKI_TEST_DATABASE_URL`.
- `wsl bash -lc 'cargo fmt --manifest-path backend/Cargo.toml --all -- --check'`, `wsl bash -lc 'cargo check --manifest-path backend/Cargo.toml -p shared -p app -p api -p server'`, `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p api -- --test-threads=1 --nocapture'` and `wsl bash -lc 'cargo clippy --manifest-path backend/Cargo.toml -p app -p api --all-targets -- -D warnings'` passed after adding an explicit PostgreSQL-mode API test for disabled public registration. The new test is present but skips in WSL when `WIKI_TEST_DATABASE_URL` is unset.
- Attempted to start the dedicated `backend/docker-compose.test.yml` PostgreSQL service on `127.0.0.1:3458`, but host Docker CLI calls (`docker compose`, `docker ps`) did not return status and were interrupted. `Test-NetConnection 127.0.0.1 -Port 3458` reported the test DB port closed, so PostgreSQL-backed registration smoke remains blocked by the local Docker environment in this run.
- `wsl bash -lc 'cargo fmt --manifest-path backend/Cargo.toml --all -- --check'`, `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p wiki-cli -- --nocapture'`, `wsl bash -lc 'cargo clippy --manifest-path backend/Cargo.toml -p wiki-cli --all-targets -- -D warnings'`, host `npm run typecheck`, `npm run test`, `npm run build`, `npm run lint`, `npm run format:check`, `npx playwright test --project=chromium` and `npm run shoot:evidence` passed after wiring the MVP `document_type` search filter through CLI and UI. Screenshot reference and dimension checks passed for 22 PNGs.
- `wsl bash -lc 'cargo tree --manifest-path backend/Cargo.toml -p api -e normal --depth 1'` confirms the API crate has no direct `argon2`, `jsonwebtoken`, `sha2`, `hex`, `rand_core` or `infra` normal dependency.
- `wsl bash -lc 'cargo clippy -p app -p api --all-targets -- -D warnings'` passed after cleaning the helper extraction and multipart upload shape.
- `wsl bash -lc 'cargo clippy --manifest-path backend/Cargo.toml -p app -p api --all-targets -- -D warnings'` passed after the Wiki auth/session helper extraction.
- `wsl bash -lc 'cargo fmt --manifest-path backend/Cargo.toml --all -- --check'`, `wsl bash -lc 'cargo check --manifest-path backend/Cargo.toml -p app -p api -p server'`, `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p api -- --test-threads=1 --nocapture'` and `wsl bash -lc 'cargo clippy --manifest-path backend/Cargo.toml -p app -p api --all-targets -- -D warnings'` passed after the API dependency cleanup.
- `wsl bash -lc 'cargo test -p wiki-cli -- --nocapture'` passed: 3 mocked HTTP smoke tests cover filtered search, document create and file-evidence upload/claim request flow.
- `wsl bash -lc 'cargo test -p api -- --test-threads=1 --nocapture'` passed: memory MVP contract green; PostgreSQL persistence smoke skipped because `WIKI_TEST_DATABASE_URL` was not set in WSL.
- `wsl bash -lc 'cargo test --workspace -- --test-threads=1 --nocapture'` passed: workspace unit/integration suite green; DB-dependent infra repository tests remain ignored and API PostgreSQL smoke is skipped without `WIKI_TEST_DATABASE_URL`.
- `docker run ... cargo test -p api -- --test-threads=1 --nocapture` passed: memory MVP contract plus PostgreSQL persistence, space-permission, file-evidence/download, missing-file and FTS search smoke.
- `docker run ... cargo test -p shared`, `cargo test -p domain`, `cargo test -p app` and `cargo test -p server` passed on Linux container.
- `docker run ... cargo test -p api wiki_postgres_routes_persist_across_router_rebuilds -- --test-threads=1 --nocapture` passed against explicit `wiki-test` compose PostgreSQL on `host.docker.internal:3458`, including FTS multi-token positive and substring-negative checks.
- `docker run ... cargo run -p api --bin openapi-gen -- /workspace/openapi/openapi.json` regenerated OpenAPI successfully.
- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` is blocked on this Windows host by missing MSVC `link.exe`; the checked Linux image also has no bundled `cargo-clippy`/`rustup` component manager.
- `cargo metadata --no-deps --format-version 1` passed; `openapi-gen` is the single OpenAPI generator binary.
- Host-side Rust linking for `cargo check` / `cargo test` is blocked before project code by missing Windows MSVC `link.exe` / Windows SDK libs; use Linux Docker for backend checks until the host toolchain is fixed.
- `tsc --noEmit` passed.
- `eslint . --max-warnings=0` passed.
- `prettier --check .` passed after mechanical frontend formatting cleanup.
- `vitest run` passed: 5 files, 17 tests.
- `vite build` passed.
- Host `npm run typecheck`, `npm run test`, `npm run lint`, `npm run format:check`, `npm run build` passed after the API dependency cleanup; `pnpm` itself remains blocked by the local Corepack error noted below.
- `playwright test --project=chromium` passed: 1 smoke test.
- Screenshot script regenerated 22 screenshots against `vite preview`; capture wait is 1 second after navigation.
- Screenshot dimensions passed: 17 desktop screenshots at `1920x1080`, 5 mobile full-page screenshots at `375px` width.
- Direct `@eslint/js` dev dependency is declared because `eslint.config.js` imports it.
- README/manifest screenshot references resolve to existing PNG files.
- README/manifest screenshot reference check passed: 22 unique PNGs, `missing=0`, `extra=0`.
- Screenshot manifest dimensions match actual PNG dimensions.
- OpenAPI path parity passed: 40 expected Wiki MVP paths, `missing=0`, `extra=0`, legacy paths `0`.
- `docs/API.md`, `docs/PRODUCT_REQUIREMENTS.md` and `openapi/openapi.json` path parity passed.
- Evidence vocabulary check passed: active API/CLI/OpenAPI defaults use `external_url` / `uploaded_file`; invalid legacy `manual_check` is covered by a negative API test.
- Evidence payload shape is aligned between runtime validation, `domain::wiki` and SQLx schema: `external_url` uses URL only; `uploaded_file` uses `attachment_id` only.
- SQLx migration baseline smoke passed on local `postgres:17.6-alpine`: `up` creates the fresh Wiki schema without task-tracker tables, and `down` leaves the public schema empty.
- Traceability coverage passed: 28 PRD requirement IDs, `missing=0`, `extra=0`.
- CI/CD docs filename parity passed with `missing=0`.
- Markdown documentation checks passed: no placeholder markers, no Markdown document under 20 non-empty lines.
- Shell script syntax check passed: `bash -n scripts/*.sh` for operational helper scripts.
- Local setup/deployment docs distinguish Docker Compose `.env` from process env used by host-side `cargo run`.
- PostgreSQL schema check passed after runtime migrations: legacy tracker tables `0`, `users.username` present, `auth_sessions` present.
- CI/CD docs filename parity passed with `missing=0` and Wiki-specific extras allowed.
- Active route/API cleanup check passed: no `/integrations`, `/reports`, `/notifications` routes or old task-tracker API groups in active frontend/API/server/CLI/OpenAPI; only package name `@sentry/integrations` remains in `pnpm-lock.yaml`.

## Known Local Environment Limits

- MSVC `link.exe` is required for full Rust linking on the current Windows host.
- `pnpm add` is blocked on this host by Corepack/Node `ERR_VM_DYNAMIC_IMPORT_CALLBACK_MISSING`; package/lock changes were reviewed manually when needed.
- If `pnpm` blocks, direct package binaries under `frontend/node_modules/.bin` can still be used for TypeScript/tests/build/lint verification.
- In the latest run, host Docker CLI did not return for `docker compose`/`docker ps`, so fresh PostgreSQL integration smoke should be rerun after Docker Desktop/daemon responsiveness is restored.
