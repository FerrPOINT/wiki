# API Standards - Wiki

## 1. Общие принципы

- API - REST поверх HTTP/1.1 и HTTP/2.
- Формат обмена данными - JSON.
- Кодировка - UTF-8.
- Base path - `/api/v1`.
- Версионирование описано в `docs/API_VERSIONING.md`.
- UI по умолчанию русский, технические поля и коды ошибок - английские.

## 2. OpenAPI

- Спецификация генерируется из Rust handlers и DTO через `utoipa-axum`.
- Swagger UI доступен по `/swagger-ui/` в dev-режиме.
- Каждый endpoint должен иметь summary, схемы request/response, статус-коды и ошибки `400`, `401`, `403`, `404`, `409`, `422`, если они применимы.
- `openapi/openapi.json` коммитится для hermetic frontend build, но не редактируется вручную.

## 3. URL и ресурсы

- Имена ресурсов - множественное число: `/spaces`, `/documents`, `/attachments`, `/evidence`.
- Вложенность допускается там, где она выражает ownership:
  - `/spaces/{space_key}/documents`
  - `/documents/{document_id}/revisions`
  - `/task-dossiers/{task_dossier_id}/phases`
  - `/phase-dossiers/{phase_dossier_id}/evidence`
- UUID ресурсов - UUIDv7.
- Внешние task keys передаются URL-encoded и всегда сопровождаются `source_system`, если возможна неоднозначность.

## 4. HTTP Methods

| Метод | Операция | Семантика |
|---|---|---|
| `GET` | Чтение | Идемпотентный, без side effects |
| `POST` | Создание/команда | `201 Created` для создания, `200/202` для команд |
| `PUT` | Полная замена | Идемпотентный |
| `PATCH` | Частичное обновление | JSON Merge Patch по умолчанию |
| `DELETE` | Удаление | `204 No Content` или soft archive command |

## 5. Пагинация

- Для списков - cursor-based pagination.
- Параметры: `cursor`, `limit` (max 100, default 20).
- Справочники могут использовать offset pagination.
- Подробнее: `docs/PAGINATION.md`.

## 6. Сортировка и фильтрация

- Сортировка: `?sort=-updated_at,title`.
- Фильтры: `?space=ENG&type=document&tag=release`.
- Поиск: `?q=deployment`.
- Archived объекты исключаются по умолчанию; включаются через `include_archived=true`.

## 7. Error Format

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Request validation failed",
    "details": [
      { "field": "title", "message": "required" }
    ],
    "request_id": "req_01J..."
  }
}
```

## 8. Realtime Events

- MVP может использовать SSE endpoint `/api/v1/events`.
- События и payloads описаны в `docs/EVENTS.md`.
- Клиент обязан уметь refetch-ить документ/dossier после reconnect.

## 9. Idempotency

- `POST /documents`, `POST /documents/{id}/publish`, `POST /evidence`, `POST /attachments` поддерживают `Idempotency-Key`.
- Ключ - UUIDv4/UUIDv7, хранится в Redis 24 часа.

## 10. Security

- Protected endpoints требуют JWT/session или scoped API token.
- Permissions проверяются на уровне space и entity.
- API не возвращает storage secrets, internal errors и stack traces.

## 11. References

- `docs/API.md`
- `docs/API_VERSIONING.md`
- `docs/ERROR_HANDLING.md`
- `docs/PAGINATION.md`
- `docs/EVENTS.md`
- `docs/SECURITY.md`
