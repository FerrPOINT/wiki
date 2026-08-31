# Требования к продукту Wiki

## 1. Назначение

Wiki - self-hosted база знаний для SDLC. Продукт хранит страницы, версии, вложения и evidence, связанные с документами, задачами и фазами workflow.

Wiki не исполняет workflow, не управляет задачами и не заменяет CI/CD. Она отвечает только за знания: что было решено, какие документы актуальны, какие материалы приложены к документу, задаче или фазе.

## 2. Клиенты продукта

У продукта один публичный backend API и два официальных клиента:

| Клиент | Назначение                                                                  |
| ------ | --------------------------------------------------------------------------- |
| UI     | Основной web-интерфейс для чтения, редактирования и администрирования Wiki  |
| CLI    | Консольный клиент к тому же API для людей, скриптов и внешней автоматизации |

CLI не имеет отдельной доменной модели и специальных команд под конкретный тип потребителя. Всё, что делает CLI, является обычной операцией публичного API.

## 3. Границы MVP

MVP включает только базовую Wiki-функциональность:

- пользователи и роли;
- spaces;
- документы и дерево страниц;
- Markdown draft/publish;
- история версий;
- связь документов с task key;
- связь документов и evidence с phase key;
- файлы и ссылки как evidence;
- поиск;
- шаблоны документов;
- audit write-действий;
- read-only runtime settings для admin UI/CLI;
- API, UI и CLI.

MVP не включает:

- real-time collaborative editing;
- inline comments;
- сложные approval chains;
- Confluence marketplace/macros/import;
- OCR и индексацию бинарных вложений;
- advanced reports;
- Git hosting, CI/CD execution или task tracker workflow.

## 4. Роли

| Роль   | Права                                                          |
| ------ | -------------------------------------------------------------- |
| Admin  | Управляет пользователями/spaces и просматривает settings/audit |
| Editor | Создаёт и редактирует документы/evidence в доступных spaces    |
| Viewer | Читает опубликованные документы и evidence в доступных spaces  |

Права проверяются до чтения документа, evidence, файла, дерева страниц и результатов поиска.

## 5. Функциональные требования MVP

| REQ-ID        | Capability       | Требование                                                                                                                                |
| ------------- | ---------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| REQ-AUTH-001  | Auth             | Пользователь может войти, выйти и получить текущий профиль                                                                                |
| REQ-AUTH-002  | Roles            | Admin управляет пользователями и назначает роли в space                                                                                   |
| REQ-AUTH-003  | Registration     | Пользователь может создать учётную запись через public register flow, если регистрация включена настройкой инстанса                       |
| REQ-SPC-001   | Spaces           | Пользователь видит список доступных spaces                                                                                                |
| REQ-SPC-002   | Space management | Admin создаёт, редактирует и архивирует space                                                                                             |
| REQ-SPC-003   | Space members    | Admin управляет участниками space                                                                                                         |
| REQ-DOC-001   | Documents        | Editor создаёт страницу с title, slug, type и Markdown body                                                                               |
| REQ-DOC-002   | Document view    | Viewer открывает опубликованную страницу                                                                                                  |
| REQ-DOC-003   | Draft edit       | Editor редактирует черновик страницы                                                                                                      |
| REQ-DOC-004   | Publish          | Публикация создаёт неизменяемую ревизию                                                                                                   |
| REQ-DOC-005   | Revision history | Пользователь видит список ревизий и открывает конкретную ревизию                                                                          |
| REQ-DOC-006   | Archive          | Editor архивирует документ; archived pages скрыты из обычного дерева                                                                      |
| REQ-TREE-001  | Page tree        | Документы имеют parent/child структуру внутри space                                                                                       |
| REQ-TREE-002  | Move page        | Editor перемещает страницу внутри одного space                                                                                            |
| REQ-TASK-001  | Task link        | Документ можно связать с внешним task key, например `SDLC-42`                                                                             |
| REQ-TASK-002  | Task page        | Страница task dossier показывает документы и evidence по task key                                                                         |
| REQ-PHASE-001 | Phase link       | Документ/evidence можно связать с phase key                                                                                               |
| REQ-PHASE-002 | Phase page       | Страница phase dossier показывает документы и evidence по phase key                                                                       |
| REQ-EVID-001  | Link evidence    | Editor добавляет URL evidence к документу, задаче или фазе                                                                                |
| REQ-EVID-002  | File evidence    | Editor загружает файл evidence с metadata и checksum                                                                                      |
| REQ-EVID-003  | Evidence list    | Пользователь видит evidence по документу, задаче и фазе                                                                                   |
| REQ-SRCH-001  | Search           | Пользователь ищет по title/body документа                                                                                                 |
| REQ-SRCH-002  | Search filters   | Поиск фильтруется по space, task key, phase key и document type                                                                           |
| REQ-TPL-001   | Templates        | Editor создаёт документ из базового шаблона                                                                                               |
| REQ-SET-001   | Settings         | Admin видит безопасный runtime snapshot настроек инстанса: API path, регистрацию, storage/search backend, лимит загрузки, язык и timezone |
| REQ-AUD-001   | Audit            | Система пишет audit для login/logout, document create/edit/publish/archive, evidence add, member/role changes                             |
| REQ-API-001   | API              | Все MVP-операции доступны через `/api/v1`                                                                                                 |
| REQ-CLI-001   | CLI              | CLI покрывает те же базовые операции, что и API, и возвращает JSON по умолчанию                                                           |
| REQ-UI-001    | UI               | UI покрывает основные сценарии spaces, documents, task/phase dossiers, evidence, search и admin                                           |

## 6. Базовые шаблоны

MVP поставляет только пять шаблонов:

| Шаблон              | Назначение                       |
| ------------------- | -------------------------------- |
| Requirements        | Требования и acceptance criteria |
| Research note       | Исследование и варианты решения  |
| Implementation note | Технические заметки реализации   |
| Test plan           | План проверки                    |
| Release note        | Итог релиза                      |

## 7. Доменные сущности

| Сущность         | Назначение                                                   |
| ---------------- | ------------------------------------------------------------ |
| User             | Учётная запись и автор действий                              |
| Space            | Верхний контейнер базы знаний                                |
| SpaceMember      | Роль пользователя внутри space                               |
| Document         | Логическая страница в дереве                                 |
| DocumentRevision | Неизменяемая опубликованная версия страницы                  |
| DocumentDraft    | Черновик страницы                                            |
| TaskDossier      | Представление внешней задачи по task key                     |
| PhaseDossier     | Представление фазы workflow по phase key                     |
| EvidenceItem     | URL или файл, подтверждающий работу по документу/задаче/фазе |
| Attachment       | Metadata файла; байты лежат в storage                        |
| DocumentTemplate | Markdown-шаблон                                              |
| AuditEntry       | Append-only запись о важном действии                         |

## 8. API v1 overview

API является единственным контрактом backend. UI и CLI используют один и тот же API.

### Auth

- `POST /api/v1/auth/register`
- `POST /api/v1/auth/login`
- `POST /api/v1/auth/refresh`
- `POST /api/v1/auth/logout`
- `GET /api/v1/users/me`

### Users and access

- `GET /api/v1/users`
- `POST /api/v1/users`
- `PUT /api/v1/users/{user_id}`
- `GET /api/v1/spaces/{space_key}/members`
- `PUT /api/v1/spaces/{space_key}/members/{user_id}`
- `DELETE /api/v1/spaces/{space_key}/members/{user_id}`

### Spaces

- `GET /api/v1/spaces`
- `POST /api/v1/spaces`
- `GET /api/v1/spaces/{space_key}`
- `PUT /api/v1/spaces/{space_key}`
- `POST /api/v1/spaces/{space_key}/archive`
- `GET /api/v1/spaces/{space_key}/tree`

### Documents

- `POST /api/v1/spaces/{space_key}/documents`
- `GET /api/v1/documents/{document_id}`
- `PUT /api/v1/documents/{document_id}/draft`
- `POST /api/v1/documents/{document_id}/publish`
- `POST /api/v1/documents/{document_id}/archive`
- `POST /api/v1/documents/{document_id}/move`
- `GET /api/v1/documents/{document_id}/revisions`
- `GET /api/v1/documents/{document_id}/revisions/{revision_id}`

### Task dossiers

- `GET /api/v1/spaces/{space_key}/tasks`
- `GET /api/v1/spaces/{space_key}/tasks/{task_key}`
- `POST /api/v1/spaces/{space_key}/tasks/{task_key}/links/documents`
- `GET /api/v1/spaces/{space_key}/tasks/{task_key}/documents`
- `GET /api/v1/spaces/{space_key}/tasks/{task_key}/evidence`

### Phase dossiers

- `GET /api/v1/spaces/{space_key}/phases`
- `GET /api/v1/spaces/{space_key}/phases/{phase_key}`
- `POST /api/v1/spaces/{space_key}/phases/{phase_key}/links/documents`
- `GET /api/v1/spaces/{space_key}/phases/{phase_key}/documents`
- `GET /api/v1/spaces/{space_key}/phases/{phase_key}/evidence`

### Evidence and attachments

- `POST /api/v1/evidence`
- `GET /api/v1/evidence/{evidence_id}`
- `GET /api/v1/evidence`
- `POST /api/v1/attachments`
- `GET /api/v1/attachments/{attachment_id}`
- `GET /api/v1/attachments/{attachment_id}/download`

### Search, templates, audit

- `GET /api/v1/search`
- `GET /api/v1/templates`
- `POST /api/v1/templates`
- `GET /api/v1/settings`
- `GET /api/v1/audit-log`

## 9. CLI overview

CLI повторяет базовые группы API:

| Группа     | Команды MVP                                                       |
| ---------- | ----------------------------------------------------------------- |
| `auth`     | `login`, `logout`, `whoami`                                       |
| `space`    | `list`, `create`, `get`, `tree`, `members`                        |
| `doc`      | `create`, `get`, `draft`, `publish`, `archive`, `move`, `history` |
| `task`     | `get`, `docs`, `evidence`, `link-doc`                             |
| `phase`    | `get`, `docs`, `evidence`, `link-doc`                             |
| `evidence` | `add-link`, `add-file`, `get`, `list`                             |
| `template` | `list`, `apply`                                                   |
| `search`   | `query`                                                           |
| `settings` | `get`                                                             |

CLI requirements:

- JSON output по умолчанию;
- ненулевой exit code для ошибок;
- ошибки совместимы с API error envelope;
- ввод Markdown из файла или stdin;
- write-команды передают `Idempotency-Key`;
- CLI не обращается к PostgreSQL, storage или внутренним backend-модулям напрямую.

## 10. Нефункциональные требования

### Security

- Все protected endpoints требуют аутентификации.
- Markdown рендерится только через sanitizer.
- Search не возвращает документы без прав.
- Attachment download проверяет права на owner entity.
- Secrets, bearer tokens и private storage keys не попадают в ответы API, audit и logs.

### Data integrity

- Published revision immutable.
- Восстановление старого содержания создаёт новую ревизию.
- Документы, evidence и файлы не могут пересекать границы space.
- Удаление в MVP является archive/soft-delete.
- Файл сохраняется атомарно: metadata без bytes или bytes без metadata не остаются как валидный attachment.

### Performance

- Списки имеют pagination или limit.
- Открытие обычной страницы до 200 KB должно быть интерактивным.
- Поиск MVP реализуется через PostgreSQL full-text search.

## 11. Критерии приёмки MVP

1. Admin создаёт space, добавляет editor/viewer и видит audit действий.
2. Editor создаёт страницу из шаблона, редактирует Markdown и публикует ревизию.
3. Viewer открывает опубликованную страницу, но не может изменить её.
4. Editor строит parent/child дерево и перемещает страницу внутри space.
5. Документ связывается с task key; task page показывает связанные документы.
6. Документ и evidence связываются с phase key; phase page показывает связанные материалы.
7. Editor добавляет URL evidence и file evidence, checksum виден в metadata.
8. Поиск находит документ по title/body и уважает права пользователя.
9. UI и CLI выполняют одинаковые базовые операции через `/api/v1`.
10. README содержит актуальные скриншоты основных frontend-страниц.

## 12. Roadmap

### Phase 1 - Base Wiki

- Auth/users/roles.
- Spaces and members.
- Documents, drafts, publish, revision history.
- Page tree and move.
- Markdown render and sanitizer.
- PostgreSQL migrations.
- API, UI and CLI for core flows.

### Phase 2 - SDLC Links

- Task dossiers by task key.
- Phase dossiers by phase key.
- Evidence links/files.
- Templates.
- Search filters.
- Audit coverage.

### Deferred

- Comments and mentions.
- Reports beyond basic lists.
- Object storage beyond local adapter.
- Advanced approvals.
- Import/export bundles.
- Real-time collaboration.

## 13. Rust-ready parts

| Зона          | Библиотека                             | Назначение                                   |
| ------------- | -------------------------------------- | -------------------------------------------- |
| HTTP API      | `axum`, `tokio`, `tower`, `tower-http` | REST API и middleware                        |
| DB            | `sqlx` + PostgreSQL                    | Явные SQL-запросы, транзакции, FTS           |
| OpenAPI       | `utoipa`, `utoipa-axum`                | API schema                                   |
| CLI           | `clap`, `reqwest`, `serde_json`        | Консольный клиент                            |
| Markdown      | `comrak`                               | Markdown parsing/rendering                   |
| Sanitization  | `ammonia`                              | Очистка HTML                                 |
| Storage       | local FS за trait                      | Файлы MVP                                    |
| Auth          | `argon2`, JWT/session middleware       | Пароли и сессии                              |
| Config        | `config` + process env                 | Typed config from TOML and `WIKI_` variables |
| Observability | `tracing`, `metrics`                   | Логи, health, базовые метрики                |

## 14. Связанные документы

- `docs/API.md`
- `docs/CLI.md`
- `docs/DATA_MODEL.md`
- `docs/DOMAIN_MODEL.md`
- `docs/ROADMAP.md`
- `docs/TZ.md`
