# Проверка CLI Wiki

Проверено 2026-10-01 в отдельном task checkout `feat/cli-workflows`.

## Пройденные проверки

Среда: Ubuntu WSL, Rust 1.88.0, PostgreSQL 16 в отдельном временном кластере/БД. Services Base не изменён этой задачей.

Из `backend`:

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace -- --test-threads=1
WIKI_TEST_DATABASE_URL=postgres://... cargo test --locked -p api --test wiki_mvp wiki_postgres_ -- --test-threads=1
cargo build --locked --release --workspace
```

- Workspace: 151 passed, 0 failed. CLI: 21 unit, 5 binary, 5 subprocess/HTTP workflows и 1 real API lifecycle.
- Env-gated PostgreSQL tests в workspace без URL возвращаются без persistence проверки. Все 16 `wiki_postgres_` дополнительно выполнены с URL отдельной БД и прошли.
- Real API: space/tree, template apply/document create, draft/publish/history/revision/move/archive, task/phase dossier и evidence, summaries/search, standalone attachment upload/download.
- Два отдельных процесса CLI повторяют document create и attachment upload с одинаковым явным key; возвращается прежний ID. Изменённое содержимое файла с тем же key возвращает JSON conflict 409.
- Multipart hash regression: случайный boundary не меняет hash, filename/content type/content меняют; некорректный multipart отклоняется, upload больше 2 MiB сохраняет внешний configured limit.
- HTTP fixtures: token precedence flag→WIKI_TOKEN→SDLC_API_TOKEN, timeout, explicit key, file/stdin, pages/filters, JSON/empty success, error status/code/message/request ID, credentials redaction и atomic download/no-clobber.
- Проверены PostgreSQL persistence/rebuild, role matrix/revocation, idempotency replay, search FTS index, paged summaries и cursor stability.

## Границы подтверждения

CLI real API использует production handlers с memory backend; отдельный PostgreSQL suite проверяет постоянное хранилище. Existing memory API fixtures имеют общий process store и auth sessions: full workspace tests запускаются последовательно, как предписывает [TESTING.md](TESTING.md). Параллельный запуск дал три 401 из-за взаимной инвалидации сессий; последовательный полный запуск прошёл.

UI, Docker images и runtime окружения продуктов не изменялись. Windows native linking недоступен (`link.exe`), полные gates выполнены в WSL. Совместимость старых сохранённых multipart idempotency keys и остальные ограничения описаны в [API.md](API.md) и [CLI.md](CLI.md). Новых document delete/restore и evidence edit нет; соответствующий API отсутствует.

Push, merge и deploy не выполнялись. Исходные незакоммиченные работы сохранены в исходных checkout.
