# Wiki MVP Readiness 100%

> Snapshot date: 2026-09-01. This document defines readiness for starting main Wiki development, not production launch readiness.

## 1. Purpose

Wiki is ready for main development when product scope, API, data model, frontend pages, CLI surface, tests, security boundaries, operations and screenshots are complete enough that implementation work can start without re-deciding the MVP.

This gate is intentionally narrower than production readiness. It confirms the base application shape and the evidence needed before engineers or agents continue implementation.

## 2. Readiness Scale

| Aspect | Gate for 100% pre-development readiness | Evidence | Status |
| ------ | --------------------------------------- | -------- | ------ |
| Product | MVP scope is stable and excludes deferred features | `docs/PRODUCT_REQUIREMENTS.md`, `docs/ROADMAP.md` | Ready |
| API | Public `/api/v1` contract is complete and documented | `openapi/openapi.json`, `docs/API.md` | Ready |
| Backend | Layering, runtime composition, persistence and tests are defined | `docs/ARCHITECTURE.md`, `docs/CURRENT_STATE.md` | Ready |
| Data | Tables, indexes, constraints and migration source are documented | `docs/DATA_MODEL.md`, `docs/DATABASE_INDEXES.md`, `backend/migrations` | Ready |
| Frontend | Approved MVP route set is frozen and backed by screenshots | `docs/ROUTING.md`, `docs/PAGE_DESIGN.md`, `README.md` | Ready |
| CLI | CLI groups are aligned with public API operations | `docs/CLI.md`, `backend/cli/src/main.rs` | Ready |
| Tests | Required test suites and static audits are listed and runnable | `docs/TEST_PLAN.md`, `docs/TESTING.md` | Ready |
| Security | MVP threats, trust boundaries and controls are documented | `docs/SECURITY.md`, `docs/THREAT_MODEL.md` | Ready |
| Ops | Health/readiness, metrics, backup/restore and deployment gates are documented | `docs/OPERATIONS.md`, `docs/DEPLOYMENT.md` | Ready |
| Docs | CI/CD-style documentation parity is preserved and indexed | `docs/README.md`, `docs/TRACEABILITY.md` | Ready |

## 3. Capability Coverage

API paths in this table are relative to `/api/v1` unless marked otherwise. `/metrics` is an operational endpoint outside OpenAPI v1.

| Capability | User scenario | API | CLI | UI | Data | Evidence |
| ---------- | ------------- | --- | --- | -- | ---- | -------- |
| Auth | Пользователь входит, выходит и проверяет профиль | `/auth/login`, `/auth/logout`, `/auth/refresh`, `/users/me` | `wiki auth` | `/login` | `users`, `auth_sessions` | auth tests, `01-login.png` |
| Registration | Пользователь создаёт учётную запись, если регистрация включена | `/auth/register` | no CLI MVP command; UI/API public flow | `/register` | `users`, `auth_sessions` | register tests, `02-register.png` |
| Users and roles | Admin управляет users и space roles | `/users`, `/spaces/{space_key}/members` | `wiki user`, `wiki space members/member-set/member-remove` | `/users`, `/admin` | `users`, `space_members`, `audit_log` | RBAC tests, `14-users.png`, `17-admin.png` |
| Spaces | Пользователь открывает доступные spaces и дерево страниц | `/spaces`, `/spaces/{space_key}`, `/spaces/{space_key}/tree` | `wiki space` | `/spaces`, `/` | `spaces`, `documents` | space tests, `03-dashboard.png`, `04-spaces.png` |
| Documents | Editor создаёт, редактирует, публикует, архивирует и перемещает страницы | `/spaces/{space_key}/documents`, `/documents/{document_id}` and write actions | `wiki doc` | `/documents/new`, `/documents/:documentId` | `documents`, `document_drafts`, `document_revisions` | document tests including stale publish conflict, `05-document-compose.png`, `06-document-view.png` |
| Revisions | Пользователь видит immutable history | `/documents/{document_id}/revisions?limit=20` | `wiki doc history --limit`, `wiki doc revision` | `/documents/:documentId` | `document_revisions` | revision limit/history tests, `06-document-view.png` |
| Task dossiers | Пользователь видит знания по external task key | `/spaces/{space_key}/tasks` | `wiki task` | `/tasks`, `/tasks/:taskKey` | `task_dossiers`, `document_task_links`, `evidence_items` | dossier tests, `07-task-dossiers.png`, `08-task-dossier-detail.png` |
| Phase dossiers | Пользователь видит знания по workflow phase key | `/spaces/{space_key}/phases` | `wiki phase` | `/phases`, `/phases/:phaseId` | `phase_dossiers`, `document_phase_links`, `evidence_items` | phase tests, `09-phase-dossiers.png`, `10-phase-dossier-detail.png` |
| Evidence and attachments | Editor добавляет URL/file material with checksum | `/evidence?limit=30`, `/attachments` | `wiki evidence`, `wiki attachment` | `/evidence`, document/task/phase pages | `evidence_items`, `attachments` | upload/list limit tests, `11-evidence.png` |
| Search | Пользователь ищет документы и материалы с фильтрами | `/search?limit=20` | `wiki search query --limit` | `/search` | PostgreSQL FTS projection | search limit/filter tests, `16-search.png`, `m-search.png` |
| Templates | Editor стартует документ из базового шаблона | `/templates` | `wiki template` | `/templates`, `/documents/new` | `document_templates` | template tests, `12-templates.png` |
| Settings | Admin видит безопасный runtime snapshot | `/settings` | `wiki settings get` | `/settings`, `/admin` | runtime config snapshot | settings tests, `15-settings.png` |
| Audit | Admin проверяет bounded append-only write history and request correlation | `/audit-log` | `wiki audit list --limit` | `/audit-log`, `/admin` | `audit_log.request_id` | audit tests, `13-audit-log.png` |
| Runtime probes | Operator проверяет liveness/readiness before traffic | `/health`, `/health/ready`, `/metrics` outside API v1 | curl/API-only | no route | runtime state, metrics exporter | health tests, ops docs |
| API contract | UI/CLI and agents use the same public API | `/api/v1`, OpenAPI | all CLI groups | all MVP routes | DTO schemas | OpenAPI parity check |

## 4. Design Freeze

- Approved frontend routes are exactly the route set in `docs/ROUTING.md`.
- Visible product text is Russian; routes, identifiers and API fields remain English.
- Every API-backed page must expose loading, empty, validation/error and retry states before release readiness.
- Every desktop route must have a README screenshot and manifest entry.
- Mobile smoke must cover dashboard, spaces, document view, task dossier and search.
- Operational probes, OpenAPI and metrics are API/ops artifacts and do not require frontend pages or screenshots.

## 5. Negative Case Coverage

| Area | Required negative cases before release |
| ---- | -------------------------------------- |
| Auth | bad credentials, disabled registration, expired/revoked token |
| Access | viewer write attempt, editor admin attempt, removed membership, cross-space read/write |
| Documents | duplicate slug, archived document writes, archived space content writes, move into own descendant, missing title/body |
| Revisions | latest-first order, immutable content, bounded `1..100` limit |
| Evidence | no target, cross-space target, URL/file payload mismatch, reused staged upload, bounded `1..100` limit |
| Attachments | empty file, unsafe filename, unsafe storage key, oversize payload, unauthorized download |
| Search | no-role access, archived filter, task/phase filter isolation, bounded `1..100` limit |
| Audit | write action without audit, audit diff containing secret-like values |
| Ops | readiness before DB connection, migration failure, backup restore verification failure |

## 6. Go/No-Go Checklist

- MVP scope has no active route/API/client for reports, notifications, integrations, webhooks or runners.
- `docs/API.md` and `docs/PRODUCT_REQUIREMENTS.md` document every OpenAPI path.
- `docs/TRACEABILITY.md` maps every P0/P1 requirement to API/data/UI-or-API-only/test evidence.
- `docs/PAGE_DESIGN.md` defines purpose, states and actions for every approved route.
- `docs/DATA_MODEL.md` names every migration table and required constraint category.
- `docs/CLI.md` documents every CLI group, output policy and API-only exception.
- README renders all screenshot images; manifest and screenshot script cover the same files.
- Frontend and backend verification commands are recorded in `docs/CURRENT_STATE.md`.
- Docker PostgreSQL smoke is either green on a Docker host or explicitly replaced by the accepted WSL PostgreSQL smoke for this Windows host.

## 7. Blockers, Non-Blockers And Deferred Scope

### Blockers For Main Development

None known after this readiness gate passes.

### Non-Blockers Before Production

- Docker-backed PostgreSQL smoke must still be run on a host with Docker Desktop available.
- Native Windows Rust checks require MSVC Build Tools; WSL backend verification is accepted on this host.
- Generated operation client can replace handwritten frontend endpoint wrappers after the API contract stabilizes further.
- Security scans, backup restore drills and deployment TLS/CORS review remain release gates.

### Deferred Features

Reports, notifications, integrations, webhooks, runners, comments, mentions, approvals, import/export bundles, real-time collaboration, public sharing, OCR and binary attachment indexing are not part of Wiki MVP.

## 8. Required Verification Commands

```bash
# Frontend
cd frontend
npm run typecheck
npm run test
npm run lint
npm run format:check
npm run build
npm run test:e2e -- --project=chromium
npm run shoot:evidence
```

```bash
# Backend
cd backend
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace -- --test-threads=1
cargo clippy --workspace --all-targets -- -D warnings
```

```powershell
# PostgreSQL smoke
pwsh -File scripts/postgres-smoke.ps1
pwsh -File scripts/postgres-smoke-wsl.ps1
```

## 9. Developer Handoff

Main development can start from this packet without reopening MVP scope:

1. Read `docs/CURRENT_STATE.md`, this document and `docs/NEXT_STEPS.md`.
2. Treat `openapi/openapi.json`, `backend/migrations` and `docs/ROUTING.md` as frozen baseline contracts until a reviewed requirement changes them.
3. Implement through existing Rust layers: domain value objects, app use cases/ports, infra adapters, API handlers, CLI commands and frontend endpoint wrappers.
4. Keep UI and CLI as equal public API clients; neither client may depend on PostgreSQL, local storage layout or internal Rust modules.
5. When a route, endpoint, migration or screenshot changes, update the matching PRD/API/data/page/traceability/readiness evidence in the same change.

## 10. References

- `docs/CURRENT_STATE.md`
- `docs/PRODUCT_REQUIREMENTS.md`
- `docs/API.md`
- `docs/CLI.md`
- `docs/DATA_MODEL.md`
- `docs/PAGE_DESIGN.md`
- `docs/TRACEABILITY.md`
- `docs/TEST_PLAN.md`
- `docs/NEXT_STEPS.md`
