# Интеграция wiki с Base

## Текущий source-кандидат UI

Точный UI SHA: `2de0f72b2c28f3d0930808a1c58a941f9c6be31e` ([pin](../.base-ui-revision)),
`@pdlc/ui` 0.3.26, зависимость — [Base #384](https://github.com/FerrPOINT/services-base/pull/384).
Reference: `0bb6a3074c8188354c79a5b4bdea9654d3c31a60a6b1eec3033c20ebc0f1488f`.
Компоненты и reference берутся из этой версии Base; открытый справочник другой
версии не подтверждает доступность контракта в продукте. SDK/skills pins
независимы. Это исходники собственного checkout; установка и authenticated
приёмка runtime этим значением не подтверждаются. Датированные разделы ниже
описывают прежние квалификации и не заменяют проверку итогового кандидата.

### Финальный Checkpoint 2de0f72b

Bounded fetch/body wait дополнен к owner-clock expiry. Frozen offline refresh,
exact snapshot, types, production build и shell contract PASS. Changed transport
покрыт 96 Base auth cases и 18 исходными Admin integration cases на final pin.
Нижние product auth cases относятся к d7c1d8c3 и переиспользованы по неизменным
React composition inputs; нового полного продуктового прогона нет.

### Первый P1 Checkpoint d7c1d8c3

Producer исправляет независимый JWT expiry через owner transport clock и
monotonic deadline, сохраняя закрытые 401/403 и expiry после owner outage.
Frozen offline install, exact installed snapshot, TypeScript, shell contract
и production build PASS; affected application-access suite: 4 PASS.
Production TS, API, SDK/skills pins и grants не менялись. Независимое review
требуется до ready/merge; stand и PM03 pin не затронуты. Остальная theme/browser
matrix остаётся follow-up; прежние 57 product tests не выдаются за новый run.

### История frozen UI 0.3.25 — 2026-10-29

Frozen offline install, точное совпадение установленного package snapshot,
CLI/reference, TypeScript, ESLint/semantic/shell guards, OpenAPI parity и
production build прошли. Scoped Vitest: 57 PASS в четырёх suites — app-shell,
application-access, document и settings. Production TS, API, SDK/skills pins,
права, данные и runtime не менялись; обновлены только UI pin и этот профиль.

Browser matrix финального pin остаётся открытой: light/gray/dark,
375/1920/2560, keyboard/focus/drawer/Portal, document/editor states, reload и
zoom 200%. Это не live-приёмка установленного Wiki. Hosted CI, общий
семиклиентный gate, merge и установка отдельны; Release C не установлен.
Текущий PM image с прежним UI не зависит от этой source-поставки.

Audit/evidence ячейки используют TableCell technical/size=sm Base; ссылки, запросы, форматирование и скачивание остаются Wiki.

## Исходная UI-поставка — 21 октября 2026

UI был закреплён на `94272dc5512be58ec6c88fb7c1d1d33c7599a3e4` ([pin](../.base-ui-revision)); зависит от [Base #297](https://github.com/FerrPOINT/services-base/pull/297).
Проверки продукта и границы fixture-приёмки: [отчёт квалификации](reports/ui-repin-qualification-20261021.md).
SDK и skills pins независимы и этим обновлением не меняются. Общая типографика,
плотность и отдельные радиусы применяются в Base к React, Portal и SSR;
продукт сохраняет данные, права, запросы и действия. Штатные SSR-ресурсы Workflow
обновлены из того же UI SHA. Это source-кандидат, а не установка платформы.

Датированные разделы ниже описывают историю квалификаций. Текущий pin сам по себе не подтверждает установку или authenticated-приёмку.

Для центральной проекции используется `ServiceBridge::try_token_with_name`:
текущее имя приходит из той же проверки сессии или личного токена, без отдельного
запроса за именем и без самостоятельного разбора JWT в Wiki. Закреплённая версия Base
содержит этот контракт; legacy standalone-вход не меняется.

Активный Base SHA закреплён в [`.namespace-base-revision`](../.namespace-base-revision).
При отсутствии этого файла CI и `scripts/build.py` используют legacy
[`.base-revision`](../.base-revision). `main` Base не является воспроизводимой
зависимостью. Checkout Base должен лежать соседним каталогом
`services-base`. Перед standalone build выполнить из корня продукта:

```sh
pin=.base-revision
if [ -f .namespace-base-revision ]; then pin=.namespace-base-revision; fi
python3 ../services-base/scripts/verify_base_revision.py --base ../services-base --revision "$pin"
python3 scripts/build.py
```

Docker standalone и umbrella используют соседние checkout как build context.
CI получает Base на том же SHA. Forge runner sources формируются штатным
`materialize_runner_sources.py` из чистого delivery candidate, с manifest происхождения.

Rust build toolchain: 1.88.0; Node: 22.20.0; pnpm: 10.28.1.
Cargo использует `--locked`; pnpm — `--frozen-lockfile`. MSRV проверяется отдельно.
Приёмка включает OpenAPI drift/compatibility, package consumer и effective-theme
проверки вместе с продуктовыми тестами. После frontend build запустить preview
и `pnpm theme:check http://127.0.0.1:4173`.

Base содержит технические механизмы; продукт сохраняет авторизацию, PAT scopes,
данные, ORM, миграции, HTTP статусы/коды/поля, env/defaults и навигацию.
Новых миграций БД в этой задаче нет. Откат — предыдущий проверенный набор SHA/images.

Текущая реализация и доказательства проверок учитываются в реестре Base
`docs/plans/base-unification-register.json`; незавершённые гейты не отмечаются пройденными.

Кнопка входа вызывает `beginSso(..., { interactive: true })`; автоматический
guard сохраняет неинтерактивный вызов. `isSsoNavigationInterruption` из Base
различает pending logout/отмену навигации и настоящий отказ Auth. Task Tracker
снимает блокировку кнопки при завершении или отмене promise. Identity, PKCE,
state, nonce, API и машинная авторизация не меняются. Source-регрессии покрыты
login-тестами; локальные fixture-проверки не заменяют живую SSO-приёмку.

Закреплённый Base сохраняет focus внутри общего `ConfirmDialog`, когда запрос
переводит все действия в pending: Tab/Shift+Tab остаются в диалоге, Escape не
закрывает незавершённую операцию. После её завершения действуют прежние правила
закрытия и возврата focus. Продуктовые операции и HTTP-контракты не меняются.
Обновление pin требует повторных frontend/package/theme и живых UI-проверок;
исторические результаты прежнего Base не подтверждают эту приёмку.

## Отдельный UI pin и размещение в Base

`.base-ui-revision` закрепляет frontend Base; `.base-revision` остаётся SDK pin.
Frontend job CI проверяет UI checkout независимо от backend jobs. Для frontend
сборки используйте `python scripts/build.py frontend` и соседний UI checkout
на точном SHA. Совместную пересборку backend и UI с разными pins выполняйте
отдельно, сохраняя исходный SDK checkout.

`VITE_PLATFORM_ORIGIN` передаётся как Docker build argument. Пустое значение
сохраняет standalone; настроенное включает memory router, silent SSO и общий
bridge. Продукт сохраняет собственные API, профиль, permissions и формы.
[Оболочка Base](https://github.com/FerrPOINT/services-base/blob/2523e7ac1b56a14e9038f41472df4ce7a114dc1e/frontend/platform/README.md) описывает
конфигурацию, подготовку кадра и отдельную runtime-приёмку.

## Реорганизация source Base

Общие Rust SDK находятся в `services-base/base-modules/rust`, Python SDK —
в `services-base/base-modules/python/cli-core`, UI — в `services-base/base-modules/ui`.
Auth, оператор, host и справочник имеют собственные корневые каталоги приложений.
Имена пакетов, HTTP/wire-контракты, данные и credentials этим переносом не меняются.
UI/SDK/skills pins независимы; source-комплект проверяется по точным revisions.
[Карта Base](https://github.com/FerrPOINT/services-base/blob/a4dea438ec99103ca83b51540d11de548b1ee82d/docs/platform/ARCHITECTURE.md)
и [ADR 0006](https://github.com/FerrPOINT/services-base/blob/a4dea438ec99103ca83b51540d11de548b1ee82d/docs/adr/0006-base-repository-layout.md)
определяют расположение и направление зависимостей. Установка выполняется отдельно.

## Каталог UI

UI pin принимает Base #245. При недоступном каталоге
`usePlatformServices().services` пуст: продуктовые ссылки показывают штатное
недоступное состояние. Меню сохраняет подписи без неподтверждённых переходов;
известные адреса из runtime/host работают независимо от health. Восстановление
каталога использует прежний интервал. SDK и HTTP-контракты не меняются.

## UI-кандидат 2026-10-18

Общий UI закреплён на `6f1c797a0b11380f7e36db5ced0360f25a0265a5`. SDK и skills pins остаются независимыми.
В этом кандидате shared/lib/theme.tsx был оставлен совместимым re-export Base;
позднее неиспользуемый private-модуль [удалён](#прямое-подключение-темы--2026-10-27).
Проверки source-кандидата и установленного PDLC2 учитываются отдельно;
обновление pin не означает установку.

## Общие строки действий — 2026-10-26

UI producer: `c0d2245e40303efffc04c7d8c24eb3746afd1439`. В Base добавлен ActionGroup и SSR action_group;
PageHeader, SectionHeader, PageNavigation и ChatComposerActions используют общий
каркас. Данные, права, курсоры, submit/pending, неизвестный результат записи и
readback сохраняются у продукта. Для пояснений используется ResourceListDescription.
Внешние margin/width/grid placement и sticky-контейнер принадлежат экрану;
общие gap, разделитель и мобильная компоновка — Base. Нативные формы,
тип кнопок, hidden, IDs и callbacks сохраняются. SDK и skills pins не изменены.

Проверены шесть offline frozen installs, singleton React, CLI/reference,
frontend types, guards и штатный SSR fingerprint. Изменённые React-сценарии
имеют scoped regression checks; общий UI проверен в IAB на трёх темах и
ширинах320/375/1920/2560. Это source adoption; установленный runtime, SSO
и продуктовые данные не изменялись. Полный release/hosted/runtime gate отдельно.

## Строки действий — 2026-10-26

Количество строк действий, переведённых на ActionGroup Base: 3. Дополнительно тип материала переведён на RadioGroup; сохранены значения, черновик и pending.
Внешнее размещение, видимость и предметные grid остаются у продукта; gap и
перенос — у Base. Callbacks, ref, права, native типы, URLs и сохранение
не меняются. UI producer остаётся c2ca61094a904706fceeda8eb8f3b3cfd2e7cdfc;
UI API, SDK/skills pins, зависимости и установленный runtime не обновляются.
Scoped проверки и компонентный IAB не заменяют полный integrated gate
и authenticated-приёмку страниц.

## Вертикальные группы действий — 2026-10-26

Принят UI producer `7294a7974a898f182c22034b511c6b4b15205cef`; SDK/skills pins сохранены.
Совместимый `ActionGroup orientation="vertical"` и SSR `action_group`
описаны в [контракте Base](https://github.com/FerrPOINT/services-base/blob/7294a7974a898f182c22034b511c6b4b15205cef/docs/plans/action-orientation-20261026.md).
Проверены архив UI, offline frozen install шести React-потребителей,
singleton React, CLI/reference, шесть frontend types и SSR fingerprint.
Forge дополнительно прошёл19 адресных тестов и lint двух изменённых модулей;
остальные backend/application builds не выполнялись.

IAB проверяет source-компонент Forge с синтетическими callbacks:24 сочетания
темы, ширины и направления; это не приёмка установленного приложения.
Данные, права, pending и подтверждения остаются у продукта; сетка корзины
Tracker связана с её данными и не получает общий breakpoint из Base.
Публикация, полный интеграционный gate и установка учитываются отдельно.

## Корневые коллекции навигации — 2026-10-26

-

+Корневой nav использует SidebarList asChild из текущего UI producer +`7294a7974a898f182c22034b511c6b4b15205cef`; вложенность Wiki использует nested.
+Native semantics, labels, маршруты, ACL, key и onNavigate сохранены.
+Gap и inset следуют плотности брендинга. Канонический контракт: +[списки Base](https://github.com/FerrPOINT/services-base/blob/7294a7974a898f182c22034b511c6b4b15205cef/docs/platform/DESIGN_SYSTEM.md#списки-боковой-навигации). +
+Types, lint и штатный shell:check прошли; проверены12 тестов оболочки Admin
+и38 тестов Wiki AppShell/WorkspaceNavigation. IAB проверяет source-функции
+с синтетическими данными в трёх темах, на ширинах320/375/1920/2560 и
+с тремя density. Это компонентная проверка, не установленное приложение.
+UI-архив переиспользован по совпадению863 production-файлов; UI/SDK/skills
+pins, зависимости и runtime не менялись. Полный интеграционный gate,
+публикация и hosted CI учитываются отдельно. +

## Подготовленный HTML и клавиатура — 2026-10-27

Принят совместимый UI producer `a95d9fe7b31cca23e1f6432bae43246120859efb`; SDK/skills pins сохранены.
Общий Prose keyboardScroll документирован в [Base](https://github.com/FerrPOINT/services-base/blob/a95d9fe7b31cca23e1f6432bae43246120859efb/docs/platform/DESIGN_SYSTEM.md).
Проверены настоящий архив UI, offline frozen install, singleton React,
reference/CLI и typecheck данного frontend.
Две страницы документа используют Base вместо локального DOM-цикла.
Прошли 27 адресных тестов, scoped lint и namespace-denied regression.
IAB проверяет source-компоненты с синтетическими hook-адаптерами: 36
сочетаний темы, scene и ширины; Tab, стрелки, focus, обновление и reset.
Это не приёмка установленного приложения, API или Auth.
Backend build, hosted CI, общий интеграционный gate и установка здесь
не выполнялись; source adoption не меняет установленный runtime.

## Markdown и защита ухода — 2026-10-27

Markdown preview сохраняет стабильный renderer CodeBlock Base. Проверены
сохранение узла/focus при изменении заголовка, безопасность Markdown, ввод
и namespace-сценарии: 24 адресных теста, typecheck, lint и shell guard.

Используется уже принятый UI producer
`a95d9fe7b31cca23e1f6432bae43246120859efb`; UI/SDK/skills pins и зависимости
не менялись. Production bytes UI совпадают с ранее проверенным архивом,
повторные install/pack/generation не выполнялись.

IAB проверяет source-компоненты на синтетических данных. Это не установленное
приложение, API/Auth или runtime-приёмка. Снимки 375/1920/2560 сохранены
отдельно от основной матрицы трёх тем и ширин 320/375/1920/2560.
Полный интеграционный gate, hosted CI, push и установка не выполнялись.

## Прямое подключение темы — 2026-10-27

Неиспользуемый private re-export `shared/lib/theme.tsx` удалён. В private
frontend-пакете нет его публичного export; исходники и тесты не импортируют
этот путь. `main.tsx` уже подключает ThemeProvider непосредственно из
публичного `@pdlc/ui/lib`. Реализация темы, storage и compatibility aliases
самого Base не меняются. UI/SDK/skills pins и lockfile прежние.

После удаления выполнена статическая проверка TypeScript без повторных
генерации, install или build. Проверка приложения в браузере не повторяется:
удалённый модуль не входит в его runtime import graph. Полный интеграционный
gate и установленная приёмка остаются отдельными этапами.

## Общая подпись навигации — 2026-10-27

Принят совместимый UI 0.3.4, producer `efa4babe176b42530a3e17fc25f10e8678f82d33`.
Подписи боковых ссылок используют SidebarItemLabel Base; native span,
полное доступное имя, title, маршруты, состояние и callbacks сохранены.
Ellipsis и compact visibility принадлежат Base; private CSS-класс не копируется.
Проверены настоящий архив, offline CLI и frozen adoption: шесть React consumers
с одним React, седьмой SSR consumer — через штатный export/check. SDK/skills
pins неизменны; это source-кандидат, без установки и hosted CI.

Итоговый producer проверен 38 scoped tests боковой навигации;
TypeScript, lint изменённых модулей и shell contract gate прошли отдельно.

## Нативные атрибуты SSR Button — 2026-10-27

Текущий UI 0.3.5: `8bdd8ecd54eb6a6f553bfbadcfaf07c99f4a801d`. Сохраняются React API и независимые SDK/skills pins.
React/JS/CSS и типы API не менялись; frontend/backend не пересобирались.
Новые SSR параметры доступны в этом точном Base, не являются новым React API.
Архив UI и offline frozen adoption проверены; общий интеграционный gate,
hosted CI, публикация и установка здесь не выполнялись.

## Общие нативные элементы SSR — 2026-10-27

Принят совместимый UI 0.3.6, producer `623deb7da5813fa9d1185ded7688c35bb15cd43e`.
Изменения принадлежат SSR macros и справочнику; React API, JS/CSS/types и
зависимости совпадают с 0.3.5. Повторная frontend/backend-сборка не нужна при
неизменных inputs. Настоящий архив из 868 файлов сверён с установленным UI,
offline frozen install, singleton React и CLI check пройдены. SDK/skills pins
неизменны. Это source adoption, без hosted CI и установленного runtime.

## SSR композиции и native reset — 2026-10-27

Текущий UI 0.3.8, producer `cf88f7094044a403e6bb7105cfa88b12c4208c96`.
Общие SSR parts принимают native attributes. В отдельном select-runtime
исправлено обновление trigger после нативного reset; API React и CSS/types
не менялись. Root UI/lib не импортируют этот runtime, прямых импортов
select-runtime в React runtime source продукта нет. Настоящий архив из
868 файлов сверён, offline frozen install, singleton React и CLI пройдены.
SDK/skills pins неизменны. Это source adoption, без установки и hosted CI.

## Рабочая область Base — 2026-10-27

Текущий UI 0.3.9, producer `bab93d2683367117784c2f01fcbed7f80fd3ef15`.
Workbench и семь частей добавляют общий каркас рабочей области внутри PageFrame:
высота по настоящим gutter, внутренний scroll, адаптивные Aside/AsideAction и
mobileFlow для длинной формы. SSR-аналога этого React-контракта нет.
Настоящий архив из 882 файлов сверён с установленным UI; offline frozen install,
singleton React и CLI check пройдены. SDK/skills pins сохранены. Это source
adoption; приложение, backend и Storybook не пересобирались, runtime не менялся.

## Поверхности dossier и сведения о пространстве — 2026-10-27

Формы привязки документа к задаче/фазе используют Card asChild с surface=raised
и CardContent density=compact. Native form, onSubmit, aria-label, поля, pending,
ошибки, callbacks и сообщения об успехе остаются у Wiki. Поверхность SpaceDetails
получает те же общие tokens; её grid/порядок содержимого и права сохраняются.
Подпись дерева использует SectionHeader level=3; названия, ключи, описания и
метаданные строк — ResourceListTitle/Description. Разделитель « · » между
названием документа и типом присутствует в тексте ссылки и доступном имени.

Scoped TypeScript, ESLint/format, UI guard и AST-проверка сохранности модуля,
45 callbacks, 88 bindings и 125 выражений показа пройдены. 34 unit cases трёх
страниц подтверждены; после изменения текста ссылок повторены 11 cases spaces.
Две проверки доступного имени сохраняют оба текста и прежние URL разных документов.

В IAB импортированы настоящие модули страниц и канонический Wiki index.css:
36 режимов трёх страниц × light/dark/gray × 320/375/1920/2560, без горизонтального
переполнения. Native Enter даёт один вызов, ошибка сохраняет ввод, pending
блокирует повтор; readonly не показывает управление. Меню в Card сохраняет
тему Portal и возврат focus по Escape. Четыре снимка просмотрены. Данные и
mutation adapters синтетические, только в памяти; fake login, backend, SMTP,
credentials и установленный runtime не используются. Финальный console чистый.
Снимки показывают 600px viewport source-фикстуры; full-page и установленная
приёмка ими не закрываются. Свои вкладка и сервер остановлены.

Новый Base API и обновление dependencies/pins не требовались: UI 0.3.9,
producer bab93d2683367117784c2f01fcbed7f80fd3ef15. SDK/skills pins сохранены.
Общий семантический аудит и интеграционный gate остаются открытыми.

## Общие секции и полосы данных — 2026-10-27

UI 0.3.10, точный producer `94f2ccd9e6f09eba9fbf67f4582a9fa7a91c44b9`.
Области каталогов, настроек, поиска и dossier используют ContentSection;
права, namespace-привязка, поиск и сохранение документа остаются у Wiki.

Фирменные разделители, inset и плотность принадлежат Base. Данные, DTO,
права, формы, смысловые статусы и внешний grid/gap остаются у продукта.
В справочнике доступны самостоятельные ContentSection/Form и DataBar/basic;
для статических данных применяется DataBar, для выполнения операции — Progress.

Архив 895 файлов и установленный UI сверены; offline frozen refresh, React
singleton, CLI check, TypeScript, UI contract guard и scoped tests пройдены.
Изменённые TSX также прошли scoped ESLint.
Браузерные доказательства относятся к изолированным общим композициям,
не ко всем страницам приложения/API/Auth. Полный интеграционный gate и
установленная приёмка остаются отдельными. SDK/skills pins и runtime сохранены.

## Общие метки и inline-код UI 0.3.11 — 2026-10-27

Точный UI producer `755c9f9a27ba81ddd841b43c511bb372a3c93e40` принят отдельно от SDK/skills.
Три метки документа используют Badge. Зелёная отметка теперь относится только к published; draft/archived имеют muted tone. Это исправляет прежнюю безусловную success-иконку, не меняя записи и API. Все 27 document tests прошли, включая три статуса.

Настоящий архив UI содержит 900 файлов; все production bytes сверены с
producer и шестью установленными frontend-пакетами. Offline frozen refresh,
React singleton и CLI check пройдены; runtime и зависимости backend не менялись.

IAB evidence относится к пяти изолированным композициям из настоящих owner/Base
исходников: 60 сочетаний темы/ширины, 18 branding-пресетов, три статуса Wiki,
readonly click и Enter/Space. Это не приёмка всех экранов, Auth или API продукта.
Полный интеграционный gate остаётся отдельной открытой стадией.

## Общая render-граница UI 0.3.12 — 2026-10-27

Точный UI producer `20633a6489dfd33e7afa782e0a06c23faea26b7a` принят отдельно от SDK/skills.
Предметные React-композиции не менялись; принят совместимый общий UI.

Настоящий UI-архив: 904 файла, offline CLI/reference и production bytes
проверены вне workspace без Storybook. Шесть frontend-пакетов сверены с архивом;
cached offline frozen refresh, React singleton и CLI check прошли.
Общий механизм и ограничения: Base docs/platform/DESIGN_SYSTEM.md,
раздел «Граница render-ошибки React». Данные, fallback, отчёт и reset identity
принадлежат владельцу. HTTP/async/event/SSR ошибки обслуживаются отдельно.

IAB evidence относится к фактическим Base/CatalogProviders source-композициям:
12 сценариев ввода/сбоя/reset и 12 длинных fallback в трёх темах и четырёх
ширинах; сохранены preview ready/error и черновик. Это не приёмка установленного
продукта или Auth/API. Интеграционный gate, hosted CI и публикация остаются
отдельными стадиями. Runtime не менялся.

## Общая responsive-подписка UI 0.3.13 — 2026-10-27

Точный UI producer `83c961379b25379e8c83ebc207eec60cfb4ebbef` принят независимо от SDK/skills.
Продуктовая компоновка не менялась. Общий AppShell/drawer получают один Base observer; шесть frontend прошли types и UI contract guard. Проверено 23 адресных тестов продуктовой оболочки.

Новый hook доступен через `@pdlc/ui/lib`; query задаёт владелец. Необязательный
callback выполняется до React layout update только на browser event, сохраняя
перенос focus из исчезающего control. Mount/query changes не запускают действие.
SSR/отсутствующий matchMedia дают false. Контракт, cleanup и standalone example
доступны через `pdlc-ui show useMediaQuery` и `pdlc-ui example useMediaQuery`.

Архив 907 файлов, offline CLI, шесть frozen refresh, React singleton и сравнение
установленных production bytes прошли. Lockfiles после подготовки не менялись;
backend, SDK/skills pins и установленный runtime не изменялись.
IAB source proof: 27 Tracker viewport transitions, 30 shell/drawer checks и
12 recovery cases в трёх темах. Ширина менялась без reload; ввод, focus и journal
сохранены. Это source-композиции, не приёмка полного authenticated приложения.
Прежний IAB attachment gap закрыт новым успешным run; ошибки сохранены в истории.
Полный интеграционный gate, Rust transport vectors, hosted CI и публикация отдельно.

## Общая геометрия истории UI 0.3.14 — 2026-10-27

UI producer `f3b99e3ce15a4a447d58790f118c6be70e931a18` принят независимо от SDK/skills.
Архив содержит 910 файлов; offline CLI, шесть frozen refresh и production bytes
проверены. В Workflow штатный SSR export остался byte-identical. Это обновление
source-поставки; установленный runtime не менялся.

Продуктовые представления не менялись. Новый hook доступен для подтверждённых
историй с prepend/live append через публичный @pdlc/ui/lib. Перед использованием
прочитать карточку и самостоятельный пример; готовый WorkbenchScroll/CodeBlock
не заменяется локальной визуальной обёрткой. Обычный scrollIntoView остаётся
нативным действием владельца.

Base qualification: 17 hook tests, 69 primitive checks и 37 search assertions.
IAB Base/Forge source-compositions: 24 theme/width cases плюс три intent/error
cases; это не полная authenticated-приёмка продуктов. Полный интеграционный gate,
Rust search transport, hosted CI и публикация остаются отдельными стадиями.
