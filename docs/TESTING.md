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

Container recipes имеют собственные `container_inputs`: Dockerfile/recipe ignore
и frontend nginx выбирают упаковку соответствующего сервиса. Это отличается от
binary/Vite producer; один recipe-only change не требует application compile.
Native layer reuse и реальная упаковка квалифицируются отдельно.

Общие правила: Base `docs/BUILD_PERFORMANCE.md` и `docs/TESTING.md` из принятого
quality tooling. SDK/UI/skills/operator pins независимы от quality pin.
Классификация и команды Wiki находятся в `.quality-gate.json`; CI и container
helper используют тот же exact Base selector.

Из корня PDLC workspace сначала выполнить `enter-dev.ps1`, затем выбрать Wiki:

```powershell
./check-workspace.ps1 -Profile Build -Repository wiki -BaseRef HEAD -PlanOnly
./check-workspace.ps1 -Profile Unit -Repository wiki -Component tooling -BaseRef HEAD
```

Обычный запуск учитывает staged/unstaged/untracked/deleted inputs. `-All` означает
намеренный полный обход; неизвестный путь расширяет проверки продукта.
План без изменений не подтверждает наличие образа: холодную подготовку и нужный
компонент задавать явно. Test-only сохраняет production producer при неизменных
build inputs; docs-only не создаёт application artifact.

Быстрый Rust-набор сейчас — `cargo test --locked -p domain --lib`. Backend server
и официальный HTTP CLI — отдельные producers: `-p server --bin server` и
`-p wiki-cli --bin wiki`. CLI не включается в server graph и не исчезает из поставки.
CLI rustfmt — static gate; физическое отделение его pure unit от HTTP integration
ещё требует квалификации. Оно не считается выполненным через unit domain.
CI сохраняет download cache, но `cache-targets: false` не восстанавливает compiled
target без qualified timestamp guard. Безопасное возвращение compiled dependency
reuse требует отдельной native qualification; Docker layer cache сохраняется.

Frontend `pnpm test:unit` запускает source-tool Node tests и Vitest без
`*.integration.test.*`, `*.slow.test.*`, `*.e2e.test.*`. `pnpm test:all` сохраняет
полный Vitest-набор; Playwright запускается отдельно. При сборке units выполняются
до producer. Ошибка unit не позволяет опубликовать новый candidate.

В frontend Docker recipes dependency manifest исключает продуктовые scripts;
frozen install сохраняет lifecycle зависимостей. OpenAPI, generators и `.prettierrc`
копируются после установки; отдельный `pnpm run postinstall` готовит types и bridge.
Изменение этих inputs не входит в dependency layer. Source-регрессии проверяют
порядок и сохранение dependency fields; реальные BuildKit cache hits и ускорение
этого разделения пока не измерены. Обычный frozen install вне Docker сохраняет
прежний postinstall.

Docker install использует зарегистрированный Node22/architecture download store
Base в `/pnpm/store`; integrity check включён, side-effects cache выключен.
`node_modules` остаётся локальным; реальные cache hits и ускорение пока не измерены.
Root/CI/Docker используют prepared consumers: `tsc --noEmit`, unit и `vite build`.
Отдельный CI `openapi:check` сохраняется: он генерирует временный контрольный
schema для drift comparison и не считается устранённым вызовом генератора.
`pnpm build:prepared` объединяет их без повторной генерации. Ручные
`generate:api`, `typecheck`, `build` сохраняют recovery generated outputs;
их вызов не объявляется полным generation-cache hit. Bridge сохраняет прежний
Prettier format и не перезаписывает идентичный output. Host generated files
исключены из Docker context и не затирают подготовленные внутри image types.

Tailwind сканирует явные app/index/Base UI roots, исключая tests/setup. Реальный
UI pin и product artifact проверяются отдельно от маленьких source fixtures.

`python scripts/build.py` выбирает только изменённые container producers;
`backend`/`frontend` выбирают конкретный контейнер, `--all` — оба. CLI producer
возвращает binary через Root, отдельного CLI container нет. `NO_BUILD_SERVICES_SELECTED`
не означает image ready. Legacy helper не владеет execution slot: raw heavy вызов
нужно обернуть общим Base slot; SDK/UI checkout verifier сохраняется. При разных
pins helper отказывает до Docker; пользоваться квалифицированным multi-pin route,
не переписывать pins ради запуска.

Следующие полные команды — отдельная согласованная ручная integration/release
веха, а не цикл после каждого изменения. `.github/workflows/release.yml`
(`workflow_dispatch`) сохраняет прежние четыре jobs, включая PostgreSQL/ignored,
namespace, packed consumer, themes и MSRV. Их `NOT_RUN` сообщается отдельно.

```bash
cd backend
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace -- --test-threads=1

cd frontend
pnpm typecheck
pnpm test:all
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

В ходе реализации выполнять scoped проверки. Полный checklist ниже относится
к согласованной интегрированной/manual вехе; необходимые security/data/live gates
сохраняются. Не повторять успешно проверенный graph без changed inputs.

- [ ] `cargo fmt --all -- --check` clean
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo test --workspace -- --test-threads=1` green
- [ ] `pnpm typecheck` clean
- [ ] `pnpm test:all` green на ручной полной вехе
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

Frontend dependency hooks: единственная policy — `allowBuilds.esbuild: true` в
`frontend/pnpm-workspace.yaml`. Source command: `python -B -m unittest scripts.tests.test_ci_contract_checks.PnpmLifecyclePolicyTests -q`.
