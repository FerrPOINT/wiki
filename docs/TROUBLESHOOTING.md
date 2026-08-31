# Troubleshooting

## 1. Сборка и запуск

### `cargo build` падает с ошибкой линковки

- Linux: убедиться, что установлены dev-зависимости `libssl-dev pkg-config`.
- Windows MSVC: установить Visual Studio Build Tools / Windows SDK, чтобы был доступен `link.exe`.
- Проверить версию Rust: `rustc --version` ≥ 1.86.

### Frontend dev-сервер не стартует

- Проверить Node.js: `node --version` ≥ 22.
- Удалить `node_modules` и lockfile: `rm -rf node_modules pnpm-lock.yaml`, затем `pnpm install`.
- Проверить, что порт 5173 не занят для dev server: `lsof -i :5173`.

### Docker compose не поднимается

```bash
docker compose down -v
docker compose pull
docker compose up -d --build
```

## 2. База данных

### Миграции не применяются

```bash
cd backend
DATABASE_URL=postgres://wiki:[CHANGE_ME]@localhost:3457/wiki cargo run -p migration -- status
# target after SQLx migration replacement:
sqlx migrate info --source backend/migrations
```

Если застряло — откатить вручную:

```bash
sqlx migrate revert
```

### Connection refused to postgres

- Проверить, что контейнер postgres healthy: `docker compose ps`.
- Проверить `WIKI_DATABASE__URL` — хост должен быть `localhost` для локального запуска, `postgres` для docker.
- Проверить credentials в `.env`.

### Медленные запросы

```sql
SELECT query, mean_exec_time, calls
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;
```

См. `docs/DATABASE_INDEXES.md`.

## 3. Redis

### Redis connection refused

- `docker compose ps` — redis healthy?
- Проверить, что не путаете host `redis` vs `localhost`.

Current Wiki API shell should not require Redis for normal reads/writes. Target cache/idempotency behavior may degrade until Redis is restored.

## 4. Auth

### Access token rejected

- Проверить TTL (по умолчанию 15 минут).
- Проверить `Authorization: Bearer <token>`.
- Проверить `WIKI_JWT_SECRET` — должен совпадать у сервера, выпустившего токен.

### Refresh cookie не приходит

- Проверить `Secure` flag — в локальной HTTP-среде может быть выключен.
- Проверить `SameSite=Lax`.
- См. `docs/SECURITY.md`.

## 5. API

### 422 Validation Error

- Тело ответа содержит список полей и ошибок.
- Проверить required поля и формат UUID.

### 409 Conflict

- Чаще всего duplicate key (unique constraint).
- Проверить комбинации: `space_key`, `document_slug`, `task_key`, `phase_key`, `email`, `username`.

### 429 Too Many Requests

- Проверить заголовки `X-RateLimit-*`.
- Подождать или использовать `Idempotency-Key`.

## 6. Frontend

### Белый экран после сборки

- Открыть DevTools → Console.
- Проверить, что `VITE_API_BASE_URL` доступен.
- Проверить 404 на `index.html` — настройка SPA fallback.

### Tailwind стили не применяются

- `pnpm dev` перезапустить.
- Проверить `@import "tailwindcss"` в `frontend/src/styles/index.css`.

### Проблемы с i18n

- Проверить, что JSON-локали в `frontend/src/i18n/locales/`.
- Проверить fallback locale (`ru`).

## 7. Тесты

### Playwright flaky

```bash
pnpm exec playwright install --with-deps
pnpm exec playwright test --workers=1 --retries=2
```

### Cargo тесты падают на DB

- Убедиться, что `TEST_DATABASE_URL` настроен (обычно отдельная DB `wiki_test`).
- Запускать миграции перед тестами.

## 8. Диагностика

### Health checks

```bash
curl http://localhost:3456/api/v1/health
curl http://localhost:3456/metrics
```

### Логи

```bash
# backend
cargo run --bin server 2>&1 | jq

# docker
docker compose logs -f backend
```

## 9. References

- `docs/LOCAL_SETUP.md`
- `docs/DEPLOYMENT.md`
- `docs/DATABASE_INDEXES.md`
- `docs/SECURITY.md`
- `docs/ERROR_HANDLING.md`
- `docs/MONITORING.md`
- `docs/TESTING.md`
