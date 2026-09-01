# Документация Wiki

Локальная карта документации. Быстрый старт, скриншоты и общая витрина проекта находятся в [корневом README](../README.md).

## С чего начать

- [CURRENT_STATE](CURRENT_STATE.md) - что реально готово сейчас и где границы MVP.
- [MVP_READINESS](MVP_READINESS.md) - 100% gate перед началом основной разработки.
- [PRODUCT_REQUIREMENTS](PRODUCT_REQUIREMENTS.md) - основной документ требований Wiki.
- [ARCHITECTURE_INDEX](ARCHITECTURE_INDEX.md) - вход в архитектуру, ADR и bounded contexts.
- [USER_GUIDE](USER_GUIDE.md) - пользовательские сценарии базовой Wiki.
- [DEVELOPMENT_GUIDE](DEVELOPMENT_GUIDE.md) - локальная разработка и проверки.
- [NEXT_STEPS](NEXT_STEPS.md) - оставшаяся работа после текущего MVP shell.

## MVP Контракт

- [API](API.md), [API_STANDARDS](API_STANDARDS.md), [API_VERSIONING](API_VERSIONING.md) - публичный `/api/v1`.
- [CLI](CLI.md) - консольный клиент к тому же API.
- [ROUTING](ROUTING.md), [PAGE_DESIGN](PAGE_DESIGN.md), [UI_UX](UI_UX.md) - утвержденные frontend routes и страницы.
- [TRACEABILITY](TRACEABILITY.md) - связь требований, API, UI, тестов и visual evidence.
- [MVP_READINESS](MVP_READINESS.md) - итоговый go/no-go checklist и readiness coverage.
- [assets/screens/manifest.md](assets/screens/manifest.md) - manifest скриншотов; PNG лежат в [screenshots/](screenshots/).

## Архитектура И Данные

- [ARCHITECTURE](ARCHITECTURE.md), [FUNCTIONAL_ARCHITECTURE](FUNCTIONAL_ARCHITECTURE.md), [DOMAIN_MODEL](DOMAIN_MODEL.md) - слои и доменная модель.
- [DATA_MODEL](DATA_MODEL.md), [DATABASE_INDEXES](DATABASE_INDEXES.md), [MIGRATIONS](MIGRATIONS.md) - целевая PostgreSQL схема.
- [STORAGE](STORAGE.md), [STORAGE_ARCHITECTURE](STORAGE_ARCHITECTURE.md) - файлы, checksum и storage adapter.
- [AUTHORIZATION](AUTHORIZATION.md), [SECURITY](SECURITY.md), [THREAT_MODEL](THREAT_MODEL.md) - права, безопасность и угрозы.
- [LIBRARIES](LIBRARIES.md), [TECH_CHOICES](TECH_CHOICES.md) - Rust/React библиотеки и rationale.

## Эксплуатация И Качество

- [TEST_PLAN](TEST_PLAN.md), [TESTING](TESTING.md), [API_EDGE_CASES](API_EDGE_CASES.md) - стратегия проверки.
- [OPERATIONS](OPERATIONS.md), [OPS_RUNBOOK](OPS_RUNBOOK.md), [RUNTIME](RUNTIME.md), [DEPLOYMENT](DEPLOYMENT.md) - запуск и эксплуатация.
- [MONITORING](MONITORING.md), [METRICS](METRICS.md), [SLO](SLO.md), [LOGGING_STANDARDS](LOGGING_STANDARDS.md) - наблюдаемость.
- [BACKUP_RESTORE](BACKUP_RESTORE.md), [DISASTER_RECOVERY](DISASTER_RECOVERY.md), [INCIDENT_RESPONSE](INCIDENT_RESPONSE.md) - восстановление и инциденты.
- [RISK_REGISTER](RISK_REGISTER.md), [DOCUMENTATION_GOVERNANCE](DOCUMENTATION_GOVERNANCE.md) - риски и правила изменения документации.

## Deferred Reference

- [REPORTS](REPORTS.md), [NOTIFICATIONS](NOTIFICATIONS.md), [WEBHOOKS](WEBHOOKS.md), [RUNNER_ARCHITECTURE](RUNNER_ARCHITECTURE.md) - справочные документы для будущих фаз, не MVP route/API promise.
- [CI_CD](CI_CD.md), [WORKFLOW](WORKFLOW.md), [GIT_HOSTING](GIT_HOSTING.md) - внешние SDLC связи, которые Wiki только документирует.
- [ADR](ADR.md) и [adr/](adr/) - принятые решения; менять через новые ADR или обновление существующих с датой.

## Source Of Truth

1. Код runtime router и Rust handlers.
2. `openapi/openapi.json`.
3. Committed migrations после Wiki schema migration.
4. ADR/contracts.
5. Остальная документация.
