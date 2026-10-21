# AGENTS.md — Wiki

## Docker и Kubernetes

Поддерживаются обычный Docker/Compose без Kubernetes и Kubernetes. Общие API,
данные, настройки и правила исполнения сохраняют одинаковый смысл в обоих
режимах; особенности оркестратора принадлежат deployment-адаптерам.
Перед изменением runtime/config/storage/runner читать [стандарт Base](https://github.com/FerrPOINT/services-base/blob/29a709834fba54a823873b82050731333d7e3d97/docs/REPO_STANDARDS.md#поддержка-docker-и-kubernetes).
Затронутый сценарий проверять отдельно в обоих режимах на точном кандидате;
непроверенный режим оставлять открытым gate. Docs-only правки не требуют deploy
или пересборки. Ссылка фиксирует решение Base, не меняя SDK/UI/skills pins.

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
- Frontend: публичные компоненты `@pdlc/ui` + Tailwind; предметные композиции остаются в продукте.
- Состояние: серверное — `@tanstack/react-query`, клиентское — `zustand`.
- Формы — native React forms/local validators для MVP; новые form/schema библиотеки добавлять только при явной необходимости.

### 3. Коммиты

- Conventional commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`).
- Один коммит = одна логическая единица.
- Не amend/squash без явного запроса.
- Push только после релевантных проверок. Если проверка заблокирована окружением, зафиксировать причину в `docs/CURRENT_STATE.md` и финальном отчёте.

### 4. Тестирование

- Backend: `cargo fmt`, `cargo clippy`, `cargo test`, интеграционные тесты с PostgreSQL testcontainer.
- Frontend: Vitest + Playwright.
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

- [ ] Все тесты проходят.
- [ ] Линтеры (`clippy`, `eslint`, `prettier`) чистые.
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

## Термины PDLC и общие компоненты

Перед изменением платформенных имён прочитайте [интеграцию с Base](docs/PDLC_BASE_INTEGRATION.md) и канонические TERMS/BRANDING.
Работа с UI описана в разделе «Разработка UI со справочником Base» ниже.
Не переносите продуктовые запросы, DTO и права в общий UI. SDK/UI/skills pins независимы; новое имя платформы не меняет SDLC IDs/scopes или Hermes upstream.

## Общие поля выбора Base

Сначала искать общий UI в Base: search → show → example. Общие исправления
вносить в Base; продукт передаёт данные, права, преобразования и сохранение.
Для одного известного значения использовать Select, для нескольких — MultiSelect,
для каталога с поиском/пагинацией — SearchableSelect из публичного @pdlc/ui.
Контракт Select/MultiSelect: options, value/defaultValue, onValueChange;
HTML children/option/onChange/multiple/size больше не являются их API.
Видимые raw select и локальные копии оформления не добавлять. Технический form
bridge принадлежит только Base; SSR использует штатный macro и choice-runtime,
явный refresh после программных value/options и native fallback до успешного init.
Правила выбора и подключения: Base docs/platform/CHOICE_CONTROLS.md.
UI/SDK/skills pins независимы; установка и production — отдельные операции.

## Разработка UI со справочником Base

Перед созданием или изменением UI читать [маршрут разработки со справочником Base](https://github.com/FerrPOINT/services-base/blob/ae2b41291eee5af752daff89f760fba5aff41783/base-modules/ui/reference/guides/agent.md).
Из корня продукта с подготовленным frontend: `pnpm --dir frontend exec pdlc-ui`.
Сначала сверить `.base-ui-revision` и выполнить `check` один раз в начале задачи
или после смены UI pin; затем `search "задача"` либо адресный `show`, при
необходимости `--section props` или `--section setup`, и один `example`. Использовать точный
квалифицированный адрес из поиска; композиции доступны через `example recipes`.
Без подготовленного frontend руководство описывает CLI из отдельного checkout
того же UI SHA без npm install. Не загружать весь каталог в контекст и не
подменять принятую версию через npx/dlx/latest или скрытый HTTP fallback.
Проверить providers/Router/i18n/CSS и cleanup; после интеграции — scoped проверки
продукта и IAB затронутого поведения. Источники, hosted CI и runtime — разные gates.

Button, Input, Textarea, Checkbox, RadioGroup/Radio, Switch, меню, Card, Table, Disclosure,
Dialog и Toaster/toast использовать из публичного @pdlc/ui. Общий дефект
исправлять в Base; продукт сохраняет данные, права и сохранение. Прямой Sonner,
копии примитивов и продуктовые определения общих tokens не добавлять.
Брендинг применяется общим runtime; draft preview изолируется BrandingScope.
Канонический контракт — Base docs/platform/DESIGN_SYSTEM.md и docs/BRANDING.md.
UI/SDK/skills pins независимы; source-изменения не означают установку.
