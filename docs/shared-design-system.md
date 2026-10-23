# Подключение общей дизайн-системы

Продукт использует @pdlc/ui 0.3 и опубликованный документ Admin. Общие controls,
цвета, шрифты, плотность, скругления и popup принадлежат Base; запросы,
права, данные и callbacks сохраняются здесь. UI/SDK/skills pins независимы.

Канонический контракт: services-base/docs/platform/DESIGN_SYSTEM.md.
Компоненты искать через pdlc-ui search → show → example. Не создавайте
копии примитивов, собственный Toaster и глобальные определения Base tokens.
Существующие source-компоненты и preview получают общий UI; исторические
прототипы не являются установленным приложением. Установка выполняется отдельно.

Источник UI 0.3.0: `579749e19e9088be200c62e5563a3cb2db4d8c9f` в `.base-ui-revision`.
SDK и skills pins независимы; установка этим source-кандидатом не выполняется.

Vite plugins включает `createUiFontAssetsPlugin()` из `@pdlc/ui/build`: notices
встроенных fonts попадают в dist через общую реализацию Base.

Общий feedback: LoadingState/ErrorState/EmptyState для ресурсов и Notice для
inline результатов. Ошибка и неизвестный исход не являются подтверждённой пустотой
или успехом. Запросы, тексты и callbacks остаются у Wiki; Notice не выполняет
retry автоматически. Действия передаются в actions, локальные рамки/иконки
сообщений не копируются. Продуктовые счётчики в live-region остаются данными.
Source-перенос 2026-10-22 проверен отдельно от закреплённого UI pin выше;
итоговое обновление зависимости выполняется после квалификации общего кандидата.

## Секции и рамки форм — 2026-10-22

15 интерфейсных h2 в девяти файлах используют SectionHeader Base. Сохранены
aria-labelledby и dynamic IDs; counters поиска остаются в live-region, ссылки
dashboard передаются в actions. Формы связи ревизии и проектов пространства
используют Card.asChild и CardContent: семантические form/section и обработчики
сохраняются, отступы и рамка принадлежат Base. Заголовок Markdown preview остаётся
документным содержимым. Остальные page headers проверяются следующим этапом.

Первичный scoped unit — 81/82. Устаревшая проверка accessible name Metric
воспроизведена на сохранённом исходнике до переноса; тест теперь проверяет ошибку
в именованном показателе, отсутствие ложного нуля, сохранность документов и
изолированный retry. Последние результаты покрывают 82 cases; полный повтор
не выполнялся. TypeScript, scoped ESLint, choice и shell guards прошли.
Новая браузерная приёмка этих страниц остаётся в итоговом UI gate. Полный route
gate открыт: актуальный owner docs/ui-routes.json пока отсутствует.

## Общие шапки страниц и фокус — 2026-10-22

Все 19 page headings в 17 файлах используют PageHeader Base. Продукт передаёт
название, контекст, навигацию и действия; запросы, права, namespace, URL и формы
сохранены. Единственный raw h2 остаётся заголовком документа внутри Markdown
preview. Бейджи предметных статусов и прочие композиции проверяются отдельно.

PageHeader получил совместимые headingRef/headingTabIndex. DocumentPage сохраняет
свой documentHeadingRef как target h1 и fallbackFocusRef ConfirmDialog. После
архивации инициатор исчезает, и focus возвращается к заголовку; cancel/Escape
возвращают его инициатору. Публикация и readback остаются правилами Wiki.

Scoped unit: 179/184; пять прежних ошибок поиска пользователей воспроизведены на
pre-header source. Тесты теперь используют searchbox и предметную подсказку вместо
старого textbox/h-10, сохраняя фильтрацию, URL и пагинацию. Repair — 8 PASS;
последние результаты всех 184 cases прошли. TypeScript, scoped ESLint, choice и
shell guards прошли. В Base шесть RED cases подтвердили отсутствие ref, затем
17 проверок PageHeader/page patterns прошли. Examples compile, reference freshness,
offline check и ru/en search vectors проверены отдельно.

IAB: настоящий новый Canvas в 12 сочетаниях трёх тем и четырёх ширин, явный focus,
tabIndex=-1, отсутствие autofocus и overflow. Справочник и preview используют одну
версию, reset проверен. Dispatch клавиатуры внутри вложенного iframe недоступен
в IAB; та же клавиатура проверена в прямом Canvas через IAB. Cold-start preview
потребовал явного retry после оптимизации; timeout и fallback не менялись.
Браузерная приёмка реальных страниц Wiki и итоговый package/pins gate остаются
открытыми. Installed runtime не обновлялся.
## Общие карточки-ссылки — 2026-10-22

Шесть карточных композиций Admin, документа и редактора используют
Card.asChild interactive и CardContent. Logo-link в шапке использует Button
ghost/icon, сохраняя PlatformMark, доступное имя и breakpoint видимости.
Рамки, padding, радиусы, hover/focus и размеры icon-control принадлежат Base.
Задачи, фазы, evidence URLs, namespace, права и callbacks не менялись.

Последние результаты 73 scoped unit cases прошли; старое ожидание min-h-20
заменено проверкой общего interactive Card без изменения API/URL assertions.
TypeScript, scoped lint, choice и shell guards прошли. Common linked Card
проверен отдельно через IAB; новый продуктовый browser gate остаётся открытым.

## Панели ревизий и пространств — 2026-10-22

Ещё пять рамок используют Card/CardContent Base: содержание выбранной ревизии,
Markdown preview, формы создания/изменения пространства и дерево пространства.
Нативные формы остаются через asChild; onSubmit, required, id, dirty tracking,
namespace binding и права не менялись. Markdown h2 — название документа,
а не интерфейсный заголовок секции. Min-height редакторского preview сохраняется.

60 scoped unit cases, TypeScript, scoped lint, choice и shell ownership прошли.
AST-сверка подтверждает non-render logic и поведенческие атрибуты.

## Режим документа, метка и общий вход — 2026-10-22

Read/edit использует RadioGroup appearance=segmented Base; черновик, доступ
и обработчик перехода остаются у документа. Информационная метка в compose
использует Badge tone=info вместо собственной рамки. Неактивные RegisterPage
и её тест формы удалены. Действующий /register → /login проверен при обоих
значениях namespace feature flag: replace не оставляет регистрацию в истории,
Back возвращает на предыдущую страницу, настоящий SSO не запускается.
Backend registration API не менялся.

Для режима прошли 24 scoped cases; для compose/namespace и redirect — 27.
TypeScript, scoped lint, choice и shell guards прошли. Source-проверка использует
существующую private qualification config; настоящий архив/peer graph, полный
route inventory и новый продуктовый browser gate остаются открытыми.

## Опасное действие документа — 2026-10-22

Архивирование использует Button destructive-outline вместо локальной красной
рамки. Readonly, pending, confirmation и archive callbacks не менялись.
28 scoped cases документа и router прошли; TypeScript, scoped lint и primitive
gate прошли. Общий hover исправлен в Base и проверен отдельно в IAB.
Продуктовая browser-приёмка и итоговая поставка остаются отдельными gates.

## Общие ссылки-действия — 2026-10-22

Три ссылки dashboard: пространства, документы и задачи используют Button asChild variant=link.
Цвет, hover, focus и область нажатия задаёт Base; ссылки остаются одним native
anchor, адреса и namespace/project scope сохраняются у продукта.

Scoped unit, TypeScript, lint и primitive guard прошли; AST-сверка подтверждает
прежние данные, маршруты, условия и события без подмены результата запроса.
Реальная продуктовая browser-приёмка и точный UI package/pin остаются открытыми.
Ссылки внутри текста не превращаются в полноразмерные кнопки; их общий контракт
проверяется отдельно.

## Секции диалога опубликованной ревизии — 2026-10-22

Диалог ревизии использует DialogContent layout=sectioned и DialogBody.
Удалены конфликтующие p-0, вложенные p-4/sm:p-6 и pr-14; разделитель и отступы
принадлежат Base. Запрос ревизии, состояния ошибки/загрузки, данные и возврат
фокуса сохранены по AST. Все 24 scoped document cases прошли.

TypeScript, scoped lint и primitive guard прошли. Общие компоненты проверены
в IAB на собственных синтетических сценах; фактическая продуктовая browser-
приёмка и поставка точного UI archive/pin остаются частью итогового gate.


## Общая граница query-сессии — 2026-10-23

Base предоставляет PlatformQuerySession и platformQueryDefaults через
`@pdlc/ui/platform`. Wiki/Fleet передают стабильный адаптер authVersion/subscribe,
фабрику своего QueryClient и QueryClientProvider с прежним SpaceAdmissionBoundary.
Store, права, expiry, SSO и запросы остаются у продукта. Смена подтверждённой
версии очищает старый cache и перемонтирует формы; pending sign-out не повышает
версию сам по себе. Tracker использует только общий retry/backoff: его lifecycle
SpaceSessionBoundary сохраняется отдельно и этим переносом не меняется.

Настройки чтения сведены без изменения исполняемой политики: transport/429/5xx,
не более четырёх повторов, backoff 1/2/4/8/10 секунд; mutations retry=false.
Base не получает зависимости от React Query. Адресная AST/JS-сверка трёх прежних
политик и пяти публичных platform-функций прошла; 9 scoped Base tests, compilation
самостоятельных examples и TypeScript/Lint трёх потребителей PASS.

Consumer auth-boundary unit gate пока не подтверждён: требуется изолированное
frozen окружение вместо прежнего смешанного React graph. IAB проверяет только
синтетическую композицию Base без token/SSO/backend: Enter даёт одну редакцию,
старый черновик удаляется, внешний trigger сохраняет focus. 320/1920 без overflow,
действие 44/40 px. Это source-кандидат, не обновление UI pin и не runtime-приёмка.
