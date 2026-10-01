# Проверка CLI Wiki

Проверено 2026-10-01 в отдельном task checkout `feat/cli-workflows`.

## Пройденные проверки

Среда: Ubuntu WSL, Rust stable 1.98.0, PostgreSQL 16 в отдельном временном кластере/БД, Node 22.23.3, pnpm 10.28.1, Python 3.12.3. Чистый опубликованный Services Base `main`: `c008bec701086d4f9201180ea5451f64e88ab519`; политика зависимости от `main` сохранена. Backend snapshot сверён с task checkout. Services Base не изменён этой задачей.

Из `backend`:

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
WIKI_TEST_DATABASE_URL=postgres://.../wiki_test cargo test --locked --workspace -- --test-threads=1
cargo build --locked --release --workspace
```

- Workspace: 152 passed, 0 failed. CLI: 21 unit, 5 binary, 7 subprocess/HTTP workflows и 1 real API lifecycle.
- Полный workspace suite запускается с явным URL отдельной PostgreSQL БД: все 16 `wiki_postgres_` действительно выполняются и проходят, а не возвращаются без проверки persistence.
- Real API: space/tree, template apply/document create, draft/publish/history/revision/move/archive, task/phase dossier и evidence, summaries/search, standalone attachment upload/download.
- Два отдельных процесса CLI повторяют document create и attachment upload с одинаковым явным key; возвращается прежний ID. Изменённое содержимое файла с тем же key возвращает JSON conflict 409.
- Multipart hash regression: случайный boundary не меняет hash, filename/content type/content меняют; некорректный multipart отклоняется, upload больше 2 MiB сохраняет внешний configured limit.
- HTTP fixtures: token precedence flag→WIKI_TOKEN→SDLC_API_TOKEN, timeout, explicit key, file/stdin, pages/filters, JSON/empty success, error status/code/message/request ID, credentials redaction и atomic download/no-clobber.
- Проверены PostgreSQL persistence/rebuild, role matrix/revocation, idempotency replay, search FTS index, paged summaries и cursor stability.

## Границы подтверждения

CLI real API использует production handlers с memory backend; отдельный PostgreSQL suite проверяет постоянное хранилище. Existing memory API fixtures имеют общий process store и auth sessions: full workspace tests запускаются последовательно, как предписывает [TESTING.md](TESTING.md). Последовательность сохраняет изоляцию fixtures от взаимной инвалидации сессий.

UI, Docker images и runtime окружения продуктов не изменялись. Windows native linking недоступен (`link.exe`), полные gates выполнены в WSL. Совместимость старых сохранённых multipart idempotency keys и остальные ограничения описаны в [API.md](API.md) и [CLI.md](CLI.md). Новых document delete/restore и evidence edit нет; соответствующий API отсутствует.

Ветка подготовлена для отдельного PR в `main`; merge и deploy не входят в пакет. Исходные незакоммиченные работы сохранены в исходных checkout.

## Дополнительные gates перед PR

- CLI suite после ревью: 34 tests, включая новый случай обрезанного credential в API error; stable Clippy проходит без подавления lints.
- Docs: 5 Python tests и README structural validation на Python 3.12.3; существующие CI-contract checks сохраняют PostgreSQL service, явный URL и последовательный workspace test.
- Frontend: install с `--no-frozen-lockfile`, OpenAPI check/compatibility с `origin/main`, 24 files / 182 tests, lint, format check и build — успешно. Проверены Git blobs с LF; Windows checkout CRLF не используется для Linux format gate.
- OpenAPI drift и release build проверены на том же опубликованном Base main.
- Новые CLI tests входят в существующий workspace gate; отдельный параллельный PostgreSQL запуск в CI не добавляется.
- Changelog `[Unreleased]`, CLI/API/Architecture/validation документы соответствуют diff. Production runner/deploy и постоянные Compose-стенды не запускаются.

Проверка исходной справки CLI выявила вывод значения token env variable в `--help`. В итоговой ветке `hide_env_values` скрывает значение, сохраняя имя переменной; subprocess regression выполняется с заданным fixture token и проверяет stdout/stderr. После этого изменения повторены CLI tests, Clippy и release build CLI; API/backend fixtures не меняются.
