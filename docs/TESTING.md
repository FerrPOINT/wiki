# Testing Strategy - Wiki

## 1. Principles

### Live Detail Geometry

На уже работающем локальном стенде:

```powershell
$env:SDLC_LIVE_QA = '1'
$env:PLAYWRIGHT_BASE_URL = 'http://localhost:7732'
pnpm --dir frontend exec playwright test e2e/detail-layout-live.spec.ts --project chromium --workers 1 --retries 0
```

Учётка читается из соседнего `services-base/deploy/.local/qa-session.json` либо
явного `SDLC_QA_SESSION_FILE`. Секреты и trace не публикуются. Реальные API
создают отдельные QA space/document/evidence и две ревизии; после проверки
документ и пространство архивируются штатным API. Evidence и аудит сохраняются
как исторические данные в собственном QA-пространстве до удаления изолированного
Compose-проекта; удаление пользовательских volumes не требуется.

Тест проверяет чтение/правку документа, задачу и фазу: 108 сочетаний в трёх темах,
rail/gap/position и intentional wide columns, границы 1023/1024 и 1279/1280,
ширину чтения не более 760 px, full-page screenshots и axe. Keyboard/touch
проверяют revision dialog, Escape/focus return и переходы с сохранением space.
При измерении страниц мутаций нет; fixture setup/cleanup отделены от UI-проверки.
На том же образе повторяется существующая live route matrix Wiki из Task Tracker.

Результат 1 октября: 2/2 теста, 252 сочетания, retries=0; выбранные full-page
кадры и fingerprints: [evidence](assets/screens/2026-10-01-detail-layout/README.md).

- Tests cover meaningful user paths and domain invariants.
- Backend tests prefer real PostgreSQL for repositories and focused mocks for services.
- Frontend tests use Vitest and Testing Library; E2E uses Playwright.
- UI changes should be checked on mobile and desktop widths.
- Integration tests must cover auth, permissions and data isolation between spaces.

## 2. Backend Tests

### Unit

- `domain/` - document, revision, space key and dossier invariants.
- `app/` - services for publish, archive, evidence attach, permission checks.
- `shared/` - config/env parsing, ID helpers, error mapping.

### Integration

- Repository tests against PostgreSQL.
- API tests for spaces, documents, revisions, task dossiers, phase dossiers, evidence and attachments.
- Failing repository tests for 500/error envelope behavior.
- External link ingestion tests use service-level mocks only after the base Wiki domain exists.
- PostgreSQL smoke tests are grouped by the `wiki_postgres_` test-name prefix and are enabled by `WIKI_TEST_DATABASE_URL`.
- `wiki_postgres_search_uses_fts_index_when_database_available` records query-plan evidence for MVP full-text search filters and asserts the GIN index is selected.

### Coverage Priorities

| Area                  | Target |
| --------------------- | ------ |
| Domain invariants     | high   |
| Permission checks     | high   |
| Publish/revision flow | high   |
| Evidence ingestion    | high   |
| Search indexing       | medium |
| Admin settings        | medium |

## 3. Frontend Tests

Unit/component tests:

- app shell navigation, account menu and logout;
- dashboard;
- spaces and page tree preview;
- successful archive removes the action and restores focus to the space row or
  document heading; Cancel/Escape restore the existing archive action;
- safe API error formatting for permission denied and validation details;
- document editor;
- revision history;
- evidence feed;
- task dossier page;
- phase dossier page;
- search empty/result/error states.

E2E smoke:

- login;
- open dashboard;
- create draft;
- publish document;
- attach evidence;
- search by task key;
- open phase dossier.

## 4. Commands

```bash
cd backend
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace -- --test-threads=1

cd frontend
pnpm typecheck
pnpm test -- --run
pnpm test:e2e
```

PostgreSQL-backed API smoke from the repository root:

```powershell
pwsh -File scripts/postgres-smoke.ps1
```

The smoke runner starts `backend/docker-compose.test.yml`, waits for `postgres-test`, sets `WIKI_TEST_DATABASE_URL=postgres://wiki@127.0.0.1:3458/wiki_test` and runs:

```bash
cd backend
cargo test -p api wiki_postgres_ -- --test-threads=1 --nocapture
```

If Docker is managed outside the script, set `WIKI_TEST_DATABASE_URL` manually and run the same filtered Cargo command.

When Docker Desktop is unavailable but WSL has a local PostgreSQL service and the `postgres` system user can create temporary roles/databases, use:

```powershell
pwsh -File scripts/postgres-smoke-wsl.ps1
```

The WSL runner creates an isolated temporary database and role, runs the same `wiki_postgres_` suite through WSL Cargo, and removes only those temporary objects on exit.

Backup/restore smoke from the repository root:

```powershell
pwsh -File scripts/backup-restore-smoke-wsl.ps1
```

The backup/restore smoke applies canonical SQLx migrations to an isolated source DB, writes document/evidence/attachment control data, creates a custom-format PostgreSQL dump plus attachment tar, restores them into an isolated restore DB, verifies restored metadata/checksums/file bytes and removes the temporary DBs/role on exit.

## 5. Fixtures

Baseline test fixtures:

- system admin;
- space `ENG`;
- document with two revisions;
- task dossier `SDLC-42`;
- phase dossier `implementation`;
- linked evidence artifact;
- viewer/editor users for permission checks.

## 6. Merge Checklist

- [ ] `cargo fmt --all -- --check` clean
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo test --workspace -- --test-threads=1` green
- [ ] `pnpm typecheck` clean
- [ ] `pnpm test -- --run` green
- [ ] `pnpm build` green
- [ ] Playwright critical path green
- [ ] Documentation updated

## 7. References

- Header acceptance: [2026-10-01](plan/2026-10-01-platform-header.md).
  `SDLC_LIVE_QA=1 SDLC_QA_SESSION_FILE=<private-json> E2E_BASE_URL=<wiki-url>`
  включает `frontend/e2e/platform-header-live.spec.ts` против живых сервисов.
  Секретный JSON содержит `email`/`password`, хранится вне Git; trace/video
  выключены. `SDLC_HEADER_EVIDENCE_DIR` задаёт каталог PNG и `results.json`.
  Запускать Chromium с retries=0, установленным Base package и актуальным
  production image; mock smoke не заменяет этот gate.

- `docs/ARCHITECTURE.md`
- `docs/API.md`
- `docs/DEPLOYMENT.md`
- `justfile`

## Общая база

Подключение версий, границы контрактов и проверки описаны в [BASE_INTEGRATION](BASE_INTEGRATION.md).
