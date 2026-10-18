# Testing Strategy - Wiki

Актуальная политика 2026-10-15: unit/static запускаются при обычной сборке;
integration/slow/full coverage и browser/live — вручную перед релизом.
Команды быстрых уровней приведены в разделе «Тестовые уровни (2026-10-15)» ниже.
Прежняя полная матрица и тяжёлые команды этого документа сохранены как
ручные release checks; они не запускаются автоматически после каждой правки.

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

На Windows с закреплёнными Node 22.20.0 / pnpm 10.28.1 / Vitest 4.1.11
unit runner использует `threads`, максимум 4 workers и `isolate: true`.
На остальных платформах сохранён `forks`: Windows qualification не подтверждает
профиль Linux. При изменении toolchain, native dependencies или process APIs
сначала повторить применимую квалификацию; `--pool=forks` — явный диагностический
override, а не повод менять тесты ради прохождения threads.

Три warm-пары на source `e8c822c`, одинаковые 247 tests / 35 files, один общий
host slot, порядок forks/threads → threads/forks → forks/threads:

| Pool | Wall, median | CPU Job, median | Peak committed Job, range | Processes |
| --- | --- | --- | --- | --- |
| forks | 31,703 с | 144,203 с | 932 810 752–1 004 494 848 bytes | 45 |
| threads | 30,438 с | 137,156 с | 954 155 008–1 011 957 760 bytes | 10 |

Во второй паре threads был медленнее. Профиль выбран по меньшему median wall,
CPU во всех трёх парах и числу процессов; значимой экономии RAM не показано.
Это private Windows Job с pnpm/supervisor, не physical RAM/RSS или требование
runtime сервиса. Production build, install и source export для измерений не
выполнялись; исходные test cases/assertions и browser isolation сохранены.
Полные timings/hashes и дальнейшая qualification ведутся в Base performance
execution ledger. Unit proof не заменяет integration/slow/live release gates.

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

Регрессия central_projection_distinguishes_database_failure_from_invalid_claims
проверяет закрытый DB pool и неполные claims отдельно. CI явно запускает
mixed_central_directory_preserves_identity_and_disabled_state_in_postgres
на своём PostgreSQL; workspace test не исполняет ignored-тест автоматически.

## Тестовые уровни (2026-10-15)

Обязательная общая политика: [Base TESTING](https://github.com/FerrPOINT/services-base/blob/ad724088e9a63b1e353fb5b28e4308e9ae6f45ae/docs/TESTING.md).
Выбор компонентов и native commands: `.quality-gate.json`.

### Разработка и сборка

Backend: `cargo test --locked -p domain --lib` из backend/.

Frontend из frontend/: `pnpm test:unit`, `pnpm build` включает unit.
`pnpm test` — unit alias, `pnpm test:all` — прежний полный набор.
Playwright, packed-consumer и effective-theme проверки — ручной release.

### Ручная релизная проверка

Полный прежний набор сохранён в `.github/workflows/release.yml`: только
workflow_dispatch на exact ref. Автоматический CI не поднимает PostgreSQL
и не запускает длительные/browser/live тесты.

Required security/data/compatibility checks и thresholds сохраняются.
Unit success не заменяет ручную приёмку; NOT RUN не означает PASS.

## Передача frontend artifact

Prepared quality gates `frontend:types`, `frontend:tests` и `frontend:build`
объявляют `descriptor_input_contract: "prepared-frontend-v1"`; build дополнительно
объявляет `artifact_input_contract: "git-source-v1"`. Совместимый exact Base
использует общую с preparer модель и сохраняет полную validation provenance:
несвязанная gate правка не пересобирает dist при прежних фактических build inputs.
Unknown/copied descriptor, source/config/toolchain изменения и stale plan
сохраняют обязательную invalidation. Unit receipt остаётся привязан к exact commit.
Контракт и qualification requirements принадлежат разделу 13 Base
`docs/PERFORMANCE_SPEC.md`. Старый runner сохраняет полный descriptor guard;
новый режим требует совместимого quality tooling, independent pins не меняются.

Tailwind исключает test/spec/e2e/setup sources на уровне CSS `@source not`,
включая явно подключённый UI package. Unit fixtures не являются источником
production utilities. `src/test/source-inputs.test.ts` проверяет настоящий
Tailwind compiler/oxide scanner: fixture-only classes не попадают в CSS,
production/library classes и соседний `contest.tsx` сохраняются. Одного
`exclude_inputs` в build fingerprint для этого недостаточно. Тест не меняет
production source/dependencies, использует собственный временный fixture и
входит в обычный Vitest unit-набор.

CI `frontend` передаёт `tested-frontend-dist` в `frontend-package`; отдельный consumer не устанавливает Node/dependencies.
Проверка графа: `python scripts/vendor/base_frontend_artifact.py check-ci --root .`.
[Формат/команды Base](https://github.com/FerrPOINT/services-base/blob/a9b1ae9869784676957c193a75bd92a2a0480dc1/docs/BUILD_PERFORMANCE.md#передача-frontend-artifact); packet pin — `.quality-tooling-revision`.
Node 22.20.0; OpenAPI generation/checks сохраняются в producer.

Docker unit stage явно получает `frontend/nginx.conf`: routing tests читают
фактический config. Он остаётся вне Vite application COPY; изменение config
повторяет относящиеся unit и packaging, сохраняя application compilation cache.

WikiSearch unit явно разделяет legacy search (namespace=false) и Namespace
global-search cases (true). Оба входят в ordinary unit; mocks/env очищаются
после каждого case, в том числе вне legacy describe. Production build flags
не должны молча менять смысл legacy fixture или выключать Namespace regressions.

## Одна OpenAPI generation в CI

В frontend job автоматического CI и ручного release `postinstall` выполняет
generation во время frozen install; project lifecycle явно включён. Сразу после
установки Git проверяет tracked outputs и отсутствие drift, без второго
generator invocation. Проверяемые outputs: `src/api/generated.ts`, `src/api/generated-exports.ts`.
Lockfile, backward compatibility и backend OpenAPI drift gates сохраняются.
Самостоятельный `pnpm openapi:check` остаётся проверкой для окружения, в котором
generation ещё не выполнялась; его не добавляют повторно после postinstall.

## Точные Rust artifacts обычной сборки

`backend:build` в `.quality-gate.json` выбирает только package/bin targets
поставляемого runtime: `server`.
Outputs именованы явно; отсутствие любого required executable завершает gate
ошибкой. Случайный соседний executable не попадает в artifact.
CLI `wiki`, migration и OpenAPI остаются отдельными consumers; их workspace/release проверки сохранены.
`backend:quality` сохраняет полные workspace clippy/tests в ручном release;
MSRV/features/OpenAPI и необходимые integration gates не сокращаются.
Fresh target между source exports и registry/compiler-cache policy Base сохраняются.
