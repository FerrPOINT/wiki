# Page Design Contract - Wiki MVP

## 1. Purpose

This document fixes the approved MVP page set and the expected page composition while backend persistence is being implemented. UI and CLI are ordinary clients of the same public API; page design must not introduce hidden report-only, notification-only or external-sync-only product scope.

## 2. Approved Routes

| Route                    | Страница           | Основная работа                                              |
| ------------------------ | ------------------ | ------------------------------------------------------------ |
| `/login`                 | Вход               | Авторизовать пользователя                                    |
| `/register`              | Регистрация        | Создать учётную запись                                       |
| `/`                      | Обзор              | Проверить последние документы и незакрытые связи             |
| `/spaces`                | Пространства       | Открыть пространства и деревья документов                    |
| `/documents/new`         | Создание документа | Создать Markdown-черновик с метаданными                      |
| `/documents/:documentId` | Просмотр документа | Прочитать текущую ревизию и связанный контекст               |
| `/tasks`                 | Задачи             | Найти знания по внешним ключам задач                         |
| `/tasks/:taskKey`        | Карточка задачи    | Проверить документы, фазы и материалы задачи                 |
| `/phases`                | Фазы               | Проверить состояние документации по фазам workflow           |
| `/phases/:phaseId`       | Карточка фазы      | Проверить документы и материалы фазы                         |
| `/evidence`              | Материалы          | Найти URL/файлы и метаданные                                 |
| `/templates`             | Шаблоны            | Переиспользовать структуры документов                        |
| `/audit-log`             | Аудит              | Смотреть неизменяемые события                                |
| `/users`                 | Пользователи       | Управлять пользователями, ролями и доступами                 |
| `/settings`              | Настройки          | Смотреть runtime-политики инстанса, поиска, доступа и локали |
| `/search`                | Поиск              | Искать документы, задачи, фазы и материалы                   |
| `/admin`                 | Администрирование  | Открывать админ-разделы и статус готовности MVP              |

## 3. Shared Page Rules

- Visible product text is Russian by default.
- Technical route names, API fields and code identifiers remain English.
- Every API-backed page needs loading, empty, permission denied, validation error and retry states before release readiness.
- Every mutating page must show the user what object will be affected before the command is submitted.
- Document, task, phase and evidence pages must keep the current space/task/phase context visible above the fold.
- Deferred reports, notifications, webhooks and external source sync must not appear as MVP routes, navigation items or required API clients.

## 4. Dashboard

The dashboard summarizes:

- recent published or draft documents;
- missing task/phase documentation links;
- core counts for spaces, documents, phases and materials;
- primary actions for creating a document and searching.

The dashboard is not a report builder. It can show small operational gaps that help users continue daily work.

## 5. Spaces

The spaces page shows:

- one card per space;
- document and member counts;
- last updated marker;
- a short tree preview;
- link to the document tree/root page.

Tree previews load through `/spaces/{space_key}/tree` and must preserve permission filtering after SQLx persistence is wired.

## 6. Documents

Document compose includes:

- title, space and document type;
- template shortcuts;
- Markdown editor;
- preview tab;
- task and phase links;
- visible metadata and tags.

Document view includes:

- breadcrumb, status and current revision;
- draft title/Markdown editor;
- publish action with revision summary;
- archive action with confirmation;
- parent document field for moving within the tree;
- primary body content;
- linked task and phase;
- revision timeline;
- related documents and related evidence materials.

## 7. Tasks And Phases

Task and phase pages are knowledge views, not workflow executors.

Task page shows:

- external task key;
- document count, phase count and evidence/material count;
- linked documents table;
- phase summary;
- related materials.

Phase page shows:

- phase key/name;
- documentation readiness;
- linked phase documents;
- linked materials;
- missing material hints.

## 8. Evidence

Evidence registry shows:

- file and URL material counts;
- filters by text, space, document, task and phase;
- table with material title, document, task, phase, evidence type and date;
- upload and add-link actions.

Evidence is permission-filtered by space and by linked document/task/phase visibility.

## 9. Templates

MVP templates are:

- requirements;
- research note;
- implementation note;
- test plan;
- release note.

Templates are starting points for documents, not approval workflows.

## 10. Admin Pages

Admin pages cover only MVP administration:

- users and roles;
- settings;
- audit log;
- admin overview.

Admin pages must not expose deferred integrations, reports, notification delivery or runner controls before those features receive separate requirements.

## 11. Screenshot Evidence

Screenshots are generated by `frontend/scripts/shoot-evidence.mjs` against `vite preview` and listed in `docs/assets/screens/manifest.md`. Any route, navigation or page design change must update:

- README screenshot gallery;
- screenshot manifest;
- generated PNG files.

## 12. References

- `docs/ROUTING.md`
- `docs/UI_UX.md`
- `docs/FRONTEND_ARCHITECTURE.md`
- `docs/contracts/UI_API_CONTRACT.md`
