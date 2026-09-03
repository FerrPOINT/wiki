# Resilience & Fault Tolerance — Wiki

## 1. Overview

Как система ведёт себя при сбоях зависимостей и сети.

## 2. Idempotency

### 2.1 Idempotency Keys

- CLI отправляет `Idempotency-Key` для повторяемых mutation requests.
- API дедуплицирует protected domain/admin `POST`/`PUT`/`DELETE` requests, если заголовок передан. Auth/session endpoints не входят в replay scope.
- Scope ключа: `(actor, key, method, path+query, request body hash)`.
- Успешный ответ хранится 24 часа и при retry возвращается без повторного domain write и audit write.
- Повтор ключа с другим method/path/body или повтор пока первый запрос в состоянии `processing` возвращает `409 CONFLICT`.
- Основные команды, где retry-safety особенно важна:
  - `POST /api/v1/spaces`
  - `POST /api/v1/spaces/{space_key}/documents`
  - `POST /api/v1/documents/{document_id}/publish`
  - `POST /api/v1/evidence`
  - `POST /api/v1/attachments`

### 2.2 Natural Idempotency

- Unique constraints and explicit conflict responses still protect naturally duplicated writes.
- Archive/soft-delete commands avoid destructive repeated deletes.
- `ETag` / `If-Match` remains deferred; current stale draft protection for publish uses `base_revision_id`.

## 3. Circuit Breakers

| Dependency | Failure Threshold | Recovery |
|------------|-------------------|----------|
| PostgreSQL | 5 errors in 30s | retry every 5s |
| Object storage | 5 errors in 60s | retry every 10s |

## 4. Graceful Degradation

| Component Fails | Fallback Behavior |
|-------------------|-------------------|
| PostgreSQL search projection | Degrade to bounded title search when safe |
| File storage S3 | Switch to local filesystem |
| Audit write path | Reject mutation if audit cannot be recorded |

## 5. Bulk Operations

Bulk evidence creation and document import are deferred. If added later, they must define max item count, per-item error shape and idempotency semantics before any API route is introduced.

## 6. Optimistic Locking

- Published document revisions are immutable and monotonically versioned.
- Concurrent write hardening with `ETag` / `If-Match` is deferred until the base API contract stabilizes.
- Current conflicts use unique constraints and `409` responses.

## 7. Request Limits

| Limit | Value |
|-------|-------|
| Max request body | 10 MB |
| Max attachment upload | 25 MiB default, configurable |
| Max bulk items | 100 |
| Max search result set | 1000 (paginated) |
| Max page size | 100 |
| Default page size | 20 |

## 8. Maintenance Jobs

- Backend may run an in-process maintenance loop without a separate worker.
- Each pass removes expired unclaimed staged attachments and expired idempotency replay records.
- Cleanup queries are bounded and idempotent under scale-out.
- File delete failures are logged for operator review; the public API and CLI surface do not expose maintenance routes.

## 9. Cross-Project Isolation

- Каждый repository query фильтрует по `space_id`.
- Service layer repeats permission checks.
- `space_id` взятый из URL проверяется на доступность.
- Тест: попытка доступа к document из чужого space возвращает 404 (не 403, чтобы не leak ID).

## 10. Soft Delete

- `spaces`, `documents`, `evidence_items`, `attachments` имеют `archived_at`, `quarantined_at` или `deleted_at` по правилам retention.
- MVP uses archive commands rather than a trash UI.
- Hard delete после retention policy.

## References

- `docs/ARCHITECTURE.md`
- `docs/API.md`
- `docs/STORAGE.md`
- `docs/NOTIFICATIONS.md`
- `docs/TESTING.md`
