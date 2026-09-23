# Templates workflow QA

Дата проверки: 2026-09-23.

## Область проверки

- `/templates` с реальным Central Auth и production frontend bundle;
- поиск, фильтр типа и client-side pagination на каталоге из 25 fixture-записей;
- direct URL, reload, browser Back/Forward и canonical query parameters;
- создание шаблона для system admin, trim-aware validation и focus management;
- success notification и переход к созданию документа;
- responsive, темы, accessibility, размеры целей и runtime/network errors.

Fixture-каталог и ответы на `POST /api/v1/templates` перехватывались в браузере.
Реальный Wiki API не изменялся.

## Исходное поведение

Поиск, тип и номер страницы хранились только в React state. URL оставался
`/templates?keep=1`, а reload очищал критерии и возвращал страницу 1. Поля из
одних пробелов проходили native `required`, обрезались до пустых строк и
отправлялись на API. Машинный результат сохранён в `baseline-report.json`.

## Реализованный контракт

- `q` хранит нормализованный поисковый запрос и обновляется через debounce/replace;
- `type` хранит валидный тип шаблона, явная смена создаёт history entry;
- `page` хранит страницу больше первой, пагинация создаёт history entries;
- новый поиск или тип сбрасывает страницу;
- direct URL, reload, Back и Forward восстанавливают controls и выдачу;
- неизвестные соседние query-параметры сохраняются, невалидные параметры удаляются;
- пробельные name/body не вызывают API, поля получают `aria-invalid`, описание
  ошибки и фокус на первом невалидном control;
- success actions сохраняют платформенную интерактивную цель не меньше 40 px.

## Результат

- функциональный URL/history/reload сценарий: пройден;
- validation: 0 create requests, focus `template-name`, оба поля `aria-invalid=true`;
- визуальная матрица: 27/27 состояний, 3 UI states x 3 темы x 3 ширины;
- ширины: 375, 1920 и 2560 px;
- horizontal overflow: 0;
- интерактивные цели меньше 40 px: 0;
- serious/critical Axe violations: 0;
- page/runtime errors: 0;
- HTTP errors: 0;
- реальные write requests: 0.

Полный результат находится в `candidate-report.json`. Сохранены representative
screenshots каталога, validation и success states во всех целевых layout-классах.
