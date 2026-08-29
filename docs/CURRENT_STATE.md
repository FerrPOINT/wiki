# Current State - Wiki

> Snapshot date: 2026-08-28. Authority is repository code and tests; update this file whenever capabilities move from target to current.

## Current Verified

| Capability | Status | Notes |
|---|---|---|
| Repository scaffold | Current | `task-tracker` code copied into `wiki`; `.git` from Wiki preserved |
| Product requirements | Current | `docs/PRODUCT_REQUIREMENTS.md` defines the reduced base Wiki scope |
| Documentation set | Current | CI/CD-style document set prepared for Wiki |
| CLI shape | Current | `wiki` CLI command surface drafted for public API operations |
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

- Backend domain/services/routes/migrations still include old task-tracker modules.
- `openapi/openapi.json` still describes inherited endpoints.
- Generated Wiki OpenAPI client is pending backend domain/API migration.

## Verification Commands

```bash
cd backend
cargo fmt --all -- --check
cargo check -p wiki-cli

cd frontend
.\node_modules\.bin\tsc.cmd --noEmit
.\node_modules\.bin\vitest.cmd run
.\node_modules\.bin\vite.cmd build
node scripts/shoot-evidence.mjs
```

Latest verification on 2026-08-28:

- `cargo fmt --all -- --check` passed.
- `cargo check -p wiki-cli` blocked before project code by missing Windows MSVC `link.exe`.
- `tsc --noEmit` passed.
- `vitest run` passed: 5 files, 17 tests.
- `vite build` passed.
- Screenshot script regenerated 22 screenshots.
- README/manifest screenshot references resolve to existing PNG files.
- CI/CD docs filename parity passed with `missing=0`.

## Known Local Environment Limits

- MSVC `link.exe` is required for full Rust linking on the current Windows host.
- If `pnpm` blocks on ignored native build scripts, direct package binaries under `frontend/node_modules/.bin` can still be used for TypeScript/tests/build verification.
