# Testing Strategy - Wiki

## Docker: раздельные входы проверок и компиляции

`frontend/Dockerfile` и `Dockerfile.umbrella` разделяют dependency install,
unit/typecheck и Vite compilation. Изменение scripts в package.json не меняет
dependency manifest; frozen install использует общий pnpm download store по
версии Node/архитектуре. Lifecycle scripts установки не запускаются неявно.
OpenAPI и flat-type bridge создаются один раз в `generated`; оба outputs
копируются после исходников в unit и production stages.
Unit stage читает тесты и config; production COPY исключает test/spec/setup,
Vitest config и e2e. Успешный unit-маркер обязателен до Vite. Итоговый nginx
получает тот же dist; прежние Node/UI paths, VITE defaults и runtime сохранены.
Recipe-specific `.dockerignore` ограничивает context этим frontend/OpenAPI
и нужным UI export, сохраняя поздние исключения private state и dependencies.
Context должен содержать UI source принятого pin в указанном recipe layout;
source checkout другого UI pin не заменяет такой export.

Это source-этап. Холодный install, реальный cache hit, failed-unit publication,
Vite/dist handoff и затраты CPU/RAM проверяются на общей native-вехе. До неё
ускорение в секундах и production readiness не заявляются. Общий контракт —
[Base BUILD_PERFORMANCE](../../services-base/docs/BUILD_PERFORMANCE.md).

## Production CSS source graph

Node 22.20.0, UI `@sdlc/ui` и его source layout сохранены.
Tailwind читает application src, index.html и прежний UI source; test/spec/setup/tooling исключены.
`pnpm test:unit` сначала выполняет два scanner/compiler controls, затем прежние units.
Ручной `pnpm test:all` включает те же Node controls перед прежним полным Vitest.
Отдельно из frontend с готовыми dependencies: `node --test scripts/tailwind-source-inputs.test.mjs`.
Fixture-only изменение не выбирает application artifact в quality/CI; реальный CSS сохраняет Build.
Это source checks, полный Vite/dist и native/container acceptance выполняются отдельно на общей вехе.
Общий стандарт — [Base BUILD_PERFORMANCE](../../services-base/docs/BUILD_PERFORMANCE.md).


## Bounded frontend unit workers

The ordinary `pnpm test:unit` command in frontend uses an explicit maximum of
four Vitest workers. Isolation, jsdom, test inventory and manual tiers remain.
The same installed Node 22.20.0/pnpm 10.28.1/Vitest 4.1.11 environment passed
187 cases at both two and four workers (three samples each). Command medians
were 30.942 s and 17.539 s respectively; four is the faster measured bounded
profile. This is not a comparison against the unbounded historical default,
and private committed job memory is not an application RAM requirement.
Existing compatible in-place dependencies can serve this native unit command;
this does not issue a managed Base environment, unit receipt or artifact grant.
Integration/slow/release and application production builds remain separate.


## Scoped unit/static и полный release

Из корня PDLC3 обычные проверки выбирают только нужный продукт и компонент:

```powershell
./check-workspace.ps1 -Profile Unit -Repository wiki -Component tooling -BaseRef HEAD
./check-workspace.ps1 -Profile Build -Repository wiki -BaseRef HEAD -PlanOnly
```

`.quality-gate.json` описывает docs/frontend/backend/tooling; неизвестный путь
расширяет выбор до всего продукта. Frontend test-only оставляет unit/static без
Vite build. Backend inputs сохраняются консервативными: разделение `cfg(test)` и
native/artifact reuse этим изменением не квалифицированы.

Обычные frontend-команды: `pnpm --dir frontend test:unit` и
`pnpm --dir frontend build` (один unit run перед компиляцией). Полный прежний
Vitest inventory доступен через `pnpm --dir frontend test:all`; unit исключает
только явно названные integration/slow/e2e suites, не удаляя их из release.
Backend fast lane: `cargo test --locked -p domain --lib` из `backend`.

## Повторная frontend generation

`postinstall` и явный `pnpm --dir frontend generate:api` сохраняют OpenAPI generation
и flat-type bridge. `build` и `typecheck` проверяют актуальное содержимое bridge;
если результат совпадает, generator не переписывает `generated-exports.ts` и не
меняет его mtime. Missing/corrupt output восстанавливается; changed schema с более
старым timestamp всё равно меняет aliases. Ошибочный input не затирает прежний output.
Сравнение читает не больше размера ожидаемого output плюс один byte: большой
повреждённый artifact не загружается в память целиком.

Дублирующий `prebuild` удалён: `build` уже вызывает тот же generator перед tsc/Vite.
Это работает при включённых и выключенных pnpm pre/post hooks. Команды, версии
Node22.20.0/pnpm10.28.1, unit-before-build и отдельные manual suites сохранены.
Политика dependency environment и generation принадлежит стандарту Base; script-only
изменение проверяется отдельным generation key без повторной установки тех же dependencies.

Автоматический CI использует standalone Base selector, один Rust 1.88.0 job и
условные application checks/builds. Полный прежний CI запускается вручную через
`.github/workflows/release.yml` (`workflow_dispatch`): PostgreSQL, full workspace
и minimum Rust all-targets, ignored central-directory projection, OpenAPI,
packed Base consumer и effective themes сохраняются. Полный frontend test run
передаётся compile-only step без второго unit run. UI pin `.base-ui-revision`
равен прежнему эффективному SDK SHA; frontend использует его независимо от
backend `.base-revision`. Это не обновление dependencies и не release proof.

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

## 6. Manual Release Checklist

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
проверяет закрытый DB pool и неполные claims отдельно. Ручной release workflow явно запускает
mixed_central_directory_preserves_identity_and_disabled_state_in_postgres
на своём PostgreSQL; workspace test не исполняет ignored-тест автоматически.

Scoped entrypoint, сохраняемые release gates и ограничения: [build performance](plan/2026-10-21-build-performance.md).


## Повторная генерация API bridge

`node frontend/scripts/generate-exports.mjs` публикует `generated-exports.ts`
только при изменении фактических bytes. В PDLC1 применён существующий bounded
content comparison PDLC3; формат output и оба Base pins сохраняются. Повторный
parse и перезапись различаются: generator может проверить вход, не меняя inode/mtime
готового bridge. Missing/corrupt output восстанавливается, старый timestamp
изменённого schema не скрывает новое имя, parse failure сохраняет прежний output.

`pnpm --dir frontend test:unit` и `test:all` теперь включают существующие Node
controls для bridge. Изменение только `frontend/scripts/tests/generate-exports.test.mjs`
выбирает frontend tests без production artifact/backend. Mixed изменение самого
generator сохраняет application build. Изолированная быстрая команда без установки
зависимостей: `node --test frontend/scripts/tests/generate-exports.test.mjs`.

На копии actual PDLC1 `generated.ts` три warm CLI invocations до исправления
перезаписывали output3 раза, после —0; содержимое совпало. Медианы отдельного
helper0,107/0,096с, samples3; это не benchmark полной сборки, CPU/RAM приложения
не измерялись. По6 Node controls PASS в PDLC1/3; PDLC2 сохраняет свой formatter
и уже подключённые controls. Полный frontend gate остаётся на согласованной вехе.

Frontend dependency hooks: единственная policy — `allowBuilds.esbuild: true` в
`frontend/pnpm-workspace.yaml`. Source command: `python -B -m unittest scripts.tests.test_ci_contract_checks.PnpmLifecyclePolicyTests -q`.
