# Архитектура Wiki

## 1. Контекст

Wiki - self-hosted база знаний для SDLC. Продукт хранит документы, версии, вложения и evidence, связанные с внешними task key и phase key.

Wiki не владеет задачами, фазами, pipeline execution или Git-источниками. Она хранит только страницы, связи и подтверждающие материалы.

## 2. Клиенты и API

```text
UI  ─┐
     ├── public REST API ── application layer ── domain ── infra
CLI ─┘
```

UI и CLI используют один публичный `/api/v1`. CLI не имеет отдельной доменной модели или специальных команд под отдельный тип потребителя: это обычный клиент к API.

## 3. Технологический стек

### Backend

| Компонент | Библиотека | Назначение |
| --------- | ---------- | ---------- |
| Язык | Rust 2024 | Backend и CLI |
| Web framework | axum | REST API, middleware, routing |
| Async runtime | tokio | Асинхронный runtime |
| DB | PostgreSQL + SQLx | Документы, версии, права, audit, search |
| Markdown | comrak | Markdown parsing/rendering |
| Sanitization | ammonia | Очистка HTML перед показом |
| OpenAPI | utoipa + utoipa-axum | Генерация API schema |
| CLI | clap + reqwest | HTTP-only CLI |
| Observability | tracing + metrics | Логи, health, базовые метрики |

### Frontend

| Компонент | Библиотека | Назначение |
| --------- | ---------- | ---------- |
| Framework | React + TypeScript | SPA |
| Build | Vite | Dev/build |
| Styling | Tailwind CSS + shadcn/ui | UI kit |
| Server state | TanStack Query | API cache |
| Routing | react-router | Навигация |
| i18n | i18next | Русский/английский |

## 4. Структура

```text
wiki/
├── backend/
│   ├── domain/      # Wiki value objects, entities and invariants
│   ├── app/         # Wiki use cases, validation and repository ports
│   ├── infra/       # SQLx PostgreSQL adapter and local attachment storage
│   ├── api/         # axum routes, DTO/OpenAPI exposure, memory test backend
│   ├── server/      # composition root
│   ├── cli/         # wiki CLI
│   ├── migrations/  # canonical SQLx migration SQL files
│   ├── migration/   # thin SQLx migration runner
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

Current export surface of `domain` contains only Wiki value objects and shared value-object helpers needed by Wiki.

## 7. Application Layer

`app::wiki` owns MVP use cases and business validation:

- login/logout/current user and session token helpers;
- users, roles and space members;
- spaces and page tree;
- document create/draft/publish/archive/move/revision history;
- task and phase dossier document links;
- URL/file evidence and attachment command assembly;
- search criteria normalization and result shaping;
- template create/list rules;
- audit command normalization.

Application code calls repository ports and does not know SQL or local filesystem details. Each write use case is expected to stay transactional at repository level when the command and audit record must be committed together.

## 8. Infrastructure Layer

`infra` contains only Wiki runtime adapters:

- `wiki_postgres`: SQLx/PostgreSQL backend behind `shared::wiki_contract::WikiBackendPort`.
- `wiki_storage`: local filesystem implementation of `domain::wiki::WikiAttachmentStorage`.

The PostgreSQL adapter is split by operation area:

- `connection` and bootstrap;
- SQL constants in `queries`;
- row mapping in `mapping`;
- identity/auth/users/settings;
- spaces/members/tree;
- documents/revisions;
- task/phase dossiers;
- evidence/attachments;
- templates;
- audit;
- search.

## 9. API Layer

Axum API owns:

- request extraction;
- DTO validation;
- auth/session extraction;
- idempotency key extraction;
- use case calls through `WikiBackendPort`;
- unified error envelope;
- OpenAPI generation.

The API crate does not own production SQLx code. Memory backend is available only through explicit test/dev router builders.

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

Visible UI text is Russian by default. Routes and code identifiers stay English.

## 11. CLI

`wiki` is an HTTP-only client for the same MVP operations as UI:

- `auth`;
- `user`;
- `space`;
- `doc`;
- `task`;
- `phase`;
- `evidence`;
- `attachment`;
- `template`;
- `search`;
- `audit`;
- `settings`.

CLI returns JSON by default, exits non-zero on API errors and sends `Idempotency-Key` for write commands.

## 12. Deferred

- Comments and mentions.
- Advanced reports.
- Notification center and delivery channels.
- Webhook ingestion and outbound delivery.
- Import/export bundles.
- Approval chains.
- Real-time collaboration.
- OCR and binary attachment indexing.

## 13. Статус миграции

| Срез | Статус |
| ---- | ------ |
| Репозиторий скопирован и переименован под Wiki | Готово |
| Product requirements reduced to base app | Готово |
| Task-tracker-only frontend pages/screenshots removed | Готово |
| Public API/router reduced to Wiki MVP | Готово |
| Wiki domain value objects and invariants | Готово |
| Fresh SQLx Wiki schema baseline | Готово |
| SQLx migration runner over `backend/migrations` | Готово |
| Public Wiki DTOs and backend port moved to `shared::wiki_contract` | Готово |
| Production PostgreSQL runtime through private `infra::wiki_postgres` adapter | Готово |
| API/server runtime through `app::WikiAppContext` | Готово |
| Copied task-tracker backend modules removed from domain/app/infra | Готово |
| Frontend Wiki MVP shell and API-backed pages | Готово |
| OpenAPI MVP artifact | Готово |
| CLI MVP command surface | Готово |
| PostgreSQL smoke with fresh disposable DB | Runner готов: `pwsh -File scripts/postgres-smoke.ps1`; успешный прогон нужен на host с Docker/Postgres |
| Search FTS plan/index evidence | Env-gated API test готов; успешный `EXPLAIN` output нужно сохранить после DB smoke |

## References

- `docs/PRODUCT_REQUIREMENTS.md`
- `docs/API.md`
- `docs/CLI.md`
- `docs/ROADMAP.md`
