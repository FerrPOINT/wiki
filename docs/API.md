# API v1 Specification - Wiki

## 1. Назначение

REST API Wiki предоставляет базовые операции продукта: auth, users, spaces, documents, revisions, task links, phase links, evidence, attachments, templates, settings, search и audit.

Текущий `openapi/openapi.json` зафиксирован под Wiki MVP API. Production server требует `WIKI_DATABASE__URL` и использует SQLx/PostgreSQL persistence; memory backend остаётся только явным test/dev router mode для быстрых API tests.

## 2. Общие правила

- Base path: `/api/v1`.
- UI и CLI используют один и тот же публичный API.
- Все ответы JSON, кроме download endpoints.
- Ошибки MVP возвращаются единым JSON envelope `{ "error": { "code": "CODE", "message": "message" } }`; `requestId` и `details` добавляются как опциональные поля, когда доступны.
- Backend возвращает `X-Request-ID` на каждый ответ: echo валидного клиентского заголовка или новый `req_` UUIDv7 для запроса без request id.
- Большие списки MVP используют bounded `limit` и стабильную сортировку: revisions, evidence, search, audit.
- CLI может отправлять `Idempotency-Key` для повторяемых write-команд; серверная дедупликация ключей вынесена в hardening после MVP.
- Protected endpoints требуют session/JWT.
- API не раскрывает секреты, bearer tokens, private storage keys и stack traces.

## 3. Health and Readiness

Operational endpoints are part of the public API surface but do not create Wiki domain data and do not require a frontend route.

| Method | Path            | Назначение                                                    |
| ------ | --------------- | ------------------------------------------------------------- |
| `GET`  | `/health`       | Liveness check для процесса API                               |
| `GET`  | `/health/ready` | Readiness check после инициализации runtime dependencies      |

## 4. Auth

| Method | Path             | Назначение                                                                               |
| ------ | ---------------- | ---------------------------------------------------------------------------------------- |
| `POST` | `/auth/register` | Регистрация пользователя; возвращает `403`, если `WIKI_AUTH__REGISTRATION_ENABLED=false` |
| `POST` | `/auth/login`    | Вход                                                                                     |
| `POST` | `/auth/refresh`  | Обновление access token                                                                  |
| `POST` | `/auth/logout`   | Завершение сессии                                                                        |
| `GET`  | `/users/me`      | Текущий пользователь                                                                     |

## 5. Users and Access

| Method   | Path                                    | Назначение                        |
| -------- | --------------------------------------- | --------------------------------- |
| `GET`    | `/users`                                | Список пользователей для admin UI |
| `POST`   | `/users`                                | Создать пользователя              |
| `PUT`    | `/users/{user_id}`                      | Обновить пользователя             |
| `GET`    | `/spaces/{space_key}/members`           | Участники space                   |
| `PUT`    | `/spaces/{space_key}/members/{user_id}` | Назначить роль в space            |
| `DELETE` | `/spaces/{space_key}/members/{user_id}` | Удалить участника из space        |

`DELETE /spaces/{space_key}/members/{user_id}` возвращает `404 NOT_FOUND`, если такой membership уже отсутствует. Успешный `204` означает, что membership реально был удалён; обычный пользователь без другой роли больше не проходит проверки чтения/поиска для этого space.

## 6. Spaces

| Method | Path                          | Назначение              |
| ------ | ----------------------------- | ----------------------- |
| `GET`  | `/spaces`                     | Список доступных spaces |
| `POST` | `/spaces`                     | Создать space           |
| `GET`  | `/spaces/{space_key}`         | Получить space          |
| `PUT`  | `/spaces/{space_key}`         | Обновить metadata       |
| `POST` | `/spaces/{space_key}/archive` | Архивировать space      |
| `GET`  | `/spaces/{space_key}/tree`    | Page tree               |

Архивированный space остаётся доступен на чтение и администрирование пользователям с правами, но content write-команды внутри него возвращают `400 VALIDATION_ERROR`: создание/изменение/публикация/перемещение/архивирование документов, создание evidence и task/phase document links.

## 7. Documents

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

Архивированный документ остаётся доступен на чтение пользователям с правом доступа к space, но write-команды `draft`, `publish`, `move`, `archive` и task/phase document links возвращают `400 VALIDATION_ERROR`.

`GET /documents/{document_id}/revisions` возвращает историю в порядке от последней опубликованной ревизии к первой. Endpoint bounded: без `limit` отдаёт последние 20 ревизий, `limit` ограничивается диапазоном `1..100`. Опубликованные ревизии immutable: новый draft или повторная публикация не меняют тело, заголовок и summary уже созданных ревизий.

`DocumentResponse` and `DocumentRevisionResponse` expose both `body_markdown` and `body_html`. `body_markdown` is the canonical source for editing and CLI export; `body_html` is the sanitized HTML rendered by the backend from the published revision and is the only HTML surface the UI should render.

## 8. Task Links

Task dossier в MVP - это представление документов/evidence, связанных одним внешним `task_key`. Wiki не владеет статусом задачи.

| Method | Path                                                   | Назначение                       |
| ------ | ------------------------------------------------------ | -------------------------------- |
| `GET`  | `/spaces/{space_key}/tasks`                            | Список task keys, известных Wiki |
| `GET`  | `/spaces/{space_key}/tasks/{task_key}`                 | Сводка по task key               |
| `POST` | `/spaces/{space_key}/tasks/{task_key}/links/documents` | Привязать документ к task key    |
| `GET`  | `/spaces/{space_key}/tasks/{task_key}/documents`       | Документы task key               |
| `GET`  | `/spaces/{space_key}/tasks/{task_key}/evidence`        | Evidence task key                |

## 9. Phase Links

Phase dossier в MVP - это представление документов/evidence по `phase_key`. Wiki не управляет переходами фаз.

| Method | Path                                                     | Назначение                        |
| ------ | -------------------------------------------------------- | --------------------------------- |
| `GET`  | `/spaces/{space_key}/phases`                             | Список phase keys, известных Wiki |
| `GET`  | `/spaces/{space_key}/phases/{phase_key}`                 | Сводка по phase key               |
| `POST` | `/spaces/{space_key}/phases/{phase_key}/links/documents` | Привязать документ к phase key    |
| `GET`  | `/spaces/{space_key}/phases/{phase_key}/documents`       | Документы phase key               |
| `GET`  | `/spaces/{space_key}/phases/{phase_key}/evidence`        | Evidence phase key                |

## 10. Evidence and Attachments

| Method | Path                                    | Назначение                  |
| ------ | --------------------------------------- | --------------------------- |
| `POST` | `/evidence`                             | Создать URL/file evidence   |
| `GET`  | `/evidence`                             | Список evidence с фильтрами |
| `GET`  | `/evidence/{evidence_id}`               | Получить evidence           |
| `POST` | `/attachments`                          | Загрузить файл              |
| `GET`  | `/attachments/{attachment_id}`          | Metadata файла              |
| `GET`  | `/attachments/{attachment_id}/download` | Скачать файл                |

Canonical `evidence_type` values for MVP are `external_url` and `uploaded_file`. `external_url` accepts a non-empty `url` and must not include `attachment_id` or `checksum`; `uploaded_file` accepts `attachment_id` without `url` and stores checksum from the staged attachment. Evidence can be linked to `document_id`, `task_key`, `phase_key` or their combination inside one space. If `document_id` is present and `space` is omitted, API uses the document's space; if explicit `space` conflicts with the document's space, API returns `400 VALIDATION_ERROR`. `GET /evidence` supports `space`, `document_id`, `task_key`, `phase_key` and `limit` filters and returns only spaces visible to the caller. Without `limit`, API returns the latest 30 evidence items; `limit` is clamped to `1..100`. Specific source categories such as CI job, pull request, deployment or test artifact are metadata, not separate evidence types.

## 11. Search

| Method | Path      | Назначение       |
| ------ | --------- | ---------------- |
| `GET`  | `/search` | Поиск документов |

Фильтры MVP: `space`, `task_key`, `phase_key`, `document_type`, `include_archived`, `limit`. Без `limit` поиск возвращает 20 результатов; `limit` ограничивается диапазоном `1..100`.

Для опубликованных документов поиск использует текущую опубликованную ревизию. Если у опубликованного документа есть новый непубликованный draft, его текст не попадает в общий search response до следующей публикации.

## 12. Templates, Settings and Audit

| Method | Path         | Назначение                                                         |
| ------ | ------------ | ------------------------------------------------------------------ |
| `GET`  | `/templates` | Список шаблонов                                                    |
| `POST` | `/templates` | Создать шаблон, system admin only                                  |
| `GET`  | `/settings`  | Admin-only read-only snapshot безопасных runtime настроек инстанса |
| `GET`  | `/audit-log` | Последние audit events для admin UI; `limit` clamps to `1..200` |

`GET /settings` не возвращает секреты, connection strings, storage paths или bootstrap credentials. MVP endpoint показывает только значения, нужные UI/CLI: API path, регистрацию, storage/search backend, лимит загрузки, язык и timezone.

`GET /audit-log` возвращает append-only события с `request_id` в порядке от новых к старым. Endpoint всегда bounded: без параметра отдаёт последние 50 событий, `limit` ограничивается диапазоном `1..200`. Для mutating HTTP-запросов audit entry использует тот же `X-Request-ID`, который backend вернул клиенту в response header; если запрос пришёл без валидного id, middleware создаёт `req_` id и он попадает в audit.

## 13. Deferred API Areas

- Comments.
- Reports.
- Notifications.
- Integrations and source-specific tokens.
- Approval chains.
- Import/export bundles.
- Webhook ingestion or delivery.
- Runner/worker control protocols.

## 14. Contract Freeze For Main Development

The pre-development API contract is frozen when these checks pass:

- every runtime `/api/v1` route appears in `openapi/openapi.json`;
- every OpenAPI path appears in this document and in `docs/PRODUCT_REQUIREMENTS.md`;
- frontend generated DTOs compile against the committed OpenAPI artifact;
- UI and CLI call only public API endpoints, never PostgreSQL or storage internals;
- The Prometheus metrics endpoint is documented in operations but is not part of the versioned `/api/v1` OpenAPI contract.

## 15. Required Negative Cases

| Area | Required behavior |
| ---- | ----------------- |
| Auth | Bad credentials return an auth error; disabled registration returns `403`; missing/invalid bearer tokens return the standard error envelope; refresh rotates access and refresh token paths; logout invalidates both token paths for the current session. |
| Access | No role or removed membership blocks document, tree, evidence, attachment and search reads for that space. |
| Spaces | Archived spaces reject document, evidence and task/phase link write commands while keeping read/admin visibility. |
| Documents | Archived documents reject `draft`, `publish`, `move`, `archive` and task/phase link writes; duplicate slugs return conflict; cyclic moves return validation error. |
| Evidence | Evidence must target at least one document/task/phase; explicit `space` cannot conflict with document space. |
| Attachments | Empty upload, unsafe filename, oversized body and unauthorized download are rejected. |
| Search | Results are bounded, permission-filtered and do not expose unpublished draft text. |
| Settings | Secrets, connection strings, storage paths and bootstrap credentials are never returned. |
| Health | `/health/ready` fails until runtime dependencies are initialized. |
