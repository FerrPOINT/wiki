# Platform shell browser QA

Дата проверки: 2026-09-21.

## Матрица

| Viewport  | Темы        | Ожидаемая навигация |
| --------- | ----------- | ------------------- |
| 375x812   | light, dark | mobile drawer       |
| 768x900   | light, dark | rail 72 px          |
| 1440x900  | light, dark | sidebar 264 px      |
| 2560x1200 | light, dark | sidebar 264 px      |

## Результат

- Header: 60 px на всех viewport.
- Main offset: 0 / 72 / 264 px по breakpoint-контракту.
- Horizontal overflow: 0; console/page errors: 0.
- Активная ссылка `/settings`: `aria-current="page"` во всей матрице.
- Интерактивные элементы shell: не меньше 40x40 px и имеют доступные имена.
- Drawer: 320 px; фокус удерживается внутри, Escape закрывает drawer и возвращает
  фокус на trigger.
- Страница настроек сохраняет читаемую ширину на 2560 px.

Полные числовые результаты находятся в `qa-results.json`; PNG рядом являются
снимками production preview после финальной сборки.
