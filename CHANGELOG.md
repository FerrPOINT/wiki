# Changelog

Все значимые изменения проекта документируются здесь.
Формат основан на [Keep a Changelog](https://keepachangelog.com/ru/1.1.0/),
версионирование - [SemVer](https://semver.org/lang/ru/).

## [Unreleased]

### Added

- Frontend MVP pages подключены к публичному Wiki API для spaces, documents, tasks, phases, evidence, templates, users, audit и search.
- Добавлены рабочие UI-формы для создания документа, создания пользователя, URL evidence и file evidence.
- Screenshot/E2E mocks обновлены под API-backed страницы с минимальным честным seed dataset.

### Changed

- README, CURRENT_STATE, NEXT_STEPS и screenshot manifest синхронизированы с текущим API-backed frontend состоянием.
- Backend API warning по unused route import устранён.

## [0.1.0] - 2026-08-27

### Added

- Проект `wiki` развёрнут на основе кодовой базы `task-tracker`.
- Сохранена существующая git-история репозитория `wiki`; внутрь скопирован frontend/backend/CLI/tooling каркас.
- Добавлен основной документ требований `docs/PRODUCT_REQUIREMENTS.md`.
- Корневая документация переключена на Wiki: README, архитектура, API, roadmap, CLI, operations, governance, contracts и skill description.
- Проектная идентичность переименована в `wiki` / `WIKI_`: env vars, docker-compose, package names, docker binary name.
- CLI заменён на целевой wiki-интерфейс: spaces, documents, task dossiers, phase dossiers, evidence, templates and search.
- Frontend route shell расширен до Wiki-страниц: documents, spaces, dossiers, evidence, templates, audit, users, settings and search.

### Removed

- Удалены task-tracker-only документы и пользовательские обещания старого backlog/kanban/sprint продукта.
- Удалены старые screenshots, Jira sample assets и временный Hermes-план по time tracking.
- Из основной пользовательской документации убраны kanban/backlog/sprint-обещания старого продукта.
- Удалены унаследованные task-tracker frontend pages/features/entities/API modules; оставлен тонкий Wiki API-клиент.

### Known Legacy

- Backend всё ещё содержит унаследованные модули старого task-tracker домена. Следующий шаг: заменить модель задач/досок на Wiki-сущности `Space`, `Document`, `DocumentRevision`, `TaskDossier`, `PhaseDossier`, `Evidence` и `Attachment`.
- Runtime persistence still needs SQLx repositories; current public Wiki API shell is in-memory.
