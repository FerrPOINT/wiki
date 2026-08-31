# Current State - Wiki

> Snapshot date: 2026-08-31. Authority is repository code and tests; update this file whenever capabilities move from target to current.

## Current Verified

| Capability                 | Status  | Notes                                                                                                                                                                                                                                                                                                                                                                                                             |
| -------------------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Repository scaffold        | Current | `task-tracker` code copied into `wiki`; `.git` from Wiki preserved                                                                                                                                                                                                                                                                                                                                                |
| Product requirements       | Current | `docs/PRODUCT_REQUIREMENTS.md` defines the reduced base Wiki scope                                                                                                                                                                                                                                                                                                                                                |
| Documentation set          | Current | CI/CD-style document set prepared for Wiki                                                                                                                                                                                                                                                                                                                                                                        |
| CLI shape                  | Current | `wiki` CLI command surface drafted for public API operations; mocked HTTP smoke tests cover auth, spaces, documents, task/phase dossiers, templates, settings, search, URL/file evidence request flows and simple/structured API error envelopes, plus one compiled-binary non-zero exit test                                                                                                                     |
| API runtime                | Current | Runtime router and OpenAPI expose Wiki MVP endpoints only, including admin-only read-only settings snapshot; errors use the shared structured envelope with `error.code` and `error.message`; memory backend remains explicit for fast tests/dev router composition, while production server runtime requires `WIKI_DATABASE__URL` and uses SQLx/PostgreSQL through internal `WikiBackendPort` for MVP operations |
| Wiki runtime composition   | Current | API/server state now uses `app::WikiAppContext` with runtime config only; the real Wiki server no longer constructs the inherited task-tracker `AppContext`, repository bundle or report/notification/issue service graph; `server::run` refuses to start without PostgreSQL configuration                                                                                                                        |
| App legacy quarantine      | Current | The default `app` crate build exports `app::wiki` only; inherited task-tracker `auth/authz/commands/context/dto/services` modules and their `tokio`/`async-trait`/`serde_json` deps are available only through feature `legacy-tracker`                                                                                                                                                                           |
| Registration policy        | Current | Public `/auth/register` is controlled by `WIKI_AUTH__REGISTRATION_ENABLED`; disabled registration returns `403` in memory and PostgreSQL runtime modes                                                                                                                                                                                                                                                            |
| Domain baseline            | Current | `domain::wiki` defines Wiki-owned value objects, roles, documents, revisions, evidence, attachments and core invariants                                                                                                                                                                                                                                                                                           |
| SQLx schema baseline       | Current | `backend/migrations/202608310001_create_wiki_mvp.*.sql` creates a fresh Wiki MVP schema without task-tracker tables; `202608310002_add_auth_runtime.*.sql` adds usernames and auth sessions                                                                                                                                                                                                                       |
| SQLx runtime persistence   | Current | Users, auth sessions, spaces, members, documents, drafts, revisions, task/phase links, evidence, attachments, templates, audit and search are backed by PostgreSQL through internal `WikiBackendPort`; concrete `PostgresWikiBackend` is private inside `api::routes::wiki::postgres`, while the main `api::routes::wiki` module keeps router/DTO plus explicit memory test/dev backend responsibilities          |
| Wiki backend port          | Current | The main route module stores `Arc<dyn WikiBackendPort>` for persistent behavior internally; memory remains the explicit fast test/dev backend, and `postgres::connect_persistent_backend` wires the private SQLx adapter for production runtime                                                                                                                                                                   |
| Attachment storage port    | Current | Attachment bytes are written/read through `domain::wiki::WikiAttachmentStorage`; server wires `infra::LocalWikiAttachmentStorage` for the PostgreSQL runtime, and local storage rejects unsafe or platform-ambiguous storage keys                                                                                                                                                                                 |
| Wiki app helpers           | Current | Shared Wiki normalization, role/access predicates, search criteria normalization, Markdown text extraction, checksums, safe download names, password hashing, Wiki JWT/session token helpers and access/refresh token-pair TTL assembly live in `app::wiki` instead of private API route helpers; the API crate no longer declares direct Wiki auth crypto dependencies                                           |
| Frontend route shell       | Current | Wiki MVP routes and screenshots exist for the approved page set only                                                                                                                                                                                                                                                                                                                                              |
| Frontend API-backed pages  | Current | Dashboard, spaces, documents, tasks, phases, evidence, templates, users, settings, admin overview, audit and search read from the public Wiki API; search sends the MVP `document_type` filter; create document, edit/publish/archive/move document, create user, URL evidence and file evidence forms call the same API; `/evidence` exposes document/task/phase owner filters                                   |
| Page design contract       | Current | `docs/PAGE_DESIGN.md` fixes page composition, states and deferred boundaries before backend work                                                                                                                                                                                                                                                                                                                  |
| Refined MVP page design    | Current | Spaces, documents, tasks, phases, evidence and search pages include API-ready layouts and metadata blocks                                                                                                                                                                                                                                                                                                         |
| Screenshot evidence        | Current | 17 desktop and 5 mobile screenshots regenerated for the MVP page set; document screenshots show draft/publish/move/archive controls; `/evidence` screenshot shows document owner field and document/task/phase filters                                                                                                                                                                                            |
| MVP documentation cleanup  | Current | Removed visible/technical integrations, reports and notifications scope from frontend routes, README gallery and screenshot manifest                                                                                                                                                                                                                                                                              |
| Development readiness docs | Current | README, local setup, env, migrations, storage, security, ops and runbooks are aligned with Wiki MVP/current-vs-target boundaries; host-side `cargo run` env is documented separately from Docker Compose `.env`                                                                                                                                                                                                   |
| Frontend API shell         | Current | Generated OpenAPI DTO types live in `frontend/src/api/generated.ts`; thin handwritten auth/Wiki endpoint wrappers use those types; old tracker generated client removed; non-2xx responses throw typed `ApiError` with status, code, optional requestId and field details                                                                                                                                         |
| Env/project identity       | Current | `WIKI_` prefix, docker names and frontend package identity                                                                                                                                                                                                                                                                                                                                                        |

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
| Settings        | admin-only safe runtime snapshot for UI/CLI                                                                           |
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

- Backend infra/domain internals and the old SeaORM migration crate still include task-tracker modules outside the active Wiki API/server runtime path; inherited `app` modules are quarantined behind feature `legacy-tracker`.
- SQLx persistence currently lives in the API route subtree as a pragmatic MVP runtime adapter; route handlers already call the explicit internal `WikiBackendPort` and the concrete PostgreSQL adapter is private inside `api::routes::wiki::postgres`, but extracting dedicated app use cases/repositories is still target architecture work. Production server composition is now PostgreSQL-only, while memory mode is explicit test/dev composition. Attachment bytes already cross a Wiki-specific storage port instead of direct route-level filesystem access, shared Wiki validation/auth/search helpers and the Wiki runtime context live in the application layer, and the API crate no longer depends directly on Argon2/JWT/SHA hashing crates for Wiki auth.
- Per-space permission checks are implemented in the PostgreSQL runtime for core read/write paths; staged attachment, claimed file evidence, attachment-download access, missing-file handling and FTS search semantics are covered by the PostgreSQL API smoke. Deeper repository/API coverage is still needed for less common edge-case combinations.
- Public registration enable/disable behavior is implemented through shared auth config and covered by the API memory smoke; PostgreSQL mode uses the same config gate before creating a user.
- Full generated Wiki frontend operation client is pending backend domain/repository stabilization; DTO schemas are already generated from `openapi/openapi.json`.

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
- `wsl bash -lc 'cargo fmt --manifest-path backend/Cargo.toml --all -- --check'`, `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p wiki-cli -- --nocapture'` and `wsl bash -lc 'cargo clippy --manifest-path backend/Cargo.toml -p wiki-cli --all-targets -- -D warnings'` passed after making `wiki template apply requirements` match the real PostgreSQL seed template by `document_type` instead of relying on UUID id or localized template name.
- `wsl bash -lc 'cargo tree --manifest-path backend/Cargo.toml -p api -e normal --depth 1'` confirms the API crate has no direct `argon2`, `jsonwebtoken`, `sha2`, `hex`, `rand_core` or `infra` normal dependency.
- `wsl bash -lc 'cargo clippy -p app -p api --all-targets -- -D warnings'` passed after cleaning the helper extraction and multipart upload shape.
- `wsl bash -lc 'cargo clippy --manifest-path backend/Cargo.toml -p app -p api --all-targets -- -D warnings'` passed after the Wiki auth/session helper extraction.
- `wsl bash -lc 'cargo fmt --manifest-path backend/Cargo.toml --all -- --check'`, `wsl bash -lc 'cargo check --manifest-path backend/Cargo.toml -p app -p api -p server'`, `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p api -- --test-threads=1 --nocapture'` and `wsl bash -lc 'cargo clippy --manifest-path backend/Cargo.toml -p app -p api --all-targets -- -D warnings'` passed after the API dependency cleanup.
- `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p wiki-cli -- --nocapture'` passed: 12 mocked HTTP unit smokes cover auth, spaces, document get/create/draft/publish/archive/move/history, task/phase dossier get/docs/evidence/link-doc, template list/apply, filtered search, URL/file evidence request flows and simple/structured API error envelopes; one integration smoke verifies the compiled `wiki` binary returns non-zero exit for an API error envelope.
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
- OpenAPI path parity passed: 41 expected Wiki MVP paths, `missing=0`, `extra=0`, legacy paths `0`.
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
- Document-level evidence support check passed: API memory smoke creates and lists evidence by `document_id`; CLI tests cover `wiki evidence add-link/add-file/list --document`; Playwright smoke verifies `/evidence` sends `document_id` as a list filter.
- Frontend evidence screenshot regenerated and visually inspected: `docs/screenshots/11-evidence.png`, 1920x1080.
- Frontend document editing check passed: Playwright smoke saves a document draft through `PUT /documents/{document_id}/draft` and publishes a revision through `POST /documents/{document_id}/publish`; document screenshots regenerated and visually inspected at `1920x1084` and `375x1892`.
- CLI document lifecycle check passed: mocked HTTP smoke verifies `wiki doc draft`, `publish`, `archive`, `move` and `history` use the public `/api/v1/documents/{document_id}` endpoints, send JSON payloads and apply `Idempotency-Key` only to write commands.
- CLI parity smoke check passed: mocked HTTP smoke verifies auth, space, document, task, phase, template, search and evidence command groups use public `/api/v1` paths, encode route/query parameters and keep `Idempotency-Key` off read commands.
- API/frontend/CLI error envelope check passed: backend renders structured `{"error":{"code","message"}}` responses; frontend renders legacy string and structured envelopes without `[object Object]`; CLI renders legacy string and structured `{"error":{"code","message","requestId","details"}}` responses as concise command errors.
- Frontend typed API error check passed: `apiRequest` preserves HTTP status, API code, optional `requestId` and validation details while keeping readable `error.message` for existing pages.
- `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p shared -- --nocapture'` passed after moving shared `AppError` responses to structured public envelopes: 18 tests.
- `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p api -- --test-threads=1 --nocapture'` passed after updating API forbidden-response expectations; PostgreSQL smoke still skips without `WIKI_TEST_DATABASE_URL`.
- Host `npm run typecheck`, `npm run test`, `npm run lint`, `npm run format:check` and `npm run build` passed after adding frontend structured error-envelope parsing and tests.
- `wsl bash -lc 'cargo fmt --manifest-path backend/Cargo.toml --all -- --check'` and `wsl bash -lc 'cargo clippy --manifest-path backend/Cargo.toml -p shared -p app -p api --all-targets -- -D warnings'` passed after the structured error-envelope change.
- `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml --workspace -- --test-threads=1 --nocapture'` and host `npx playwright test --project=chromium` passed after the final MVP contract review; DB-dependent infra repository tests remain ignored and API PostgreSQL smoke still skips without `WIKI_TEST_DATABASE_URL`.
- CLI binary exit check passed: compiled `wiki` returns a non-zero process status and readable stderr for a structured API error envelope.
- Search criteria extraction check passed: `app::wiki::build_wiki_search_criteria` now owns q/filter/limit normalization and escapes evidence `LIKE` wildcards; `wsl bash -lc 'cargo test --manifest-path backend/Cargo.toml -p app wiki_search_criteria -- --nocapture'`, `cargo test -p app`, `cargo test -p api` and `cargo clippy -p app -p api --all-targets -- -D warnings` passed. Real PostgreSQL syntax smoke for `LIKE ... ESCAPE` remains blocked in this run because host `docker ps` did not return and was interrupted.
- Wiki runtime context check passed: `wsl bash -lc 'cd /mnt/c/git/azhukov/sdlc/wiki/backend && cargo fmt --all -- --check'`, `cargo check -p api -p server`, `cargo test -p app wiki_ -- --nocapture`, `cargo test -p api -- --test-threads=1 --nocapture`, `cargo test -p server -- --nocapture` and `cargo clippy -p app -p api -p server --all-targets -- -D warnings` passed after switching API/server to `app::WikiAppContext`; PostgreSQL API smokes still skip without `WIKI_TEST_DATABASE_URL`.
- Frontend and page evidence check passed after the runtime-context change: host `npm run typecheck`, `npm run test`, `npm run lint`, `npm run format:check`, `npm run build` and `npm run shoot:evidence` passed; the screenshot run captured 17 desktop and 5 mobile MVP screenshots against `vite preview` on `127.0.0.1:4173`.
- Static MVP surface check passed: frontend router contains the 17 approved MVP routes, active API/server/frontend/README screenshot manifest contain no `/integrations`, `/reports` or `/notifications` route promises, README/manifest screenshot references resolve to existing PNGs, touched docs pass Prettier, and docs contain no unresolved placeholder markers or Markdown files under 20 non-empty lines.
- App legacy quarantine check passed: `wsl bash -lc 'cd /mnt/c/git/azhukov/sdlc/wiki/backend && cargo check --workspace'` and `cargo test --workspace -- --test-threads=1 --nocapture` passed for the default Wiki-only app build; `cargo tree -p app -e normal --depth 1` no longer lists `tokio`, `async-trait`, `serde_json`, `anyhow`, `thiserror` or `tracing`; `cargo test -p app --features legacy-tracker -- --nocapture` and `cargo clippy -p app --features legacy-tracker --all-targets -- -D warnings` passed for the explicit compatibility build. Before the successful rerun, the first full workspace test exhausted the full `C:` drive while linking test binaries; `cargo clean` removed 39.3 GiB of build artifacts and freed enough space to complete validation.
- Host frontend verification after the app legacy quarantine passed: `npm run typecheck`, `npm run test`, `npm run lint`, `npm run format:check` and `npm run build`.
- Production PostgreSQL runtime guard check passed: `server::run` now builds persistent Wiki backend only and returns a configuration error when `WIKI_DATABASE__URL` is empty; memory runtime is available through explicit test/dev builders. `wsl bash -lc 'cd /mnt/c/git/azhukov/sdlc/wiki/backend && cargo check -p api -p server && cargo test -p api -- --test-threads=1 --nocapture && cargo test -p server -- --nocapture && cargo clippy -p api -p server --all-targets -- -D warnings'` and `cargo check --workspace && cargo test --workspace -- --test-threads=1 --nocapture` passed; API PostgreSQL smokes still skip without `WIKI_TEST_DATABASE_URL`.
- Wiki backend port check passed: route handlers now call internal `WikiBackendPort`, `PostgresWikiBackend` is private inside `api::routes::wiki::postgres`, and `wsl bash -lc 'cd /mnt/c/git/azhukov/sdlc/wiki/backend && cargo fmt --all -- --check && cargo check -p api -p server && cargo test -p api -- --test-threads=1 --nocapture && cargo test -p server -- --nocapture && cargo clippy -p api -p server --all-targets -- -D warnings'` passed; API PostgreSQL smokes still skip without `WIKI_TEST_DATABASE_URL`. Static search confirmed the parent route module/server no longer use the old concrete PostgreSQL accessor.
- Full backend workspace regression passed after the backend-port change: `wsl bash -lc 'cd /mnt/c/git/azhukov/sdlc/wiki/backend && cargo check --workspace'` and `wsl bash -lc 'cd /mnt/c/git/azhukov/sdlc/wiki/backend && cargo test --workspace -- --test-threads=1 --nocapture'` passed; infra Docker repository tests remain ignored and API PostgreSQL smokes still skip without `WIKI_TEST_DATABASE_URL`.
- Host frontend sanity passed after the backend-port change: `npm run typecheck`, `npm run test`, `npm run lint`, `npm run format:check` and `npm run build`.
- Frontend OpenAPI DTO generation check passed: `npm run generate:api` writes `frontend/src/api/generated.ts`, auth/Wiki endpoint wrappers import generated request/response aliases, and host `npm run typecheck`, `npm run test`, `npm run lint`, `npm run format:check`, `npm run build` and `npm run test:e2e -- --project=chromium` passed against those generated types. `openapi-gen` now preserves the proprietary OpenAPI license metadata and writes a trailing newline.
- Settings/admin MVP hardening check passed: API/OpenAPI/CLI/frontend expose `GET /api/v1/settings` as an admin-only read-only runtime snapshot, `/settings` and `/admin` read the same public API, `wsl bash -lc 'cd /mnt/c/git/azhukov/sdlc/wiki/backend && cargo fmt --all -- --check && cargo check -p api -p server -p wiki-cli && cargo test -p wiki-cli -- --nocapture && cargo test -p api -- --test-threads=1 --nocapture && cargo clippy -p api -p wiki-cli --all-targets -- -D warnings'`, host `npm run typecheck`, `npm run lint`, `npm run test`, `npm run format:check`, `npm run build`, `npm run test:e2e -- --project=chromium` and `npm run shoot:evidence` passed, and README/manifest reference 22 existing screenshot PNGs with `missing=0`, `extra=0`.

## Known Local Environment Limits

- MSVC `link.exe` is required for full Rust linking on the current Windows host.
- `pnpm add` is blocked on this host by Corepack/Node `ERR_VM_DYNAMIC_IMPORT_CALLBACK_MISSING`; package/lock changes were reviewed manually when needed.
- If `pnpm` blocks, direct package binaries under `frontend/node_modules/.bin` can still be used for TypeScript/tests/build/lint verification.
- In the latest run, host Docker CLI did not return for `docker compose`/`docker ps`, so fresh PostgreSQL integration smoke should be rerun after Docker Desktop/daemon responsiveness is restored.
