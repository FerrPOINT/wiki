# Deployment — Wiki

## 1. Overview

Self-hosted Wiki для SDLC knowledge base. MVP поставляется как Docker Compose: backend (Rust), frontend (Vite static), PostgreSQL and persistent attachment volume. Reverse proxy по желанию.

## 2. System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | 2 cores | 4+ cores |
| RAM | 4 GB | 8+ GB |
| Disk | 20 GB SSD | 100+ GB SSD |
| OS | Linux x86_64 | Ubuntu 22.04 LTS |
| Docker | 24.0+ | 27.0+ |
| Docker Compose | 2.20+ | 2.27+ |

## 3. Services

| Service | Image | Port | Description |
|---------|-------|------|-------------|
| `backend` | build from `backend/Dockerfile` | `3456` | Axum API |
| `postgres` | `postgres:17.6-alpine` | `5432` | PostgreSQL |
| `frontend` | build from `frontend/Dockerfile` | `19877` | Static Wiki UI |

## 4. Quick Start

```bash
cp .env.example .env
# отредактируйте секреты и задайте bootstrap admin для fresh DB
# Для backend в контейнере с PostgreSQL persistence используйте host `postgres:5432`.
docker compose up -d postgres backend
curl -sf http://localhost:3456/api/v1/health/ready
```

## 5. Local Development

```bash
# Terminal 1
docker compose up -d postgres
cd backend
export WIKI_JWT_SECRET=dev-secret-32-chars-minimum
export WIKI_DATABASE__URL=postgres://wiki:[CHANGE_ME]@localhost:3457/wiki
export WIKI_BOOTSTRAP__ADMIN_EMAIL=admin@example.com
export WIKI_BOOTSTRAP__ADMIN_PASSWORD=change-me-before-use
cargo run --bin server

# Terminal 2
cd frontend
pnpm install
pnpm dev
```

Frontend dev-server ожидает backend по `http://127.0.0.1:3456/api/v1` (env `VITE_API_BASE_URL`).

## 6. Production Build

```bash
cd frontend
pnpm install
pnpm build
```

Результат — `frontend/dist`, который можно раздать nginx или встроить в контейнер.

## 7. First Admin

Для PostgreSQL runtime нет demo-пароля по умолчанию. На fresh database задайте `WIKI_BOOTSTRAP__ADMIN_EMAIL` и `WIKI_BOOTSTRAP__ADMIN_PASSWORD` перед первым стартом backend; startup создаст или обновит этого admin, базовое пространство `SDLC` и стандартные шаблоны. Memory fallback в API-тестах по-прежнему использует `demo@example.com` / `demo`.

## 8. Health Checks

| Endpoint | Service |
|----------|---------|
| `GET /api/v1/health` | api liveness |
| `GET /api/v1/health/ready` | PostgreSQL-backed runtime readiness |

## 9. Backup

```bash
./scripts/backup.sh
./scripts/cleanup_old_backups.sh 30
```

Archive format and restore procedure are described in `docs/BACKUP_RESTORE.md`.

## 10. Update

```bash
git pull origin main
docker compose down -v   # при изменениях миграций
docker compose up -d postgres backend
```

## 11. Reverse Proxy Example (nginx)

```nginx
server {
  listen 19877;

  location /api/ {
    proxy_pass http://127.0.0.1:3456;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
  }

  location / {
    root /var/www/wiki/frontend/dist;
    try_files $uri $uri/ /index.html;
  }
}
```

## References

- `docs/ARCHITECTURE.md`
- `docs/LOCAL_SETUP.md`
- `docs/OPS_RUNBOOK.md`
- `docs/SECURITY.md`
