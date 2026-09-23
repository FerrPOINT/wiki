# Audit log URL history QA

Дата проверки: 2026-09-23.

## Область проверки

- `/audit-log` с реальным Central Auth, Wiki API и production frontend bundle;
- фильтры action, entity type, actor и локальный период;
- cursor pagination, direct URL, reload и browser Back/Forward;
- канонизация невалидных query parameters без потери соседних параметров;
- closed/open filter states, responsive, три темы, accessibility и размеры целей;
- runtime, console, network errors и отсутствие write requests.

Для детерминированной проверки трёх cursor-страниц использовался read-only
browser fixture ответа `GET /api/v1/audit-log`. Вход, каталог пользователей и
основной фильтрованный happy path прошли на реальных Central Auth и Wiki API.
Write-запросы к Wiki API не выполнялись.

## Исходное поведение

Фильтры и стек cursor-страниц хранились только в React state. После применения
URL оставался `/audit-log?keep=1`, reload сбрасывал фильтры, а direct URL с
`action` и `entity_type` не восстанавливал controls или API request. Основная
кнопка фильтров имела высоту 32 px. Машинный результат сохранён в
`baseline-report.json`.

## Реализованный контракт

- `action`, `entity_type`, `actor_id`, `from` и `to` хранят применённые фильтры;
- `from` и `to` канонизируются в RFC3339/UTC, а форма показывает местное время;
- `cursor` хранит текущую страницу, повторяемый `previous_cursor` — путь назад;
- direct URL, reload, Back и Forward восстанавливают controls, API query и page;
- смена или сброс фильтров возвращает первую страницу;
- пустые, дублированные и невалидные параметры канонизируются через replace;
- неизвестные соседние query parameters сохраняются;
- неизвестные action/entity codes остаются доступными в форме для будущих событий;
- интерактивные controls фильтра имеют стабильную цель не меньше 40 px;
- названия событий в desktop table переносятся по словам, технический code —
  внутри собственного идентификатора.

## Результат

- functional URL/history/reload/direct-link/canonicalization: пройден;
- cursor page 3: `cursor=cursor-2&previous_cursor=cursor-1` восстановлен;
- визуальная матрица: 18/18 состояний, 2 UI states x 3 темы x 3 ширины;
- ширины: 375, 1920 и 2560 px;
- horizontal overflow: 0;
- интерактивные цели меньше 40 px: 0;
- serious/critical Axe violations: 0;
- page и console errors: 0;
- HTTP errors: 0;
- реальные write requests: 0.

Полный результат находится в `candidate-report.json`. Сохранены representative
screenshots closed/open states во всех целевых layout-классах.
