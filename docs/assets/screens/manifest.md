# Wiki screenshots manifest

Скриншоты фиксируют текущий frontend-shell Wiki для проверки покрытия страниц, визуального состояния.

## Capture

| Параметр         | Значение                                         |
| ---------------- | ------------------------------------------------ |
| Tool             | Playwright Chromium                              |
| Command          | `cd frontend && node scripts/shoot-evidence.mjs` |
| Build source     | `vite preview` production bundle                 |
| Theme            | Dark                                             |
| Auth state       | Mocked authenticated user for private pages      |
| Desktop viewport | 1920x1080                                        |

## Desktop pages

| Файл                                                                         | Route                             | Назначение                                       | Размер    |
| ---------------------------------------------------------------------------- | --------------------------------- | ------------------------------------------------ | --------- |
| [03-dashboard.png](../../screenshots/03-dashboard.png)                       | `/`                               | Dashboard Wiki, последние документы и task-связи | 1920x1080 |
| [04-spaces.png](../../screenshots/04-spaces.png)                             | `/spaces`                         | Пространства, дерево документов и доступы        | 1920x1080 |
| [05-document-compose.png](../../screenshots/05-document-compose.png)         | `/documents/new`                  | Создание документа                               | 1920x1080 |
| [06-document-view.png](../../screenshots/06-document-view.png)               | `/documents/product-requirements` | Просмотр, редактирование, публикация и ревизии   | 1920x1294 |
| [07-task-dossiers.png](../../screenshots/07-task-dossiers.png)               | `/tasks`                          | Карточки задач                                   | 1920x1080 |
| [08-task-dossier-detail.png](../../screenshots/08-task-dossier-detail.png)   | `/tasks/SDLC-42`                  | Документы и фазы задачи                          | 1920x1080 |
| [09-phase-dossiers.png](../../screenshots/09-phase-dossiers.png)             | `/phases`                         | Карточки фаз workflow                            | 1920x1080 |
| [10-phase-dossier-detail.png](../../screenshots/10-phase-dossier-detail.png) | `/phases/implementation`          | Карточка workflow phase                          | 1920x1080 |
| [11-evidence.png](../../screenshots/11-evidence.png)                         | `/evidence`                       | Реестр материалов                                | 1920x1080 |
| [12-templates.png](../../screenshots/12-templates.png)                       | `/templates`                      | Шаблоны документов                               | 1920x1080 |
| [13-audit-log.png](../../screenshots/13-audit-log.png)                       | `/audit-log`                      | Журнал аудита                                    | 1920x1080 |
| [14-users.png](../../screenshots/14-users.png)                               | `/users`                          | Пользователи и роли                              | 1920x1080 |
| [15-settings.png](../../screenshots/15-settings.png)                         | `/settings`                       | Настройки инстанса                               | 1920x1080 |
| [16-search.png](../../screenshots/16-search.png)                             | `/search`                         | Поиск по документам, задачам, фазам и материалам | 1920x1080 |
| [17-admin.png](../../screenshots/17-admin.png)                               | `/admin`                          | Администрирование                                | 1920x1080 |

## Повторная проверка поиска, 2026-09-19

Полностраничные снимки Playwright Chromium из локального развёртывания.
Для плотной выдачи подставлены ответы `GET /api/v1/search`; вход через SSO и
первый запрос поиска выполнены на живых сервисах. Изменяющих запросов нет.

| Файл | Экран | Состояние |
| --- | --- | --- |
| [search-375.png](2026-09-19-search/search-375.png) | 375x812 | Результаты, фильтры закрыты |
| [search-1920.png](2026-09-19-search/search-1920.png) | 1920x900 | Результаты, фильтры закрыты |
| [search-2560.png](2026-09-19-search/search-2560.png) | 2560x900 | Результаты, фильтры закрыты |
| [search-filters-375.png](2026-09-19-search/search-filters-375.png) | 375x812 | Фильтры применены |

## Review checklist

- Every route listed in `README.md` has a desktop screenshot.
- Key authenticated flows are covered with deterministic mocked API responses.
- Any frontend route change must update this manifest and regenerate screenshots.
