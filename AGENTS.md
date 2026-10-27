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

Свободная строка с локальными подсказками использует AutocompleteInput Base:
options/value/defaultValue/onValueChange(string). Произвольные значения остаются допустимыми;
закрытый Select для этого не подходит. Native input, ref, FormData, reset, popup и клавиатура
принадлежат Base; запросы и сохранение — продукту. Не добавлять list/datalist или локальный
popup.

Сначала искать общий UI в Base: search → show → example. Общие исправления
вносить в Base; продукт передаёт данные, права, преобразования и сохранение.
Для закрытого списка с одним значением — Select; для нескольких — MultiSelect.
Одно значение из всех видимых вариантов — RadioGroup, взаимноисключающие режимы — appearance="segmented".
Вкладки с панелями используют Tabs. Каталог с поиском/пагинацией — SearchableSelect.
Использовать публичный @pdlc/ui и сверять наличие API по точному UI pin.
Контракт Select/MultiSelect: options, value/defaultValue, onValueChange;
HTML children/option/onChange/multiple/size больше не являются их API.
Видимые raw select и локальные копии оформления не добавлять. Технический form
bridge принадлежит только Base; SSR использует штатный macro и choice-runtime,
явный refresh после программных value/options и native fallback до успешного init.
Правила выбора и подключения: Base docs/platform/CHOICE_CONTROLS.md.
UI/SDK/skills pins независимы; установка и production — отдельные операции.

## Разработка UI со справочником Base

Открытая страница справочника может быть новее зависимости продукта: сверять
контракт по принятому UI pin. Если поиск пуст, уточнить запрос/фильтры и
проверить рецепты; отсутствие API проверить отдельно в актуальном Base.
Общий контракт добавлять в Base вместе с metadata и примером, затем явно
обновлять UI pin потребителя. Локальная копия не заменяет эту интеграцию.

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
Dialog, ActionGroup и Toaster/toast использовать из публичного @pdlc/ui. Общий дефект
исправлять в Base; продукт сохраняет данные, права и сохранение. Прямой Sonner,
копии примитивов и продуктовые определения общих tokens не добавлять.
Брендинг применяется общим runtime; draft preview изолируется BrandingScope.
Размеры однострочных контролов и типографика/padding TabsTrigger принадлежат
Base; локальные h-10/min-h-10 и мобильные копии не добавлять. Textarea,
ширина предметной панели, внешний margin и компоновка сохраняют свой контракт.
Строки явных действий используют ActionGroup (SSR — action_group): перенос и gap
принадлежат Base, внешние margin/width/grid — экрану. Выбор значения использует
RadioGroup/Select; контекст ресурса не превращается в фиктивную строку действий.
Канонический контракт — Base docs/platform/DESIGN_SYSTEM.md и docs/BRANDING.md.
Иконки действий используют Button size="icon", текстовые компактные действия —
штатный size="sm". Кнопки шаблонов сохраняют Wiki-обработчик и pending, но не
копируют padding, border, цвета и hover/focus общих контролов.
UI/SDK/skills pins независимы; source-изменения не означают установку.
ThemeProvider/useTheme подключаются напрямую из публичного @pdlc/ui/lib.
Новый локальный ThemeContext, storage runtime или ненужный private re-export
не создавать; исправления общего поведения темы принадлежат Base.


Разделитель под заголовком задаёт SectionHeader separated. Для прозрачной области
содержимого используйте ContentSection (SSR content_section): dividers/inset и density
принадлежат Base; native section/form, сетка, внешние отступы и handlers — продукту.
ResourceList предназначен для строк, Fieldset — для группы полей. Сначала сверить UI pin.
Фон CodeBlock/JsonBlock и выбранная TableRow selected оформляются Base; продукт
сохраняет viewport, содержимое, выбранный ID и доступные действия. Не копировать
border/padding заголовка и bg на этих общих компонентах. Наличие props сверять по UI pin.

## Единое пространство PDLC

[Согласованный план](docs/plans/unified-spaces-20261019.md) вводит M:N и общий read admission. До полного
совместимого cohort это цель, не установленное поведение. Строгие V1/V2 и
applied migrations сохранять; ProjectRef/SpaceRef/V3 отдельны. Полный gate на
интегрированной вехе, в ходе реализации scoped checks. Runtime не активировать
по готовности одного продукта; не выбирать первый Project/Space неявно.

Записи Wiki content/member используют existing SQLx begin_content_write и
write_authorization::commit с immutable исходным proof. Предварительная роль
до row wait не достаточна. Порядок locks и личный staged upload:
[PROJECT_CONTENT_AUTHORIZATION_V2](docs/PROJECT_CONTENT_AUTHORIZATION_V2.md).
Owner lifecycle/Tracker projection/maintenance имеют отдельные узкие ports;
не переносить их полномочия в обычный human запрос.

SQLx errors внутри wiki_postgres передавать через typed database_error: SQLSTATE
должен сохраняться до выбора публичного AppError. Не добавлять авто-повтор
записи при conflict. Правила кодов и private diagnostics — в документе выше.

## Подписи и карточки вариантов выбора

После проверки API в принятом UI pin используйте label или children Checkbox/Radio
для обычной подписи, appearance="card" — для варианта с рамкой и описанием.
Не копируйте label с border/rounded/padding и не оборачивайте такой выбор в
Card asChild. Base владеет hit target, focus, checked/disabled и нативной формой;
продукт — данными, правами и обработчиком. Внешняя grid/flex-компоновка допустима.
SSR использует checkbox_control/radio_control с appearance='card'; отдельный
click-handler или повторное input/change для оформления не нужен.
Контракт и пример: Base docs/platform/CHOICE_CONTROLS.md, pdlc-ui example Checkbox --case cards.


Видимый выбор файла использует `Input type="file"` Base: кнопка выбора,
focus и disabled следуют теме и branding. Native файловый диалог остаётся у ОС;
чтение, проверка содержимого/размера и отправка — у продукта. Скрытый технический
file input за Button допустим; видимый raw input не добавлять. Для обновления
оформления сверять UI pin; CLI/Telegram без web UI не получают новых зависимостей.

Подписи web-полей оформлять публичным Label; подпись/подсказку/ошибку одного
поля — FormField Base. Продукт сохраняет текст, htmlFor/id, required, данные
и обработчики; не копировать типографику подписи и фон/рамку/радиусы/padding
Input/Textarea. Внешняя компоновка, высота Textarea и моноширинный код допустимы.


## Композиция списков и страниц

Пагинация использует PageNavigation либо подходящий OffsetPageNavigation Base.
URL, курсоры, проверка допустимости страницы и callbacks принадлежат продукту.
Невалидное состояние блокируется disabled, запрос — pending; не создавать
фиктивные страницы или повторы кнопок. ResourceList linked-actions оформляет
сам anchor через interactive ResourceListRow, а независимое действие — рядом.
Сначала сверять наличие API по точному UI pin; локальные hover/focus/размеры
вместо общего контракта не добавлять.
Разделитель и внутренний отступ после содержимого задаются PageNavigation separated;
внешние margin, ширина и выравнивание остаются у продукта. Border/padding не копировать.

Границу перед группой полей задаёт Fieldset separated; native legend/disabled/FormData
и данные остаются у владельца. Для Disclosure используйте appearance, tone=muted/danger
и size=sm вместо локальных border/padding, цветов и типографики. Внешние margin,
ширина и предметное содержимое остаются у продукта; наличие props сверять по UI pin.
## Меню аккаунта Base

При принятом UI pin с AccountMenu использовать общую композицию аккаунта,
темы и выхода. AppShell применяет её сам; renderHeader требует явного
AccountMenu в actions общего PlatformHeader. Identity, permissions,
logout/pending и предметные пункты beforeTheme/children остаются у продукта;
не копировать trigger, popup или theme/logout layout. Контракт и пример
сначала читать через pdlc-ui; custom API/SSO и установленный runtime не меняются.

## Коллекции боковой навигации

После проверки UI pin используйте SidebarList Base для списка и separated
для его разделителя, nested — для отступа вложенной коллекции. asChild
сохраняет native nav/ul/li/section, hidden, ID и обработчики без DOM-оболочки.
Ключ mapped-узла передавать внешнему SidebarList. Данные, ACL, URL, раскрытие
и внешние margin/width остаются у продукта; внутренний base-sidebar-list и
его border/padding/gap не копировать. Сначала pdlc-ui show и один example.
Корневой список боковых ссылок также использует SidebarList asChild;
map/filter, маршруты и вызов onNavigate остаются у владельца приложения.

Подготовленный HTML документа и ревизии использует Prose keyboardScroll из
проверенного UI pin. Base владеет tab stops/focus paint для raw pre/table;
Wiki сохраняет Markdown/sanitization, права, body_html и пустые состояния.
Не восстанавливать локальный querySelectorAll(pre/table) для tabindex.
JSX-содержимое использует CodeBlock/Table, без скрытого HTML-преобразования.

React Markdown preview использует CodeBlock Base со стабильным renderer mapping
вне компонента страницы. Inline renderer пересоздаёт DOM и теряет focus при
изменении заголовка/темы. Парсер, skipHtml, политика ссылок и черновик остаются
у Wiki; оформление и клавиатурный контракт CodeBlock не копировать.


Текст бокового пункта после проверки UI pin использует SidebarItemLabel Base.
Не копировать base-sidebar-item-label/truncate: compact и сокращение общие.
Иконка, полное aria-label/title, маршрут, current state и callbacks остаются
продукту. У текстового slot нет собственного SSR macro; отдельную кнопку
или focus target вокруг подписи не добавлять.

## Общий inline-код и выделение записи

После проверки UI pin используйте InlineCode для оформленного одиночного кода,
Prose для prepared текста, CodeBlock для многострочного блока и
ResourceListDescription technical для подписи без поверхности. Не копируйте
фон/padding/radius native code. ResourceListRow highlighted задаёт только
оформление: текущий ID, aria-current, ref/tabIndex и действия остаются у продукта.
SSR: inline_code и resource_list_row из штатного экспорта Base.
Каноническое руководство — Base docs/platform/DESIGN_SYSTEM.md и pdlc-ui.
