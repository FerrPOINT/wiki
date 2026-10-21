# Выборочные проверки Wiki PDLC3

Дата: 2026-10-21. Общие правила принадлежат Base: `docs/BUILD_PERFORMANCE.md`
из `qualityTooling.path` в inventory workspace. Команды и исключения — Wiki.

## Штатный маршрут

Из корня workspace: `./check-workspace.ps1 -Profile Unit -Repository wiki`.
`-Profile Build` выбирает producers, `-PlanOnly -DetailedPlan` показывает выбор.
`-Component tooling -BaseRef HEAD` проверяет сам маршрут без Cargo/Vitest.
Standalone CI потребляет exact selector81467ded; Root packet остаётся a36cd64f.
SDK и UI разделены без обновления прежнего эффективного SHA19a7a381.

Docs не выбирают application producer. Один frontend не собирает backend;
frontend tests/setup сохраняют unit/static без Vite. Неизвестный input расширяет
scope. Backend inputs консервативны: новый cfg(test) projection сюда не переносится.
Domain units используют `cargo test --locked -p domain --lib`.
Service artifact собирает только `-p server --bin server`: оба runtime Dockerfile
поставляют этот binary. CLI/migrations/full workspace остаются ручным release
контрактом, а не удаляются из продукта. Native artifact reuse выключен до
квалификации именно этого peer tuple; dependency caches сохраняются.

## Подготовка и полная проверка

Rust unit/server/OpenAPI используют один automatic job и один toolchain/cache
setup. Frontend `build` выполняет units один раз; manual `test:all` передаётся
compile-only. Полный исходный CI сохранён в `release.yml` с `workflow_dispatch`:
PostgreSQL/ignored contract, workspace clippy/check/test, MSRV, OpenAPI,
packed consumer и themes. Live/security/data gates остаются обязательными.

Bridge generator вызывается postinstall и typecheck: удалять typecheck refresh
без проверки warm environment нельзя. Его repeated invocation пока не устранён;
implicit prebuild hook execution не измерено. Эта граница остаётся в плане,
без нового cache framework ради короткого parser.

## Граница доказательств

Source/static и route tests выполняются отдельно от native Cargo/Vitest/build,
hosted CI, manual release и live. Скорость компиляции и память приложения из
route tests не выводятся. Финальный readback и measurements сохраняет Base.
