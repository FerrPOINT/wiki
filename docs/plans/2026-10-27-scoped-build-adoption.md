# Wiki PDLC2: выбор только необходимой сборки

Baseline `8a773b6`. Общие правила принадлежат Base BUILD_PERFORMANCE/TESTING;
SDK/UI/quality pins независимы. Rust1.88/Node22.20/pnpm10.28.1 сохраняются.

1. Подключить product descriptor и exact canonical selector, игнорировать own
   `.local` без удаления данных. Docs/test-only не создают application artifact;
   неизвестные inputs расширяют продукт. Обычная команда выбирает изменения,
   полный обход задаётся явно.
2. Production targets — отдельные server и официальный Wiki CLI. CLI сохраняется
   в поставке и полном manual workspace gate, не добавляется в server build без
   consumer edge. Quick domain --lib сохраняется; CLI rustfmt не выдаётся за unit.
3. CI использует тот же exact candidate head и Base plan. Все прежние jobs,
   PostgreSQL/ignored/namespace/compatibility/MSRV/packed/theme/security checks
   сохраняются в ручном release. Type/unit/compile выполняются по одному разу.
4. Сохранить postinstall как подготовку API/bridge. Уже подготовленные Root/CI/
   Docker consumers не повторяют generator. Явные generate:api/typecheck/build
   остаются recovery/manual entrypoints. Bridge не переписывает равный output.
5. Перед test-only exclusion сделать Tailwind source graph явным и проверить
   test/setup/tooling negative плюс app/index/UI-fixture positive case.
6. Убрать unsafe shared compiled target между exports, сохранить download/layer
   caches и bounded workers. Ограничить recipe contexts прежним COPY closure.
7. Scoped source checks и штатные Root no-change/unit routes; native compile/
   Vitest/Vite/Docker, artifact reuse/invalidation, resource/visual/live gates —
   согласованная финальная веха. До неё application cacheable=false.

Raw container helper сохраняет проверку SDK/UI и отказывает mixed-pin контексту
до Docker, если один checkout не может доказать оба pins. Ordinary Root candidate
routes и exact native caller adoption квалифицируются отдельно. Runtime,
production, соседние ресурсы и полезные caches не меняются.
