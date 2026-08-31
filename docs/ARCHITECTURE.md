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
│   └── shared/      # config, errors, ids
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

Текущий первый application-layer slice: `app::wiki` содержит общие правила нормализации Wiki keys/types/roles, access predicates, search criteria normalization, Markdown text extraction, checksums, safe download filenames, password hashing, Wiki JWT/session token helpers и сборку access/refresh token pair с TTL. API использует эти helpers вместо приватных route-level validator/security functions и не декларирует прямые crypto dependencies для Wiki auth.

## 8. Infrastructure Layer

- PostgreSQL repositories и migrations.
- Local storage adapter для attachments через `domain::wiki::WikiAttachmentStorage`.
- Markdown rendering через `comrak`.
- HTML sanitization через `ammonia`.
- Search projection: PostgreSQL full-text search.
- Audit repository.

## 9. API Layer

Целевая граница Axum API отвечает за:

- request extraction;
- DTO validation;
- auth/session extraction;
- idempotency key;
- вызов application use cases;
- единый error envelope;
- OpenAPI generation.

В целевой архитектуре API не содержит SQL и не пишет файлы напрямую. Текущий переходный MVP runtime уже отделяет router/DTO/memory fallback от PostgreSQL-реализации: SQLx-запросы живут в `api::routes::wiki::postgres`, а следующий архитектурный шаг - перенос этого поведения в application use cases и infra repositories.

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

Текущий React shell уже заменён на Wiki-навигацию и страницы целевого продукта. Backend API переключён на Wiki MVP router/OpenAPI: публичный слой больше не экспонирует task-tracker endpoints. Runtime использует memory fallback для быстрых тестов и переходный SQLx/PostgreSQL adapter `api::routes::wiki::postgres` при заданном `WIKI_DATABASE__URL`; полноценные app use cases/repositories и infra wiring ещё должны заменить унаследованные backend modules.

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

| Срез                                                                | Статус                                                                                                                                                                                                                                                        |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Репозиторий скопирован из `task-tracker`                            | Готово                                                                                                                                                                                                                                                        |
| Идентичность проекта `wiki`/`WIKI_`                                 | Готово                                                                                                                                                                                                                                                        |
| Product requirements reduced to base app                            | Готово                                                                                                                                                                                                                                                        |
| Удаление task-tracker-only documentation/screenshots/frontend pages | Готово                                                                                                                                                                                                                                                        |
| Замена публичного API/router на Wiki MVP                            | Готово                                                                                                                                                                                                                                                        |
| Wiki domain value objects and invariants                            | Готово                                                                                                                                                                                                                                                        |
| Fresh SQLx Wiki schema baseline                                     | Готово                                                                                                                                                                                                                                                        |
| Route-level SQLx runtime persistence                                | Готово для MVP: PostgreSQL adapter вынесен из основного router/DTO файла в `api::routes::wiki::postgres`; базовые space-role checks включены; attachment bytes вынесены за Wiki storage port; shared Wiki validation/auth/search helpers вынесены в app layer |
| Замена app/repositories/runtime wiring на Wiki persistence          | Следующий шаг                                                                                                                                                                                                                                                 |
| Замена frontend страниц на Wiki UI                                  | Готово                                                                                                                                                                                                                                                        |
| Перегенерация OpenAPI                                               | Готово для MVP API                                                                                                                                                                                                                                            |
| Generated frontend client                                           | После стабилизации PostgreSQL-backed API                                                                                                                                                                                                                      |

## References

- `docs/PRODUCT_REQUIREMENTS.md`
- `docs/API.md`
- `docs/CLI.md`
- `docs/ROADMAP.md`
