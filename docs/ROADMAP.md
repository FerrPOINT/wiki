# Roadmap - Wiki

Roadmap фиксирует только базовое приложение. Расширения не должны попадать в MVP, пока не закрыты основные сценарии страниц, связей и evidence.

## Phase 1 - Base Wiki

- Поддерживать идентичность проекта `wiki` и env prefix `WIKI_`.
- Поддерживать auth: register/login/logout/refresh/current user.
- Поддерживать пользователей, роли `admin/editor/viewer` и доступ к spaces.
- Поддерживать spaces, members, archive.
- Поддерживать документы: create, view, draft edit, publish, archive, move.
- Поддерживать immutable revision history.
- Поддерживать page tree: parent/child, breadcrumbs, move within space.
- Поддерживать Markdown render и HTML sanitization.
- Поддерживать PostgreSQL migrations для базовых сущностей.
- Поддерживать Docker Compose, `/api/v1/health` и readiness endpoint.

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

## Phase 3 - Clients Completion And Hardening

- Поддерживать UI для всех MVP-сценариев.
- Поддерживать CLI для тех же MVP-операций, что поддерживает API.
- Обновлять OpenAPI после изменения endpoints.
- Держать UI/CLI на публичном API без прямого доступа к backend internals.
- Расширять contract tests для API.
- Расширять smoke/e2e для основных UI-сценариев.

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
