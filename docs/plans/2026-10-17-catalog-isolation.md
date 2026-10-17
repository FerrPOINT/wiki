# Обновление общего каталога Wiki

Принять Base UI `e0759e070326c905f365182d0107f8c87a2e67f6` (Base #245) через независимый
`.base-ui-revision`, сохранив SDK/skills pins, backend, HTTP-контракты и данные.
Обновить только дерево `@pdlc/ui` в frontend lockfile; прямые зависимости
продукта и overrides сохраняются.

При недоступном каталоге общий provider не выдаёт destinations продуктовым
ссылкам. Меню сохраняет подписи без переходов; runtime-каталог и настроенный
host сохраняют обычные ссылки, health и namespace transport. Продуктовые
запросы, права и формы остаются здесь.

Проверить frozen install/build, профильные frontend tests/typecheck/lint,
OpenAPI/package/effective-theme и применимые форматные проверки. Подтвердить
неизменность backend/source pins и связать повторно используемые backend gates
с точным исходником. Установку собственного web image проводить отдельно,
после квалификации, с живыми SSO/reload/logout и снимками UI на 375/1920/2560.

Открытая разработка SDLC и Fleet не включается в этот кандидат. Пользовательские
изменения canonical checkout и существующий initialized runtime сохраняются.
Фактические результаты будут дополнены после проверок.

## Source gate — 17 октября

Frozen builder и nginx image, 247 frontend tests, lint, typecheck,
Prettier, OpenAPI drift, package/reference и effective theme в трёх темах прошли.
Прямые dependencies/overrides, SDK и backend сохраняются. Полный unit-прогон
соответствует CI и установленному профилю `Namespace=false`; отдельные
Namespace-тесты явно включают feature внутри своих сценариев.

Дополнительный глобальный прогон всех legacy fixtures с принудительным
`Namespace=true` не квалифицирован: обнаружены отсутствующий QueryClient и
ожидания legacy навигации. Этот кандидат не включает Namespace в runtime и
не объявляет полную приёмку этого режима. Исполняемая функциональность и
live SSO проверяются отдельно после адресного обновления web.

Проверка каталога пользователей теперь использует настоящий Base provider:
без каталога ссылки Admin нет, а с runtime-каталогом сохранён точный origin
и прежняя геометрия кнопки. Зашитый `7772` не считается проверенным адресом.
