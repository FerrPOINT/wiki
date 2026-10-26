# AGENTS.md — Wiki

## Репозиторий

- **GitHub**: `git@github.com:FerrPOINT/wiki.git`
- **Стек**: backend Rust (Axum + SQLx/PostgreSQL), frontend React 19.1.0 + Vite 6.2.0 + Tailwind CSS 4.1.0
- **Env prefix**: `WIKI_`
- **Публичные порты по умолчанию**: frontend docker `19877`, backend `3456`, PostgreSQL `3457`

## Правила работы

### 1. Перед началом работы

1. Прочитать `docs/PRODUCT_REQUIREMENTS.md`, `docs/ARCHITECTURE.md`, `docs/DATA_MODEL.md`.
2. Проверить текущее состояние ветки: `git status`.
3. Составить план, показать пользователю, получить подтверждение.

### 2. Код

- Backend: целевая слоистая архитектура `api/routes → app/services → domain → infra/repositories`.
- Runtime DI для Wiki идёт через `WikiAppContext`; backend runtime не собирает task-tracker services/entities/repositories.
- API routes обращаются к постоянному хранилищу через `shared::wiki_contract::WikiBackendPort`; конкретные SQLx/PostgreSQL adapters не вызывать напрямую из handlers. Production composition создаётся в `server` через `infra`.
- Все публичные API покрыты OpenAPI через `utoipa-axum`.
- Rust-хендлеры и DTO — единственный источник правды для схемы; frontend DTO types генерируются из `openapi/openapi.json`, а thin endpoint wrappers остаются временным слоем до полного generated operation client.
- Новую Wiki persistence-логику писать на SQLx по ADR-0001. Не возвращать task-tracker domain/app/infra services, routes, DTO или ORM-модули в MVP.
- Все endpoint тестируются интеграционно; PostgreSQL smoke запускается через Docker Postgres или изолированную WSL PostgreSQL database.
- Frontend: компоненты на `shadcn/ui` + Tailwind.
- Состояние: серверное — `@tanstack/react-query`, клиентское — `zustand`.
- Формы — native React forms/local validators для MVP; новые form/schema библиотеки добавлять только при явной необходимости.

### 3. Коммиты

- Conventional commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`).
- Один коммит = одна логическая единица.
- Не amend/squash без явного запроса.
- Push только после релевантных проверок. Если проверка заблокирована окружением, зафиксировать причину в `docs/CURRENT_STATE.md` и финальном отчёте.

### 4. Тестирование

- Обычная сборка/разработка: backend `cargo test --locked -p domain --lib`, frontend `pnpm test:unit`; команды и scope — `.quality-gate.json`.
- Integration с PostgreSQL, Playwright и полный набор — ручной release gate по `docs/TESTING.md`.
- После UI-изменений — скриншоты full-page (375 / 1920 / 2560).
- Все новые endpoint — curl-проверка.

### 5. Документация

- При изменении API обновлять `docs/API.md`.
- При изменении дата-модели обновлять `docs/DATA_MODEL.md`.
- При новом функционале обновлять `docs/PRODUCT_REQUIREMENTS.md` или соответствующий `docs/*.md`.
- Любые неочевидные решения фиксировать в `docs/ARCHITECTURE.md`.

### 6. Безопасность

- Никогда не коммитить credentials, токены, пароли, email коллег, реальные данные клиентов.
- Все secrets — через env vars.
- Перед push проверять, что в diff нет чувствительных данных.

### 7. Docker

- Сборка: `docker compose build`.
- Пересоздание контейнера: `docker compose up -d` (не `docker compose restart`).
- Проверка: `docker compose ps` и health endpoint.

### 8. Проверка перед завершением

- [ ] Пройдены unit/static checks затронутых компонентов; повтор использует подходящие receipts.
- [ ] Для релиза отдельно подтверждены integration/slow/full coverage и необходимые security/data/live gates на exact candidate; невыполненные проверки обозначены NOT RUN.
- [ ] Документация актуальна.
- [ ] Коммиты запушены, если пользователь просил push.
- [ ] Пользователь увидел результат (скриншот / curl / лог).

## Контакты

- Техлид: Александр Жуков.
- Основной язык общения и документов: русский.

## References

- `docs/ARCHITECTURE.md`
- `docs/CODE_STYLE.md`
- `docs/TESTING.md`

## Base и воспроизводимость

См. [BASE_INTEGRATION](docs/BASE_INTEGRATION.md). Обязательны pinned Base SHA,
locked/frozen зависимости и проверка актуальных checkout до сборки.
Rust build toolchain 1.88.0, Node 22.20.0, pnpm 10.28.1; MSRV отдельно.

## Быстрые проверки и сборка

Перед тестами, build и CI читать обязательный [стандарт Base](https://github.com/FerrPOINT/services-base/blob/f910017264a730089b38c11ee8ee53b29eed8fe2/docs/BUILD_PERFORMANCE.md).
При разработке и сборке unit/static. Integration/slow/full coverage —
ручной gate при подготовке релиза по docs/TESTING.md.
Сохранять полезный cache, не переустанавливать неизменные dependencies и не
пересобирать приложение из-за test-only. До повторного тяжёлого запуска назвать
изменившиеся inputs/недостающее evidence; передавать один artifact между этапами.
Required security/data/compatibility/live checks и product toolchains сохраняются.
Компоненты и команды: `.quality-gate.json`; общие правила не копировать.

Уровни и команды: [TESTING](docs/TESTING.md). Полный `.github/workflows/release.yml`
запускается вручную на exact ref; зелёный unit build не означает release acceptance.

## Выбор сборки и повторное использование

Перед build/test/install определить нужные компоненты и изменившиеся реальные
inputs; проверить готовый artifact, environment и cache. Неизменные inputs и
проверенный artifact означают отсутствие повторных install/compile/source export.
Docs/test-only не пересобирают приложение без изменения его build inputs.
Во время работы выполнять scoped unit/static; полный gate — на согласованной
вехе. Integration/slow/full coverage — вручную перед релизом; необходимые
security/data/compatibility/live gates сохраняются. Полные команды в этом
документе — каталог проверок, а не требование выполнять всё после каждой правки.
Перед повторным heavy назвать changed inputs или missing proof; REUSED не
выдавать за новый тест. Полезный прогретый cache сохранять в его бюджете.
В source workspace читать `docs/BUILD_PERFORMANCE.md` из `qualityTooling.path`
в `../source-workspace.json` и [правила workspace](../AGENTS.md).
В standalone checkout использовать стандарт принятой версии Base.
