# Runtime Behavior — Wiki

## 1. Overview

Как приложение стартует, работает и корректно завершается в production.

## 2. Health Probes

### 2.1 Endpoints

| Probe | Path | Success | Failure |
|-------|------|---------|---------|
| Liveness | `GET /api/v1/health` | HTTP 200 | Process unavailable |
| Readiness | `GET /api/v1/health/ready` | PostgreSQL backend reachable | HTTP 503 |
| Startup | covered by backend startup | migrations and bootstrap done before bind | process exits on failure |

### 2.2 Startup Probe

- Выполняется только во время старта.
- Проверяет, что миграции применены и seed-данные на месте.
- Период: 10s, failureThreshold: 30 (≈5 минут).
- После success не повторяется.

### 2.3 Readiness Probe

- Проверяет соединение с PostgreSQL для persistent runtime.
- Если БД недоступна — readiness 503, трафик не направляется.
- Период: 5s.

### 2.4 Liveness Probe

- Простой ping.
- Если не отвечает 3 раза подряд — контейнер перезапускается.

## 3. Startup Order

1. Загрузка и validation конфигурации (`WIKI_*`).
2. Если `WIKI_ENVIRONMENT=production`, startup fails fast on unsafe CORS, weak JWT secret, insecure refresh cookie or empty PostgreSQL URL.
3. Подключение к PostgreSQL с retry:
   - Initial delay: 1s.
   - Max delay: 30s.
   - Max retries: 30.
4. Применение Wiki SQLx migrations.
5. Seed default data (admin, default spaces, document templates).
6. Запуск HTTP сервера.
7. Readiness probe начинает отдавать `ready`.

## 4. Retry / Backoff

| Dependency | Strategy |
|------------|----------|
| PostgreSQL | exponential backoff 1s → 30s |
| Attachment storage | validate configured location before production rollout |

## 5. Graceful Shutdown

1. Получение `SIGTERM` / `SIGINT`.
2. Stop accepting new HTTP connections.
3. Wait for active requests (timeout 30s).
4. Flush pending audit/search writes if any are in process.
5. Close DB connection pool.
6. Exit.

## 6. Resource Limits

| Resource | Limit | Why |
|----------|-------|-----|
| `nofile` | 65536 | uploads + concurrent HTTP |
| `max_connections` PostgreSQL | 200 | connection pool |
| Backend connection pool | 20-50 | per instance |
| Request body | 10 MB | JSON payloads |
| Upload file | 50 MB | attachments |

## 7. Maintenance Jobs

- MVP can run without a separate worker process.
- Optional in-process maintenance jobs may cleanup expired temporary files.
- Retry policy: 3 attempts, then audit/admin event.

## 8. Watchdogs

- Если readiness падает более 2 минут — алерт.
- Если upload/search error rate растёт — алерт.
- Если liveness падает — автоматический restart.

## 9. Multi-instance Notes

- Stateless HTTP tier.
- Optional maintenance jobs must be idempotent under scale-out.

## References

- `docs/DEPLOYMENT.md`
- `docs/OPS_RUNBOOK.md`
- `docs/MONITORING.md`
- `docs/EVENTS.md`
