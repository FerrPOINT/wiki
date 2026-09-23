# Search URL state QA

Дата: 2026-09-23

## Дефект

На production bundle запрос `Wiki` и переключение типа результата на
`Документы` не меняли `/search`. После reload запрос и выбранный тип
сбрасывались. Baseline зафиксирован в `baseline-before-reload.png` и
`baseline-after-reload.png`.

## Изменение

- `q`, `space`, `task_key`, `phase_key`, `document_type` и `result_type`
  читаются из URL как применённые критерии.
- Изменение строки поиска обновляет URL через debounce + replace без заполнения
  browser history каждым введённым символом.
- Явные фильтры и тип результата создают history entries и сохраняют неизвестные
  соседние query-параметры.
- Прямая ссылка, reload, Back и Forward синхронизируют controls и API request.
- Несовместимый `document_type` удаляется при выборе материалов; неизвестные
  значения фильтров канонизируются без нового history entry.
- Курсорная страница возвращается к первой только после изменения критериев.

## Production candidate QA

Проверено с настоящими Central Auth и Wiki API:

- viewport: `375`, `1920`, `2560`;
- темы: light, gray, dark;
- сценарии: direct URL, `Документы`, `Материалы`, Back, Forward, filters,
  reload и сохранение соседнего `keep=1`;
- 9/9 состояний прошли;
- root overflow: 0;
- видимые controls меньше 40 px: 0;
- serious/critical Axe: 0;
- page errors: 0;
- неожиданные API errors: 0;
- write requests: 0.

Полный машинный отчёт находится в `report.json`. PNG-файлы содержат итоговые
mobile/desktop состояния во всех трёх темах.

## Automated gates

- Wiki search tests: 6/6;
- frontend suite: 24 файла / 139 тестов;
- TypeScript, ESLint, semantic color lint и production build: green;
- OpenAPI schema и compatibility: green;
- focused Prettier для изменённых source/test файлов: green.

Общий `pnpm format:check` сохраняет baseline-расхождение 77 файлов в `main`;
изменённые файлы проверены отдельно.
