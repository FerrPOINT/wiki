# Проверка CLI Wiki

Проверено 2026-10-02 в изолированном task checkout `feat/cli-workflows`.

## Среда и обязательные gates

Ubuntu WSL, rustc 1.98.0 (88d9e12ae 2026-08-18), Node 22.23.3 / pnpm 10.28.1, Python 3.12.3. Чистый опубликованный Services Base `main`: `69bd8ef0fe424c2018bcdc509ddd25f7fce02a7e`. Политика зависимости от `main` сохраняется; Base и исходные dirty checkout не изменены этой задачей. Product lockfile обновлён под изменившиеся зависимости опубликованного Base без обновления registry versions. Wiki дополнительно использует существующие workspace sha2/hex для составных ключей.

Backend: fmt, workspace/all-target Clippy с `-D warnings`, workspace tests, OpenAPI drift и release workspace — успешно. Workspace: **155 passed, 0 ignored**, 0 failed. CI/CD дополнительно проверен штатным параллельным workspace invocation; Task Tracker и Wiki — последовательным invocation их workflows.

```bash
cd backend
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace -- --test-threads=1
cargo build --locked --release --workspace
WIKI_TEST_DATABASE_URL=postgres://.../wiki_test cargo test --locked --workspace -- --test-threads=1
```

Docs validators и существующие CI-contract tests проходят. Frontend: install с `--no-frozen-lockfile`, OpenAPI check/compat с `origin/main`, tests, lint и build — успешно. Task Tracker дополнительно typecheck; Task Tracker/Wiki — предусмотренный format check. Frontend tests: Task Tracker 253, CI/CD 191, Wiki 182.

## Регрессии и проверенные сценарии

- Real API lifecycle расширен первым запуском и повтором всей `evidence add-file` с ключом длиной 128 байт: одинаковые evidence/attachment IDs, evidence count увеличивается только на один, скачанное содержимое совпадает. Изменение файла или metadata с прежним key возвращает JSON conflict 409. До исправления второй этап первой команды возвращал conflict.
- HTTP fixture проверяет восстановление после upload success/evidence failure: отдельные процессы повторяют те же stage keys, upload переиспользуется. Проверены SHA-256/trim/длина child keys и отказ до upload для недопустимого parent key.
- Самостоятельные document create/attachment upload передают явный ключ без изменения и сохраняют replay. Генерация ключей без аргумента не меняется; автоматических write retries нет.
- Workspace запускается последовательно с отдельной PostgreSQL 16 БД и явным WIKI_TEST_DATABASE_URL: все 16 PostgreSQL tests выполняются. CLI real API использует production handlers с memory backend; persistence/replay/access/FTS/pages проверяются отдельным PostgreSQL suite.

Существующие проверки file/stdin, pages, JSON/204, access/validation/conflict, transport timeout, credential redaction и download no-clobber сохраняются и проходят. Новые regressions воспроизвели замечания на исходной ветке, затем прошли после исправлений.

## Границы подтверждения

Проверки используют только fixture данные и собственные временные ресурсы. Постоянные Compose-группы, runtime images, volumes и production deployment не менялись. Windows native linking недоступен (`link.exe`); Rust gates выполнены в WSL. Новых endpoint, миграций или изменений Services Base нет. Merge и deploy не выполняются.

Описание команд, configuration, input/output/errors и ограничения: [CLI.md](CLI.md).
