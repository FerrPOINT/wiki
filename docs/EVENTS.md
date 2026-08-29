# Event Catalog - Wiki

## 1. Overview

Доменные события в MVP нужны для audit и обновления search projection. Они не являются публичным webhook-контрактом.

## 2. Event Structure

```json
{
  "event_id": "018f...",
  "event_type": "document.published",
  "version": 1,
  "occurred_at": "2026-08-28T12:34:56Z",
  "actor_id": "018f...",
  "entity": {
    "type": "document",
    "id": "018f..."
  },
  "payload": {}
}
```

## 3. MVP Event Types

| Event Type | When | Consumers |
|---|---|---|
| `user.logged_in` | Пользователь вошёл | Audit |
| `user.logged_out` | Пользователь вышел | Audit |
| `space.created` | Создан space | Audit |
| `space.archived` | Архивирован space | Audit |
| `space.member_changed` | Изменён участник space | Audit |
| `document.created` | Создан документ | Audit |
| `document.draft_updated` | Обновлён черновик | Audit |
| `document.published` | Опубликована ревизия | Audit, search |
| `document.archived` | Документ архивирован | Audit, search |
| `document.moved` | Документ перемещён | Audit |
| `task_dossier.linked` | Документ/evidence связан с task key | Audit |
| `phase_dossier.linked` | Документ/evidence связан с phase key | Audit |
| `evidence.added` | Добавлено evidence | Audit |
| `attachment.uploaded` | Загружен файл | Audit |

## 4. Delivery

MVP may process events synchronously inside the application service. Durable queues and external delivery are deferred.

Rules:

- event payloads must not contain secrets;
- event writes should happen in the same transaction as the domain change when possible;
- consumers must be idempotent if async processing is added later.

## 5. Audit Mapping

| Event | Audit Action |
|---|---|
| `document.published` | `document.publish` |
| `document.archived` | `document.archive` |
| `document.moved` | `document.move` |
| `space.member_changed` | `space.member_change` |
| `evidence.added` | `evidence.add` |
| `attachment.uploaded` | `attachment.upload` |

## 6. Deferred

- Public webhook payloads.
- Notification fanout.
- Durable outbox workers.
- Dead-letter replay.

## 7. References

- `docs/DOMAIN_MODEL.md`
- `docs/API.md`
- `docs/SECURITY.md`
