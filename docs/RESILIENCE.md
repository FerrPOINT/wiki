# Resilience & Fault Tolerance — Wiki

## 1. Overview

Как система ведёт себя при сбоях зависимостей и сети.

## 2. Idempotency

### 2.1 Idempotency Keys

- Каждый mutation request может содержать заголовок `Idempotency-Key: <uuid>`.
- Сервер сохраняет mapping `key → response` на 24 часа.
- При повторном запросе с тем же ключом возвращается сохранённый ответ.
- Применяется к:
  - `POST /api/v1/documents`
  - `POST /api/v1/spaces`
  - `POST /api/v1/comments`
  - bulk operations

### 2.2 Natural Idempotency

- `PUT` обновления с ETag/If-Match.
- `DELETE` повторный — 404 без side effects.

## 3. Circuit Breakers

| Dependency | Failure Threshold | Recovery |
|------------|-------------------|----------|
| PostgreSQL | 5 errors in 30s | retry every 5s |
| Redis | 10 errors in 30s | retry every 5s |
| Object storage | 5 errors in 60s | retry every 10s |

## 4. Graceful Degradation

| Component Fails | Fallback Behavior |
|-------------------|-------------------|
| Redis | Cache miss → DB query; sessions stateless via JWT |
| Search service | Degrade to `LIKE`/`tsvector` query |
| File storage S3 | Switch to local filesystem |
| Audit write path | Reject mutation if audit cannot be recorded |

## 5. Bulk Operations

- `POST /api/v1/evidence/bulk` - создание до 100 evidence records.
- `POST /api/v1/spaces/{space_key}/documents/import` - импорт до 100 документов.
- Ответ содержит `processed`, `failed`, `errors`.
- Bulk requests обязательно с `Idempotency-Key`.

## 6. Optimistic Locking

- Document/space/config имеют `version` поле.
- `PUT` с `If-Match: <version>`.
- При conflict — `409` с актуальной версией.

## 7. Request Limits

| Limit | Value |
|-------|-------|
| Max request body | 10 MB |
| Max attachments total | 50 MB |
| Max bulk items | 100 |
| Max search result set | 1000 (paginated) |
| Max page size | 100 |
| Default page size | 20 |

## 8. Retry for Maintenance Jobs

- Maintenance jobs are optional after MVP backend migration.
- Retry: 3 attempts with exponential delay.
- After 3 failures, record an audit/admin event for operator review.

## 9. Cross-Project Isolation

- Каждый repository query фильтрует по `space_id`.
- Service layer repeats permission checks.
- `space_id` взятый из URL проверяется на доступность.
- Тест: попытка доступа к document из чужого space возвращает 404 (не 403, чтобы не leak ID).

## 10. Soft Delete

- `spaces`, `documents`, `comments`, `attachments` имеют `archived_at` или `deleted_at` по правилам retention.
- DELETE endpoint помечает `deleted_at`.
- Trash UI позволяет восстановить в течение 30 дней.
- Hard delete после retention policy.

## References

- `docs/ARCHITECTURE.md`
- `docs/API.md`
- `docs/STORAGE.md`
- `docs/NOTIFICATIONS.md`
- `docs/TESTING.md`
