# Admin overview UX QA

Дата: 2026-09-24

## Дефект

`/admin` скрывал все метрики при ошибке одного источника, а общий retry
повторял все четыре запроса. Число событий аудита показывало
размер первой страницы как полный total. Три одинаковые ссылки
`Открыть` имели интерактивную область около `54 x 19 px`, а верстка была
перегружена вложенными карточками.

## Изменение

- Переходы к профилям, настройкам и аудиту стали полноразмерными
  строками с уникальными названиями и точным описанием границ Wiki.
- Каждая метрика имеет независимые loading, error и retry; сбой одного
  API не скрывает остальные данные.
- Аудит явно показывает, что загружена первая страница с лимитом 50,
  и не выдаёт page size за общее число событий.
- Разделы стали невложенными, компактными и устойчивыми к длинным
  значениям на mobile и desktop.

## Production candidate QA

Проверено с настоящими Central Auth и Wiki API на production bundle:

- 12 success-состояний: `375`, `768`, `1920`, `2560` в light, gray и dark;
- 2 retry-сценария: управляемый `503` настроек в light `375` и dark `1920`;
- 14/14 сценариев прошли, failures: 0;
- root overflow: 0;
- видимые controls меньше 40 px: 0;
- nested scrollers: 0;
- serious/critical Axe: 0;
- page errors и неожиданные HTTP errors: 0;
- write requests: 0;
- во всех audit requests использован `limit=50`;
- максимальная высота страницы: 1122 px, DOM: 235 узлов.

Ожидаемые console-сообщения контролируемого `503` учтены только в
retry-сценариях. Полные измерения находятся в `results.json`.

## Automated gates

- Admin page tests: 2/2;
- frontend suite: 24 файла / 157 тестов;
- TypeScript, ESLint, semantic color lint и production build: green;
- OpenAPI schema и compatibility: green;
- focused Prettier и `git diff --check`: green.

Общий `pnpm format:check` сохраняет baseline-расхождение 79 файлов в `main`;
изменённые source/test файлы проверены отдельно.
