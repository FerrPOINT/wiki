# API v1 Specification - Wiki

## 1. Назначение

REST API Wiki предоставляет базовые операции продукта: auth, users, spaces, documents, revisions, task links, phase links, evidence, attachments, templates, search и audit.

Текущий `openapi/openapi.json` зафиксирован под Wiki MVP API. Runtime использует memory fallback для быстрых тестов и SQLx/PostgreSQL persistence при заданном `WIKI_DATABASE__URL`.

## 2. Общие правила

- Base path: `/api/v1`.
- UI и CLI используют один и тот же публичный API.
- Все ответы JSON, кроме download endpoints.
- Ошибки MVP возвращаются единым JSON envelope `{ "error": "message" }`.
- Списки используют `limit` и стабильную сортировку.
- CLI может отправлять `Idempotency-Key` для повторяемых write-команд; серверная дедупликация ключей вынесена в hardening после MVP.
- Protected endpoints требуют session/JWT.
- API не раскрывает секреты, bearer tokens, private storage keys и stack traces.

## 3. Auth

| Method | Path | Назначение |
|---|---|---|
| `POST` | `/auth/register` | Регистрация пользователя; возвращает `403`, если `WIKI_AUTH__REGISTRATION_ENABLED=false` |
| `POST` | `/auth/login` | Вход |
| `POST` | `/auth/refresh` | Обновление access token |
| `POST` | `/auth/logout` | Завершение сессии |
| `GET` | `/users/me` | Текущий пользователь |

## 4. Users and Access

| Method | Path | Назначение |
|---|---|---|
| `GET` | `/users` | Список пользователей для admin UI |
| `POST` | `/users` | Создать пользователя |
| `PUT` | `/users/{user_id}` | Обновить пользователя |
| `GET` | `/spaces/{space_key}/members` | Участники space |
| `PUT` | `/spaces/{space_key}/members/{user_id}` | Назначить роль в space |
| `DELETE` | `/spaces/{space_key}/members/{user_id}` | Удалить участника из space |

## 5. Spaces

| Method | Path | Назначение |
|---|---|---|
| `GET` | `/spaces` | Список доступных spaces |
| `POST` | `/spaces` | Создать space |
| `GET` | `/spaces/{space_key}` | Получить space |
| `PUT` | `/spaces/{space_key}` | Обновить metadata |
| `POST` | `/spaces/{space_key}/archive` | Архивировать space |
| `GET` | `/spaces/{space_key}/tree` | Page tree |

## 6. Documents

| Method | Path | Назначение |
|---|---|---|
| `POST` | `/spaces/{space_key}/documents` | Создать документ |
| `GET` | `/documents/{document_id}` | Открыть документ |
| `PUT` | `/documents/{document_id}/draft` | Обновить draft |
| `POST` | `/documents/{document_id}/publish` | Опубликовать новую ревизию |
| `POST` | `/documents/{document_id}/archive` | Архивировать документ |
| `POST` | `/documents/{document_id}/move` | Переместить в дереве |
| `GET` | `/documents/{document_id}/revisions` | История ревизий |
| `GET` | `/documents/{document_id}/revisions/{revision_id}` | Конкретная ревизия |

## 7. Task Links

Task dossier в MVP - это представление документов/evidence, связанных одним внешним `task_key`. Wiki не владеет статусом задачи.

| Method | Path | Назначение |
|---|---|---|
| `GET` | `/spaces/{space_key}/tasks` | Список task keys, известных Wiki |
| `GET` | `/spaces/{space_key}/tasks/{task_key}` | Сводка по task key |
| `POST` | `/spaces/{space_key}/tasks/{task_key}/links/documents` | Привязать документ к task key |
| `GET` | `/spaces/{space_key}/tasks/{task_key}/documents` | Документы task key |
| `GET` | `/spaces/{space_key}/tasks/{task_key}/evidence` | Evidence task key |

## 8. Phase Links

Phase dossier в MVP - это представление документов/evidence по `phase_key`. Wiki не управляет переходами фаз.

| Method | Path | Назначение |
|---|---|---|
| `GET` | `/spaces/{space_key}/phases` | Список phase keys, известных Wiki |
| `GET` | `/spaces/{space_key}/phases/{phase_key}` | Сводка по phase key |
| `POST` | `/spaces/{space_key}/phases/{phase_key}/links/documents` | Привязать документ к phase key |
| `GET` | `/spaces/{space_key}/phases/{phase_key}/documents` | Документы phase key |
| `GET` | `/spaces/{space_key}/phases/{phase_key}/evidence` | Evidence phase key |

## 9. Evidence and Attachments

| Method | Path | Назначение |
|---|---|---|
| `POST` | `/evidence` | Создать URL/file evidence |
| `GET` | `/evidence` | Список evidence с фильтрами |
| `GET` | `/evidence/{evidence_id}` | Получить evidence |
| `POST` | `/attachments` | Загрузить файл |
| `GET` | `/attachments/{attachment_id}` | Metadata файла |
| `GET` | `/attachments/{attachment_id}/download` | Скачать файл |

Canonical `evidence_type` values for MVP are `external_url` and `uploaded_file`. `external_url` accepts `url` without `attachment_id`; `uploaded_file` accepts `attachment_id` without `url`. Specific source categories such as CI job, pull request, deployment or test artifact are metadata, not separate evidence types.

## 10. Search

| Method | Path | Назначение |
|---|---|---|
| `GET` | `/search` | Поиск документов |

Фильтры MVP: `space`, `task_key`, `phase_key`, `document_type`, `include_archived`.

## 11. Templates and Audit

| Method | Path | Назначение |
|---|---|---|
| `GET` | `/templates` | Список шаблонов |
| `POST` | `/templates` | Создать шаблон |
| `GET` | `/audit-log` | Audit log для admin UI |

## 12. Deferred API Areas

- Comments.
- Advanced reports.
- Approval chains.
- Import/export bundles.
- Webhook delivery.
