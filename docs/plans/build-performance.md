# Быстрые повторные проверки и сборка

Два существующих Rust test-only файла `backend/domain/src/value_objects/tests.rs`
и `backend/domain/src/wiki/tests.rs` сохраняют все13domain units и cfg(test)
подключения. Оба backend recipes копируют их в unit stage и исключают только из
production COPY; descriptor использует те же точные artifact exclusions и
`backend_unit_inputs`. Изменение этих файлов выбирает backend units без
application build; production owners `value_objects.rs`/`wiki.rs` сохраняют build.
Docker COPY использует [штатный --exclude](https://docs.docker.com/reference/dockerfile/#copy---exclude).
Source regression не заменяет native unchanged/unit-only/failed-unit gate.

Существующий CI contract module является pure tooling component. Он исполняется
один раз в ordinary CI и выбирает собственный `tooling:build-contracts` через
installed selector, без frontend/backend producers. SDK/quality pins сохранены.

Ordinary backend CI после workspace build запускает
`${CARGO_TARGET_DIR:-target}/debug/openapi-gen` с прежним output path.
Обязательный schema diff сохранён; готовый generator не компилируется повторно.

Применить [стандарт Base](https://github.com/FerrPOINT/services-base/blob/197ead9d9b6cf6901e8f9dd31f43f719885502c3/docs/BUILD_PERFORMANCE.md) без изменения runtime и SDK/UI/skills pins.
Product-owned команды описаны в .quality-gate.json. При разработке/сборке —
unit/static; integration/slow/full coverage — вручную перед релизом.
Security/data/compatibility/live gates сохранены; reuse только для допустимых
deterministic exact-candidate проверок.
Проверить no-change, один компонент, dependency/toolchain change и corrupt output;
измерить duration и cache hit/miss. Сохранять прогретые caches и чистить own scratch.

Docker frontend: раздельные unit/compile stages, unit/static перед compilation;
TypeScript проверяется один раз в обязательном unit stage; compile получает
готовые generated outputs и запускает Vite без повторного `generate:api`.
production COPY исключает тесты. Frozen dependencies сохраняют общий совместимый
pnpm download store. Parent digests закреплены; Dockerfile frontend, общий RUN slot и
ограничения Windows producers определяет стандарт Base.
Runtime и package build scripts сохраняют прежние публичные команды.

Frontend CI handoff: один `dist` с manifest/digests, затем verified nginx packaging.
Команда self-check и product flags: [TESTING](../TESTING.md#передача-frontend-artifact). SDK/UI/skills pins сохраняются.

Shell unit использует QueryClientProvider с actor-scoped workspace-catalog cache;
wrapper сохраняется и при rerender. Namespace mode определяет ожидаемый context
slot. Native qualification проверяет false/true без сетевых запросов; production
source, UI pin и release набор не меняются.

Отдельные test setup и `vitest.config.ts` исключены из production COPY
и `frontend:build` inputs, но остаются входами unit/static. Только эти явно
объявленные пути допускают пропуск CI compilation; общий `vite.config.ts`
с aliases/plugins остаётся полноценным build input. Ошибка unit блокирует
получение нового образа даже при сохранённой production compilation.

Docker installation получает manifest без `scripts` через отдельный cheap
BuildKit stage и content-addressed COPY. Установка выполняется с
`--frozen-lockfile --ignore-scripts`, сохраняет lock/workspace и настоящий Base
package. Полный product manifest возвращается перед проверками и compilation.
OpenAPI postinstall выполняется один раз в отдельном generated stage; spec,
generator и полный manifest определяют его cache key. Непустые outputs
проверяются и копируются после frontend sources в unit/application stages.
Изменение scripts/spec не запускает повторный install; отсутствие output или
ошибка generation блокируют результат, включая прогретый compile cache.

Vitest ограничен четырьмя workers. Это предел параллелизма, не RAM quota;
изменение только vitest config проверяет unit и сохраняет application artifact.

CI Rust target после restore готовится через `python3 ../scripts/vendor/base_rust_target_cache.py` из `backend/`; версия и checksum включены в отдельный Base quality packet.
