# Roadmap - Wiki

Roadmap фиксирует только базовое приложение. Расширения не должны попадать в MVP, пока не закрыты основные сценарии страниц, связей и evidence.

## Phase 1 - Base Wiki

- Завершить переименование проекта в `wiki` и env prefix `WIKI_`.
- Заменить унаследованную task-tracker domain model на Wiki-сущности.
- Реализовать auth: login/logout/current user.
- Реализовать пользователей, роли `admin/editor/viewer` и доступ к spaces.
- Реализовать spaces, members, archive.
- Реализовать документы: create, view, draft edit, publish, archive.
- Реализовать immutable revision history.
- Реализовать page tree: parent/child, breadcrumbs, move within space.
- Реализовать Markdown render и HTML sanitization.
- Реализовать PostgreSQL migrations для базовых сущностей.
- Поднять Docker Compose, `/health`, `/ready`.

## Phase 2 - SDLC Links

- Реализовать task dossier по внешнему `task_key`.
- Реализовать phase dossier по `phase_key`.
- Связать документы с task key и phase key.
- Добавить URL evidence.
- Добавить file evidence с checksum.
- Добавить списки evidence по документу, задаче и фазе.
- Добавить базовые templates: requirements, research note, implementation note, test plan, release note.
- Реализовать PostgreSQL full-text search по title/body.
- Добавить фильтры поиска: space, task key, phase key, document type.
- Покрыть write-действия audit log.

## Phase 3 - Clients Completion

- Довести UI до всех MVP-сценариев.
- Довести CLI до тех же MVP-операций, что поддерживает API.
- Обновить OpenAPI после реализации endpoints.
- Подключить UI/CLI к публичному API без прямого доступа к backend internals.
- Добавить contract tests для API.
- Добавить smoke/e2e для основных UI-сценариев.

## Deferred

- Comments and mentions.
- Advanced reports.
- Object storage adapter beyond local files.
- Import/export bundles.
- Approval chains.
- Real-time collaborative editing.
- Confluence import/macros.

## Definition of Done

- Capability имеет REQ-ID в `docs/PRODUCT_REQUIREMENTS.md`.
- API endpoint отражён в OpenAPI.
- Есть migration и persistence test для новой таблицы или связи.
- UI использует публичный API.
- CLI команда использует публичный API.
- Документы не обещают capability как MVP, если она не входит в базовый scope.
