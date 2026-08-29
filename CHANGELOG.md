# Changelog

Все значимые изменения проекта документируются здесь.
Формат основан на [Keep a Changelog](https://keepachangelog.com/ru/1.1.0/),
версионирование - [SemVer](https://semver.org/lang/ru/).

## [0.1.0] - 2026-08-27

### Added

- Проект `wiki` развёрнут на основе кодовой базы `task-tracker`.
- Сохранена существующая git-история репозитория `wiki`; внутрь скопирован frontend/backend/CLI/tooling каркас.
- Добавлен основной документ требований `docs/PRODUCT_REQUIREMENTS.md`.
- Корневая документация переключена на Wiki: README, архитектура, API, roadmap, CLI, operations, governance, contracts и skill description.
- Проектная идентичность переименована в `wiki` / `WIKI_`: env vars, docker-compose, package names, docker binary name.
- CLI заменён на целевой wiki-интерфейс: spaces, documents, task dossiers, phase dossiers, evidence, search, export.
- Frontend route shell расширен до Wiki-страниц: documents, spaces, dossiers, evidence, templates, integrations, reports, audit, users, settings.

### Removed

- Удалены task-tracker-only документы `docs/TZ.md`, `docs/UI_UX.md`, `docs/WORKFLOW.md`, `docs/REPORTS.md`.
- Удалены старые screenshots, Jira sample assets и временный Hermes-план по time tracking.
- Из основной пользовательской документации убраны kanban/backlog/sprint-обещания старого продукта.
- Удалены унаследованные task-tracker frontend pages/features/entities/API modules; оставлен тонкий Wiki API-клиент для auth/notifications.

### Known Legacy

- Backend всё ещё содержит унаследованные модули старого task-tracker домена. Следующий шаг: заменить модель задач/досок на Wiki-сущности `Space`, `Document`, `DocumentRevision`, `TaskDossier`, `PhaseDossier`, `Evidence` и `Attachment`.
- `openapi/openapi.json` пока унаследован из исходного проекта и должен быть перегенерирован после реализации Wiki API.
