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

Проверки используют только fixture данные и собственные временные ресурсы. Постоянные Compose-группы, runtime images, volumes и production deployment не менялись. Windows native linking недоступен (`link.exe`); Rust gates выполнены в WSL. Новых endpoint, миграций или изменений Services Base нет. Слияние PR в main и deploy не выполняются.

Описание команд, configuration, input/output/errors и ограничения: [CLI.md](CLI.md).
