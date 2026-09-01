# API v1 Specification - Wiki

## 1. Назначение

REST API Wiki предоставляет базовые операции продукта: auth, users, spaces, documents, revisions, task links, phase links, evidence, attachments, templates, settings, search и audit.

Текущий `openapi/openapi.json` зафиксирован под Wiki MVP API. Production server требует `WIKI_DATABASE__URL` и использует SQLx/PostgreSQL persistence; memory backend остаётся только явным test/dev router mode для быстрых API tests.

## 2. Общие правила

- Base path: `/api/v1`.
- UI и CLI используют один и тот же публичный API.
- Все ответы JSON, кроме download endpoints.
- Ошибки MVP возвращаются единым JSON envelope `{ "error": { "code": "CODE", "message": "message" } }`; `requestId` и `details` добавляются как опциональные поля, когда доступны.
- Списки используют `limit` и стабильную сортировку.
- CLI может отправлять `Idempotency-Key` для повторяемых write-команд; серверная дедупликация ключей вынесена в hardening после MVP.
- Protected endpoints требуют session/JWT.
- API не раскрывает секреты, bearer tokens, private storage keys и stack traces.

## 3. Auth

| Method | Path             | Назначение                                                                               |
| ------ | ---------------- | ---------------------------------------------------------------------------------------- |
| `POST` | `/auth/register` | Регистрация пользователя; возвращает `403`, если `WIKI_AUTH__REGISTRATION_ENABLED=false` |
| `POST` | `/auth/login`    | Вход                                                                                     |
| `POST` | `/auth/refresh`  | Обновление access token                                                                  |
| `POST` | `/auth/logout`   | Завершение сессии                                                                        |
| `GET`  | `/users/me`      | Текущий пользователь                                                                     |

## 4. Users and Access

| Method   | Path                                    | Назначение                        |
| -------- | --------------------------------------- | --------------------------------- |
| `GET`    | `/users`                                | Список пользователей для admin UI |
| `POST`   | `/users`                                | Создать пользователя              |
| `PUT`    | `/users/{user_id}`                      | Обновить пользователя             |
| `GET`    | `/spaces/{space_key}/members`           | Участники space                   |
| `PUT`    | `/spaces/{space_key}/members/{user_id}` | Назначить роль в space            |
| `DELETE` | `/spaces/{space_key}/members/{user_id}` | Удалить участника из space        |

## 5. Spaces

| Method | Path                          | Назначение              |
| ------ | ----------------------------- | ----------------------- |
| `GET`  | `/spaces`                     | Список доступных spaces |
| `POST` | `/spaces`                     | Создать space           |
| `GET`  | `/spaces/{space_key}`         | Получить space          |
| `PUT`  | `/spaces/{space_key}`         | Обновить metadata       |
| `POST` | `/spaces/{space_key}/archive` | Архивировать space      |
| `GET`  | `/spaces/{space_key}/tree`    | Page tree               |

## 6. Documents

| Method | Path                                               | Назначение                 |
| ------ | -------------------------------------------------- | -------------------------- |
| `POST` | `/spaces/{space_key}/documents`                    | Создать документ           |
| `GET`  | `/documents/{document_id}`                         | Открыть документ           |
| `PUT`  | `/documents/{document_id}/draft`                   | Обновить draft             |
| `POST` | `/documents/{document_id}/publish`                 | Опубликовать новую ревизию |
| `POST` | `/documents/{document_id}/archive`                 | Архивировать документ      |
| `POST` | `/documents/{document_id}/move`                    | Переместить в дереве       |
| `GET`  | `/documents/{document_id}/revisions`               | История ревизий            |
| `GET`  | `/documents/{document_id}/revisions/{revision_id}` | Конкретная ревизия         |

Архивированный документ остаётся доступен на чтение пользователям с правом доступа к space, но write-команды `draft`, `publish` и `move` возвращают `400 VALIDATION_ERROR`.

`GET /documents/{document_id}/revisions` возвращает историю в порядке от последней опубликованной ревизии к первой. Опубликованные ревизии immutable: новый draft или повторная публикация не меняют тело, заголовок и summary уже созданных ревизий.

## 7. Task Links

Task dossier в MVP - это представление документов/evidence, связанных одним внешним `task_key`. Wiki не владеет статусом задачи.

| Method | Path                                                   | Назначение                       |
| ------ | ------------------------------------------------------ | -------------------------------- |
| `GET`  | `/spaces/{space_key}/tasks`                            | Список task keys, известных Wiki |
| `GET`  | `/spaces/{space_key}/tasks/{task_key}`                 | Сводка по task key               |
| `POST` | `/spaces/{space_key}/tasks/{task_key}/links/documents` | Привязать документ к task key    |
| `GET`  | `/spaces/{space_key}/tasks/{task_key}/documents`       | Документы task key               |
| `GET`  | `/spaces/{space_key}/tasks/{task_key}/evidence`        | Evidence task key                |

## 8. Phase Links

Phase dossier в MVP - это представление документов/evidence по `phase_key`. Wiki не управляет переходами фаз.

| Method | Path                                                     | Назначение                        |
| ------ | -------------------------------------------------------- | --------------------------------- |
| `GET`  | `/spaces/{space_key}/phases`                             | Список phase keys, известных Wiki |
| `GET`  | `/spaces/{space_key}/phases/{phase_key}`                 | Сводка по phase key               |
| `POST` | `/spaces/{space_key}/phases/{phase_key}/links/documents` | Привязать документ к phase key    |
| `GET`  | `/spaces/{space_key}/phases/{phase_key}/documents`       | Документы phase key               |
| `GET`  | `/spaces/{space_key}/phases/{phase_key}/evidence`        | Evidence phase key                |

## 9. Evidence and Attachments

| Method | Path                                    | Назначение                  |
| ------ | --------------------------------------- | --------------------------- |
| `POST` | `/evidence`                             | Создать URL/file evidence   |
| `GET`  | `/evidence`                             | Список evidence с фильтрами |
| `GET`  | `/evidence/{evidence_id}`               | Получить evidence           |
| `POST` | `/attachments`                          | Загрузить файл              |
| `GET`  | `/attachments/{attachment_id}`          | Metadata файла              |
| `GET`  | `/attachments/{attachment_id}/download` | Скачать файл                |

Canonical `evidence_type` values for MVP are `external_url` and `uploaded_file`. `external_url` accepts a non-empty `url` and must not include `attachment_id` or `checksum`; `uploaded_file` accepts `attachment_id` without `url` and stores checksum from the staged attachment. Evidence can be linked to `document_id`, `task_key`, `phase_key` or their combination inside one space. If `document_id` is present and `space` is omitted, API uses the document's space; if explicit `space` conflicts with the document's space, API returns `400 VALIDATION_ERROR`. `GET /evidence` supports `space`, `document_id`, `task_key` and `phase_key` filters and returns only spaces visible to the caller. Specific source categories such as CI job, pull request, deployment or test artifact are metadata, not separate evidence types.

## 10. Search

| Method | Path      | Назначение       |
| ------ | --------- | ---------------- |
| `GET`  | `/search` | Поиск документов |

Фильтры MVP: `space`, `task_key`, `phase_key`, `document_type`, `include_archived`.

Для опубликованных документов поиск использует текущую опубликованную ревизию. Если у опубликованного документа есть новый непубликованный draft, его текст не попадает в общий search response до следующей публикации.

## 11. Templates, Settings and Audit

| Method | Path         | Назначение                                                         |
| ------ | ------------ | ------------------------------------------------------------------ |
| `GET`  | `/templates` | Список шаблонов                                                    |
| `POST` | `/templates` | Создать шаблон, system admin only                                  |
| `GET`  | `/settings`  | Admin-only read-only snapshot безопасных runtime настроек инстанса |
| `GET`  | `/audit-log` | Audit log для admin UI                                             |

`GET /settings` не возвращает секреты, connection strings, storage paths или bootstrap credentials. MVP endpoint показывает только значения, нужные UI/CLI: API path, регистрацию, storage/search backend, лимит загрузки, язык и timezone.

## 12. Deferred API Areas

- Comments.
- Advanced reports.
- Approval chains.
- Import/export bundles.
- Webhook delivery.
