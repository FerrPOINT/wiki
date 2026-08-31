# Current State - Wiki

> Snapshot date: 2026-08-31. Authority is repository code and tests; update this file whenever capabilities move from target to current.

## Current Verified

| Capability | Status | Notes |
|---|---|---|
| Repository scaffold | Current | `task-tracker` code copied into `wiki`; `.git` from Wiki preserved |
| Product requirements | Current | `docs/PRODUCT_REQUIREMENTS.md` defines the reduced base Wiki scope |
| Documentation set | Current | CI/CD-style document set prepared for Wiki |
| CLI shape | Current | `wiki` CLI command surface drafted for public API operations |
| API shell | Current | Runtime router and OpenAPI expose Wiki MVP endpoints only; evidence type validation uses `external_url` / `uploaded_file`; implementation is in-memory until domain/repository migration |
| Frontend route shell | Current | Static Wiki pages and screenshots exist for the approved MVP page set only |
| Page design contract | Current | `docs/PAGE_DESIGN.md` fixes page composition, states and deferred boundaries before backend work |
| Refined MVP page design | Current | Spaces, documents, tasks, phases, evidence and search pages include API-ready layouts and metadata blocks |
| Screenshot evidence | Current | 17 desktop and 5 mobile screenshots regenerated for the MVP page set |
| MVP documentation cleanup | Current | Removed visible/technical integrations, reports and notifications scope from frontend routes, README gallery and screenshot manifest |
| Development readiness docs | Current | README, local setup, env, migrations, storage, security, ops and runbooks are aligned with Wiki MVP/current-vs-target boundaries; host-side `cargo run` env is documented separately from Docker Compose `.env` |
| Frontend API shell | Current | Thin handwritten auth client; old tracker generated client removed |
| Env/project identity | Current | `WIKI_` prefix, docker names and frontend package identity |

## Target MVP

| Capability | Target |
|---|---|
| Users and roles | login/logout, current user, admin/editor/viewer |
| Spaces | CRUD, members, permissions, archive |
| Documents | Markdown draft, publish, revision history, archive |
| Page tree | parent/child, breadcrumbs, move within space |
| Task links | link documents/evidence by external task key |
| Phase links | link documents/evidence by phase key |
| Evidence | `external_url` and `uploaded_file` evidence, checksum, lists by owner |
| Attachments | local storage metadata and download |
| Search | PostgreSQL FTS over title/body with basic filters |
| Templates | requirements, research note, implementation note, test plan, release note |
| Audit | write actions and access/role changes |
| API/UI/CLI | same MVP operations through public `/api/v1` |

## Explicitly Deferred

- Comments, mentions and inline reviews.
- Advanced reports.
- Approval chains.
- Import/export bundles.
- Real-time collaborative editing.
- OCR and binary attachment indexing.

## Inherited To Replace

- Backend app/domain/infra/migrations still include old task-tracker modules outside the active Wiki API shell.
- PostgreSQL persistence for Wiki spaces/documents/revisions/evidence/search/audit is still target work.
- Generated Wiki frontend OpenAPI client is pending backend domain/repository stabilization.

## Verification Commands

```bash
cd backend
cargo fmt --all -- --check
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

- `cargo fmt --all -- --check` passed.
- `cargo metadata --no-deps --format-version 1` passed; `openapi-gen` is the single OpenAPI generator binary.
- `cargo check -p api`, `cargo check -p wiki-cli`, `cargo check -p server` and `cargo test -p api wiki_mvp_routes_cover_public_contract` blocked before project code by missing Windows MSVC `link.exe` / Windows SDK libs.
- `tsc --noEmit` passed.
- `eslint . --max-warnings=0` passed.
- `prettier --check .` passed after mechanical frontend formatting cleanup.
- `vitest run` passed: 5 files, 17 tests.
- `vite build` passed.
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
- Traceability coverage passed: 28 PRD requirement IDs, `missing=0`, `extra=0`.
- CI/CD docs filename parity passed with `missing=0`.
- Markdown documentation checks passed: no open placeholder markers, no Markdown document under 20 non-empty lines.
- Local setup/deployment docs distinguish Docker Compose `.env` from process env used by host-side `cargo run`.
- Active route/API cleanup check passed: no `/integrations`, `/reports`, `/notifications` routes or old task-tracker API groups in active frontend/API/server/CLI/OpenAPI; only package name `@sentry/integrations` remains in `pnpm-lock.yaml`.

## Known Local Environment Limits

- MSVC `link.exe` is required for full Rust linking on the current Windows host.
- `pnpm add` is blocked on this host by Corepack/Node `ERR_VM_DYNAMIC_IMPORT_CALLBACK_MISSING`; package/lock changes were reviewed manually when needed.
- If `pnpm` blocks, direct package binaries under `frontend/node_modules/.bin` can still be used for TypeScript/tests/build/lint verification.
