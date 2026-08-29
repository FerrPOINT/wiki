# Routing - Wiki

## 1. Overview

Frontend-роуты объявлены в `frontend/src/app/router.tsx`. На первом экране должна открываться рабочая Wiki-панель, а не landing page.

## 2. Route Groups

| Group | Auth | Layout |
|---|---|---|
| Public | no | standalone |
| App | yes | `AppShell` |
| Catch-all | any | redirect to `/` |

## 3. Public Routes

| Route | Page | Notes |
|---|---|---|
| `/login` | `pages/login` | Вход |
| `/register` | `pages/register` | Регистрация |

## 4. Protected Routes

| Route | Page | Notes |
|---|---|---|
| `/` | `pages/dashboard` | Обзор Wiki и последние документы |
| `/spaces` | `pages/spaces` | Список пространств и дерево документов |
| `/documents/new` | `pages/document-compose` | Создание документа |
| `/documents/:documentId` | `pages/document` | Просмотр документа |
| `/tasks` | `pages/task-dossier` | Список карточек задач |
| `/tasks/:taskKey` | `pages/task-dossier` | Документы и материалы по задаче |
| `/phases` | `pages/phase-dossier` | Список фаз workflow |
| `/phases/:phaseId` | `pages/phase-dossier` | Документы и материалы по фазе workflow |
| `/evidence` | `pages/evidence` | Реестр материалов, ссылок и файлов |
| `/templates` | `pages/templates` | Шаблоны документов |
| `/audit-log` | `pages/audit-log` | Журнал аудита |
| `/users` | `pages/users` | Пользователи и роли |
| `/settings` | `pages/settings` | Настройки инстанса |
| `/search` | `pages/wiki-search` | Поиск по документам, задачам, фазам и материалам |
| `/admin` | `pages/admin` | Администрирование |

## 5. URL Parameters

| Param | Pattern | Example |
|---|---|---|
| `:documentId` | document slug or UUID | `product-requirements` |
| `:taskKey` | external task key | `SDLC-42` |
| `:phaseId` | workflow phase key | `implementation` |

## 6. Query Parameters

| Param | Used On | Description |
|---|---|---|
| `q` | `/search` | Полнотекстовый запрос |
| `space` | `/search`, `/spaces` | Фильтр по пространству |
| `type` | `/search` | `document`, `task`, `phase`, `evidence` |
| `tag` | `/search` | Фильтр по тегу |
| `cursor` | list pages | Cursor pagination |
| `limit` | list pages | Page size |

## 7. UX Rules

- MVP navigation: Обзор, Пространства, Задачи, Фазы, Материалы, Шаблоны, Поиск.
- Admin navigation: Аудит, Пользователи, Настройки, Администрирование.
- Create action opens `/documents/new`.
- Document pages must keep title, breadcrumbs, revision status and linked task/phase visible above the fold.
- Task and phase pages must show linked documents and materials.

## 8. References

- `docs/FRONTEND_ARCHITECTURE.md`
- `docs/API.md`
- `docs/PRODUCT_REQUIREMENTS.md`
