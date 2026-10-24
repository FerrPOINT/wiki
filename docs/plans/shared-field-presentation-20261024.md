# Общие подписи полей Base

## Текущая волна: фирменные подсказки — 25 октября 2026

UI pin `56cf8a2f63b2b853103c164bbf1f7758a90e8db0`, версия 0.3.2; reference
`3d88aadb70081f614a1ef51481c32724c53bcd7c6135045d96bc7be2a1dfd1d4`.

Тип документа использует AutocompleteInput вместо datalist. Сохранены свободный ввод,
namespace-authority и черновик. Прошли 23 scoped cases.

Новый контрол и его popup принадлежат Base. IAB проверяет общую реализацию,
а не authenticated страницы продукта; прежние результаты сохраняют свой scope.
Scoped source/static прошёл: шесть strict TypeScript/CLI/choice consumers,
Base types, examples, reference/coverage, настоящий offline UI-архив, SSR/Auth.
Данные Radix подтверждают один layer stack у шести потребителей. Отдельно
прошли 26 Base, 104 product cases и 8 Forge preview cases. Полный frontend
gate и hosted CI этой волны ещё открыты; прежний полный PASS не подменяет их.
SDK/skills pins, установленный runtime и пользовательские данные не менялись.
Девять owner AGENTS уточняют выбор AutocompleteInput/Select/SearchableSelect.
Общий остаток ведётся в Base `docs/plans/shared-ui-consolidation-20261021.md`.

## Изменение

Фильтры аудита, обзор и выбор пространства используют публичный Label вместо локального
оформления семи полей в трёх файлах. Всего перенесено 7 подписей. Текст, htmlFor/id,
required, значение, обработчики, запросы и права сохраняет продукт.
Внешняя компоновка не переносится в библиотеку.

Первый срез использовал UI producer `8efc6c8d62b7d2c152b007215d6ce91f4c8f8d17` (0.3.1).
Label существовал ранее; нового API или зависимости для этих подписей нет.
SDK/skills pins сохранены; cached offline frozen install уже подготовленного
UI-пакета прошёл без обновления внешних версий.

## Проверка

Прошли 25 scoped test cases затронутых форм/экранов, scoped ESLint и
существующий primitive/choice contract. AST-сопоставление до/после не обнаружило
изменений вне импорта, тега подписи и её оформления.

Общий маршрут проверен в IAB на настоящей форме спринта Tracker: три темы,
standard/large и 320/375/1920/2560. Подписи имеют 14/16 px из BrandingScope
внутри Dialog Portal; htmlFor сохраняет focus и исходный callback получает
введённые данные. Это 24 source-composition проверки с синтетическим onSubmit,
не приёмка backend этого продукта. Полная новая сборка и hosted CI ещё открыты.

## Обновление общей source-политики

Текущий UI pin — `948b492b36ff2c95215d43cf945f15a849d593f3`. Он добавляет
общую проверку типографики подписей и оформления полей. UI-payload совпадает
с ранее проверенным архивом по всем 772 файлам; binary bytes сравнивались точно.
Повторного install не было. Installed CLI check и React singleton подтвердили
сохранённый пакет. Предыдущие 25 cases не обозначаются новым прогоном.
SDK/skills pins и внешние версии сохранены; новый frontend build остаётся gate.

## Составные поля и сообщения операций

Три FormField Base заменяют разметку имени, типа и Markdown шаблона. Сохранены
id, refs, native required, noValidate и продуктовые trim/focus/submit правила.
Base задаёт связь ошибки с контролом и оформление Notice. Ошибки создания,
обновления пространства и скачивания evidence также используют inline Notice
с role=alert. Запросы, права и форматирование безопасной API-ошибки остаются в Wiki.

38 уникальных cases прошли: evidence 15, spaces 11, templates 12. Четыре первых
отказа templates были связаны с буквальным label-текстом после появления общей
required-звёздочки: тесты используют точное доступное имя textbox и дополнительно
проверяют обе accessible descriptions и два alert. Требования trim, focus,
сохранности черновика и permissions не ослаблялись. AST, scoped lint, format и
primitive/choice guard прошли; другие файлы после исправления не перезапускались.

IAB настоящей TemplatesPage с синтетическими чтениями подтвердил 12 сочетаний
трёх тем и 320/375/1920/2560, 40/44 px без overflow. Обе ошибки связаны с полями;
после исправления имени focus переходит к Markdown, имя сохраняется. Дополнительный
large-показ даёт Label 16 px. Внешних API и записей не было; собственные сервер
и вкладка закрыты. Это не backend/installed acceptance и не browser zoom 200%.
UI pin и зависимости не менялись; общий build/reference gate остаётся открыт.

## Совместимое обновление локальных состояний Base

UI pin — 2e29b403b3987905494ed40fe812608cf998cc3a; reference
`e5cad6ebab4369dbd56831f566ab94141ca4de050b253f9b93191ddbcb75c23a`.
LoadingState/ErrorState и их общие композиции сохраняют page-scope по умолчанию;
local-scope доступен явно для отдельного блока. В этом владельце вызовы не менялись.
Cached frozen offline refresh обновил UI без downloads; остальные версии графа,
React singleton и SDK/skills pins сохранены. Installed CLI check прошёл.
AI Hub обновляет только URI immutable snapshot в package/lockfile; остальные
lockfiles не меняются. Прежние product tests не обозначаются новым прогоном.
Новый общий архив и standalone CLI проверены отдельно; полный frontend gate открыт.

## Общие текстовые ссылки

14 локальных оформлений в семи файлах используют TextLink Base. NamespaceLink
сохраняет URL-контекст; внешние href сохраняют target/rel. Убраны повторные цвет,
hover и focus; ширина, перенос, минимальная высота и предметные подписи сохранены.
Связь шаблона с созданием документа остаётся обычным переходом. Составные поля,
разворачивание и права этой миграцией не меняются.

51 уникальный scoped case пяти файлов прошёл; TypeScript, ESLint и общий
primitive/choice guard прошли. Проверка типов также нашла прежнюю ошибку нашей
волны 554248e: getByRole не принимает exact. Неподдерживаемая опция убрана
отдельным test-коммитом; строковое accessible name по-прежнему сравнивается точно,
assertions описаний/ошибок и продуктовые требования не ослаблены.

IAB настоящих DocumentRevisionPage Wiki и WorkspaceLinks Forge с синтетическими
read-only данными проверил 24 сочетания: три темы, две палитры и четыре ширины.
Цвет меняется у обеих композиций одинаково; подтверждены underline, отсутствие
вложенных anchors/overflow, видимый keyboard focus и Enter с сохранением context.
Это source proof двух композиций, не визуальная приёмка каждого маршрута.
AST 25 файлов обоих владельцев сохраняет непредставительные contracts; при map
key переносится на внешнюю композицию. Сервер и вкладка закрыты.

UI producer/reference, lockfile и независимые pins не меняются. Дополнительный
guard пока проверен из текущего source Base; установленный UI pin не считается
adoption непоставляемого source checker. Новые frontend bundles, полный route/
reference gate и установленная платформа остаются открытыми.

## Общая композиция простых полей

В этом срезе 30 повторных композиций подписи и одного контрола заменены
существующим FormField Base. Внешняя предметная компоновка и сложные группы
с несколькими контролами сохранены. Тексты и обязательность не меняются;
id/label association, все props данных и callbacks проверены AST для 25 файлов
пяти владельцев: 131 control и 111 подписей, включая нетронутых соседей.
Новые запросы, UI API и зависимости не добавлены. Проверенный UI producer —
2e29b403b3987905494ed40fe812608cf998cc3a; независимые SDK/skills pins сохранены.

Прошли 120 scoped cases этого владельца; вместе пять продуктов дали
320 уникальных cases. TypeScript и scoped ESLint пяти владельцев прошли.
IAB пяти реальных композиций с синтетическими read-only адаптерами подтвердил
120 сочетаний трёх тем, standard/large и 320/375/1920/2560. Подписи связаны
с контролами, высоты 40/44 px и отсутствие overflow сохранены. Дополнительно
проверены label click, выбор/focus и native required Tracker без записи.
Это source proof пяти композиций, не приёмка каждого маршрута и не zoom 200%.

Первая private-фикстура Wiki пропустила data.space.status: исправлен только
адаптер и повторены её сценарии. Этот fixture FAIL не объявляется продуктовым
дефектом. Собственные сервер и вкладка закрыты, неизменность исходников
браузерного среза подтверждена hash. Общий build/route gate, публикация source
и установленная платформа остаются открытыми. Независимое исправление SSR
брендинга Base квалифицируется отдельно от этого среза React-форм.

## Сведённый producer и локальные состояния

SearchResultLink использует LoadingState scope="local" и Notice вместо
локальной разметки сообщений. Query keys, transport и URL не меняются. Пять
scoped cases, TypeScript и scoped ESLint прошли. IAB настоящей композиции
проверил pending/503/403: подтверждённый результат сохраняется при временной
ошибке и скрывается после потери доступа; три темы и четыре ширины без overflow.

UI producer — d6c0914ae7bab37fc91b5f10602ad05543551ec0, reference
`5f4ff5f4ada180236c2d6e014b52e78ad60586824980c7b45316eb20b44a69a8`.
Frozen offline refresh прошёл без downloads и изменения lockfile; React singleton
и независимые SDK/skills pins сохранены. Настоящий архив Base проверен отдельно.
IAB использовал собственные исходники и read-only adapters; новый полный build,
hosted CI, zoom 200%, публикация и installed acceptance этим срезом не заявляются.

## Компактные сообщения и актуальная UI-поставка

Предупреждение о несохранённом черновике использует Notice tone=warning
appearance=inline. Dirty tracking, выбор режима и запросы сохранения не меняются.
24 scoped cases, TypeScript и scoped ESLint прошли. IAB настоящего DocumentPage
проверил три темы и четыре ширины с вводом названия; предупреждение видно,
ввод сохранён, записи не выполнялись.

UI pin — eb3ceceeb1b1c5029ccaaa9b863d6646b35b1a46; reference
`5a702cb74a6eff5ec0e298ebdee609e70703e4fb01eda9e101cf7be84f019d36`.
Cached frozen refresh обновил UI без downloads и сторонних dependency upgrades;
React singleton и независимые SDK/skills pins сохранены. Настоящий UI-архив
проверен отдельно. Общий integrated gate, source publication и hosted evidence
остаются открытыми; установленный runtime этим изменением не обновляется.


## Списки и пагинация — 24 октября 2026

Семь ручных пагинаторов заменены PageNavigation: страницы, курсоры и версии документа. Права, сброс выбранной версии, курсорная история и URL остаются у владельца. Устранено двойное объявление номера ревизии; счётчик пользователей сохранён отдельно. Проверены 93 scoped cases, TypeScript, lint и IAB каталога пользователей.

UI producer `ff303ac5a273dc6b265292465b8b9db9a9476f9f`, fingerprint `206bf4889e50889e8c7bfab9aec36ef52df2e1e76dfcaa94a3ac756a7b80e2bf`.
Проверки относятся к исходникам собственных checkout; установка, hosted CI и общий
интеграционный gate сохраняют отдельный статус. SDK/skills pins не изменены.


## Сообщения, статусы и семантическая пустота — 24 октября 2026

Сообщения успешной привязки dossier используют Notice; статус пространства — inline Badge. Данные и условия отображения сохранены. Проверены 34 scoped cases, TypeScript и scoped ESLint.

UI producer `56f27ee55bb0481ecc4733a0bb58144ae3cfed60`, reference `ba456575dd41f6237911ee97d0cec9de7c5f39941965714d3e4d86d09b0d8a2d`. Семь UI pins согласованы; шесть frozen offline refresh без downloads сохранили React singleton. SDK/skills pins не менялись.

Доказательства сохранены в private evidence/shared-message-audit-20261024/. Общий интеграционный gate, hosted CI, source publication и установленная приёмка остаются отдельными открытыми этапами.


## Общие заголовки и видимый фокус — 25 октября 2026

Production React source этой волной не менялся. Обновлён точный UI pin общего оформления focus-visible PageHeader; AI Hub дополнительно обновляет только URI immutable UI snapshot в package/lockfile. Прикладные unit/type результаты неизменных inputs переиспользуются, новый продуктовый browser PASS не заявляется.

UI producer `88e4ed54546181c6ffa46a38651b6553cd4e3aff`, reference `9d8689e5c5e52fd6bc3dd3d2b4b030bc992af26cfa22d626c15268765bb06316`. Семь UI pins согласованы, шесть frozen offline refresh без downloads и сторонних upgrades сохранили React singleton. SDK/skills pins неизменны. Настоящий UI-архив и offline CLI проверены вне workspace. Общий интеграционный gate, hosted CI, публикация веток и установленная приёмка остаются открытыми.


## Общие пары описания и показатели — 25 октября 2026

Мобильный аудит и его технические сведения используют DescriptionItem; read-only настройки — тот же компонент с сохранением индикаторов enabled и исходных значений. Форматирование, opaque IDs, фильтры, cursors и API остаются у Wiki.

Прошли 18 scoped cases, TypeScript и scoped ESLint; общий UI guard сохранён. IAB использовал настоящие owner-компоненты с синтетическими read-only данными: общий срез четырёх владельцев — 240 сочетаний десяти композиций, трёх тем, standard/large и 320/375/1920/2560. Это не приёмка всех продуктовых маршрутов или backend.

UI остаётся 88e4ed54546181c6ffa46a38651b6553cd4e3aff / reference 9d8689e5c5e52fd6bc3dd3d2b4b030bc992af26cfa22d626c15268765bb06316. API Base, семь UI pins, SDK/skills pins и зависимости не менялись; install/build/export не повторялись. Собственный QA завершён без обращений к продуктовым API. Доказательства: private evidence/definition-pairs-20261025/. Общий integrated gate, публикация и installed acceptance остаются открытыми.

После исправления лишней вложенности трёх групп Forge их 24 UI-сценария квалифицированы отдельно; 216 сценариев остальных композиций переиспользованы по неизменным соответствующим inputs. Дополнительно проверена валидная структура dt/dd во всех десяти композициях. Новый семантический RED-тест прошёл после исправления; он не заменяет проверку значений. Финальный QA 55828 завершён, inputs неизменны и API requests отсутствуют.


## Общий возврат фокуса и presentation — 25 октября 2026

Снимок ревизии использует returnFocusRef вместо локального focus handler. EvidenceItemTitle использует Button variant=link вместо локальной окраски. Строка шаблона и раскрытый CodeBlock получают оформление Base; имя, body, URL, обработчики и управление раскрытием сохранены. Тест снимка проверяет фокус исходной ревизии после закрытия. Scoped TypeScript, ESLint и 51 product cases прошли.

UI producer `ed8edd288f3980b521258bcac9082bfb88856b69`, reference `1424252c8cf384eb42385d5692daa148d0df30796f2a79cf100dd87e56e54743`. Семь UI pins согласованы; шесть frozen
offline refresh без downloads и сторонних upgrades сохранили React singleton.
SDK/skills pins неизменны. Настоящий архив UI содержит 780 файлов; offline CLI,
новые карточки и самостоятельные примеры проверены вне workspace без Storybook.

Общий IAB срез: 144 уникальных сочетания шести source-композиций, трёх тем,
двух type_scale и ширин 320/375/1920/2560. Из-за пропущенной утилиты временной
QA-страницы Table 24 сочетания перепроверены; 120 остальных сохранены после
доказанного сравнения CSS. Дополнительно 18 уникальных открытых Portal-сцен,
native Tab/ArrowRight таблицы и URL-навигация Forge. Продуктовые записи отключены
синтетическими adapters; это не backend или установленная приёмка. Снимок
ревизии Wiki и context drawer Fleet имеют unit, но не отдельный IAB proof этой волны.

Доказательства: private evidence/dialog-compositions-20261025/. Временные
процессы и IAB-вкладка закрыты, source inputs сохранены, API-запросов нет.
Общий интеграционный gate, HTTP parity, настоящий zoom 200%, публикация,
hosted CI и установленная приёмка остаются открытыми. Runtime не изменён.

## Общая типографика строк ресурсов — 25 октября 2026

Названия/подписи ресурсов используют Base. Запросы, выбор пространства, document/task routes и DTO сохранены. Scoped cases: 31.

UI producer `1848c08d68f8a60f7dc179751984e8c821647373`, reference `250d3c802e3a22b4095efb70defe6b0baff379fd6b1c1f7bbcf9c10d2e30b983`. SDK/skills pins этой UI-волной не менялись; принятые upstream SDK-переходы сохранены. Семь UI pins согласованы. Frozen offline refresh шести React-потребителей сохранил locks и React singleton. Workflow получил штатный SSR export, Auth export не изменился.

IAB: 168 уникальных сочетаний Base и шести реальных source-композиций, трёх тем, двух type_scale и ширин 320/375/1920/2560 без overflow; семь снимков просмотрены. Изменённый Fleet проверен повторно вместо прежних 24 результатов. Данные синтетические, записи отключены; backend/установленная приёмка не заявляется. Собственные временные процессы и вкладка закрыты.

Доказательства: private evidence/remaining-ui-20261025/. Общий integrated gate, HTTP parity, zoom 200%, source publication, hosted CI и установленная приёмка остаются открытыми. Продолжается классификация других локальных компонентов и SSR/dev-preview.

## Общая offset-навигация — 25 октября 2026

В этой волне изменён только UI pin; runtime sources и backend не менялись. UI producer `97399e00e275c953c28a77d1b178a22a79ca2a93`, reference `48721f30d2841d0c76e086876340f5b5367953df4519118c9d92956b08c9f4cb`. Семь pins согласованы; frozen offline refresh сохранил locks и React singleton. SDK/skills pins неизменны. SSR/Auth assets проверены без byte-diff и повторного экспорта.

Base: 17 scoped cases; IAB Base/Admin — 120 уникальных сочетаний, пять просмотренных снимков, keyboard/URL transitions без записи. Архив и offline CLI проверены вне workspace. Это source proof с синтетическими GET-only данными; интеграционный gate, публикация, hosted CI и установленная приёмка остаются открытыми. Доказательства: private evidence/offset-navigation-20261025/.

## Интегрированная frontend-проверка — 25 октября 2026

Квалифицирован source head `e22a4010d3019b3fde6f957290bb63fa0cff307a`:
279 frontend cases, owner static/lint и production frontend build прошли.
UI producer `97399e00e275c953c28a77d1b178a22a79ca2a93`; dependency inputs,
SDK/skills pins и SSR/Auth exports этой проверкой не менялись.

ESLint исключает только приватный generated cache `.local/**`. Исходный FAIL
сохранён; production rules и проверка AppShell не ослаблены. Реальный ESLint
API подтвердил неизменный набор правил production-файлов.

Общий gate Base/справочника/шести frontend завершён: 29 стадий, 4270 test cases.
Это единый прогон; прежние scoped cases в него повторно не суммируются.
153 поисковых vectors совпали в JS, Rust HTTP API, offline и HTTP CLI.
Свежие built assets справочника проверены по bytes и через IAB.

Доказательства: private `evidence/ui-integrated-20261025/`; общий статус и
оставшиеся переносы — [план Base](https://github.com/FerrPOINT/services-base/blob/refactor/shared-ui-residuals-20261024/docs/plans/shared-ui-consolidation-20261021.md).
Docs-only фиксация результата не пересобирает прежние inputs. Hosted CI,
source publication и установленная приёмка здесь не заявляются. Реальный zoom
200% не применился доступной IAB-командой; CSS/DPR-подмена не использована.
