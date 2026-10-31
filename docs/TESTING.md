# Testing Strategy - Wiki

Обычный `just test` использует reviewed domain units и `pnpm test:unit`.
Полный backend-набор вынесен в явный ручной `just test-backend-full`.
`just build` без scope отказывает; выбрать `build-backend`,
`build-frontend`, `build-cli` либо намеренный `build-all`.
Backend/CLI packaging зависит от того же unit prerequisite;
в одном полном вызове Just он выполняется один раз.
Настоящие Rust/Node/Just qualification и release/live — отдельная веха.

В automatic CI совместимые Rust 1.88.0 minimum и backend проверки используют
одну подготовку в job `backend`. Прежний `minimum-rust` проверяет его успешный
результат без второго compiler; unit failure останавливает дальнейшую работу.
Команды, environment exception и открытая проверка времени:
[контракт Base](../../services-base/docs/plans/rust-ci-shared-preparation-20261030.md).

## Prepared generator inputs

LF-правило frontend-генератора находится в `frontend/.gitattributes`, рядом с
его потребителем. Общие backend rules остаются в корневом `.gitattributes`;
frontend-only добавление не должно менять native backend input key. При переносе
rules сравнивать `git check-attr --all -z --stdin` для tracked paths и соседних
новых путей, сохраняя фактические source bytes и все проверки artifact identity.

Wiki принимает quality tooling `f910017264a730089b38c11ee8ee53b29eed8fe2`
независимо от SDK/UI/skills/operator pins. В source workspace используйте
`check-workspace.ps1 -Profile Fast -Repository wiki -Component frontend` из
корня после `enter-dev.ps1`; подготовка и проверка используют один environment.
Профиль `frontend_generation` в `.quality-gate.json` сохраняет postinstall,
его hooks и actual environment; test-only/no-change не повторяют генерацию.

Квалифицированы OpenAPI local fragments, существующий exports helper и
отсутствие четырёх cwd Redocly configs. Изменение helper, generator toolchain,
external refs или добавление config/plugins требует проверки нового closure.
`FrontendGenerationContractTests` в CI docs/tooling gate проверяет эту границу;
не обновлять approved hash вместо исследования новых imports/reads. Base
проверяет новые inputs и generated outputs, неизвестные cache-only configs
сохраняются с отказом. Общий контракт — [Base delivery standard](https://github.com/FerrPOINT/services-base/blob/f910017264a730089b38c11ee8ee53b29eed8fe2/docs/plans/frontend-generation-delivery-20261026.md).
Standalone CI с новым dependency environment выполняет необходимый postinstall;
этот профиль не объявляет CI dependency installation готовым локальным cache hit.

## Production CSS source graph

Node 22.20.0, UI `@sdlc/ui` и его source layout сохранены.
Tailwind читает application src, index.html и прежний UI source; test/spec/setup/tooling исключены.
`pnpm test:unit` сначала выполняет два scanner/compiler controls, затем прежние units.
Ручной `pnpm test:all` включает те же Node controls перед прежним полным Vitest.
Отдельно из frontend с готовыми dependencies: `node --test scripts/tailwind-source-inputs.test.mjs`.
Fixture-only изменение не выбирает application artifact в quality/CI; реальный CSS сохраняет Build.
Это source checks, полный Vite/dist и native/container acceptance выполняются отдельно на общей вехе.
Общий стандарт — [Base BUILD_PERFORMANCE](../../services-base/docs/BUILD_PERFORMANCE.md).


Pure unit suites for api-error/search-scope/space-selection, raw nginx config
and the two mocked Wiki API query suites use explicit Vitest Node environment.
Their imports need no browser. Auth store, real client and UI suites retain the
default jsdom environment; unknown suites are not moved automatically. Browser
matchers, i18n and storage initialize only in the browser setup branch.
Node/browser controls guard that boundary; isolation and four workers remain.
Run only affected suites with `pnpm exec vitest run <test-paths>` from frontend,
or use root `check-workspace.ps1 -Profile Fast -Repository wiki -Component frontend`.
Use `-BaseRef HEAD -PlanOnly -DetailedPlan` for the current dirty delta; after
commit select an explicit component or the intended comparison range. Empty
diff is not test evidence. App build, integration and release stay separate.

Known Python helper tests (`test_build_pin`, `test_verify_readme`,
`test_tooling_ownership`) select only `tooling:python-helpers` in Fast/Unit/Build.
The existing CI-contract test keeps `tooling:build-contracts`; unknown scripts
retain full product fallback. Root preview:
`check-workspace.ps1 -Profile Unit -Repository wiki -Component tooling -PlanOnly`.
Explicit tooling selects both lightweight gates. From Wiki, run helpers with
`python -B -m unittest scripts.tests.test_build_pin scripts.tests.test_verify_readme scripts.tests.test_tooling_ownership -q`.
See [source qualification](plans/python-tooling-test-selection-20261024.md).

Актуальная политика 2026-10-15: unit/static запускаются при обычной сборке;
integration/slow/full coverage и browser/live — вручную перед релизом.
Команды быстрых уровней приведены в разделе «Тестовые уровни (2026-10-15)» ниже.
Прежняя полная матрица и тяжёлые команды этого документа сохранены как
ручные release checks; они не запускаются автоматически после каждой правки.

Windows backend использует declared Base adapter `rust-compose-v1` и тот же
`cargo fmt --all -- --check; cargo test --locked -p domain --lib` в pinned Linux
image. Из root выбирать только Wiki backend:

```powershell
./check-workspace.ps1 -Profile Unit -Repository wiki -Component backend `
  -NativeDockerContext desktop-linux `
  -RustNativeInputs $wikiNativeReference.path `
  -RustNativeInputsSha256 $wikiNativeReference.sha256
```

Reference выдаётся tooling owner и связывает original checkout с уже имеющимися
SDK/tools exports; не создавать второй SDK export ради имени каталога.
Общий cache/slot/receipt и порядок reuse принадлежат [Base](../../services-base/docs/BUILD_PERFORMANCE.md).
Backend-only не запускает frontend; Build сначала требует Unit и передаёт один
проверенный `server` artifact дальше. Лимит 2 CPU / 1024 MiB — исходный профиль
пилота, не измеренное потребление и не утверждение о самом быстром варианте.
Native qualification фиксируется отдельно от source controls в Base; release,
integration, PostgreSQL и live gates остаются ручной вехой.

Docker target `unit` в обеих backend recipes имеет отдельный dependency graph:
`manifests → unit-dependencies → unit`. Он не наследует компиляцию server.
Полная сборка расширяет тот же immutable слой до `dependencies`, а `builder`
проверяет настоящий unit marker перед production compilation. Registry cache
и общий RUN slot сохранены; writable compiled target между экспортами не добавлен.
Source guards графа запускаются без Cargo:
`check-workspace.ps1 -Profile Unit -Repository wiki -Component tooling` из root.
Изменение recipes требует отдельной qualification нового tuple на финальной
вехе; source tests не являются замером CPU/RAM, новым проходом 13 domain units
или приёмкой готового image. Старый N4 proof остаётся историческим.

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

## Проверка generated outputs в CI

`postinstall` выполняется при frozen install с `--ignore-scripts=false`.
Текущие outputs: `src/api/generated.ts`, `src/api/generated-exports.ts`; они намеренно
игнорируются Git. Поэтому `git diff` не доказывает их актуальность, а требование
tracked output было ошибкой CI. Git проверяет только tracked lockfile; для outputs
проверяются ignore policy, обычный непустой файл без symlink и прежний
`pnpm openapi:check`. Проверки `test` записаны отдельными командами: failed
non-final `test` внутри `&&` не гарантирует остановку Bash с `-e`.

Повторный compiler в drift check здесь имеет конкретную причину: нет переданного
проверяемого результата generation для ignored outputs. Удалять его можно после
подключения квалифицированного postinstall handoff существующего Base owner,
с сохранением input/output identity и corruption controls. Новый cache не нужен.
Это не разрешает повторный application build или install. Backend schema drift,
OpenAPI compatibility и ручной release gate сохраняются. CI contract проверяется
штатным `base_frontend_artifact.py check-ci` до подготовки приложения.

## Точные Rust artifacts обычной сборки

`backend:build` в `.quality-gate.json` выбирает только package/bin targets
поставляемого runtime: `server`.
Outputs именованы явно; отсутствие любого required executable завершает gate
ошибкой. Случайный соседний executable не попадает в artifact.
CLI `wiki`, migration и OpenAPI остаются отдельными consumers; их workspace/release проверки сохранены.
`backend:quality` сохраняет полные workspace clippy/tests в ручном release;
MSRV/features/OpenAPI и необходимые integration gates не сокращаются.
Fresh target между source exports и registry/compiler-cache policy Base сохраняются.

Изменение только `scripts/verify_readme.py` выбирает docs/tooling: Fast выполняет
README validator и Python helper units, Unit/Build — только helper units.
Application producers для этого пути не выбираются. Неизвестные scripts и
`scripts/build.py` сохраняют полный product fallback; соседний backend input
по-прежнему включает backend checks/build. Public plan допускает
`--changed-file scripts/verify_readme.py`; root `-Repository wiki -Component tooling`
проверяет весь явно выбранный tooling scope. Общие правила — стандарт Base выше.


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

## Scoped standalone container helper

`python scripts/build.py` использует existing Base plan относительно HEAD.
Явный service задаёт один scope; `--all` выбирает все buildable services.
Пустой clean/docs/test-only выбор не вызывает verifier/Docker и не является
readiness/artifact proof. Cold bootstrap выбирает service явно. Packaging
recipes/nginx имеют отдельную карту; unknown paths сохраняют product fallback.
Invalid ref или unsupported map отказывают до подготовки. SDK/UI pins независимы;
несовместимый single-context mixed build требует qualified multi-pin route.
Raw heavy остаётся под общим host slot, Root owner вторым slot не оборачивать.
Source команда: `python -B -m unittest scripts.tests.test_build_scope -q`.
Контракт/границы: [план](plans/container-selection-20261030.md).
