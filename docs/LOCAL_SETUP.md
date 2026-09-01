# Local Setup

## 1. Требования

| Инструмент | Минимальная версия | Примечание |
|---|---|---|
| Docker + Compose | 24.x | для Postgres, backend/frontend containers and optional Traefik |
| Rust | 1.86+ | backend workspace |
| cargo | 1.86+ | backend workspace |
| Node.js | 22 LTS | frontend |
| pnpm | 9.x | frontend package manager |
| just | — | task runner (опционально) |
| git | 2.40+ | — |

## 2. Быстрый старт

```bash
git clone git@github.com:FerrPOINT/wiki.git /opt/dev/wiki
cd /opt/dev/wiki

cp .env.example .env
# отредактируй .env под себя

docker compose up -d postgres
```

`.env` читает Docker Compose. Если backend запускается с хоста через `cargo run`, передайте override-переменные в окружение процесса:

```bash
cd backend
export WIKI_JWT_SECRET=dev-secret-32-chars-minimum
export WIKI_DATABASE__URL=postgres://wiki:[CHANGE_ME]@localhost:3457/wiki
export WIKI_BOOTSTRAP__ADMIN_EMAIL=admin@example.com
export WIKI_BOOTSTRAP__ADMIN_PASSWORD=change-me-before-use
cargo run --bin server
```

```powershell
cd backend
$env:WIKI_JWT_SECRET = "dev-secret-32-chars-minimum"
$env:WIKI_DATABASE__URL = "postgres://wiki:[CHANGE_ME]@localhost:3457/wiki"
$env:WIKI_BOOTSTRAP__ADMIN_EMAIL = "admin@example.com"
$env:WIKI_BOOTSTRAP__ADMIN_PASSWORD = "change-me-before-use"
cargo run --bin server
```

Frontend:

```bash
cd frontend
pnpm install
pnpm dev
```

Frontend dev-сервер доступен по `http://localhost:5173`. Docker frontend публикуется на `http://localhost:19877`; backend API - `http://localhost:3456/api/v1`.

## 3. Переменные окружения

Основные для локальной разработки:

```env
WIKI_DATABASE__URL=postgres://wiki:[CHANGE_ME]@localhost:3457/wiki
WIKI_JWT_SECRET=[CHANGE_ME_32BYTES_MIN]
WIKI_STORAGE__DIR=/var/lib/wiki/uploads
WIKI_STORAGE__MAX_UPLOAD_BYTES=26214400
WIKI_BOOTSTRAP__ADMIN_EMAIL=admin@example.com
WIKI_BOOTSTRAP__ADMIN_PASSWORD=change-me-before-use
VITE_API_BASE_URL=http://127.0.0.1:3456/api/v1
```

Полный список — в `.env.example`.

## 4. Backend

```bash
cd backend

# Установка зависимостей
cargo build

# SQLx migrations are applied by backend startup when WIKI_DATABASE__URL is set.
# Manual migration checks use sqlx-cli:
DATABASE_URL=postgres://wiki:[CHANGE_ME]@localhost:3457/wiki sqlx migrate info --source migrations

# Запуск API сервера
export WIKI_JWT_SECRET=dev-secret-32-chars-minimum
export WIKI_DATABASE__URL=postgres://wiki:[CHANGE_ME]@localhost:3457/wiki
export WIKI_BOOTSTRAP__ADMIN_EMAIL=admin@example.com
export WIKI_BOOTSTRAP__ADMIN_PASSWORD=change-me-before-use
cargo run --bin server

# Запуск тестов
cargo test

# Запуск с watch
cargo watch -x run --bin server
```

## 5. Frontend

```bash
cd frontend

pnpm install
pnpm dev

# Типизация
pnpm typecheck

# Линтер
pnpm lint

# Тесты
pnpm test
pnpm test:e2e
```

## 6. Docker

```bash
# Всё через compose
docker compose up -d --build

# Только инфраструктура
docker compose up -d postgres

# Пересоздать контейнеры после изменений
docker compose build
docker compose up -d

# Логи
docker compose logs -f backend
```

## 7. Тестовые данные

После первого запуска:

```bash
# Автосоздание admin пользователя из .env происходит при старте backend,
# если заданы WIKI_BOOTSTRAP__ADMIN_EMAIL и WIKI_BOOTSTRAP__ADMIN_PASSWORD.
docker compose up -d --force-recreate backend

# Seed demo-проекта и задач (опционально)
./scripts/seed-demo.sh
```

## 8. IDE

Рекомендуемые расширения:

- Rust Analyzer
- Tailwind CSS IntelliSense
- ESLint
- Prettier
- GitLens
- Docker

## 9. Частые проблемы

| Проблема | Решение |
|---|---|
| Порт 19877 занят | Изменить frontend port mapping в `docker-compose.override.yml` |
| Порт 3456 занят | `WIKI_SERVER__PORT` в `.env` / `docker-compose.override.yml` |
| Postgres не стартует | `docker compose down -v` и пересоздать volume |
| `cargo` долго компилирует | `sccache` + `cargo nextest` |

Больше диагностики — в `docs/TROUBLESHOOTING.md`.

## 10. Pre-commit

```bash
# Установить hooks (после создания)
just install-hooks
# или
pre-commit install
```

## 11. References

- `.env.example`
- `docker-compose.yml`
- `docs/DEPLOYMENT.md`
- `docs/TESTING.md`
- `docs/TROUBLESHOOTING.md`
- `docs/CODE_STYLE.md`
- `docs/AGENTS.md`
