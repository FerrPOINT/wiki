# Wiki screenshots manifest

Скриншоты фиксируют текущий frontend-shell Wiki для проверки покрытия страниц, визуального состояния и базовой адаптивности.

## Capture

| Параметр | Значение |
|---|---|
| Tool | Playwright Chromium |
| Command | `cd frontend && node scripts/shoot-evidence.mjs` |
| Build source | `vite preview` production bundle |
| Theme | Dark |
| Auth state | Mocked authenticated user for private pages |
| Desktop viewport | 1920x1080 |
| Mobile viewport | 375x812 |

## Desktop pages

| Файл | Route | Назначение | Размер |
|---|---|---|---|
| [01-login.png](../../screenshots/01-login.png) | `/login` | Вход пользователя | 1920x1080 |
| [02-register.png](../../screenshots/02-register.png) | `/register` | Регистрация пользователя | 1920x1080 |
| [03-dashboard.png](../../screenshots/03-dashboard.png) | `/` | Dashboard Wiki, последние документы и незакрытые связи | 1920x1080 |
| [04-spaces.png](../../screenshots/04-spaces.png) | `/spaces` | Пространства и дерево документов | 1920x1080 |
| [05-document-compose.png](../../screenshots/05-document-compose.png) | `/documents/new` | Создание документа | 1920x1080 |
| [06-document-view.png](../../screenshots/06-document-view.png) | `/documents/product-requirements` | Просмотр документа и связей | 1920x1080 |
| [07-task-dossiers.png](../../screenshots/07-task-dossiers.png) | `/tasks` | Карточки задач | 1920x1080 |
| [08-task-dossier-detail.png](../../screenshots/08-task-dossier-detail.png) | `/tasks/SDLC-42` | Документы и фазы задачи | 1920x1080 |
| [09-phase-dossiers.png](../../screenshots/09-phase-dossiers.png) | `/phases` | Карточки фаз workflow | 1920x1080 |
| [10-phase-dossier-detail.png](../../screenshots/10-phase-dossier-detail.png) | `/phases/implementation` | Карточка workflow phase | 1920x1080 |
| [11-evidence.png](../../screenshots/11-evidence.png) | `/evidence` | Реестр материалов | 1920x1080 |
| [12-templates.png](../../screenshots/12-templates.png) | `/templates` | Шаблоны документов | 1920x1080 |
| [13-audit-log.png](../../screenshots/13-audit-log.png) | `/audit-log` | Журнал аудита | 1920x1080 |
| [14-users.png](../../screenshots/14-users.png) | `/users` | Пользователи и роли | 1920x1080 |
| [15-settings.png](../../screenshots/15-settings.png) | `/settings` | Настройки инстанса | 1920x1080 |
| [16-search.png](../../screenshots/16-search.png) | `/search` | Поиск по документам, задачам, фазам и материалам | 1920x1080 |
| [17-admin.png](../../screenshots/17-admin.png) | `/admin` | Администрирование | 1920x1080 |

## Mobile checks

| Файл | Route | Назначение | Размер |
|---|---|---|---|
| [m-dashboard.png](../../screenshots/m-dashboard.png) | `/` | Dashboard mobile layout | 375x1376 |
| [m-spaces.png](../../screenshots/m-spaces.png) | `/spaces` | Spaces mobile layout | 375x1138 |
| [m-document-view.png](../../screenshots/m-document-view.png) | `/documents/product-requirements` | Document view mobile layout | 375x1940 |
| [m-task-dossier.png](../../screenshots/m-task-dossier.png) | `/tasks/SDLC-42` | Task page mobile layout | 375x1089 |
| [m-search.png](../../screenshots/m-search.png) | `/search` | Search mobile layout | 375x918 |

## Review checklist

- Every route listed in `README.md` has a desktop screenshot.
- Key authenticated flows are covered with deterministic mocked API responses.
- Mobile smoke screenshots cover dashboard, navigation-heavy pages, document reading, task reading and search.
- Any frontend route change must update this manifest and regenerate screenshots.
