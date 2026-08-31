# Архитектура Wiki

## 1. Контекст

Wiki - self-hosted база знаний для SDLC. Продукт хранит документы, версии, вложения и evidence, связанные с внешними task key и phase key.

Главное отличие от `task-tracker`, `project-workflow` и `CI-CD`: Wiki не владеет задачами, фазами, pipeline execution или Git-источниками. Она хранит только страницы, связи и подтверждающие материалы.

## 2. Клиенты и API

```text
UI  ─┐
     ├── public REST API ── application layer ── domain ── infra
CLI ─┘
```

UI и CLI используют один публичный `/api/v1`. CLI не имеет отдельной доменной модели или специальных команд под отдельный тип потребителя: это обычный клиент к API.

## 3. Технологический стек

### Backend

| Компонент     | Библиотека           | Назначение                              |
| ------------- | -------------------- | --------------------------------------- |
| Язык          | Rust 2024            | Backend и CLI                           |
| Web framework | axum                 | REST API, middleware, routing           |
| Async runtime | tokio                | Асинхронный runtime                     |
| DB            | PostgreSQL + sqlx    | Документы, версии, права, audit, search |
| Markdown      | comrak               | Markdown parsing/rendering              |
| Sanitization  | ammonia              | Очистка HTML перед показом              |
| OpenAPI       | utoipa + utoipa-axum | Генерация API schema                    |
| CLI           | clap + reqwest       | HTTP-only CLI                           |
| Observability | tracing + metrics    | Логи, health, базовые метрики           |

### Frontend

| Компонент    | Библиотека               | Назначение         |
| ------------ | ------------------------ | ------------------ |
| Framework    | React + TypeScript       | SPA                |
| Build        | Vite                     | Dev/build          |
| Styling      | Tailwind CSS + shadcn/ui | UI kit             |
| Server state | TanStack Query           | API cache          |
| Routing      | react-router             | Навигация          |
| i18n         | i18next                  | Русский/английский |

## 4. Целевая структура

```text
wiki/
├── backend/
│   ├── domain/      # Space, Document, Revision, Task/Phase links, Evidence
│   ├── app/         # use cases и транзакционные границы
│   ├── infra/       # PostgreSQL, local storage, search
│   ├── api/         # axum routes, DTO, OpenAPI
│   ├── server/      # composition root
│   ├── cli/         # wiki CLI
│   ├── migration/   # versioned migrations
│   └── shared/      # config, errors, ids, public Wiki API contract
├── frontend/
│   └── src/{api,app,pages,features,entities,shared,widgets}
├── openapi/
├── docs/
└── scripts/
```

## 5. Dependency Flow

```text
frontend -> API client -> REST API
cli      -> HTTP client -> REST API
api      -> app         -> domain
infra    -> app/domain  -> PostgreSQL/local storage/search
server   -> api/app/infra runtime composition
```

`domain` не импортирует Axum, SQLx, файловую систему, HTTP clients или Markdown renderer.

## 6. Domain Layer

Основные сущности MVP:

- `User` - учётная запись.
- `Space` - верхний контейнер базы знаний.
- `SpaceMember` - роль пользователя в space.
- `Document` - логическая страница.
- `DocumentDraft` - редактируемый черновик.
- `DocumentRevision` - immutable опубликованная версия.
- `TaskDossier` - срез документов/evidence по внешнему task key.
- `PhaseDossier` - срез документов/evidence по phase key.
- `EvidenceItem` - URL или файл evidence.
- `Attachment` - metadata файла в storage.
- `DocumentTemplate` - Markdown-шаблон.
- `AuditEntry` - append-only audit.

Инварианты:

- опубликованная ревизия не меняется задним числом;
- восстановление старого содержания создаёт новую ревизию;
- document/task/phase/evidence/attachment не пересекают `space_id`;
- Wiki не меняет состояние внешних задач, фаз или pipeline;
- секреты и токены не попадают в search index, audit и пользовательский HTML.

Default `domain` crate export surface содержит Wiki-модель и общие `value_objects`, нужные Wiki. Унаследованные task-tracker `email_port`, `entities`, `events`, `jql`, `repositories`, memory compatibility repositories, `IssueQuery` и `StatusDto` доступны только через feature `legacy-tracker`.

## 7. Application Layer

Use cases MVP:

- login/logout/current user;
- управление users, roles и space members;
- управление spaces;
- создание, редактирование draft, публикация и archive документов;
- управление деревом страниц;
- связь документов с task key;
- связь документов/evidence с phase key;
- добавление URL/file evidence;
- поиск документов;
- применение шаблонов;
- audit write-действий.

Каждый write use case выполняется в транзакции там, где это требуется целостностью данных.

Текущий первый application-layer slice: `app::wiki` содержит лёгкий `WikiAppContext` для runtime config, общие правила нормализации Wiki keys/types/roles, access predicates, search criteria normalization, Markdown text extraction, checksums, safe download filenames, password hashing, Wiki JWT/session token helpers и сборку access/refresh token pair с TTL. Shared contract `shared::wiki_contract` содержит публичные Wiki DTO, `WikiSettingsSnapshot` и `WikiBackendPort`, чтобы API handlers, CLI/UI contract generation и infra adapter не зависели друг от друга циклически. API использует эти helpers вместо приватных route-level validator/security functions и не декларирует прямые crypto dependencies для Wiki auth. Унаследованные task-tracker app modules (`auth`, `authz`, `commands`, `context`, `dto`, `services`) исключены из default-сборки и доступны только через feature `legacy-tracker`.

## 8. Infrastructure Layer

- PostgreSQL repositories и migrations.
- Local storage adapter для attachments через `domain::wiki::WikiAttachmentStorage`.
- Markdown rendering через `comrak`.
- HTML sanitization через `ammonia`.
- Search projection: PostgreSQL full-text search.
- Audit repository.

Default `infra` crate export surface contains the Wiki local attachment storage adapter and the transitional SQLx/PostgreSQL Wiki backend adapter. Inherited task-tracker `cache`, `db`, `email`, `entities`, `event_bus`, `jql`, `repos` and issue attachment `storage` modules are compatibility code behind feature `legacy-tracker`.

## 9. API Layer

Целевая граница Axum API отвечает за:

- request extraction;
- DTO validation;
- auth/session extraction;
- idempotency key;
- вызов application use cases;
- единый error envelope;
- OpenAPI generation.

В целевой архитектуре API не содержит SQL и не пишет файлы напрямую. Текущий переходный MVP runtime отделяет HTTP handlers/OpenAPI, shared contract, явный memory test/dev backend и постоянное хранилище: route handlers вызывают `shared::wiki_contract::WikiBackendPort`, публичные request/response DTO живут в `shared::wiki_contract`, а конкретный `PostgresWikiBackend` приватен внутри `infra::wiki_postgres`. Переходный PostgreSQL adapter уже разделён на `connection`, `queries`, `mapping`, `identity`, `spaces`, `documents` и `dossiers`; auth/users/settings, spaces/space_members/tree, document/revision и task/phase dossier операции отделены от основного operation module, но все операции ещё должны переехать в application use cases и dedicated infra repositories. `server` запускает Wiki через `app::WikiAppContext` и `infra::connect_postgres_wiki_backend` без сборки унаследованного task-tracker service graph. Production `server::run` требует `WIKI_DATABASE__URL`; memory backend доступен только через явно названный test/dev router builder. Default domain/app/infra dependency surface теперь содержит только Wiki-needed exports.

## 10. Frontend

MVP pages:

- login/register;
- dashboard;
- spaces and page tree;
- document view/editor/history;
- task page;
- phase page;
- evidence list/upload;
- search;
- templates;
- users/settings/audit for admin.

Текущий React shell уже заменён на Wiki-навигацию и страницы целевого продукта. Backend API переключён на Wiki MVP router/OpenAPI: публичный слой больше не экспонирует task-tracker endpoints. Production server использует переходный SQLx/PostgreSQL adapter из `infra` за `WikiBackendPort` и не стартует без `WIKI_DATABASE__URL`; memory backend остаётся явным режимом быстрых тестов. Server/API state уже использует Wiki-specific context, а полноценные app use cases/repositories ещё должны заменить переходный persistence adapter.

## 11. CLI

`wiki` - HTTP-only клиент для тех же MVP-операций, что доступны через API: `auth`, `space`, `doc`, `task`, `phase`, `evidence`, `template`, `search`.

## 12. Deferred

- Comments and mentions.
- Advanced reports.
- Object storage beyond local adapter.
- Import/export bundles.
- Approval chains.
- Real-time collaboration.
- Confluence import/macros.

## 13. Статус миграции

| Срез                                                                | Статус                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Репозиторий скопирован из `task-tracker`                            | Готово                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| Идентичность проекта `wiki`/`WIKI_`                                 | Готово                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| Product requirements reduced to base app                            | Готово                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| Удаление task-tracker-only documentation/screenshots/frontend pages | Готово                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| Замена публичного API/router на Wiki MVP                            | Готово                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| Wiki domain value objects and invariants                            | Готово                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| Domain legacy quarantine                                            | Готово: default `domain` экспортирует только Wiki/value-object слой; унаследованные task-tracker domain modules доступны только через feature `legacy-tracker`                                                                                                                                                                                                                                                                                                                                  |
| Fresh SQLx Wiki schema baseline                                     | Готово                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| Route-level SQLx runtime persistence                                | Готово для MVP: PostgreSQL adapter закрыт за `shared::wiki_contract::WikiBackendPort` и вынесен из API crate в `infra::wiki_postgres`; `connection`, SQL constants, row mapping, identity/auth/users/settings, spaces/space_members/tree, document/revision и task/phase dossier operation modules отделены от основного transition module; базовые space-role checks включены; attachment bytes вынесены за Wiki storage port; shared Wiki validation/auth/search helpers вынесены в app layer |
| Wiki contract boundary                                              | Готово: публичные Wiki DTO, `WikiSettingsSnapshot` и `WikiBackendPort` живут в `shared::wiki_contract`; API route module re-export'ит DTO для OpenAPI, но не владеет persistence contract                                                                                                                                                                                                                                                                                                       |
| Wiki backend port                                                   | Готово: route handlers завязаны на `WikiBackendPort`, а конкретный `PostgresWikiBackend` приватен внутри переходного `infra::wiki_postgres`                                                                                                                                                                                                                                                                                                                                                     |
| Wiki runtime context                                                | Готово: API/server используют `app::WikiAppContext` и больше не собирают унаследованный task-tracker `AppContext`/services при запуске Wiki                                                                                                                                                                                                                                                                                                                                                     |
| Production PostgreSQL runtime                                       | Готово: `server::run` требует `WIKI_DATABASE__URL`; memory backend используется только через явный test/dev builder                                                                                                                                                                                                                                                                                                                                                                             |
| App legacy quarantine                                               | Готово: унаследованные task-tracker app modules исключены из default-сборки и доступны только через feature `legacy-tracker`                                                                                                                                                                                                                                                                                                                                                                    |
| Infra legacy quarantine                                             | Готово: default `infra` экспортирует Wiki storage/PostgreSQL runtime adapters; унаследованные task-tracker infra modules и их SeaORM/JQL/email/cache dependencies доступны только через feature `legacy-tracker`                                                                                                                                                                                                                                                                                |
| Разбиение transition SQLx adapter на app/repositories               | В работе: identity/auth/users/settings, spaces/space_members/tree, document/revision и task/phase dossier operation modules отделены; следующий шаг - evidence/attachment/template/audit/search modules, затем полноценные app use cases и dedicated repositories                                                                                                                                                                                                                               |
| Замена frontend страниц на Wiki UI                                  | Готово                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| Перегенерация OpenAPI                                               | Готово для MVP API                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| Generated frontend client                                           | После стабилизации PostgreSQL-backed API                                                                                                                                                                                                                                                                                                                                                                                                                                                        |

## References

- `docs/PRODUCT_REQUIREMENTS.md`
- `docs/API.md`
- `docs/CLI.md`
- `docs/ROADMAP.md`
