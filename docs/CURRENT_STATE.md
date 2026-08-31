# Current State - Wiki

> Snapshot date: 2026-08-31. Authority is repository code and tests; update this file whenever capabilities move from target to current.

## Current Verified

| Capability | Status | Notes |
|---|---|---|
| Repository scaffold | Current | `task-tracker` code copied into `wiki`; `.git` from Wiki preserved |
| Product requirements | Current | `docs/PRODUCT_REQUIREMENTS.md` defines the reduced base Wiki scope |
| Documentation set | Current | CI/CD-style document set prepared for Wiki |
| CLI shape | Current | `wiki` CLI command surface drafted for public API operations |
| API shell | Current | Runtime router and OpenAPI expose Wiki MVP endpoints only; old API dto/middleware/route/test files were removed; implementation is in-memory until domain/repository migration |
| Frontend route shell | Current | Static Wiki pages and screenshots exist for the approved MVP page set only |
| Page design contract | Current | `docs/PAGE_DESIGN.md` fixes page composition, states and deferred boundaries before backend work |
| Refined MVP page design | Current | Spaces, documents, tasks, phases, evidence and search pages include API-ready layouts and metadata blocks |
| Screenshot evidence | Current | 17 desktop and 5 mobile screenshots regenerated for the MVP page set |
| MVP documentation cleanup | Current | Removed visible/technical integrations, reports and notifications scope from frontend routes, README gallery and screenshot manifest |
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
| Evidence | URL evidence, file evidence, checksum, lists by owner |
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
.\node_modules\.bin\vitest.cmd run
.\node_modules\.bin\vite.cmd build
.\node_modules\.bin\playwright.cmd test --project=chromium
node scripts/shoot-evidence.mjs
```

Latest verification on 2026-08-31:

- `cargo fmt --all -- --check` passed.
- `cargo metadata --no-deps --format-version 1` passed; `openapi-gen` is the single OpenAPI generator binary.
- `cargo check -p api`, `cargo check -p wiki-cli` and `cargo check -p server` blocked before project code by missing Windows MSVC `link.exe` / Windows SDK libs.
- `tsc --noEmit` passed.
- `vitest run` passed: 5 files, 17 tests.
- `vite build` passed.
- `playwright test --project=chromium` passed: 1 smoke test.
- Screenshot script regenerated 22 screenshots.
- `eslint . --max-warnings=0` is blocked by missing `@eslint/js` in the local frontend install/config.
- `prettier --check .` reports existing format drift in frontend files; broad formatting cleanup was not mixed into this MVP contract change.
- README/manifest screenshot references resolve to existing PNG files.
- OpenAPI path parity passed: 40 expected Wiki MVP paths, `missing=0`, `extra=0`, legacy paths `0`.
- CI/CD docs filename parity passed with `missing=0`.

## Known Local Environment Limits

- MSVC `link.exe` is required for full Rust linking on the current Windows host.
- If `pnpm` blocks on ignored native build scripts, direct package binaries under `frontend/node_modules/.bin` can still be used for TypeScript/tests/build verification.
- Frontend lint needs the local ESLint dependency set fixed before it can be used as a release gate on this host.
