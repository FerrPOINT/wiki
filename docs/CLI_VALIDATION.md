# Проверка CLI Wiki

Проверено 2026-10-02 в изолированном task checkout `feat/cli-workflows`.

## Среда и обязательные gates

Ubuntu WSL, rustc 1.88.0 (6b00bc388 2025-06-23), Node 22.20.0 / pnpm 10.28.1, Python 3.12.3. Проверено после объединения с актуальным продуктовым `main`, с чистым опубликованным Services Base `c083783a37791e277db796361203884b87828a7d`, закреплённым в `.base-revision`. Принятая в `main` политика pinned Base, Cargo `--locked` и pnpm `--frozen-lockfile` сохранена. Base и исходные dirty checkout этой задачей не изменены.

Backend: fmt, workspace/all-target Clippy с `-D warnings`, workspace tests, отдельный MSRV check (`cargo check --locked --workspace --all-targets`), OpenAPI drift и release workspace — успешно. Workspace: **155 passed, 0 ignored**, 0 failed. Workspace выполнен последовательно, как предусмотрено workflow этого продукта.

```bash
cd backend
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace -- --test-threads=1
cargo build --locked --release --workspace
WIKI_TEST_DATABASE_URL=postgres://.../wiki_test cargo test --locked --workspace -- --test-threads=1
```

Docs validators и существующие CI-contract tests проходят. Frontend: install с `--frozen-lockfile`, OpenAPI check/compat с `origin/main`, tests, lint и build — успешно. Дополнительно проходят неизменность generated contracts/lockfile, packed Base consumer и effective themes (dark/gray/light) на built preview. Task Tracker дополнительно typecheck; Task Tracker/Wiki — предусмотренный format check. Frontend tests этого продукта: 174.

## Регрессии и проверенные сценарии

- Real API lifecycle расширен первым запуском и повтором всей `evidence add-file` с ключом длиной 128 байт: одинаковые evidence/attachment IDs, evidence count увеличивается только на один, скачанное содержимое совпадает. Изменение файла или metadata с прежним key возвращает JSON conflict 409. До исправления второй этап первой команды возвращал conflict.
- HTTP fixture проверяет восстановление после upload success/evidence failure: отдельные процессы повторяют те же stage keys, upload переиспользуется. Проверены SHA-256/trim/длина child keys и отказ до upload для недопустимого parent key.
- Самостоятельные document create/attachment upload передают явный ключ без изменения и сохраняют replay. Генерация ключей без аргумента не меняется; автоматических write retries нет.
- Workspace запускается последовательно с отдельной PostgreSQL 16 БД и явным WIKI_TEST_DATABASE_URL: все 16 PostgreSQL tests выполняются. CLI real API использует production handlers с memory backend; persistence/replay/access/FTS/pages проверяются отдельным PostgreSQL suite.

Существующие проверки file/stdin, pages, JSON/204, access/validation/conflict, transport timeout, credential redaction и download no-clobber сохраняются и проходят. Новые regressions воспроизвели замечания на исходной ветке, затем прошли после исправлений.

## Границы подтверждения

Проверки используют только fixture данные и собственные временные ресурсы. Постоянные Compose-группы, runtime images, volumes и production deployment не менялись. Windows native linking недоступен (`link.exe`); Rust gates выполнены в WSL. Новых endpoint, миграций или изменений Services Base нет. Эти проверки были выполнены до merge; статус main и живая приёмка приведены ниже. Deploy не выполнялся.

Описание команд, configuration, input/output/errors и ограничения: [CLI.md](CLI.md).

## После merge и приёмка sdlc1 — 2026-10-02

Merge commit: `4a85f6a7c1de82e0869528788710da258c355ffe`. [Post-merge CI](https://github.com/FerrPOINT/wiki/actions/runs/37000178443): docs/backend/frontend/minimum-rust — 4/4 success на этом точном SHA; содержимое merge tree совпало с reviewed head. CLI release binary собран отдельно с закреплённым Base; checksum и установка: [CLI_INSTALL.md](CLI_INSTALL.md).

На собственном space выполнены tree, применение существующего template, document create со stdin и replay, draft/publish, history/revision, stale base conflict, move, task/phase summaries и dossiers, link/file evidence, standalone upload, metadata/download/no-clobber, evidence filters и cursor continuation, document/evidence search. Первый evidence add-file с ключом 128 байт проходит; повтор с тем же файлом/metadata/key получает HTTP 409 на upload. Изменение файла также получает conflict. Это блокирует replay на принятом runtime image, хотя соответствующие main/CI regression tests зелёные. Read-only PAT получает 403. Собственные документы и space архивированы. Attachment/evidence, idempotency и audit rows сохраняются согласно текущему API: permanent deletion в пакет не входит. Новые глобальные templates не создавались.

Проверка выполнялась с временными Central Auth PAT, ограниченными тремя продуктами; read/write и read-only tokens отозваны после прогона. Значения tokens/credentials не сохранялись в логах или артефактах. Исходные dirty checkout, постоянные Compose-группы, runtime images/pins и volumes не изменялись. Fixtures использовали реальные API и PostgreSQL работающего sdlc1, но только собственные project/repository/space и файлы. Ошибки исправленного smoke (имя флага search, when: manual, начальный deployment status и вывод terminal wait) отделены от воспроизведённых отказов runtime.

**Статус:** CLI main/CI проверен; полная совместимость с текущим sdlc1 не принята. Требуется отдельная сверка/обновление backend до согласованных main-кандидатов и повтор блокирующих операций. Публичный release не объявляется готовым.
